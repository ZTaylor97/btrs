use std::fmt;
use std::net::IpAddr;

use serde::{Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};

use serde::de;
use serde::de::Visitor;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TrackerRequest {
    #[serde(skip_serializing)]
    pub info_hash: String,
    pub peer_id: String,
    pub port: u64,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: Option<u64>,
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
            event: None,
            compact: Some(0),
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

        let expected_result = "peer_id=-RS0001-kONXltkhXIr5&port=6882&uploaded=0&downloaded=0&left=0&compact=0&numwant=50&event=started&info_hash=%DA%BFr%01%9D%EFM0%AF%00%F4%BFM%DF%8Ais%0C%02%B4";

        assert_eq!(request.to_query_string(), expected_result);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TrackerResponse {
    #[serde(rename = "failure reason")]
    failure_reason: Option<String>,
    #[serde(rename = "warning message")]
    warning_message: Option<String>,
    interval: u64,
    #[serde(rename = "min interval")]
    min_interval: Option<u64>,
    #[serde(rename = "tracker id")]
    tracker_id: Option<String>,
    complete: u64,
    incomplete: u64,
    peers: PeersEnum,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
pub enum PeersEnum {
    Dict(Vec<PeersDict>),
    Compact(Vec<u8>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PeersDict {
    #[serde(rename = "peer id")]
    peer_id: String,
    ip: String,
    port: u64,
}

// 👇 custom deserialization logic
impl<'de> Deserialize<'de> for PeersEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PeersEnumVisitor;

        impl<'de> Visitor<'de> for PeersEnumVisitor {
            type Value = PeersEnum;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("either a byte string (compact) or a list of peer dicts")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PeersEnum::Compact(v.to_vec()))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let peers: Vec<PeersDict> =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(PeersEnum::Dict(peers))
            }
        }

        deserializer.deserialize_any(PeersEnumVisitor)
    }
}
