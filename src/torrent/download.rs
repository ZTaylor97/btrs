use crate::torrent::{download::tracker::TrackerRequest, metainfo::MetaInfo};

pub mod tracker;

pub async fn download(
    torrent: &MetaInfo,
    request: &TrackerRequest,
) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!("{}?{}", torrent.announce, request.to_query_string());
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let bytes = res.bytes().await?;

    Ok(bytes.to_vec())
}
