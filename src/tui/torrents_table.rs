use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, HighlightSpacing, Row, Table, TableState},
};

use crate::app::ui_models::TorrentItem; // adjust as needed

pub struct TorrentsTable {
    pub selected: usize,
    pub active: bool,
}

impl TorrentsTable {
    pub fn render(&self, f: &mut Frame, area: Rect, torrents: &[TorrentItem]) {
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
                    Cell::from(t.name.clone()),
                    Cell::from(t.status.clone()),
                    Cell::from(t.info_hash.clone()),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ];

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
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        let mut state = TableState::default();
        if self.active {
            state.select(Some(self.selected));
        }

        f.render_stateful_widget(table, area, &mut state);
    }
}
