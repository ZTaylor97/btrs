use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState,
    },
};

use crate::{
    app::{App, ui_models::TorrentItem},
    torrent::Peer,
};

const INFO_TEXT: &str = "(Esc) quit | (⏎) toggle torrent start/stop | (↑) move up | (↓) move down";

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
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(vertical_chunks[1]);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled("BTRS", Style::default().fg(Color::Green)))
        .centered()
        .block(title_block);

    frame.render_widget(title, vertical_chunks[0]);

    let torrent_items: Vec<TorrentItem> = app.torrents.iter().map(TorrentItem::from).collect();

    render_torrents_table(frame, middle_chunks[0], &torrent_items, app.selected);
    render_peers(
        frame,
        middle_chunks[1],
        &torrent_items[app.selected].peer_list,
    );
    render_footer(frame, vertical_chunks[2]);

    frame.render_widget(Block::default().borders(Borders::ALL), middle_chunks[1]);
}

pub fn render_torrents_table(f: &mut Frame, area: Rect, torrents: &[TorrentItem], selected: usize) {
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
                .title("Torrents")
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
        .block(Block::default().title("Peers").borders(Borders::ALL));

    f.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let info_footer = Paragraph::new(Text::from(INFO_TEXT))
        .centered()
        .block(Block::bordered().border_type(BorderType::Double));

    frame.render_widget(info_footer, area);
}
