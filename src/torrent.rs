use anyhow::Error;

use crate::torrent::metainfo::MetaInfo;

mod download;
mod metainfo;

pub struct Torrent {
    torrents: Vec<metainfo::MetaInfo>,
}

impl Torrent {
    pub fn new() -> Self {
        Torrent { torrents: vec![] }
    }

    pub fn add_torrent(&mut self, file_path: &str) -> Result<(), Error> {
        let metainfo = MetaInfo::from_file(file_path)?;

        println!("{:?}", metainfo.get_info_hash());
        self.torrents.push(metainfo);

        Ok(())
    }
}
