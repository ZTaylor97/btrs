pub mod app;
pub mod torrent;
pub mod tui;

use anyhow::Error;
use app::App;

use ratatui::{
    Terminal,
    crossterm::event::{self, Event, KeyEventKind},
    prelude::Backend,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut app = App::new();

    let mut terminal = ratatui::init();

    run_app(&mut terminal, &mut app).await?;

    ratatui::restore();

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Error> {
    loop {
        if app.should_exit {
            break;
        }

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                app.handle_key(key_event).await?
            }
            _ => {}
        };

        terminal.draw(|f| tui::draw(f, &app))?;
    }

    Ok(())
}
