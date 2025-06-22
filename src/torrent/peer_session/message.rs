use std::io::Read;

use anyhow::{anyhow, bail};
use bytes::{Buf, BytesMut};

pub enum MessageType {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
    Port(u16),
    KeepAlive,
}

impl MessageType {
    pub fn from_bytes(bytes: &mut BytesMut, id: u8, len: u32) -> Result<Self, anyhow::Error> {
        if bytes.len() < 4 {
            bail!("Message {bytes:?} invalid");
        }

        let message_length = bytes.get_u32();
        if message_length == 0 {
            return Ok(Self::KeepAlive);
        }

        if bytes.len() < message_length as usize {
            bail!("Message {bytes:?} has less than")
        }

        let idx = bytes.get_u8();

        match idx {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => {
                let index = bytes.get_u32();
                Ok(Self::Have(index))
            }
            5 => (),
            6 => (),
            7 => (),
            8 => (),
            9 => (),
            _ => (),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut message: Vec<u8> = vec![];

        match self {
            MessageType::Choke => {
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(0u8);
            }
            MessageType::Unchoke => {
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(1u8);
            }
            MessageType::Interested => {
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(2u8);
            }
            MessageType::NotInterested => {
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(3u8);
            }
            MessageType::Have(idx) => {
                message.extend_from_slice(&5u32.to_be_bytes());
                message.push(4u8);
                message.extend_from_slice(&idx.to_be_bytes());
            }
            MessageType::Bitfield(items) => {
                let len = items.len() + 1;
                message.extend_from_slice(&len.to_be_bytes());
                message.push(5u8);
                message.extend(items);
            }
            MessageType::Request {
                index,
                begin,
                length,
            } => {
                message.extend_from_slice(&13u32.to_be_bytes());
                message.push(6u8);
                message.extend_from_slice(&index.to_be_bytes());
                message.extend_from_slice(&begin.to_be_bytes());
                message.extend_from_slice(&length.to_be_bytes());
            }
            MessageType::Piece {
                index,
                begin,
                block,
            } => {
                let len = 9 + block.len();
                message.extend_from_slice(&len.to_be_bytes());
                message.push(7u8);
                message.extend_from_slice(&index.to_be_bytes());
                message.extend_from_slice(&begin.to_be_bytes());
                message.extend(block);
            }
            MessageType::Cancel {
                index,
                begin,
                length,
            } => {
                message.extend_from_slice(&13u32.to_be_bytes());
                message.push(8u8);
                message.extend_from_slice(&index.to_be_bytes());
                message.extend_from_slice(&begin.to_be_bytes());
                message.extend_from_slice(&length.to_be_bytes());
            }
            MessageType::Port(port) => {
                message.extend_from_slice(&3u32.to_be_bytes());
                message.push(9u8);
                message.extend_from_slice(&port.to_be_bytes());
            }
            MessageType::KeepAlive => message.extend_from_slice(&0u32.to_be_bytes()),
        }

        message
    }
}
