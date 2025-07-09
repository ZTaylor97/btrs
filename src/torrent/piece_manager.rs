use std::{collections::VecDeque, sync::Arc};

use tokio::sync::{Mutex, mpsc::Receiver};

pub struct PieceManager {
    work_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
    results: Receiver<PieceResponse>,
    piece_metadata: Vec<PieceMetadata>,
}

pub struct PieceMetadata {
    pub index: u32,
    pub hash: [u8; 20],
    pub length: usize,
    pub offset: usize,
}

impl PieceManager {
    pub fn new(
        work_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
        results: Receiver<PieceResponse>,
    ) -> Self {
        Self {
            work_queue,
            results,
            piece_metadata: vec![],
        }
    }

    pub async fn run(&mut self) {
        // Receive completed pieces
        while let Some(result) = self.results.recv().await {
            println!("Got piece: {:?}", result.piece_index);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PieceRequest {
    pub piece_index: u32,
    pub length_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct PieceResponse {
    pub piece_index: u32,
    pub result: Result<Vec<u8>, PieceError>,
}
#[derive(Debug, Clone)]
pub enum PieceError {
    Timeout,
    InvalidData(String),
    PeerChoked,
    ConnectionLost,
    PieceUnavailable,
}
