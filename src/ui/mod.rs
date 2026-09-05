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
pub use history::HistoryScreen;
pub use results::ResultsScreen;
pub use settings::SettingsScreen;
pub use typing::TypingScreen;

use crate::app::{App, AppState};

pub fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        if event::poll(app.poll_timeout())?
            && let event::Event::Key(key) = event::read()?
        {
            app.handle_key(&key);
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
        AppState::Typing => TypingScreen.render(frame, area, app),
        AppState::Menu => ConfigScreen.render(frame, area, app),
        AppState::Results => ResultsScreen.render(frame, area, app),
        AppState::Settings => SettingsScreen.render(frame, area, app),
        AppState::History => HistoryScreen.render(frame, area, app),
        AppState::Paused => {}
    }
}