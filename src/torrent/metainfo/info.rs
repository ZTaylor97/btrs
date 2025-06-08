//! Submodule containing structures related to the `Info` dictionary.

use serde::{Deserialize, Deserializer, de};
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use urlencoding::encode_binary;

/// InfoMultiFile format contains the files key.
/// Present when torrent consists of multiple files.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct InfoMultiFile {
    pub(super) name: String,
    pub(super) length: Option<u64>,
    pub(super) md5sum: Option<String>,
    #[serde(rename = "piece length")]
    pub(super) piece_length: u64,
    pub(super) pieces: ByteBuf,
    pub(super) files: Vec<FilesDict>,
}

/// Fields to deserialize the files list into for
/// a multi file torrent.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FilesDict {
    pub(super) length: u64,
    pub(super) md5sum: Option<String>,
    pub(super) path: Vec<String>,
}

/// InfoSingleFile format does not contain the files key.
/// Present when torrent is only one file.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct InfoSingleFile {
    pub(super) name: String,
    pub(super) length: u64,
    pub(super) md5sum: Option<String>,
    #[serde(rename = "piece length")]
    pub(super) piece_length: u64,
    pub(super) pieces: ByteBuf,
}

/// Allow automatic serialization to correct `Info` format
/// out of `MultiFile` or `SingleFile`.
#[derive(Serialize, PartialEq, Eq, Debug)]
pub enum InfoEnum {
    MultiFile(InfoMultiFile),
    SingleFile(InfoSingleFile),
}

impl InfoEnum {
    pub fn get_hash(&self) -> String {
        let mut hasher = Sha1::new();

        let bytes = serde_bencode::to_bytes(&self).unwrap();
        hasher.update(&bytes);

        let result = hasher.finalize();

        let slice = result.as_slice();

        encode_binary(slice).into_owned()
    }
}

impl<'de> Deserialize<'de> for InfoEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Extract Value out to check for files field
        let value = Value::deserialize(deserializer)?;

        if let Value::Dict(ref dict) = value {
            let encoded = serde_bencode::to_bytes(&value).map_err(de::Error::custom)?;

            // If files key is present, then info must be multi file, otherwise assume single file.
            if dict.contains_key(&b"files".to_vec()) {
                let multi: InfoMultiFile =
                    serde_bencode::from_bytes(&encoded).map_err(de::Error::custom)?;
                Ok(InfoEnum::MultiFile(multi))
            } else {
                let single: InfoSingleFile =
                    serde_bencode::from_bytes(&encoded).map_err(de::Error::custom)?;
                Ok(InfoEnum::SingleFile(single))
            }
        } else {
            Err(de::Error::custom("Expected a dictionary for InfoEnum"))
        }
    }
}
