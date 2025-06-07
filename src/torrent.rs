use anyhow::Error;
use rand::{Rng, distr::Alphanumeric};
use urlencoding::encode_binary;

use crate::torrent::metainfo::MetaInfo;

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

        println!("{peer_id}");

        Torrent {
            torrents: vec![],
            peer_id,
        }
    }

    pub fn add_torrent(&mut self, file_path: &str) -> Result<(), Error> {
        let metainfo = MetaInfo::from_file(file_path)?;

        println!("{:?}", metainfo.get_info_hash());
        self.torrents.push(metainfo);

        Ok(())
    }
}
