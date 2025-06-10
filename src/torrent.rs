//! Contains all state and structures for the
//! torrent client, including loading METAINFO and
//! making requests to trackers.

use std::net::Ipv4Addr;

use anyhow::{Context, Error};
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use urlencoding::encode_binary;

use metainfo::MetaInfo;

mod download;
pub mod metainfo;

pub struct Torrent {
    metainfo: MetaInfo,
    info_hash: String,
}

impl Torrent {
    /// Adds a torrent to the client from bytes loaded from a .torrent file.
    pub fn load(bytes: &[u8]) -> Result<Self, Error> {
        let metainfo = MetaInfo::from_bytes(&bytes)?;
        let info_hash = Self::calculate_info_hash(&bytes)?;

        Ok(Self {
            metainfo,
            info_hash,
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

    pub async fn download(&self, peer_id: &str) -> Result<(), Error> {
        let result = download::download(
            &self.metainfo,
            &download::tracker::TrackerRequest::new(&self.info_hash, &peer_id),
        )
        .await?;

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

        Ok(())
    }

    pub fn get_metainfo(&self) -> &MetaInfo {
        &self.metainfo
    }
    pub fn get_info_hash(&self) -> &str {
        &self.info_hash
    }
}
