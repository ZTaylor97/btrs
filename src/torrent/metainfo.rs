//! Torrent metadata parsing.
//!
//! Contains the structures and deserialization logic
//! for parsing `.torrent` files into usable Rust types.
use anyhow::Result;
use info::InfoEnum;
use serde_derive::{Deserialize, Serialize};

pub mod info;

/// Metadata for a torrent for clients to configure sessions.
///
/// Deserialize from .torrent files.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MetaInfo {
    pub(super) info: InfoEnum,
    pub(super) announce: String,
    #[serde(rename = "announce-list")]
    pub(super) announce_list: Option<Vec<Vec<String>>>,
    #[serde(rename = "creation date")]
    pub(super) creation_date: Option<u64>,
    pub(super) comment: Option<String>,
    #[serde(rename = "created by")]
    pub(super) created_by: Option<String>,
    pub(super) encoding: Option<String>,
}

impl MetaInfo {
    /// Deserializes a metainfo dictionary bytes into a [MetaInfo] struct.
    ///
    /// Returns an [`anyhow::Error`] if file is not found or .torrent file
    /// is invalid.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(serde_bencode::from_bytes(bytes)?)
    }

    pub fn info(&self) -> &InfoEnum {
        return &self.info;
    }

    pub fn get_tracker_urls(&self) -> &str {
        return &self.announce;
    }
}

#[cfg(test)]
mod metainfo_tests {
    use serde_bytes::ByteBuf;

    use super::info::*;
    use super::*;

    fn mock_metainfo() -> MetaInfo {
        MetaInfo {
            announce: "http://tracker.test/multi/announce".to_string(),
            announce_list: Some(vec![vec!["http://backup.tracker".to_string()]]),
            creation_date: Some(1_700_000_001),
            comment: Some("Multi file test".to_string()),
            created_by: Some("btrs-test".to_string()),
            encoding: Some("UTF-8".to_string()),
            info: InfoEnum::MultiFile(InfoMultiFile {
                name: "test_folder".to_string(),
                piece_length: 32768,
                pieces: ByteBuf::from(vec![0u8; 40]), // two pieces
                files: vec![
                    FilesDict {
                        length: 1000,
                        md5: None,
                        path: vec!["subfolder".to_string(), "file1.txt".to_string()],
                    },
                    FilesDict {
                        length: 2000,
                        md5: None,
                        path: vec!["file2.txt".to_string()],
                    },
                ],
            }),
        }
    }

    #[test]
    fn it_works() {
        let test1 = "d5:filesld6:lengthi1000e4:pathl9:subfolder9:file1.txteed6:lengthi2000e4:pathl9:file2.txteee4:name11:test_folder12:piece lengthi32768e6:pieces40:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0e";

        let info: InfoEnum = serde_bencode::from_str(test1).unwrap();

        let test_info = mock_metainfo();
        assert_eq!(info, test_info.info);
    }
}
