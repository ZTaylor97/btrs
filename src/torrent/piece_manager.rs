use std::{collections::VecDeque, sync::Arc};

use tokio::sync::{Mutex, mpsc::Receiver};

pub struct PieceManager {
    work_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
    results: Receiver<PieceResult>,
    piece_length: u64,
    num_pieces: u32,
}

impl PieceManager {
    pub fn new(
        work_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
        results: Receiver<PieceResult>,
        piece_length: u64,
        num_pieces: u32,
    ) -> Self {
        Self {
            work_queue,
            results,
            piece_length,
            num_pieces,
        }
    }

    pub async fn run(&mut self) {
        {
            let mut queue = self.work_queue.lock().await;
            queue.reserve(self.num_pieces as usize);
            queue.extend((0..self.num_pieces).map(|i| PieceRequest {
                piece_index: i,
                length_bytes: self.piece_length as usize,
            }));
        }

        // Receive completed pieces
        while let Some(result) = self.results.recv().await {
            // Add piece back to queue if peer session returns an error while working on that piece.

            // TODO: log error
            if let Err(_piece_error) = result.result {
                let mut queue = self.work_queue.lock().await;

                queue.push_front(PieceRequest {
                    piece_index: result.piece_index,
                    length_bytes: self.piece_length as usize,
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
pub struct PieceResult {
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
