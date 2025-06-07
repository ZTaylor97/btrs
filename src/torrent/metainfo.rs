//! Torrent metadata parsing.
//!
//! Contains the structures and deserialization logic
//! for parsing `.torrent` files into usable Rust types.
use std::fs;

use anyhow::Result;

use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MetaInfo {
    info: Info,
    announce: String,
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
    pieces: serde_bytes::ByteBuf,
    files: Vec<FilesDict>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FilesDict {
    length: u64,
    md5sum: Option<String>,
    path: Vec<String>,
}

impl MetaInfo {
    pub fn from_file(file_path: &str) -> Result<Self, anyhow::Error> {
        let bytes: Vec<u8> = fs::read(file_path).unwrap();

        Ok(serde_bencode::from_bytes(&bytes)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(serde_bencode::from_bytes(bytes)?)
    }
}
