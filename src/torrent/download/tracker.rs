use std::net::IpAddr;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TrackerRequest {
    #[serde(skip_serializing)]
    pub info_hash: String,
    pub peer_id: String,
    pub port: u64,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: Option<bool>,
    pub no_peer_id: Option<bool>,
    pub numwant: u64,
    pub event: Option<TrackerEvent>,
    pub ip: Option<IpAddr>,
    pub key: Option<String>,
    pub trackerid: Option<String>,
}

impl TrackerRequest {
    pub fn new(info_hash: &str, peer_id: &str) -> Self {
        Self {
            info_hash: String::from(info_hash),
            peer_id: String::from(peer_id),
            port: 6882,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            event: Some(TrackerEvent::Started),
            compact: None,
            no_peer_id: None,
            ip: None,
            numwant: 50,
            key: None,
            trackerid: None,
        }
    }
    pub fn to_query_string(&self) -> String {
        let mut encoded = serde_urlencoded::to_string(&self).unwrap();
        encoded.push_str("&info_hash=");

        encoded.push_str(self.info_hash.as_str());

        encoded
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum TrackerEvent {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "completed")]
    Completed,
}

#[cfg(test)]
mod tracker_tests {
    use super::*;

    #[test]
    fn test_to_query_string() {
        let request = TrackerRequest::new(
            "%DA%BFr%01%9D%EFM0%AF%00%F4%BFM%DF%8Ais%0C%02%B4",
            "-RS0001-kONXltkhXIr5",
        );

        println!("{}", request.to_query_string());
    }
}
