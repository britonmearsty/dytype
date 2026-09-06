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
    let mut first_frame = true;
    loop {
        let mut redraw = first_frame;
        first_frame = false;
        if event::poll(app.poll_timeout())? {
            match event::read()? {
                event::Event::Key(key) => {
                    app.handle_key(&key);
                    redraw = true;
                }
                event::Event::Resize(_, _) => {
                    // Force a reflow to the new size even if the screen was
                    // otherwise static.
                    redraw = true;
                }
                _ => {}
            }
        }
        let now = Instant::now();
        app.tick(now);
        // Skip the draw when nothing changed: a key event, the interactive
        // typing state (blink/timer/live stats), or a running animation all
        // require a new frame, but a static menu does not. Keeping the poll
        // cadence frame-rate keeps input latency under one frame.
        let needs_frame = app.state == AppState::Typing || app.state == AppState::Paused;
        if redraw || needs_frame || app.anim.is_active(now) {
            terminal.draw(|frame| render(frame, app))?;
        }
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