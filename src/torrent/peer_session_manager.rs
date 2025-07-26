use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use tokio::{
    net::TcpStream,
    sync::{Mutex, mpsc::Sender},
    task::JoinHandle,
};

use crate::torrent::{
    Peer,
    peer_session::PeerSession,
    piece_manager::{PieceRequest, PieceResult},
    tracker::TrackerSession,
};

pub struct PeerSessionManager {
    pub(super) piece_result_sender: Sender<PieceResult>,
    pub(super) tracker_lock: Arc<Mutex<TrackerSession>>,
    pub(super) active_peers_lock: Arc<Mutex<BTreeMap<Peer, JoinHandle<()>>>>,
    pub(super) info_hash: [u8; 20],
    pub(super) work_queue_lock: Arc<Mutex<VecDeque<PieceRequest>>>,
}

impl PeerSessionManager {
    pub async fn manage_peer_sessions(&mut self) -> ! {
        // TODO: Move to configuration
        let max_peers = 10;
        let client_id = { self.tracker_lock.lock().await.peer_id.clone() };
        let client_id_raw = client_id
            .as_bytes()
            .try_into()
            .expect("Failed to convert client id to bytes");
        loop {
            let mut active_peers = self.active_peers_lock.lock().await;
            let known_peers = { self.tracker_lock.lock().await.peer_list.clone() };

            // Remove completed peer sessions
            let mut to_remove = vec![];
            for (peer, handle) in active_peers.iter() {
                if handle.is_finished() {
                    to_remove.push(peer.clone())
                }
            }
            for add in to_remove {
                active_peers.remove(&add);
            }

            // Only add new peers if we need to.
            if active_peers.len() < max_peers {
                for peer in known_peers {
                    if !active_peers.contains_key(&peer) {
                        let mut peer_session = PeerSession::new(
                            &format!("{}:{}", peer.ip, peer.port),
                            client_id_raw,
                            self.info_hash.clone(),
                        )
                        .await;

                        // TODO: Find better way to avoid connecting to self.
                        if peer.port == 6882 {
                            continue;
                        }

                        let queue = self.work_queue_lock.clone();
                        let piece_sender = self.piece_result_sender.clone();

                        let addr = if peer.ip.contains(":") {
                            &format!("[{}]:{}", peer.ip, peer.port)
                        } else {
                            &format!("{}:{}", peer.ip, peer.port)
                        };

                        match TcpStream::connect(addr).await {
                            Ok(tcp_stream) => {
                                let handle = tokio::spawn(async move {
                                    if let Err(e) =
                                        peer_session.start(tcp_stream, queue, piece_sender).await
                                    {
                                        eprintln!("Peer session failed unexpectedly: {e}")
                                    }
                                });

                                active_peers.insert(peer.clone(), handle);
                            }
                            Err(e) => {
                                eprintln!("Error making tcp connection to {addr}: {e}")
                            }
                        }
                    }
                }
            }

            // Start new peer sessions etc infrequently
            // TODO: Move sleep time to configuration
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}
