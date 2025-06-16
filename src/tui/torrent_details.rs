use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarState, Table, TableState},
};

use crate::torrent::Peer;

pub struct TorrentDetails {
    pub selected: usize,
    pub active: bool,
}

impl TorrentDetails {
    pub fn render_peers(&mut self, f: &mut Frame, area: Rect, peers: &[Peer]) {
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

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title("[P]eers")
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED),
        );

        let mut scroll_state = ScrollbarState::default().content_length(peers.len());

        let mut table_state = TableState::default();

        let peer_scrollbar = Scrollbar::default();
        if self.active {
            self.selected = usize::clamp(self.selected, 0, peers.len());
            scroll_state = scroll_state.position(self.selected);
            table_state.select(Some(self.selected));
        }

        f.render_stateful_widget(table, area, &mut table_state);
        f.render_stateful_widget(peer_scrollbar, area, &mut scroll_state);
    }
}
