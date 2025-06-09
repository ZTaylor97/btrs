pub mod app;
pub mod torrent;
pub mod tui;

use anyhow::Error;
use app::App;

use ratatui::Terminal;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::{Backend, CrosstermBackend};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut app = App::new();

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, &app).await?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), Error> {
    let mut exit = false;
    while !exit {
        terminal.draw(|f| {})?;

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Esc => exit = true,
                    _ => (),
                }
            }
            _ => {}
        };
    }

    app.download_torrents().await?;
    Ok(())
}
