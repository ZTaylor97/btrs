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
        let num_pieces: u32 = 2021;
        let piece_length: u32 = 2048 * 1024;

        {
            let mut queue = self.work_queue.lock().await;

            for i in 0..num_pieces {
                queue.push_back(PieceRequest {
                    piece_index: i,
                    length_bytes: piece_length as usize,
                });
            }
        }

        // Receive completed pieces
        while let Some(result) = self.results.recv().await {
            // Add piece back to queue if peer session returns an error while working on that piece.
            if let Err(piece_error) = result.result {
                let mut queue = self.work_queue.lock().await;

                queue.push_front(PieceRequest {
                    piece_index: result.piece_index,
                    length_bytes: piece_length as usize,
                });
            }
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
