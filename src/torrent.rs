//! Contains all state and structures for the
//! torrent client, including loading METAINFO and
//! making requests to trackers.

use std::net::Ipv4Addr;

use anyhow::Error;
use rand::{Rng, distr::Alphanumeric};
use serde_bencode::value::Value;
use urlencoding::encode_binary;

use metainfo::MetaInfo;

use download::tracker::TrackerRequest;

mod download;
mod metainfo;

pub struct Torrent {
    torrents: Vec<metainfo::MetaInfo>,
    peer_id: String,
}

impl Torrent {
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

        Torrent {
            torrents: vec![],
            peer_id,
        }
    }

    pub fn add_torrent(&mut self, file_path: &str) -> Result<(), Error> {
        let metainfo = MetaInfo::from_file(file_path)?;

        self.torrents.push(metainfo);

        Ok(())
    }

    pub async fn download_torrents(&self) {
        for torrent in &self.torrents {
            let result = download::download(
                &torrent,
                &TrackerRequest::new(&torrent.get_info_hash(), &self.peer_id),
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
