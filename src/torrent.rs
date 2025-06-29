//! Contains all state and structures for the
//! torrent client, including loading METAINFO and
//! making requests to trackers.

use std::{net::Ipv4Addr, sync::Arc};

use anyhow::{Context, Error};
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use urlencoding::encode_binary;

use metainfo::MetaInfo;

use crate::torrent::{
    metainfo::info::InfoEnum,
    tracker::{PeersEnum, TrackerSession},
};

pub mod files;
pub mod metainfo;
pub mod peer_session;
mod piece_manager;
pub mod tracker;
pub struct Torrent {
    metainfo: MetaInfo,
    info_hash: String,
    tracker_session: Arc<Mutex<TrackerSession>>, // TODO: PieceStorage
                                                 // TODO: Vec<PeerSession>
}

#[derive(Clone)]
pub struct Peer {
    pub ip: String,
    pub port: u64,
}

impl From<PeersEnum> for Vec<Peer> {
    fn from(peers_enum: PeersEnum) -> Self {
        let mut peers: Vec<Peer> = vec![];

        match peers_enum {
            tracker::PeersEnum::Dict(peers_dicts) => {
                for peer_raw in peers_dicts {
                    peers.push(Peer {
                        ip: peer_raw.ip.clone(),
                        port: peer_raw.port,
                    });
                }
            }
            tracker::PeersEnum::Compact(items) => {
                for chunk in items.chunks_exact(6) {
                    let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]).to_string();
                    let port: u64 = u16::from_be_bytes([chunk[4], chunk[5]]) as u64;
                    peers.push(Peer { ip, port })
                }
            }
        }

        peers
    }
}

impl Torrent {
    /// Adds a torrent to the client from bytes loaded from a .torrent file.
    pub fn load(bytes: &[u8], peer_id: &str) -> Result<Self, Error> {
        let metainfo = MetaInfo::from_bytes(&bytes)?;
        let info_hash = Self::calculate_info_hash(&bytes)?;

        let tracker_session = TrackerSession::new(&metainfo, &info_hash, peer_id);

        Ok(Self {
            metainfo,
            info_hash: String::from(info_hash),
            tracker_session: Arc::new(Mutex::new(tracker_session)),
        })
    }

    /// Calculates an `info_hash` from the info dictionary bytes found in
    /// the .torrent file.
    ///
    /// Returns an [`Error`](`anyhow::Error`) if:
    ///     - bytes are not valid bencode,
    ///     - info key is missing from bencode,
    ///     - an error happens converting back to bytes
    fn calculate_info_hash(bytes: &[u8]) -> Result<String, Error> {
        let value: Value = serde_bencode::from_bytes(&bytes)
            .context("Failed to decode .torrent file as bencode")?;

        let info_value = match value {
            Value::Dict(ref dict) => dict
                .get(&b"info".to_vec())
                .context("Missing 'info' key in .torrent file")?,
            _ => anyhow::bail!("Top-level bencode structure is not a dictionary"),
        };

        let info_bytes = serde_bencode::to_bytes(info_value)
            .context("Failed to re-encode 'info' value to bencode")?;

        let mut hasher = Sha1::new();
        hasher.update(&info_bytes);
        let hash = hasher.finalize();

        Ok(encode_binary(&hash).into_owned())
    }

    pub fn start_tracker(&mut self) {
        let tracker = Arc::clone(&self.tracker_session);

        tokio::spawn(async move {
            {
                let mut session = tracker.lock().await;
                if session.started {
                    return;
                }

                session.started = true;
            }
            loop {
                let wait_time = {
                    let mut session = tracker.lock().await;
                    session.started = true;
                    if let Err(e) = session.update().await {
                        eprintln!("[Tracker] Update failed: {:?}", e);
                    }

                    // Wait 5 seconds if wait time is in the past
                    if Instant::from_std(session.next_announce) < Instant::now() {
                        Instant::now() + Duration::from_secs(5)
                    } else {
                        Instant::from_std(session.next_announce)
                    }
                };

                tokio::time::sleep_until(wait_time).await;
            }
        });
    }

    pub fn name(&self) -> &str {
        match &self.metainfo.info {
            InfoEnum::MultiFile(info_multi_file) => &info_multi_file.name,
            InfoEnum::SingleFile(info_single_file) => &info_single_file.name,
        }
    }

    pub fn info_hash(&self) -> &str {
        &self.info_hash
    }

    pub async fn peer_list(&self) -> Vec<Peer> {
        let tracker = Arc::clone(&self.tracker_session);

        let session = tracker.lock().await;

        session.peer_list.clone()
    }

    pub fn get_file_tree(&self) -> Result<files::FileEntry, anyhow::Error> {
        let mut root = files::FileEntry::new(".");

        match &self.metainfo.info {
            InfoEnum::MultiFile(info_multi_file) => {
                for file in &info_multi_file.files {
                    root.insert_path(&file.path)?;
                }
            }
            InfoEnum::SingleFile(info_single_file) => {
                root.insert_path(&[info_single_file.name.clone()])?;
            }
        }
        Ok(root)
    }
}
