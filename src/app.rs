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
use crate::typing::generator::{Generator, TestConfig, WordPools};
use crate::typing::test::{BACKSPACE_KEY, TestMode};
use crate::ui;
use crate::ui::config::ConfigMenu;

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
    pub config: TestConfig,
    pub config_menu: ConfigMenu,
    pub pools: WordPools,
    pub generator: Generator,
    pub missed_words: Vec<String>,
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
        let config = TestConfig::default();
        Ok(Self {
            state: AppState::Menu,
            settings,
            keybindings,
            history,
            audio,
            engine: Engine::with_test(TestMode::Words(0), &[]),
            config_menu: ConfigMenu::from_config(&config),
            config,
            pools: WordPools::default(),
            generator: Generator::new(),
            missed_words: Vec::new(),
            live_stats: LiveStats::default(),
            should_quit: false,
        })
    }

    pub fn start_new_test(&mut self) {
        self.state = AppState::Typing;
        let (mode, words) = self.generator.generate(&self.config, &self.pools, &self.missed_words);
        self.engine = Engine::with_test(mode, &words);
        self.live_stats = LiveStats::default();
    }

    fn collect_missed_words(&mut self) {
        let test = &self.engine.test;
        let mut missed: Vec<String> = test
            .errors
            .iter()
            .filter_map(|error| test.word_at(error.position))
            .collect();
        missed.sort();
        missed.dedup();
        missed.truncate(50);
        self.missed_words = missed;
    }

    pub fn handle_key(&mut self, key: &KeyEvent) {
        let now = Instant::now();
        match self.keybindings.handle(key) {
            Action::Quit => self.should_quit = true,
            Action::Restart => match self.state {
                AppState::Typing => self.start_new_test(),
                AppState::Results => self.open_config_menu(),
                AppState::Menu => self.config_menu.select_start(),
                _ => {}
            },
            Action::Submit => match self.state {
                AppState::Typing => self.engine.test.submit(now),
                AppState::Results => self.start_new_test(),
                AppState::Menu => {
                    self.config = self.config_menu.apply();
                    self.start_new_test();
                }
                _ => {}
            },
            Action::MoveUp if self.state == AppState::Menu => self.config_menu.move_selection(-1),
            Action::MoveDown if self.state == AppState::Menu => self.config_menu.move_selection(1),
            Action::MoveLeft if self.state == AppState::Menu => self.config_menu.cycle(-1),
            Action::MoveRight if self.state == AppState::Menu => self.config_menu.cycle(1),
            Action::Backspace if self.state == AppState::Typing => {
                self.engine.handle_key(BACKSPACE_KEY, now);
            }
            Action::TypeChar(c) if self.state == AppState::Typing => {
                self.engine.handle_key(c, now);
            }
            _ => {}
        }
    }

    fn open_config_menu(&mut self) {
        self.config_menu = ConfigMenu::from_config(&self.config);
        self.state = AppState::Menu;
    }

    pub fn poll_timeout(&self) -> Duration {
        if self.state == AppState::Typing
            && let Some(deadline) = self.engine.test.deadline()
        {
            let remaining = deadline.saturating_duration_since(Instant::now());
            return remaining.min(Duration::from_millis(100));
        }
        Duration::from_millis(50)
    }

    pub fn tick(&mut self, now: Instant) {
        if self.state == AppState::Typing {
            self.engine.test.tick(now);
            self.live_stats = LiveStats::calculate(&self.engine.test, now);
            if self.engine.test.is_finished() {
                self.collect_missed_words();
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