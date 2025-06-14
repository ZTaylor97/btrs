use std::{
    collections::BTreeMap,
    fs,
};

use anyhow::{Error, anyhow};
use rand::{Rng, distr::Alphanumeric};
use ratatui::crossterm::event::{self, KeyEvent};
use urlencoding::encode_binary;

use crate::{app::ui_models::TorrentItem, torrent::Torrent};

pub mod ui_models;

pub enum CurrentScreen {
    Main,
}

pub struct App {
    torrents: BTreeMap<String, Torrent>,
    pub peer_id: String,
    pub should_exit: bool,
    pub selected: usize, // selected torrent from sorted list
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
            should_exit: false,
            selected: 0,
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

        self.torrents
            .insert(torrent.get_info_hash().into(), torrent);

        Ok(())
    }

    pub fn tick(&mut self) {}

    pub async fn handle_key(&mut self, key_event: KeyEvent) -> Result<(), Error> {
        match key_event.code {
            event::KeyCode::Esc | event::KeyCode::Char('q') => self.should_exit = true,
            event::KeyCode::Up | event::KeyCode::Char('j') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            event::KeyCode::Down | event::KeyCode::Char('k') => {
                if self.selected + 1 < self.torrents.len() {
                    self.selected += 1;
                }
            }
            event::KeyCode::Enter => self.download_torrent().await?,
            _ => (),
        }

        Ok(())
    }

    pub async fn download_torrent(&mut self) -> Result<(), Error> {
        let key = self
            .torrents
            .iter()
            .nth(self.selected)
            .ok_or(anyhow!("Failed getting nth element"))?
            .0
            .clone();

        self.torrents
            .get_mut(&key).ok_or( anyhow!("Element not found"))?
            .download(&self.peer_id)
            .await?;

        Ok(())
    }

    pub fn torrent_items(&self) -> Vec<TorrentItem> {
        // By default sorted based on key, which is info hash
        self.torrents
            .iter()
            .map(|(_k, v)| TorrentItem::from(v))
            .collect()
    }
}
