pub mod app;
pub mod torrent;
pub mod tui;

use app::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.download_torrents().await.unwrap();
}
