use crate::torrent::{Peer, Torrent, files::FileEntry};

#[derive(Clone)]
pub struct TorrentItem {
    pub name: String,
    pub progress: f64,
    pub status: String,
    pub download_speed: String,
    pub info_hash: String,
    pub peer_list: Vec<Peer>,
    pub files: FileEntry,
}

impl From<&Torrent> for TorrentItem {
    fn from(t: &Torrent) -> Self {
        let info_hash = t.get_info_hash();

        let name = t.get_name();

        let files = t.get_file_tree();

        TorrentItem {
            name: name,
            progress: 0.0,
            status: String::from("Stopped"),
            download_speed: String::from("0.0kb/s"),
            info_hash: String::from(info_hash),
            peer_list: t.get_peer_list().to_vec(),
            files,
        }
    }
}
