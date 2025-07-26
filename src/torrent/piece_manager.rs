use std::{collections::VecDeque, sync::Arc};

use tokio::sync::{Mutex, mpsc::Receiver};

pub struct PieceManager {
    work_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
    results: Receiver<PieceResult>,
}

impl PieceManager {
    pub fn new(
        work_queue: Arc<Mutex<VecDeque<PieceRequest>>>,
        results: Receiver<PieceResult>,
    ) -> Self {
        Self {
            work_queue,
            results,
        }
    }

    pub async fn run(&mut self) {
        let num_pieces: u32 = 2021;
        let piece_length: u32 = 2048 * 1024;

        {
            let mut queue = self.work_queue.lock().await;
            queue.reserve(num_pieces as usize);
            queue.extend((0..num_pieces).map(|i| PieceRequest {
                piece_index: i,
                length_bytes: piece_length as usize,
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
