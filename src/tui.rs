use anyhow::Error;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState,
    },
};
use tokio::sync::mpsc::Sender;

use crate::{AppEvent, AppEventType, app::ui_models::TorrentItem, torrent::Peer};

const INFO_TEXT: &str = "(Esc) quit | (⏎) toggle torrent start/stop | (↑) move up | (↓) move down";

pub struct Tui {
    pub selected: usize,
    torrent_items: Vec<TorrentItem>,
    event_tx: Sender<AppEvent>,
}

pub enum NavDirection {
    Up,
    Right,
    Down,
    Left,
}

impl Tui {
    pub fn new(event_tx: Sender<AppEvent>) -> Self {
        Self {
            selected: 0,
            torrent_items: vec![],
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
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(vertical_chunks[1]);

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let title = Paragraph::new(Text::styled("BTRS", Style::default().fg(Color::Green)))
            .centered()
            .block(title_block);

        frame.render_widget(title, vertical_chunks[0]);

        self.torrent_items = torrent_items.to_vec();

        // TODO: TUI components in separate structs/module\s
        Self::render_torrents_table(frame, middle_chunks[0], &self.torrent_items, self.selected);
        Self::render_peers(
            frame,
            middle_chunks[1],
            &self.torrent_items[self.selected].peer_list,
        );
        Self::render_footer(frame, vertical_chunks[2]);

        frame.render_widget(Block::default().borders(Borders::ALL), middle_chunks[1]);
    }

    pub fn render_torrents_table(
        f: &mut Frame,
        area: Rect,
        torrents: &[TorrentItem],
        selected: usize,
    ) {
        let header = Row::new(vec![
            Cell::from("Name"),
            Cell::from("Status"),
            Cell::from("Info Hash"),
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        let rows: Vec<Row> = torrents
            .iter()
            .map(|t| {
                Row::new(vec![
                    Cell::from(Text::from(t.name.clone())),
                    Cell::from(Text::from(t.status.clone())),
                    Cell::from(Text::from(t.info_hash.clone())),
                ])
                .height(4)
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ];
        let bar = " █ ";

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title("[T]orrents")
                    .borders(Borders::ALL)
                    .border_set(symbols::border::ROUNDED),
            )
            .row_highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::LightBlue),
            )
            .column_highlight_style(Style::new().fg(Color::LightMagenta))
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .highlight_spacing(HighlightSpacing::Always);

        let mut state = TableState::default();
        state.select(Some(selected));

        f.render_stateful_widget(table, area, &mut state);
    }

    pub fn render_peers(f: &mut Frame, area: Rect, peers: &[Peer]) {
        let header = Row::new(vec![Cell::from("IP"), Cell::from("Port")]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        let rows: Vec<Row> = peers
            .iter()
            .map(|peer| {
                Row::new(vec![
                    Cell::from(peer.ip.clone()),
                    Cell::from(peer.port.to_string()),
                ])
            })
            .collect();

        let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().title("[P]eers").borders(Borders::ALL));

        f.render_widget(table, area);
    }

    fn render_footer(frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from(INFO_TEXT))
            .centered()
            .block(Block::bordered().border_type(BorderType::Double));

        frame.render_widget(info_footer, area);
    }

    pub fn navigate(&mut self, direction: NavDirection) {
        match direction {
            NavDirection::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            NavDirection::Down => {
                if self.selected + 1 < self.torrent_items.len() {
                    self.selected += 1;
                }
            }
            _ => {}
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
                let key = self.torrent_items[self.selected].info_hash.clone();

                self.event_tx
                    .send(AppEvent::Custom(AppEventType::Download(key)))
                    .await?;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.event_tx
                    .send(AppEvent::Custom(AppEventType::Exit))
                    .await?
            }
            KeyCode::Char('T') => {}
            _ => (),
        }

        Ok(())
    }
}
