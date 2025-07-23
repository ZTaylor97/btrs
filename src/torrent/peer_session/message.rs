use anyhow::bail;
use bytes::{Buf, BytesMut};

const MSG_CHOKE: u8 = 0;
const MSG_UNCHOKE: u8 = 1;
const MSG_INTERESTED: u8 = 2;
const MSG_NOT_INTERESTED: u8 = 3;
const MSG_HAVE: u8 = 4;
const MSG_BITFIELD: u8 = 5;
const MSG_REQUEST: u8 = 6;
const MSG_PIECE: u8 = 7;
const MSG_CANCEL: u8 = 8;
const MSG_PORT: u8 = 9;

/// Represents the peer protocol Message type for serializing and deserializing.
#[derive(Clone, PartialEq, Debug)]
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
    /// Extract a peer protocol message from the bytes read from a peer.
    pub fn from_bytes(bytes: &mut BytesMut, id: u8, len: u32) -> Result<Self, anyhow::Error> {
        if bytes.len() < 4 {
            bail!("Message {bytes:?} invalid");
        }

        let message_length = bytes.try_get_u32()?;
        if message_length == 0 {
            return Ok(Self::KeepAlive);
        }

        if bytes.len() < message_length as usize {
            bail!("Message {bytes:?} has less than length {message_length} bytes")
        }

        let idx = bytes.try_get_u8()?;

        Ok(match idx {
            MSG_CHOKE => Self::Choke,
            MSG_UNCHOKE => Self::Unchoke,
            MSG_INTERESTED => Self::Interested,
            MSG_NOT_INTERESTED => Self::NotInterested,
            MSG_HAVE => {
                let index = bytes.try_get_u32()?;
                Self::Have(index)
            }
            MSG_BITFIELD => {
                // 1 byte has already been requested from bytes so take length - 1 bytes.
                let bitfield = bytes.split_to(len as usize - 1).to_vec();
                Self::Bitfield(bitfield)
            }
            MSG_REQUEST => {
                let index = bytes.try_get_u32()?;
                let begin = bytes.try_get_u32()?;
                let length = bytes.try_get_u32()?;

                Self::Request {
                    index,
                    begin,
                    length,
                }
            }
            MSG_PIECE => {
                let index = bytes.try_get_u32()?;
                let begin = bytes.try_get_u32()?;
                // 9 bytes (1 + 4 + 4) have already been taken from bytes so take length - 9 bytes.
                let block = bytes.split_to(len as usize - 9).to_vec();

                Self::Piece {
                    index,
                    begin,
                    block,
                }
            }
            MSG_CANCEL => {
                let index = bytes.try_get_u32()?;
                let begin = bytes.try_get_u32()?;
                let length = bytes.try_get_u32()?;

                Self::Cancel {
                    index,
                    begin,
                    length,
                }
            }
            MSG_PORT => {
                let port = bytes.try_get_u16()?;
                Self::Port(port)
            }
            _ => bail!("Invalid message id {id}"),
        })
    }

    /// Serialize a peer protocol message into bytes to send across the wire to a listener.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut message: Vec<u8> = vec![];

        match self {
            MessageType::Choke => {
                // Choke message has a fixed length of 1 byte from the message index.
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(MSG_CHOKE);
            }
            MessageType::Unchoke => {
                // Unchoke message has a fixed length of 1 byte from the message index.
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(MSG_UNCHOKE);
            }
            MessageType::Interested => {
                // Interested message has a fixed length of 1 byte from the message index.
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(MSG_INTERESTED);
            }
            MessageType::NotInterested => {
                // NotInterested message has a fixed length of 1 byte from the message index.
                message.extend_from_slice(&1u32.to_be_bytes());
                message.push(MSG_NOT_INTERESTED);
            }
            MessageType::Have(idx) => {
                // Have messages have a fixed length of 5 bytes, 4 for the piece index and 1 for the message index.
                message.extend_from_slice(&5u32.to_be_bytes());
                message.push(MSG_HAVE);
                message.extend_from_slice(&idx.to_be_bytes());
            }
            MessageType::Bitfield(items) => {
                // other clients will expect that len is indexed from 1 not 0.
                let len: u32 = items.len() as u32 + 1;
                message.extend_from_slice(&len.to_be_bytes());
                message.push(MSG_BITFIELD);
                message.extend(items);
            }
            MessageType::Request {
                index,
                begin,
                length,
            } => {
                // Request has a fixed length of 13 bytes 4 for each of the u32s which are index, begin, and length. and one for the message index.
                message.extend_from_slice(&13u32.to_be_bytes());
                message.push(MSG_REQUEST);
                message.extend_from_slice(&index.to_be_bytes());
                message.extend_from_slice(&begin.to_be_bytes());
                message.extend_from_slice(&length.to_be_bytes());
            }
            MessageType::Piece {
                index,
                begin,
                block,
            } => {
                // Piece has a variable length of 9 from the index, begin, and message index + the length of the block.
                let len: u32 = 9 + block.len() as u32;
                message.extend_from_slice(&len.to_be_bytes());
                message.push(MSG_PIECE);
                message.extend_from_slice(&index.to_be_bytes());
                message.extend_from_slice(&begin.to_be_bytes());
                message.extend(block);
            }
            MessageType::Cancel {
                index,
                begin,
                length,
            } => {
                // Request has a fixed length of 13 bytes 4 for each of the u32s which are index, begin, and length. and one for the message index.
                message.extend_from_slice(&13u32.to_be_bytes());
                message.push(MSG_CANCEL);
                message.extend_from_slice(&index.to_be_bytes());
                message.extend_from_slice(&begin.to_be_bytes());
                message.extend_from_slice(&length.to_be_bytes());
            }
            MessageType::Port(port) => {
                // Request has a fixed length of 3 bytes 4 for each of the u32s which are index, begin, and length. and one for the message index.
                message.extend_from_slice(&3u32.to_be_bytes());
                message.push(MSG_PORT);
                message.extend_from_slice(&port.to_be_bytes());
            }
            MessageType::KeepAlive => message.extend_from_slice(&0u32.to_be_bytes()),
        }

        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn round_trip(original: MessageType, expected_bytes: &[u8]) {
        let actual_bytes = original.to_bytes();

        assert_eq!(actual_bytes, expected_bytes, "Serialized bytes don't match");

        // Peek len and id.
        let len = u32::from_be_bytes([
            actual_bytes[0],
            actual_bytes[1],
            actual_bytes[2],
            actual_bytes[3],
        ]);
        let id = if len > 0 { actual_bytes[4] } else { 0 };

        let mut bytes = BytesMut::from(&actual_bytes[..]);
        let parsed = MessageType::from_bytes(&mut bytes, id, len).unwrap();

        assert_eq!(original, parsed, "Round-trip MessageType does not match");
    }

    #[test]
    fn test_choke_round_trip() {
        round_trip(MessageType::Choke, &vec![0, 0, 0, 1, 0]);
    }

    #[test]
    fn test_unchoke_round_trip() {
        round_trip(MessageType::Unchoke, &vec![0, 0, 0, 1, 1]);
    }

    #[test]
    fn test_interested_round_trip() {
        round_trip(MessageType::Interested, &vec![0, 0, 0, 1, 2]);
    }

    #[test]
    fn test_not_interested_round_trip() {
        round_trip(MessageType::NotInterested, &vec![0, 0, 0, 1, 3]);
    }

    #[test]
    fn test_have_round_trip() {
        round_trip(MessageType::Have(42), &{
            let mut v = vec![0, 0, 0, 5, 4];
            v.extend_from_slice(&42u32.to_be_bytes());
            v
        });
    }

    #[test]
    fn test_bitfield_round_trip() {
        round_trip(MessageType::Bitfield(vec![0b10101010, 0b11110000]), &{
            let mut v = vec![0, 0, 0, 3, 5];
            v.extend_from_slice(&[0b10101010, 0b11110000]);
            v
        });
    }

    #[test]
    fn test_request_round_trip() {
        round_trip(
            MessageType::Request {
                index: 1,
                begin: 2,
                length: 3,
            },
            &{
                let mut v = vec![0, 0, 0, 13, 6];
                v.extend_from_slice(&1u32.to_be_bytes());
                v.extend_from_slice(&2u32.to_be_bytes());
                v.extend_from_slice(&3u32.to_be_bytes());
                v
            },
        );
    }

    #[test]
    fn test_piece_round_trip() {
        round_trip(
            MessageType::Piece {
                index: 42,
                begin: 0,
                block: vec![1, 2, 3, 4, 5],
            },
            &{
                let mut v = vec![0, 0, 0, 14, 7];
                v.extend_from_slice(&42u32.to_be_bytes());
                v.extend_from_slice(&0u32.to_be_bytes());
                v.extend_from_slice(&[1, 2, 3, 4, 5]);
                v
            },
        );
    }

    #[test]
    fn test_cancel_round_trip() {
        round_trip(
            MessageType::Cancel {
                index: 1,
                begin: 2,
                length: 3,
            },
            &{
                let mut v = vec![0, 0, 0, 13, 8];
                v.extend_from_slice(&1u32.to_be_bytes());
                v.extend_from_slice(&2u32.to_be_bytes());
                v.extend_from_slice(&3u32.to_be_bytes());
                v
            },
        );
    }

    #[test]
    fn test_port_round_trip() {
        round_trip(MessageType::Port(6881), &{
            let mut v = vec![0, 0, 0, 3, 9];
            v.extend_from_slice(&6881u16.to_be_bytes());
            v
        });
    }

    #[test]
    fn test_keep_alive_round_trip() {
        round_trip(MessageType::KeepAlive, &vec![0, 0, 0, 0]);
    }
}
