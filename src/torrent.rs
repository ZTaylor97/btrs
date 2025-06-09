//! Contains all state and structures for the
//! torrent client, including loading METAINFO and
//! making requests to trackers.

use std::{fs, net::Ipv4Addr, path::Path};

use anyhow::{Context, Error};
use rand::{Rng, distr::Alphanumeric};
use serde::de;
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use urlencoding::encode_binary;

use metainfo::MetaInfo;

use download::tracker::TrackerRequest;

mod download;
mod metainfo;

pub struct Torrents {
    torrents: Vec<Torrent>,
    peer_id: String,
}

pub struct Torrent {
    metainfo: MetaInfo,
    info_hash: String,
}

impl Torrents {
    pub fn new() -> Self {
        let prefix = b"-RS0001-";
        let mut peer_id_bytes = [0u8; 20];

        peer_id_bytes[..8].copy_from_slice(prefix);

        let rand_part: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        peer_id_bytes[8..].copy_from_slice(rand_part.as_bytes());

        let peer_id = encode_binary(&peer_id_bytes).into_owned();

        Torrents {
            torrents: vec![],
            peer_id,
        }
    }

    /// Adds a torrent to the client from a .torrent `file_path`
    pub fn add_torrent(&mut self, file_path: &str) -> Result<(), Error> {
        let bytes: Vec<u8> = fs::read(file_path).expect("{file_path} not found.");
        let metainfo = MetaInfo::from_bytes(&bytes)?;

        let info_hash = Self::info_hash(&bytes)?;

        self.torrents.push(Torrent {
            metainfo,
            info_hash,
        });

        Ok(())
    }

    /// Calculates an `info_hash` from the info dictionary bytes found in
    /// the .torrent file.
    ///
    /// Returns an [`Error`](`anyhow::Error`) if:
    ///     - bytes are not valid bencode,
    ///     - info key is missing from bencode,
    ///     - an error happens converting back to bytes
    pub fn info_hash(bytes: &[u8]) -> Result<String, Error> {
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

    /// Basic WIP function to make a request to a tracker.
    pub async fn download_torrents(&self) {
        for torrent in &self.torrents {
            let result = download::download(
                &torrent.metainfo,
                &TrackerRequest::new(&torrent.info_hash, &self.peer_id),
            )
            .await
            .unwrap();

            if let Ok(Value::Dict(mut map)) = serde_bencode::from_bytes::<Value>(&result[..]) {
                if let Some(peers) = map.remove(&b"peers".to_vec()) {
                    match peers {
                        Value::Bytes(compact_peers) => {
                            println!("Compact peers: {} bytes", compact_peers.len());

                            for chunk in compact_peers.chunks_exact(6) {
                                let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                                let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                                println!("Peer: {}:{}", ip, port);
                            }
                        }
                        Value::List(peer_list) => {
                            println!("Non-compact peer list with {} entries", peer_list.len());
                        }
                        _ => {
                            println!("Unexpected format for peers field: {:?}", peers);
                        }
                    }
                } else {
                    println!("No peers field in response.");
                }
            }
        }
    }
}
