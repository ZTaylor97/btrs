use anyhow::bail;
use bytes::{Buf, BytesMut};

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

        Ok(match idx {
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => {
                let index = bytes.get_u32();
                Self::Have(index)
            }
            5 => {
                let bitfield = bytes[..(len as usize - 1)].to_vec();
                Self::Bitfield(bitfield)
            }
            6 => {
                let index = bytes.get_u32();
                let begin = bytes.get_u32();
                let length = bytes.get_u32();

                Self::Request {
                    index,
                    begin,
                    length,
                }
            }
            7 => {
                let index = bytes.get_u32();
                let begin = bytes.get_u32();
                let block = bytes[..(len as usize - 9)].to_vec();

                Self::Piece {
                    index,
                    begin,
                    block,
                }
            }
            8 => {
                let index = bytes.get_u32();
                let begin = bytes.get_u32();
                let length = bytes.get_u32();

                Self::Cancel {
                    index,
                    begin,
                    length,
                }
            }
            9 => {
                let port = bytes.get_u16();
                Self::Port(port)
            }
            _ => bail!("Invalid message id {id}"),
        })
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
                let len: u32 = items.len() as u32 + 1;
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
                let len: u32 = 9 + block.len() as u32;
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
