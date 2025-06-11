use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, ui_models::TorrentItem};

const INFO_TEXT: &str = "(Esc) quit | (↑) move up | (↓) move down | (←) move left | (→) move right";

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
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
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
    render_footer(frame, vertical_chunks[2]);

    frame.render_widget(Block::default().borders(Borders::ALL), middle_chunks[1]);
}

use ratatui::widgets::{Cell, Row, Table, TableState};

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
                Cell::from(t.name.clone()),
                Cell::from(t.status.clone()),
                Cell::from(t.info_hash.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
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
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), bar.into()]))
        .highlight_spacing(HighlightSpacing::Always);

    let mut state = TableState::default();
    state.select(Some(selected));

    f.render_stateful_widget(table, area, &mut state);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let info_footer = Paragraph::new(Text::from(INFO_TEXT))
        .centered()
        .block(Block::bordered().border_type(BorderType::Double));

    frame.render_widget(info_footer, area);
}
