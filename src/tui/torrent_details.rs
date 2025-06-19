use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{
        Cell, List, ListItem, ListState, Row, Scrollbar, ScrollbarState, Table, TableState, Tabs,
    },
};

use crate::{
    app::ui_models::TorrentItem,
    torrent::{
        Peer,
        files::{FileEntry, FileKind},
    },
};

pub struct TorrentDetails {
    pub selected: usize,
    pub selected_tab: usize,
}

impl TorrentDetails {
    pub fn render_tabs(
        &mut self,
        f: &mut Frame,
        area: Rect,
        torrent_item: &TorrentItem,
        active: bool,
    ) {
        // Split into tab bar and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(area);

        // Tab bar
        let titles: Vec<Span> = vec!["[P]eers", "[F]iles"]
            .iter()
            .enumerate()
            .map(|(idx, t)| {
                let title = if idx == self.selected_tab {
                    format!("{}{}", &t[1..2], &t[3..])
                } else {
                    String::from(*t)
                };
                let style = Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD);
                Span::styled(format!(" {} ", title), style)
            })
            .collect();

        let tabs = Tabs::new(titles).select(self.selected_tab);

        f.render_widget(tabs, chunks[0]);

        match self.selected_tab {
            0 => self.render_peers(f, chunks[1], &torrent_item.peer_list, active),
            1 => self.render_files(f, chunks[1], &torrent_item.files, active),
            _ => (),
        }
    }

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

        let table = Table::new(rows, widths).header(header);

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

        let list = List::new(items).highlight_style(Style::default().fg(Color::LightBlue));

        f.render_stateful_widget(list, area, &mut state);
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
