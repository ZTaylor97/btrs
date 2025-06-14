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
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::tui::Tui;

#[derive(Debug)]
enum AppEvent {
    Terminal(Event),
    Custom(AppEventType),
}

#[derive(Debug)]
pub enum AppEventType {
    Download(String),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut app = App::new();

    let mut terminal = ratatui::init();

    run_app(&mut terminal, &mut app).await?;

    ratatui::restore();

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Error> {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);

    // Start terminal event thread
    let tx1 = tx.clone();
    tokio::spawn(async move {
        loop {
            // Block in a separate thread to poll for terminal events
            if let Ok(Ok(true)) =
                tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(100))).await
            {
                if let Ok(evt) = tokio::task::spawn_blocking(event::read).await {
                    if let Ok(evt) = evt {
                        if tx1.send(AppEvent::Terminal(evt)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    let mut tui = Tui::new(tx.clone());

    while let Some(event) = rx.recv().await {
        if app.should_exit {
            break;
        }

        match event {
            AppEvent::Terminal(event) => match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    app.handle_key(key_event, &mut tui).await?
                }
                _ => {}
            },
            AppEvent::Custom(AppEventType::Download(key)) => app.download_torrent(&key).await?,
        }
    }
    loop {
        terminal.draw(|f| tui.draw(f, &app.torrent_items()))?;
    }

    Ok(())
}
