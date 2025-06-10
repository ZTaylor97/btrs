use std::fs;

use anyhow::Error;
use rand::{Rng, distr::Alphanumeric};
use urlencoding::encode_binary;

use crate::torrent::Torrent;

pub mod ui_models;

pub enum CurrentScreen {
    Main,
}

pub struct App {
    pub torrents: Vec<Torrent>,
    pub peer_id: String,
}

impl App {
    pub fn new() -> Self {
        let prefix = b"-RS0001-";
        let mut peer_id_bytes = [0u8; 20];

        peer_id_bytes[..8].copy_from_slice(prefix);

        let rand_part: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        peer_id_bytes[8..].copy_from_slice(rand_part.as_bytes());

        let peer_id = encode_binary(&peer_id_bytes).into_owned();

        let mut app = Self {
            torrents: vec![],
            peer_id,
        };

        app.add_torrent("test_files/A_Little_Princess_WB39_WOC_2001-07_archive.torrent")
            .unwrap();

        app.add_torrent("test_files/ubuntu-24.04.2-desktop-amd64.iso.torrent")
            .unwrap();
        app
    }

    pub fn add_torrent(&mut self, file_path: &str) -> Result<(), Error> {
        let bytes: Vec<u8> = fs::read(file_path).expect("{file_path} not found.");

        let torrent = Torrent::load(&bytes)?;

        self.torrents.push(torrent);

        Ok(())
    }

    pub fn tick(&mut self) {}

    pub async fn download_torrents(&self) -> Result<(), Error> {
        for torrent in &self.torrents {
            torrent.download(&self.peer_id).await?;
        }

        Ok(())
    }
}
