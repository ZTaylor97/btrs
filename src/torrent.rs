//! Contains all state and structures for the
//! torrent client, including loading METAINFO and
//! making requests to trackers.

use std::net::Ipv4Addr;

use anyhow::{Context, Error};
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use urlencoding::encode_binary;

use metainfo::MetaInfo;

use crate::torrent::{metainfo::info::InfoEnum, tracker::PeersEnum};

pub mod files;
pub mod metainfo;
pub mod tracker;

pub struct Torrent {
    metainfo: MetaInfo,
    info_hash: String,
    peer_list: Vec<Peer>,
    // TODO: TrackerSession
    // TODO: PieceStorage
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
    pub fn load(bytes: &[u8]) -> Result<Self, Error> {
        let metainfo = MetaInfo::from_bytes(&bytes)?;
        let info_hash = Self::calculate_info_hash(&bytes)?;

        Ok(Self {
            metainfo,
            info_hash,
            peer_list: vec![],
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

    pub async fn download(&mut self, peer_id: &str) -> Result<(), Error> {
        let result = tracker::fetch_peers(
            &self.metainfo,
            &tracker::TrackerRequest::new(&self.info_hash, &peer_id),
        )
        .await?;

        if let Some(peers) = result.peers {
            self.peer_list = peers.into();
        }

        Ok(())
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

    pub fn peer_list(&self) -> &[Peer] {
        &self.peer_list
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
