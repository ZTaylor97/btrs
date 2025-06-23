use tokio::sync::mpsc;

pub struct PieceManager {
    txs: Vec<mpsc::Sender<PieceRequest>>,
    results: mpsc::Receiver<PieceResponse>,
}

impl PieceManager {
    pub async fn run(&mut self) {
        tokio::spawn(async move {
            for tx in &self.txs {
                let request = PieceRequest { piece_index: 0 };
                let _ = tx.send(request).await;
            }
        });

        // Receive completed pieces
        while let Some(result) = self.results.recv().await {
            println!("Got piece: {:?}", result.piece_index);
        }
    }
}

pub struct PieceRequest {
    pub piece_index: u32,
}

pub struct PieceResponse {
    pub piece_index: u32,
    pub result: Result<Vec<u8>, PieceError>,
}
#[derive(Debug)]
pub enum PieceError {
    Timeout,
    InvalidData(String),
    PeerChoked,
    ConnectionLost,
}
