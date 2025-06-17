use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{
        Block, Borders, Cell, List, ListItem, ListState, Row, Scrollbar, ScrollbarState, Table,
        TableState,
    },
};

use crate::torrent::{
    Peer,
    files::{FileEntry, FileKind},
};

pub struct TorrentDetails {
    pub selected: usize,
}

impl TorrentDetails {
    pub fn render_peers(&mut self, f: &mut Frame, area: Rect, peers: &[Peer], active: bool) {
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
        if active {
            self.selected = usize::clamp(self.selected, 0, peers.len());
            scroll_state = scroll_state.position(self.selected);
            table_state.select(Some(self.selected));
        }

        f.render_stateful_widget(table, area, &mut table_state);
        f.render_stateful_widget(peer_scrollbar, area, &mut scroll_state);
    }

    pub fn render_files(&mut self, f: &mut Frame, area: Rect, files: &FileEntry, active: bool) {
        let mut flat = Vec::new();
        flatten_all(files, 0, &mut flat);

        let items: Vec<ListItem> = flat
            .iter()
            .map(|(depth, entry)| {
                let indent = "  ".repeat(*depth);
                let prefix = match entry.kind {
                    FileKind::Directory { .. } => "📁 ",
                    FileKind::File => "📄 ",
                };
                ListItem::new(Span::raw(format!("{}{}{}", indent, prefix, entry.name)))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(self.selected));

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Files")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(Style::default().fg(Color::LightBlue));

        f.render_stateful_widget(list, area, &mut state);
    }
}

fn flatten_file_entries<'a>(
    entry: &'a FileEntry,
    depth: usize,
    output: &mut Vec<(usize, &'a FileEntry)>,
) {
    output.push((depth, entry));

    if let FileKind::Directory { children } = &entry.kind {
        for child in children {
            flatten_file_entries(child, depth + 1, output);
        }
    }
}

fn flatten_all<'a>(entry: &'a FileEntry, depth: usize, out: &mut Vec<(usize, &'a FileEntry)>) {
    out.push((depth, entry));
    if let FileKind::Directory { children } = &entry.kind {
        for child in children {
            flatten_all(child, depth + 1, out);
        }
    }
}
