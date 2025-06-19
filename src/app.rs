use std::{collections::BTreeMap, fs};

use anyhow::{Error, anyhow};
use rand::{Rng, distr::Alphanumeric};
use urlencoding::encode_binary;

use crate::{app::ui_models::TorrentItem, torrent::Torrent};

pub mod ui_models;

pub enum CurrentScreen {
    Main,
}

pub struct App {
    torrents: BTreeMap<String, Torrent>,
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
            torrents: BTreeMap::new(),
            peer_id,
        };

        // TODO remove once TUI implemented
        app.add_torrent("test_files/A_Little_Princess_WB39_WOC_2001-07_archive.torrent")
            .unwrap();
        app.add_torrent("test_files/ubuntu-24.04.2-desktop-amd64.iso.torrent")
            .unwrap();

        app
    }

    pub fn add_torrent(&mut self, file_path: &str) -> Result<(), Error> {
        let bytes: Vec<u8> = fs::read(file_path).expect("{file_path} not found.");

        let torrent = Torrent::load(&bytes)?;

        self.torrents.insert(torrent.info_hash().into(), torrent);

        Ok(())
    }

    pub fn tick(&mut self) {}

    pub async fn download_torrent(&mut self, selected: &str) -> Result<(), Error> {
        self.torrents
            .get_mut(selected)
            .ok_or(anyhow!("Element not found"))?
            .download(&self.peer_id)
            .await?;

        Ok(())
    }

    pub fn torrent_items(&self) -> Result<Vec<TorrentItem>, anyhow::Error> {
        // By default sorted based on key, which is info hash
        self.torrents
            .iter()
            .map(|(_k, v)| TorrentItem::try_from(v))
            .collect()
    }
}
