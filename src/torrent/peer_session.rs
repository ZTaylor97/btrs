use tokio::net::{TcpListener, TcpStream};

mod message;

const PSTR: &[u8; 19] = b"BitTorrent protocol";

pub struct PeerSession {
    stream: TcpStream,
    peer_id: String,
    info_hash: String,
    url: String,
    is_choked: bool,
    is_choking: bool,
    is_peer_interested: bool,
    is_interested: bool,
}

impl PeerSession {
    pub async fn new(url: &str, peer_id: &str, info_hash: &str) -> Result<Self, anyhow::Error> {
        let stream = TcpStream::connect(url).await?;

        Ok(Self {
            stream,
            peer_id: String::from(peer_id),
            info_hash: String::from(info_hash),
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
        request_bytes.extend_from_slice(&self.info_hash.as_bytes());
        request_bytes.extend_from_slice(&self.peer_id.as_bytes());

        self.stream.writable().await?;
        self.stream.try_write(&request_bytes)?;

        let mut response_bytes = [0u8; 68];
        self.stream.readable().await?;
        let count = self.stream.try_read(&mut response_bytes)?;
        println!("got {count} bytes");
        Ok(response_bytes)
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
            println!("[Mock] Sent handshake request");
        });
    }

    #[tokio::test]
    pub async fn test1() {
        let port = 6888;

        start_mock_peer_server(port).await;

        let mut peer_session = PeerSession::new(
            &format!("127.0.0.1:{port}"),
            "-TEST0-1234567890123",
            str::from_utf8(&MOCK_INFO_HASH).unwrap(),
        )
        .await
        .unwrap();

        let bytes = peer_session.handshake().await.unwrap();

        println!("{}", str::from_utf8(&bytes).unwrap());
    }
}
