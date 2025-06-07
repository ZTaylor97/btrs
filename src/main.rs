use torrent::Torrent;

mod torrent;
fn main() {
    let deserialised = torrent::Torrent::from_file(
        "test_files/A_Little_Princess_WB39_WOC_2001-07_archive.torrent",
    )
    .unwrap();
}
