//! Torrent metadata parsing.
//!
//! Contains the structures and deserialization logic
//! for parsing `.torrent` files into usable Rust types.
use std::fs;

use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

use anyhow::Result;
use urlencoding::encode_binary;

use serde_derive::{Deserialize, Serialize};

/// Metadata for a torrent for clients to configure sessions.
///
/// Deserialize from .torrent files.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MetaInfo {
    info: Info,
    pub announce: String,
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    creation_date: Option<u64>,
    comment: Option<String>,
    created_by: Option<String>,
    encoding: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Info {
    name: String,
    length: Option<u64>,
    md5sum: Option<String>,
    #[serde(rename = "piece length")]
    piece_length: u64,
    pieces: ByteBuf,
    files: Vec<FilesDict>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FilesDict {
    length: u64,
    md5sum: Option<String>,
    path: Vec<String>,
}

impl MetaInfo {
    /// Deserializes a .torrent file at `file_path` into a [MetaInfo] struct.
    ///
    /// Returns an [`anyhow::Error`] if file is not found or .torrent file
    /// is invalid.
    pub fn from_file(file_path: &str) -> Result<Self, anyhow::Error> {
        let bytes: Vec<u8> = fs::read(file_path).expect("{file_path} not found.");

        Ok(serde_bencode::from_bytes(&bytes)?)
    }

    /// Deserializes a .torrent file at `file_path` into a [MetaInfo] struct.
    ///
    /// Returns an [`anyhow::Error`] if file is not found or .torrent file
    /// is invalid.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(serde_bencode::from_bytes(bytes)?)
    }

    pub fn get_info_hash(&self) -> String {
        let mut hasher = Sha1::new();

        let bytes = serde_bencode::to_bytes(&self.info).unwrap();
        hasher.update(&bytes);

        let result = hasher.finalize();

        let slice = result.as_slice();

        encode_binary(slice).into_owned()
    }
}

#[cfg(test)]
mod metainfo_tests {
    use super::*;

    fn mock_metainfo() -> MetaInfo {
        MetaInfo {
            announce: "http://tracker.test/multi/announce".to_string(),
            announce_list: Some(vec![vec!["http://backup.tracker".to_string()]]),
            creation_date: Some(1_700_000_001),
            comment: Some("Multi file test".to_string()),
            created_by: Some("btrs-test".to_string()),
            encoding: Some("UTF-8".to_string()),
            info: Info {
                name: "test_folder".to_string(),
                length: None,
                md5sum: None,
                piece_length: 32768,
                pieces: ByteBuf::from(vec![0u8; 40]), // two pieces
                files: vec![
                    FilesDict {
                        length: 1000,
                        md5sum: None,
                        path: vec!["subfolder".to_string(), "file1.txt".to_string()],
                    },
                    FilesDict {
                        length: 2000,
                        md5sum: None,
                        path: vec!["file2.txt".to_string()],
                    },
                ],
            },
        }
    }

    #[test]
    fn it_works() {
        let test1 = "d5:filesld6:lengthi1000e4:pathl9:subfolder9:file1.txteed6:lengthi2000e4:pathl9:file2.txteee4:name11:test_folder12:piece lengthi32768e6:pieces40:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0e";

        let info: Info = serde_bencode::from_str(test1).unwrap();

        let test_info = mock_metainfo();
        assert_eq!(info, test_info.info);

        let hash = test_info.get_info_hash();

        assert_eq!(
            hash.as_str(),
            "%D7%E4%25%A6%8B%2B%B1%CC5Y%AD%93T%85%D1%A5%A4%E0%E9%CB"
        )
    }
}
