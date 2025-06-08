//! All download logic is stored here.

use crate::torrent::{
    download::tracker::{TrackerRequest, TrackerResponse},
    metainfo::MetaInfo,
};

pub mod tracker;

/// Test function which gets a list of peers for
/// the tracker found in [`torrent`](`MetaInfo`)
///
/// Returns the response bytes or a [`reqwest::Error`]
pub async fn download(
    torrent: &MetaInfo,
    request: &TrackerRequest,
) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!("{}?{}", torrent.announce, request.to_query_string());
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let bytes = res.bytes().await?;

    let test: TrackerResponse = serde_bencode::from_bytes(&bytes.to_vec()).unwrap();

    println!("{test:?}");

    Ok(bytes.to_vec())
}
