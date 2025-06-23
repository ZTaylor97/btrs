use std::sync::Arc;

use anyhow::bail;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        TcpListener, TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
};

mod message;

use message::MessageType;

const PSTR: &[u8; 19] = b"BitTorrent protocol";

pub struct PeerSession {
    peer_id: [u8; 20],
    info_hash: [u8; 20],
    url: String,
    peer_state: Arc<Mutex<PeerState>>,
    piece_request_rx: Receiver<PieceRequest>,
    piece_request_tx: Sender<PieceResponse>,
}

pub struct Piece

pub struct PeerState {
    pub is_choked: bool,
    pub is_choking: bool,
    pub is_peer_interested: bool,
    pub is_interested: bool,
    pub bitfield: Vec<u8>,
}

impl PeerSession {
    pub async fn new(
        url: &str,
        peer_id: [u8; 20],
        info_hash: [u8; 20],
    ) -> Result<Self, anyhow::Error> {
        let peer_state = PeerState {
            is_choked: true,
            is_choking: true,
            is_peer_interested: false,
            is_interested: false,
            bitfield: vec![],
        };

        Ok(Self {
            peer_id,
            info_hash,
            url: String::from(url),
            peer_state: Arc::new(Mutex::new(peer_state)),
        })
    }

    pub async fn send_handshake(
        info_hash: &[u8; 20],
        peer_id: &[u8; 20],
        writer: &mut OwnedWriteHalf,
    ) -> Result<(), anyhow::Error> {
        let mut request_bytes: Vec<u8> = Vec::new();
        request_bytes.push(19u8);
        request_bytes.extend_from_slice(PSTR);
        request_bytes.extend_from_slice(&[0u8; 8]); // Reserved bytes
        request_bytes.extend_from_slice(info_hash);
        request_bytes.extend_from_slice(peer_id);

        writer.writable().await?;
        writer.write_all(&request_bytes).await?;

        Ok(())
    }

    pub async fn read_handshake(reader: &mut OwnedReadHalf) -> Result<[u8; 68], anyhow::Error> {
        let mut response_bytes = [0u8; 68];
        reader.readable().await?;
        reader.read_exact(&mut response_bytes).await?;

        Ok(response_bytes)
    }

    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        let stream = TcpStream::connect(&self.url).await?;
        let (mut reader, mut writer) = stream.into_split();

        Self::send_handshake(&self.info_hash, &self.peer_id, &mut writer).await?;
        let handshake_response = Self::read_handshake(&mut reader).await?;
        let resp = &handshake_response[28..48];

        if resp != self.info_hash {
            drop(reader);
            drop(writer);
            bail!(
                "Dropping connection to peer, info_hash invalid {resp:?}:{:?}",
                self.info_hash
            );
        }

        Self::send_interested(&mut writer).await?;
        Self::send_unchoke(&mut writer).await?;

        // Create locks for reader and writer streams
        let reader = Arc::new(tokio::sync::Mutex::new(reader));
        let writer = Arc::new(tokio::sync::Mutex::new(writer));

