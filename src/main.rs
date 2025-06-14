use torrent::Torrents;

pub mod torrent;
pub mod tui;

#[tokio::main]
async fn main() {
    let mut torrent_state = Torrents::new();

    torrent_state
        .add_torrent("test_files/A_Little_Princess_WB39_WOC_2001-07_archive.torrent")
        .unwrap();

    torrent_state.download_torrents().await;
}
