use std::io;
use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::audio::player::AudioPlayer;
use crate::config::settings::Settings;
use crate::input::keybindings::{Action, Keybindings};
use crate::persistence::database::Database;
use crate::stats::history::History;
use crate::stats::live::LiveStats;
use crate::typing::engine::Engine;
use crate::typing::test::BACKSPACE_KEY;
use crate::typing::test::TestMode;
use crate::typing::words::Words;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Typing,
    Paused,
    Results,
    Settings,
    History,
}

pub struct App {
    pub state: AppState,
    pub settings: Settings,
    pub keybindings: Keybindings,
    pub history: History,
    pub audio: AudioPlayer,
    pub engine: Engine,
    pub words: Words,
    pub live_stats: LiveStats,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let settings = Settings::load()?;
        let keybindings = Keybindings::load()?;
        let database = Database::open()?;
        let history = database.load_history()?;
        let audio = AudioPlayer::new(&settings.audio)?;
        let engine = Engine::new(TestMode::Time(Duration::from_secs(15)));
        let mut app = Self {
            state: AppState::Typing,
            settings,
            keybindings,
            history,
            audio,
            engine,
            words: Words::builtin(),
            live_stats: LiveStats::default(),
            should_quit: false,
        };
        app.start_new_test();
        Ok(app)
    }

    pub fn start_new_test(&mut self) {
        self.state = AppState::Typing;
        let words = self.words.shuffled();
        self.engine.restart(&words);
        self.live_stats = LiveStats::default();
    }

    pub fn handle_key(&mut self, key: &KeyEvent) {
        match self.keybindings.handle(key) {
            Action::Quit => self.should_quit = true,
            Action::Restart => self.start_new_test(),
            Action::Submit => match self.state {
                AppState::Typing => self.engine.test.submit(Instant::now()),
                AppState::Results => self.start_new_test(),
                _ => {}
            },
            Action::Backspace if self.state == AppState::Typing => {
                self.engine.handle_key(BACKSPACE_KEY);
            }
            Action::TypeChar(c) if self.state == AppState::Typing => {
                self.engine.handle_key(c);
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, now: Instant) {
        if self.state == AppState::Typing {
            self.engine.test.tick(now);
            self.live_stats = LiveStats::calculate(&self.engine.test, now);
            if self.engine.test.is_finished() {
                self.state = AppState::Results;
            }
        }
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = ui::run_event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}