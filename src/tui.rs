use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::{Line, Text},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, ui_models::TorrentItem};

struct TorrentList {
    list_state: ListState,
    torrent_items: Vec<TorrentItem>,
}

pub fn draw(frame: &mut Frame, app: &App) {
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
        .constraints([Constraint::Percentage(40), Constraint::Percentage((60))])
        .split(vertical_chunks[1]);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled("BTRS", Style::default().fg(Color::Green)))
        .centered()
        .block(title_block);

    frame.render_widget(title, vertical_chunks[0]);

    let torrent_items: Vec<ListItem> = app
        .torrents
        .iter()
        .map(TorrentItem::from)
        .map(|ti| ListItem::from(&ti))
        .collect();

    let list_block = Block::default()
        .title(Line::raw("Torrents"))
        .borders(Borders::TOP)
        .border_set(symbols::border::EMPTY);

    let list = List::new(torrent_items)
        .block(list_block)
        .highlight_symbol(" > ")
        .highlight_style(Style::default().fg(Color::Blue))
        .highlight_spacing(HighlightSpacing::Always);

    let mut list_state = ListState::default();

    list_state.select_first();

    frame.render_stateful_widget(list, middle_chunks[0], &mut list_state);
    frame.render_widget(Block::default().borders(Borders::ALL), middle_chunks[1]);
}

impl From<&TorrentItem> for ListItem<'_> {
    fn from(torrent_item: &TorrentItem) -> Self {
        let line = Line::from(format!(
            "{} - {} - {}",
            torrent_item.name, torrent_item.status, torrent_item.info_hash
        ));
        ListItem::new(line)
    }
}
