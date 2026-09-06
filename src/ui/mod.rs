use std::io;
use std::time::Instant;

use crossterm::event;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub mod config;
pub mod history;
pub mod results;
pub mod settings;
pub mod typing;
pub mod widgets;

pub use config::ConfigScreen;
pub use history::{HistoryScreen, HistoryTab};
pub use results::ResultsScreen;
pub use settings::SettingsScreen;
pub use typing::TypingScreen;

use crate::app::{App, AppState};

pub fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        if event::poll(app.poll_timeout())? {
            match event::read()? {
                event::Event::Key(key) => app.handle_key(&key),
                // The next draw reflows to the new size via ratatui's
                // autoresize, so a resize needs no action here.
                event::Event::Resize(_, _) => {}
                _ => {}
            }
        }
        app.tick(Instant::now());
        terminal.draw(|frame| render(frame, app))?;
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    match app.state {
        AppState::Typing | AppState::Paused => TypingScreen.render(frame, area, app),
        AppState::Menu => ConfigScreen.render(frame, area, app),
        AppState::Results => ResultsScreen.render(frame, area, app),
        AppState::Settings => SettingsScreen.render(frame, area, app),
        AppState::History => HistoryScreen.render(frame, area, app),
    }
}