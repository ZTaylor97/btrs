use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::{Line, Text},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph},
};

use crate::{
    app::{App, ui_models::TorrentItem},
    torrent::Torrent,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled("BTRS", Style::default().fg(Color::Green)))
        .centered()
        .block(title_block);

    frame.render_widget(title, chunks[0]);

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
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_widget(list, chunks[1]);
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
