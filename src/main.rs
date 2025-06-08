use torrent::Torrent;

mod torrent;

#[tokio::main]
async fn main() {
    let mut torrent_state = Torrent::new();

    torrent_state
        .add_torrent("test_files/Adventure Time - Season 7.torrent")
        .unwrap();

    torrent_state.download_torrents().await;
}
