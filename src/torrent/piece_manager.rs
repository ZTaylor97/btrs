use tokio::sync::mpsc;

pub struct PieceManager {
    txs: Vec<mpsc::Sender<PieceRequest>>,
    results: mpsc::Receiver<PieceResult>,
}

impl PieceManager {
    pub async fn run(&mut self) {
        // Distribute work
        for tx in &self.txs {
            let request = PieceRequest { piece_index: 0 };
            let _ = tx.send(request).await;
        }

        // Receive completed pieces
        while let Some(result) = self.results.recv().await {
            println!("Got piece: {:?}", result.piece_index);
        }
    }
}

pub struct PieceRequest {
    pub piece_index: u32,
    // maybe block range etc.
}

pub struct PieceResult {
    pub piece_index: u32,
    pub data: Vec<u8>,
}
