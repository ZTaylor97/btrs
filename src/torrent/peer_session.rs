use std::{collections::VecDeque, sync::Arc, time::Duration};

use anyhow::bail;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

mod message;
mod work;

use message::MessageType;
use work::{BlockInfo, BlockResponse, BlockStatus, PieceWork};

use crate::torrent::piece_manager::{PieceError, PieceRequest, PieceResponse};

const PSTR: &[u8; 19] = b"BitTorrent protocol";

pub struct PeerSession {
    peer_id: [u8; 20],
    info_hash: [u8; 20],
    url: String,
    peer_state: Arc<Mutex<PeerState>>,
}

#[derive(Clone, Debug)]
pub struct PeerState {
    pub is_choked: bool,
    pub is_choking: bool,
    pub is_peer_interested: bool,
    pub is_interested: bool,
    pub bitfield: Vec<u8>,
}

impl PeerState {
    pub fn has_piece(&self, piece_index: usize) -> bool {
        let bit_offset = 7 - (piece_index % 8); // assume Big Endian bytes
        let byte_offset = piece_index / 8;

        let byte = self.bitfield[byte_offset];

        byte & (1 << bit_offset) != 0
    }
}

impl PeerSession {
    pub async fn new(
        url: &str,
        peer_id: [u8; 20],
        info_hash: [u8; 20],
    ) -> Result<PeerSession, anyhow::Error> {
        let peer_state = PeerState {
            is_choked: true,
            is_choking: true,
            is_peer_interested: false,
            is_interested: false,
            bitfield: vec![],
        };

        Ok(PeerSession {
            peer_id,
            info_hash,
            url: String::from(url),
            peer_state: Arc::new(Mutex::new(peer_state)),
        })
    }

