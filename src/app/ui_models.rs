use crate::torrent::metainfo::info::InfoEnum;
use crate::torrent::{Peer, Torrent};

#[derive(Clone)]
pub struct TorrentItem {
    pub name: String,
    pub progress: f64,
    pub status: String,
    pub download_speed: String,
    pub info_hash: String,
    pub peer_list: Vec<Peer>,
}

impl From<&Torrent> for TorrentItem {
    fn from(t: &Torrent) -> Self {
        let info_hash = t.get_info_hash();

        let name = t.get_name();

        TorrentItem {
            name: name,
            progress: 0.0,
            status: String::from("Stopped"),
            download_speed: String::from("0.0kb/s"),
            info_hash: String::from(info_hash),
            peer_list: t.get_peer_list().to_vec(),
        }
    }
}


