//! All download logic is stored here.

use crate::torrent::{
    download::tracker::{TrackerRequest, TrackerResponse},
    metainfo::MetaInfo,
};

pub mod tracker;

/// Test function which gets a list of peers for
/// the tracker found in [`torrent`](`MetaInfo`)
///
/// Returns the response or a [`anyhow::Error`]
pub async fn download(
    torrent: &MetaInfo,
    request: &TrackerRequest,
) -> Result<TrackerResponse, anyhow::Error> {
    let url = format!("{}?{}", torrent.announce, request.to_query_string());
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let bytes = res.bytes().await?;

    let response: TrackerResponse = serde_bencode::from_bytes(&bytes.to_vec())?;

    Ok(response)
}
