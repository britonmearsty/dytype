use std::io;
use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::animation::Animations;
use crate::audio::event::SoundEvent;
use crate::audio::manager::AudioManager;
use crate::config::settings::{Config, CursorAnimation};
use crate::input::command::{Command, Keymap};
use crate::input::keybindings::{Action, Keybindings};
use crate::persistence::database::Database;
use crate::stats::history::History;
use crate::stats::live::LiveStats;
use crate::stats::result::TestResult;
use crate::terminal::TerminalGuard;
use crate::typing::engine::Engine;
use crate::typing::generator::{Generator, TestConfig, WordPools};
use crate::typing::test::{BACKSPACE_KEY, TestMode};
use crate::ui;
use crate::ui::config::ConfigMenu;
use crate::ui::history::HistoryTab;
use crate::ui::settings::SettingsMenu;
use crate::ui::widgets::theme::Theme;

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
    pub settings: Config,
    pub theme: Theme,
    pub keybindings: Keybindings,
    pub history: History,
    pub history_tab: HistoryTab,
    pub audio: AudioManager,
    pub engine: Engine,
    pub config: TestConfig,
    pub config_menu: ConfigMenu,
    pub settings_menu: SettingsMenu,
    pub pools: WordPools,
    pub generator: Generator,
    pub missed_words: Vec<String>,
    pub live_stats: LiveStats,
    pub anim: Animations,
    pub db: Database,
    pub should_quit: bool,
    sounds_seen: usize,
    sounds_words: usize,
    sounds_finished: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let settings = Config::load()?;
        let keybindings = Keybindings::new(Keymap::default().with_overrides(&settings.keybindings.overrides));
        let database = Database::open()?;
        let history = database.load_history()?;
        let audio = AudioManager::new(&settings.sounds)?;
        let theme = Theme::resolve(&settings.theme.name);
        let config = TestConfig {
            difficulty: settings.typing.difficulty,
            ..TestConfig::default()
        };
        let settings_menu = SettingsMenu::from_config(&settings);
        Ok(Self {
            state: AppState::Menu,
            settings,
            theme,
            keybindings,
            history,
            history_tab: HistoryTab::default(),
            audio,
            engine: Engine::with_test(TestMode::Words(0), &[]),
            config_menu: ConfigMenu::from_config(&config),
            settings_menu,
            config,
            pools: WordPools::default(),
            generator: Generator::new(),
            missed_words: Vec::new(),
            live_stats: LiveStats::default(),
            anim: Animations::new(),
            db: database,
            should_quit: false,
            sounds_seen: 0,
            sounds_words: 0,
            sounds_finished: false,
        })
    }

    pub fn start_new_test(&mut self) {
        self.state = AppState::Typing;
        let (mode, words) = self.generator.generate(&self.config, &self.pools, &self.missed_words);
        self.engine = Engine::with_test(mode, &words);
        self.live_stats = LiveStats::default();
        self.anim.mark_test_start(Instant::now());
        self.sounds_seen = 0;
        self.sounds_words = 0;
        self.sounds_finished = false;
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
            Action::Command(command) => self.handle_command(command, now),
            Action::Submit => match self.state {
                AppState::Typing => {
                    self.engine.test.submit(now);
                    self.dispatch_typing_sounds();
                }
                AppState::Results => self.start_new_test(),
                AppState::Menu => self.start_from_menu(),
                AppState::Settings => self.open_config_menu(),
                AppState::History => self.start_new_test(),
                _ => {}
            },
            Action::MoveUp => match self.state {
                AppState::Menu => self.config_menu.move_selection(-1),
                AppState::Settings => self.settings_menu.move_selection(-1),
                _ => {}
            },
            Action::MoveDown => match self.state {
                AppState::Menu => self.config_menu.move_selection(1),
                AppState::Settings => self.settings_menu.move_selection(1),
                _ => {}
            },
            Action::MoveLeft => match self.state {
                AppState::Menu => self.config_menu.cycle(-1),
                AppState::Settings => {
                    self.settings_menu.cycle(-1);
                    self.commit_settings();
                }
                AppState::History => self.history_tab = self.history_tab.cycle(-1),
                _ => {}
            },
            Action::MoveRight => match self.state {
                AppState::Menu => self.config_menu.cycle(1),
                AppState::Settings => {
                    self.settings_menu.cycle(1);
                    self.commit_settings();
                }
                AppState::History => self.history_tab = self.history_tab.cycle(1),
                _ => {}
            },
            Action::Backspace if self.state == AppState::Typing && self.settings.typing.backspace => {
                self.engine.handle_key(BACKSPACE_KEY, now);
                self.anim.observe(&self.engine.test, now);
                if self.settings.display.cursor_animation == CursorAnimation::Off {
                    self.anim.snap_cursor();
                }
            }
            Action::TypeChar(c) if self.state == AppState::Typing => {
                self.engine.handle_key(c, now);
                self.anim.observe(&self.engine.test, now);
                if self.settings.display.cursor_animation == CursorAnimation::Off {
                    self.anim.snap_cursor();
                }
                self.dispatch_typing_sounds();
            }
            _ => {}
        }
    }

    fn start_from_menu(&mut self) {
        self.config = self.config_menu.apply();
        self.config.punctuation = self.settings.typing.punctuation;
        self.config.numbers = self.settings.typing.numbers;
        self.start_new_test();
    }

    fn handle_command(&mut self, command: Command, now: Instant) {
        match command {
            Command::Quit => match self.state {
                AppState::Settings | AppState::History => self.open_config_menu(),
                _ => self.should_quit = true,
            },
            Command::Restart => match self.state {
                AppState::Typing | AppState::Paused => self.start_new_test(),
                AppState::Results | AppState::History => self.open_config_menu(),
                AppState::Menu => self.config_menu.select_start(),
                _ => {}
            },
            Command::Pause => match self.state {
                AppState::Typing if self.engine.test.is_running() => {
                    self.engine.test.pause(now);
                    self.state = AppState::Paused;
                }
                AppState::Paused => {
                    self.engine.test.resume(now);
                    self.state = AppState::Typing;
                }
                _ => {}
            },
            Command::NextTest => match self.state {
                AppState::Menu => self.start_from_menu(),
                AppState::Typing
                | AppState::Paused
                | AppState::Results
                | AppState::History => self.start_new_test(),
                _ => {}
            },
            Command::PreviousTest => match self.state {
                AppState::Results | AppState::History => self.open_config_menu(),
                _ => {}
            },
            Command::OpenSettings => match self.state {
                AppState::Menu | AppState::Results | AppState::History => self.open_settings(),
                _ => {}
            },
            Command::OpenHistory => match self.state {
                AppState::Menu | AppState::Results => self.state = AppState::History,
                _ => {}
            },
            Command::ToggleStats => match self.state {
                AppState::Typing | AppState::Paused => {
                    self.settings.display.compact_mode = !self.settings.display.compact_mode;
                }
                _ => {}
            },
        }
    }

    fn open_settings(&mut self) {
        self.settings_menu = SettingsMenu::from_config(&self.settings);
        self.state = AppState::Settings;
    }

    fn commit_settings(&mut self) {
        self.settings = self.settings_menu.apply(&self.settings);
        self.theme = Theme::resolve(&self.settings.theme.name);
        self.audio.reconfigure(&self.settings.sounds);
        if let Err(error) = self.settings.save() {
            eprintln!("warning: failed to save settings: {error}");
        }
    }

    fn dispatch_typing_sounds(&mut self) {
        let events = self.collect_sound_events();
        self.sounds_seen = self.engine.test.keystrokes.len();
        self.sounds_words = self.engine.test.completed_words();
        self.sounds_finished = self.engine.test.is_finished();
        let audio = &self.settings.sounds;
        for event in events {
            self.audio.emit(event, audio);
        }
    }

    fn collect_sound_events(&self) -> Vec<SoundEvent> {
        let test = &self.engine.test;
        let mut events = Vec::new();
        let focus = self.sounds_seen.min(test.keystrokes.len());
        for stroke in &test.keystrokes[focus..] {
            events.push(if stroke.correct {
                SoundEvent::Keypress
            } else {
                SoundEvent::Error
            });
        }
        if test.completed_words() > self.sounds_words {
            events.push(SoundEvent::WordComplete);
        }
        if test.is_finished() && !self.sounds_finished {
            events.push(SoundEvent::TestComplete);
        }
        events
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
            let frame = self.frame_duration();
            return remaining.min(frame);
        }
        self.frame_duration()
    }

    fn frame_duration(&self) -> Duration {
        let fps = self.settings.display.fps.max(1);
        Duration::from_millis(u64::from((1000 + fps / 2) / fps))
    }

    pub fn tick(&mut self, now: Instant) {
        self.anim.update(now);
        if self.state == AppState::Typing {
            self.engine.test.tick(now);
            self.dispatch_typing_sounds();
            self.live_stats = LiveStats::calculate(&self.engine.test, now);
            if self.engine.test.is_finished() {
                self.collect_missed_words();
                self.record_result(now);
                self.anim.results.start(
                    self.live_stats.wpm as f32,
                    self.live_stats.raw_wpm as f32,
                    self.live_stats.accuracy as f32,
                    self.live_stats.consistency as f32,
                    now,
                );
                self.state = AppState::Results;
            }
        }
    }

    fn record_result(&mut self, now: Instant) {
        let id = self.history.next_id();
        let result = TestResult::build(id, &self.engine.test, self.config.difficulty, now);
        self.history.push(result.clone());
        if let Err(error) = self.db.append(&result) {
            eprintln!("warning: failed to save test result: {error}");
        }
    }
}

pub fn run() -> io::Result<()> {
    install_panic_hook();
    let mut guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = ui::run_event_loop(&mut terminal, &mut app);

    terminal.show_cursor()?;
    guard.restore();

    result
}

/// Ensures the terminal is restored even if the application panics, by
/// intercepting panics before unwinding and restoring terminal state, then
/// re-raising the panic.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = crossterm::execute!(
            stdout,
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        previous(info);
    }));
}