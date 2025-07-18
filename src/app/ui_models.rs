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

impl TorrentItem {
    pub async fn try_from_torrent(t: &Torrent) -> Result<Self, anyhow::Error> {
        Ok(TorrentItem {
            name: String::from(t.name()),
            progress: 0.0,
            status: String::from("Stopped"),
            download_speed: String::from("0.0kb/s"),
            info_hash: String::from(t.info_hash()),
            peer_list: t.peer_list().await.to_vec(),
            files: t.get_file_tree()?,
        })
    }
}
