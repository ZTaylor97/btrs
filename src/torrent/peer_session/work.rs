use crate::torrent::piece_manager::{PieceError, PieceRequest, PieceResponse};

const BLOCK_SIZE: usize = 16 * 1024;
pub struct BlockInfo {
    pub offset: u32,
    pub length: u32,
    pub status: BlockStatus,
    pub data: Vec<u8>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum BlockStatus {
    Full,
    Empty,
    InProgress,
}
pub struct PieceWork {
    pub index: u32,
    pub length: usize,
    pub block_size: usize,
    pub blocks: Vec<BlockInfo>,
}
pub struct BlockResponse {
    pub index: u32,
    pub begin: u32,
    pub block: Vec<u8>,
}

impl From<PieceRequest> for PieceWork {
    fn from(value: PieceRequest) -> Self {
        let mut blocks = vec![];
        let mut offset: usize = 0;

        while offset < value.length_bytes {
            let remaining = value.length_bytes - offset;
            let block_len = std::cmp::min(BLOCK_SIZE, remaining);

            let block = BlockInfo {
                offset: offset as u32,
                length: block_len as u32,
                status: BlockStatus::Empty,
                data: Vec::with_capacity(block_len),
            };

            blocks.push(block);
            offset += block_len;
        }

        debug_assert_eq!(
            blocks.iter().map(|b| b.length as usize).sum::<usize>(),
            value.length_bytes,
            "Total block length doesn't match piece length!"
        );

        Self {
            index: value.piece_index,
            length: value.length_bytes,
            block_size: BLOCK_SIZE,
            blocks,
        }
    }
}

impl PieceWork {
    pub fn is_complete(&self) -> bool {
        self.blocks
            .iter()
            .all(|block| block.status == BlockStatus::Full)
    }

    // TODO implement
    pub fn to_piece_response(self) -> PieceResponse {
        let bytes: Vec<u8> = self
            .blocks
            .into_iter()
            .map(|block| block.data)
            .flatten()
            .collect();

        if bytes.len() != self.length {
            PieceResponse {
                piece_index: self.index,
                result: Err(PieceError::InvalidData(String::from(
                    "piece data is malformed",
                ))),
            }
        } else {
            PieceResponse {
                piece_index: self.index,
                result: Ok(bytes),
            }
        }
    }
}