        let state_ref = self.peer_state.clone();

        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut reader = reader.lock().await;
                    Self::read_message(&mut reader).await.unwrap()
                };
                {
                    let mut state = state_ref.lock().await;
                    match msg {
                        MessageType::Choke => state.is_choked = true,
                        MessageType::Unchoke => state.is_choked = false,
                        MessageType::Interested => state.is_peer_interested = true,
                        MessageType::NotInterested => state.is_peer_interested = false,
                        MessageType::Have(piece_id) => println!("Peer has {piece_id}"),
                        MessageType::Bitfield(items) => state.bitfield = items,
                        MessageType::Request {
                            index,
                            begin,
                            length,
                        } => println!("Sorry buddy, but no"),
                        MessageType::Piece {
                            index,
                            begin,
                            block,
                        } => {
                            println!(
                                "Received block at index {index}, offset {begin} and length {}",
                                block.len()
                            );
                        }
                        MessageType::Cancel {
                            index,
                            begin,
                            length,
                        } => {
                            println!(
                                "Cancelled block at index {index}, offset {begin} and length {length}"
                            )
                        }
                        MessageType::Port(port) => println!("Port request {port}"),
                        MessageType::KeepAlive => println!("Received keep alive!"),
                    }
                }
            }
        });

        // Requester task
        tokio::spawn(async move {
            // let current_piece = None
            // let request_queue = Queue(size = 5)
            loop {
                // Note: Need some way to signal current_piece has had all blocks requested
                // Note: What happens if requests to peer get lost or something.
                // if let Some(piece) = current_piece {
                //    if request_queue.is_full() or (time_since_last_request > 5 seconds and not request_queue.is_empty()) {
                //     
                //     Self::write_messages(&mut writer, request_queue.get_all_work())
                //   
                //     
                //   
                //   
                //    } else {
                //      work = current_piece.get_next_block()
                //      if let Ok(work) = work { request_queue.add_work(work) }
                //    }
                //} else {
                //  current_piece = PieceManagerQueue.GiveMeWork(timeout = 100ms)
                //}
                // if current_piece.is_complete() {
                //  current_piece = None
                //}
            }
        });

        Ok(())
    }

    pub async fn read_message(reader: &mut OwnedReadHalf) -> Result<MessageType, anyhow::Error> {
        reader.readable().await?;

        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf);

        let total_len = 4 + msg_len as usize;
        let mut msg_buf = BytesMut::with_capacity(total_len);
        msg_buf.extend_from_slice(&len_buf);

        msg_buf.resize(total_len, 0);
        reader.read_exact(&mut msg_buf[4..]).await?;

        let id = if msg_len > 0 { msg_buf[4] } else { 0 };

        MessageType::from_bytes(&mut msg_buf, id, msg_len)
    }

    pub async fn send_interested(writer: &mut OwnedWriteHalf) -> Result<(), anyhow::Error> {
        let interested_bytes = MessageType::Interested.to_bytes();

        writer.writable().await?;
        writer.write_all(&interested_bytes).await?;

        Ok(())
    }
    pub async fn send_unchoke(writer: &mut OwnedWriteHalf) -> Result<(), anyhow::Error> {
        let interested_bytes = MessageType::Unchoke.to_bytes();

        writer.writable().await?;
        writer.write_all(&interested_bytes).await?;

        Ok(())
    }

    pub async fn send_request(
        writer: &mut OwnedWriteHalf,
        index: u32,
        begin: u32,
        length: u32,
    ) -> Result<(), anyhow::Error> {
        let request_bytes = MessageType::Request {
            index,
            begin,
            length,
        }
        .to_bytes();

        writer.writable().await?;

        writer.write_all(&request_bytes).await?;

        Ok(())
    }
}

#[cfg(test)]
mod peer_session_tests {
    use super::*;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::task;

    const MOCK_INFO_HASH: [u8; 20] = *b"12345678901234567890";
    const MOCK_PEER_ID: [u8; 20] = *b"-MOCK0-1234567890123";
    const MOCK_CLIENT_ID: [u8; 20] = *b"-TEST0-1234567890123";

    /// Start a mock peer server that responds with a handshake
    async fn start_mock_peer_server(port: u16) {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr).await.unwrap();

        task::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();

            // Read incoming handshake (should be 68 bytes)
            let mut handshake = [0u8; 68];
            let count = socket.read(&mut handshake).await.unwrap();
            println!("test sent {count} bytes");
            println!("[Mock] Received handshake: {:?}", &handshake);

            // Write a valid BitTorrent handshake request
            let mut request = Vec::new();
            request.push(19u8);
            request.extend_from_slice(b"BitTorrent protocol");
            request.extend_from_slice(&[0u8; 8]);
            request.extend_from_slice(&MOCK_INFO_HASH);
            request.extend_from_slice(&MOCK_PEER_ID);

            socket.write_all(&request).await.unwrap();
            // println!("[Mock] Sent handshake request");
        });
    }

    #[tokio::test]
    pub async fn test1() {
        let port = 6888;

        start_mock_peer_server(port).await;

        let peer_session =
            PeerSession::new(&format!("127.0.0.1:{port}"), MOCK_CLIENT_ID, MOCK_INFO_HASH)
                .await
                .unwrap();

        let stream = TcpStream::connect(&peer_session.url).await.unwrap();
        let (mut reader, mut writer) = stream.into_split();

        PeerSession::send_handshake(&peer_session.info_hash, &peer_session.peer_id, &mut writer)
            .await
            .unwrap();
        PeerSession::read_handshake(&mut reader).await.unwrap();
    }

    #[tokio::test]
    pub async fn real_test() {
        let info_hash = [
            0x57, 0x96, 0xd3, 0x3f, 0xda, 0x21, 0x68, 0x48, 0x68, 0x28, 0x67, 0x8f, 0x75, 0x40,
            0xf1, 0xaf, 0x72, 0xdb, 0x4a, 0x37,
        ];

        let port = 6137;

        // Connect to another client hosting the torrent locally for testing.
        let mut peer_session =
            PeerSession::new(&format!("127.0.0.1:{port}"), MOCK_CLIENT_ID, info_hash)
                .await
                .unwrap();

        let num_pieces: u32 = 2021;
        let piece_length: u32 = 2048 * 1024;

        peer_session.start().await.unwrap();
    }
}
