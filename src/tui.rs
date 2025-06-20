use anyhow::Error;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tokio::sync::mpsc::Sender;

use crate::{
    AppEvent, AppEventType,
    app::ui_models::TorrentItem,
    tui::{torrent_details::TorrentDetails, torrents_table::TorrentsTable},
};

mod torrent_details;
mod torrents_table;

const INFO_TEXT: &str = "(Esc) quit | (⏎) toggle torrent start/stop | (↑) move up | (↓) move down";

pub struct Tui {
    torrents_table: TorrentsTable,
    torrent_details: TorrentDetails,
    focused_pane: FocusedPane,
    torrent_items: Vec<TorrentItem>,
    event_tx: Sender<AppEvent>,
}

pub enum NavDirection {
    Up,
    Right,
    Down,
    Left,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedPane {
    Left,
    Right,
}

impl Tui {
    pub fn new(event_tx: Sender<AppEvent>) -> Self {
        Self {
            torrents_table: TorrentsTable { selected: 0 },
            torrent_details: TorrentDetails {
                selected: 0,
                selected_tab: 0,
            },
            torrent_items: vec![],
            focused_pane: FocusedPane::Left,
            event_tx,
        }
    }
    pub fn draw(&mut self, frame: &mut Frame, torrent_items: &[TorrentItem]) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vertical_chunks[1]);

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let title = Paragraph::new(Text::styled("BTRS", Style::default().fg(Color::Green)))
            .centered()
            .block(title_block);

        frame.render_widget(title, vertical_chunks[0]);

        self.torrent_items = torrent_items.to_vec();

        self.torrents_table.render(
            frame,
            middle_chunks[0],
            &self.torrent_items,
            self.focused_pane == FocusedPane::Left,
        );

        self.torrent_details.render_tabs(
            frame,
            middle_chunks[1],
            &torrent_items[self.torrents_table.selected],
            self.focused_pane == FocusedPane::Right,
        );

        Self::render_footer(frame, vertical_chunks[2]);
    }

    fn render_footer(frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from(INFO_TEXT))
            .centered()
            .block(Block::bordered().border_type(BorderType::Double));

        frame.render_widget(info_footer, area);
    }

    pub fn navigate(&mut self, direction: NavDirection) {
        match direction {
            NavDirection::Up => match self.focused_pane {
                FocusedPane::Left => {
                    if self.torrent_items.len() > 0 {
                        self.torrents_table.selected =
                            self.torrents_table.selected.wrapping_sub(1) % self.torrent_items.len();
                    }
                }
                FocusedPane::Right => {
                    self.torrent_details.selected = self.torrent_details.selected.saturating_sub(1);
                }
            },
            NavDirection::Down => match self.focused_pane {
                FocusedPane::Left => {
                    if self.torrent_items.len() > 0 {
                        self.torrents_table.selected =
                            (self.torrents_table.selected + 1) % self.torrent_items.len();
                    }
                }
                FocusedPane::Right => self.torrent_details.selected += 1,
            },
            NavDirection::Right => self.focused_pane = FocusedPane::Right,
            NavDirection::Left => self.focused_pane = FocusedPane::Left,
        }
    }

    pub async fn handle_key(&mut self, key_event: KeyEvent) -> Result<(), Error> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                self.navigate(NavDirection::Up);
            }
            KeyCode::Down | KeyCode::Char('k') => {
                self.navigate(NavDirection::Down);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.navigate(NavDirection::Right);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.navigate(NavDirection::Left);
            }
            KeyCode::Enter => {
                let key = self.torrent_items[self.torrents_table.selected]
                    .info_hash
                    .clone();

                self.event_tx
                    .send(AppEvent::Custom(AppEventType::Download(key)))
                    .await?;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.event_tx
                    .send(AppEvent::Custom(AppEventType::Exit))
                    .await?
            }
            KeyCode::Char('P') => {
                self.focused_pane = FocusedPane::Right;
                self.torrent_details.selected_tab = 0;
            }
            KeyCode::Char('F') => {
                self.focused_pane = FocusedPane::Right;
                self.torrent_details.selected_tab = 1;
            }
            KeyCode::Char('T') => self.focused_pane = FocusedPane::Left,
            _ => (),
        }

        Ok(())
    }
}
