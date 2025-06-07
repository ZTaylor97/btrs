use std::fs;

mod metainfo;
fn main() {
    let bytes: Vec<u8> =
        fs::read("test_files/A_Little_Princess_WB39_WOC_2001-07_archive.torrent").unwrap();

    let deserialized: metainfo::MetaInfo = serde_bencode::from_bytes(&bytes).unwrap();

    println!("{deserialized:?}");
}