    pub async fn send_handshake(
        writer: &mut OwnedWriteHalf,
        info_hash: &[u8; 20],
        peer_id: &[u8; 20],
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

    pub async fn start(
        &mut self,
        piece_request_rx: Arc<Mutex<VecDeque<PieceRequest>>>,
        piece_request_tx: Sender<PieceResponse>,
    ) -> Result<(), anyhow::Error> {
        let (block_tx, block_rx) = channel::<BlockResponse>(100);

        let stream = TcpStream::connect(&self.url).await?;
        let (mut reader, mut writer) = stream.into_split();

        PeerSession::send_handshake(&mut writer, &self.info_hash, &self.peer_id).await?;
        let handshake_response = PeerSession::read_handshake(&mut reader).await?;
        let resp = &handshake_response[28..48];

        if resp != self.info_hash {
            drop(reader);
            drop(writer);
            bail!(
                "Dropping connection to peer, info_hash invalid {resp:?}:{:?}",
                self.info_hash
            );
        }

        // Communicate intention to download from peer synchronously before starting upload/download.
        PeerSession::send_interested(&mut writer).await?;
        PeerSession::send_unchoke(&mut writer).await?;

        // Start receiving messages from the peer.
        let reader = Arc::new(Mutex::new(reader));
        let state_ref = self.peer_state.clone();
        let reader_handle =
            tokio::spawn(
                async move { PeerSession::peer_listener(state_ref, reader, block_tx).await },
            );

        // Start sending messages to the peer
        let state_ref = self.peer_state.clone();
        let piece_queue = piece_request_rx.clone();
        let piece_tx = piece_request_tx.clone();
        let writer = Arc::new(Mutex::new(writer));
        let writer_handle = tokio::spawn(async move {
            PeerSession::peer_requester(state_ref, piece_queue, piece_tx, writer, block_rx).await
        });

        Ok(())
    }

    async fn peer_requester(
        peer_state: Arc<Mutex<PeerState>>,
        piece_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
        piece_tx: Sender<PieceResponse>,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        mut block_rx: Receiver<BlockResponse>,
    ) -> Result<(), anyhow::Error> {
        let mut piece_work: Option<PieceWork> = None;
        let max_in_flight = 5;
        loop {
            // Clone latest peer state then unlock mutex, state information doesn't have to be realtime.
            let state = { peer_state.lock().await.clone() };

            // Fetch next piece to download from queue if not currently working on one.
            if piece_work.is_none() {
                let mut piece_request_queue = piece_queue.lock().await;
                let new_piece = piece_request_queue.pop_front();

                if let Some(piece_req) = new_piece {
                    if state.has_piece(piece_req.piece_index as usize) {
                        piece_work = Some(piece_req.into());
                    } else {
                        // Inform piece manager that piece is not available on this peer.
                        piece_tx
                            .send(PieceResponse {
                                piece_index: piece_req.piece_index,
                                result: Err(PieceError::PieceUnavailable),
                            })
                            .await?;
                    }
                }
            }

            // Do work if there is work to do
            if let Some(mut work) = piece_work.take() {
                // Send piece to piece manager if it is complete
                if work.is_complete() {
                    if let Err(e) = piece_tx.send(work.to_piece_response()).await {
                        eprintln!("ERROR: Failed to send piece to PieceManager: {e}")
                    }
                    continue;
                }

                // First consume all blocks from peer reader task channel if there are any.
                while let Ok(block_response) = block_rx.try_recv() {
                    let offset = block_response.begin;

                    let block = work.blocks.iter_mut().find(|block| {
                        block.offset == offset && block.status == BlockStatus::InProgress
                    });

                    if let Some(block) = block {
                        block.data = block_response.block;
                        block.status = BlockStatus::Full;
                    } else {
                        eprintln!(
                            "WARNING: Received block response from peer that did not match expected block offset."
                        );
                    }
                }

                // Only send requests if not choked.

                if !state.is_choked {
                    // Get next 5 blocks (if there are 5 to get) and make requests to peer
                    let next_blocks: Vec<&mut BlockInfo> = work
                        .blocks
                        .iter_mut()
                        .filter(|block| block.status == BlockStatus::Empty)
                        .take(max_in_flight)
                        .map(|block| {
                            block.status = BlockStatus::InProgress;
                            block
                        })
                        .collect();

                    let mut writer = writer.lock().await;
                    let resp =
                        PeerSession::send_request(&mut writer, work.index, &next_blocks).await;

                    if let Err(e) = resp {
                        eprintln!("{e}");
                    }
                }

                // Give ownership back if work not complete yet
                piece_work = Some(work);
            }

            // Give other tasks some time to execute if there is no work
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn peer_listener(
        peer_state: Arc<Mutex<PeerState>>,
        reader: Arc<Mutex<OwnedReadHalf>>,
        block_tx: Sender<BlockResponse>,
    ) -> Result<(), anyhow::Error> {
        loop {
            let msg = {
                let mut reader = reader.lock().await;
                PeerSession::read_message(&mut reader).await.unwrap()
            };
            {
                let mut state = peer_state.lock().await;
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
                        // TODO: Handle errors correctly
                        // send to block manager task
                        block_tx.try_send(BlockResponse {
                            index,
                            begin,
                            block,
                        })?;
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
        piece_index: u32,
        blocks: &[&mut BlockInfo],
    ) -> Result<(), anyhow::Error> {
        let bytes: Vec<u8> = blocks
            .iter()
            .flat_map(|block| {
                MessageType::Request {
                    index: piece_index,
                    begin: block.offset,
                    length: block.length,
                }
                .to_bytes()
            })
            .collect();

        writer.writable().await?;

        writer.write_all(&bytes).await?;

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

            // Write a valid BitTorrent handshake request
            let mut request = Vec::new();
            request.push(19u8);
            request.extend_from_slice(b"BitTorrent protocol");
            request.extend_from_slice(&[0u8; 8]);
            request.extend_from_slice(&MOCK_INFO_HASH);
            request.extend_from_slice(&MOCK_PEER_ID);

            socket.write_all(&request).await.unwrap();
        });
    }

    #[tokio::test]
    pub async fn test_handshake() {
        let port = 6888;

        start_mock_peer_server(port).await;

        let peer_session =
            PeerSession::new(&format!("127.0.0.1:{port}"), MOCK_CLIENT_ID, MOCK_INFO_HASH)
                .await
                .unwrap();

        let stream = TcpStream::connect(&peer_session.url).await.unwrap();
        let (mut reader, mut writer) = stream.into_split();

        PeerSession::send_handshake(&mut writer, &peer_session.info_hash, &peer_session.peer_id)
            .await
            .unwrap();
        PeerSession::read_handshake(&mut reader).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    pub async fn test_download() {
        // This is the info hash for the A_Little_Princess torrent from the Internet_Archive
        let info_hash = [
            0x57, 0x96, 0xd3, 0x3f, 0xda, 0x21, 0x68, 0x48, 0x68, 0x28, 0x67, 0x8f, 0x75, 0x40,
            0xf1, 0xaf, 0x72, 0xdb, 0x4a, 0x37,
        ];
        // Basic information pulled from the metainfo file.
        let num_pieces: u32 = 2021;
        let piece_length: u32 = 2048 * 1024;

        let port = 6137;

        let piece_request_rx = Arc::new(Mutex::new(VecDeque::new()));
        let (piece_request_tx, mut piece_requester_rx) = channel::<PieceResponse>(100);

        // Connect to another client hosting the torrent locally for testing.
        let mut peer_session =
            PeerSession::new(&format!("127.0.0.1:{port}"), MOCK_CLIENT_ID, info_hash)
                .await
                .unwrap();

        peer_session
            .start(piece_request_rx.clone(), piece_request_tx)
            .await
            .unwrap();

        // Mimic PieceManager
        for i in 0..num_pieces {
            let mut queue = piece_request_rx.lock().await;

            queue.push_back(PieceRequest {
                piece_index: i,
                length_bytes: piece_length as usize,
            });
        }

        loop {
            let piece_response = piece_requester_rx.recv().await;
            if let Some(resp) = piece_response {
                println!("{:?}", resp.result.unwrap().len());
            }
        }
    }
}
