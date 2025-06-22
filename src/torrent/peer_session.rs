use anyhow::bail;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

mod message;

use message::MessageType;

const PSTR: &[u8; 19] = b"BitTorrent protocol";

pub struct PeerSession {
    stream: TcpStream,
    peer_id: [u8; 20],
    info_hash: [u8; 20],
    url: String,
    is_choked: bool,
    is_choking: bool,
    is_peer_interested: bool,
    is_interested: bool,
}

impl PeerSession {
    pub async fn new(
        url: &str,
        peer_id: [u8; 20],
        info_hash: [u8; 20],
    ) -> Result<Self, anyhow::Error> {
        let stream = TcpStream::connect(url).await?;

        Ok(Self {
            stream,
            peer_id,
            info_hash,
            url: String::from(url),
            is_choked: true,
            is_choking: true,
            is_peer_interested: false,
            is_interested: false,
        })
    }

    pub async fn handshake(&mut self) -> Result<[u8; 68], anyhow::Error> {
        // Write a valid BitTorrent handshake request
        let mut request_bytes: Vec<u8> = Vec::new();
        request_bytes.push(19u8); // pstrlen
        request_bytes.extend_from_slice(PSTR); // pstr
        request_bytes.extend_from_slice(&[0u8; 8]); // Reserved bytes
        request_bytes.extend_from_slice(&self.info_hash);
        request_bytes.extend_from_slice(&self.peer_id);

        self.stream.writable().await?;
        self.stream.write_all(&request_bytes).await?;

        let mut response_bytes = [0u8; 68];
        self.stream.readable().await?;
        self.stream.read_exact(&mut response_bytes).await?;

        Ok(response_bytes)
    }

    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        let handshake_response = self.handshake().await?;
        let resp = &handshake_response[28..48];

        if resp != self.info_hash {
            self.stream.shutdown().await?;
            bail!(
                "Dropping connection to peer, info_hash invalid {resp:?}:{:?}",
                self.info_hash
            );
        }

        self.send_interested().await?;
        self.send_unchoke().await?;

        let mut request_count = 0;
        let mut total_request_count = 0;

        loop {
            let msg = self.read_message().await?;

            if request_count < 5 {
                self.send_request(total_request_count, 0, 16 * 1024).await?;
                total_request_count += 1;
                request_count += 1;
            }

            match msg {
                MessageType::Choke => self.is_choked = true,
                MessageType::Unchoke => self.is_choked = false,
                MessageType::Interested => self.is_peer_interested = true,
                MessageType::NotInterested => self.is_peer_interested = false,
                MessageType::Have(piece_id) => println!("Peer has {piece_id}"),
                MessageType::Bitfield(items) => {
                    println!("Printing bitfield: {items:?}")
                }
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
                    request_count = 0;
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
                    println!("Cancelled block at index {index}, offset {begin} and length {length}")
                }
                MessageType::Port(port) => println!("Port request {port}"),
                MessageType::KeepAlive => println!("Received keep alive!"),
            }
        }

        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<MessageType, anyhow::Error> {
        self.stream.readable().await?;

        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf);

        let total_len = 4 + msg_len as usize;
        let mut msg_buf = BytesMut::with_capacity(total_len);
        msg_buf.extend_from_slice(&len_buf);

        msg_buf.resize(total_len, 0);
        self.stream.read_exact(&mut msg_buf[4..]).await?;

        let id = if msg_len > 0 { msg_buf[4] } else { 0 };

        MessageType::from_bytes(&mut msg_buf, id, msg_len)
    }

    pub async fn send_interested(&mut self) -> Result<(), anyhow::Error> {
        let interested_bytes = MessageType::Interested.to_bytes();

        self.stream.writable().await?;
        self.stream.write_all(&interested_bytes).await?;

        Ok(())
    }
    pub async fn send_unchoke(&mut self) -> Result<(), anyhow::Error> {
        let interested_bytes = MessageType::Unchoke.to_bytes();

        self.stream.writable().await?;
        self.stream.write_all(&interested_bytes).await?;

        Ok(())
    }

    pub async fn send_request(
        &mut self,
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

        self.stream.writable().await?;

        self.stream.write_all(&request_bytes).await?;

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
            // println!("test sent {count} bytes");
            // println!("[Mock] Received handshake: {:?}", &handshake);

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

        let mut peer_session =
            PeerSession::new(&format!("127.0.0.1:{port}"), MOCK_CLIENT_ID, MOCK_INFO_HASH)
                .await
                .unwrap();

        peer_session.handshake().await.unwrap();
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

        peer_session.start().await.unwrap();
    }
}
