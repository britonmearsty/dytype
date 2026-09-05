use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::typing::generator::{Difficulty, TestConfig, TestKind};

const WORD_COUNTS: &[usize] = &[10, 25, 50, 100, 200];
const DURATIONS_SECS: &[usize] = &[15, 30, 60, 120];
const MODE_LABELS: [&str; 4] = ["Words", "Time", "Quote", "Practice"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Words,
    Time,
    Quote,
    Practice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Mode,
    Count,
    Time,
    Difficulty,
    Start,
}

impl Row {
    fn label(self) -> &'static str {
        match self {
            Row::Mode => "Mode",
            Row::Count => "Word count",
            Row::Time => "Time",
            Row::Difficulty => "Difficulty",
            Row::Start => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigMenu {
    mode: usize,
    count: usize,
    time: usize,
    difficulty: usize,
    selected: usize,
}

impl ConfigMenu {
    pub fn from_config(config: &TestConfig) -> Self {
        let (mode, count, time) = match &config.kind {
            TestKind::Words(n) => (0, position_or(WORD_COUNTS, *n, 1), 0),
            TestKind::Time(duration) => (
                1,
                1,
                position_or(DURATIONS_SECS, duration.as_secs() as usize, 0),
            ),
            TestKind::Quote => (2, 1, 0),
            TestKind::Custom(_) => (0, 1, 0),
            TestKind::Practice(n) => (3, position_or(WORD_COUNTS, *n, 1), 0),
        };
        Self {
            mode,
            count,
            time,
            difficulty: difficulty_index(config.difficulty),
            selected: 0,
        }
    }

    pub fn apply(&self) -> TestConfig {
        let kind = match self.current_mode() {
            Mode::Words => TestKind::Words(WORD_COUNTS[self.count]),
            Mode::Time => TestKind::Time(Duration::from_secs(DURATIONS_SECS[self.time] as u64)),
            Mode::Quote => TestKind::Quote,
            Mode::Practice => TestKind::Practice(WORD_COUNTS[self.count]),
        };
        TestConfig::new(kind, self.current_difficulty())
    }

    pub fn move_selection(&mut self, dir: i8) {
        let len = self.rows().len() as i8;
        self.selected = (self.selected as i8 + dir).rem_euclid(len) as usize;
    }

    pub fn cycle(&mut self, dir: i8) {
        match self.rows()[self.selected] {
            Row::Mode => {
                self.mode = (self.mode as i8 + dir).rem_euclid(MODE_LABELS.len() as i8) as usize;
                self.selected = self.selected.min(self.rows().len() - 1);
            }
            Row::Count => {
                self.count = (self.count as i8 + dir).rem_euclid(WORD_COUNTS.len() as i8) as usize;
            }
            Row::Time => {
                self.time =
                    (self.time as i8 + dir).rem_euclid(DURATIONS_SECS.len() as i8) as usize;
            }
            Row::Difficulty => {
                self.difficulty = (self.difficulty as i8 + dir).rem_euclid(4) as usize;
            }
            Row::Start => {}
        }
    }

    pub fn select_start(&mut self) {
        self.selected = self.rows().len() - 1;
    }

    fn rows(&self) -> Vec<Row> {
        match self.current_mode() {
            Mode::Words | Mode::Practice => {
                vec![Row::Mode, Row::Count, Row::Difficulty, Row::Start]
            }
            Mode::Time => vec![Row::Mode, Row::Time, Row::Difficulty, Row::Start],
            Mode::Quote => vec![Row::Mode, Row::Difficulty, Row::Start],
        }
    }

    fn current_mode(&self) -> Mode {
        match self.mode {
            0 => Mode::Words,
            1 => Mode::Time,
            2 => Mode::Quote,
            _ => Mode::Practice,
        }
    }

    fn current_difficulty(&self) -> Difficulty {
        match self.difficulty {
            0 => Difficulty::Easy,
            1 => Difficulty::Normal,
            2 => Difficulty::Hard,
            _ => Difficulty::Expert,
        }
    }

    fn value_text(&self, row: Row) -> String {
        match row {
            Row::Mode => MODE_LABELS[self.mode].to_owned(),
            Row::Count => WORD_COUNTS[self.count].to_string(),
            Row::Time => format!("{}s", DURATIONS_SECS[self.time]),
            Row::Difficulty => self.current_difficulty().label().to_owned(),
            Row::Start => "Start test".to_owned(),
        }
    }
}

fn position_or(list: &[usize], value: usize, fallback: usize) -> usize {
    list.iter().position(|&item| item == value).unwrap_or(fallback)
}

fn difficulty_index(difficulty: Difficulty) -> usize {
    match difficulty {
        Difficulty::Easy => 0,
        Difficulty::Normal => 1,
        Difficulty::Hard => 2,
        Difficulty::Expert => 3,
    }
}

pub struct ConfigScreen;

impl ConfigScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let menu = &app.config_menu;
        let mut lines = vec![Line::from("")];
        for (i, row) in menu.rows().iter().enumerate() {
            let selected = i == menu.selected;
            let marker = if selected { "▸" } else { " " };
            let value_style = if selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("   {marker} {:<12}", row.label()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(menu.value_text(*row), value_style),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(
            "   ↑/↓ move   ←/→ change   Enter start test   F2 settings   F3 history   Esc quit",
        ));
        let block = Block::default().title(" config ").borders(Borders::ALL);
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_menu() -> ConfigMenu {
        ConfigMenu::from_config(&TestConfig::default())
    }

    #[test]
    fn default_menu_applies_to_default_config() {
        assert_eq!(default_menu().apply(), TestConfig::default());
    }

    #[test]
    fn cycling_to_time_mode_builds_time_kind() {
        let mut menu = default_menu();
        menu.cycle(1);
        assert_eq!(menu.apply().kind, TestKind::Time(Duration::from_secs(15)));
    }

    #[test]
    fn cycling_time_cycles_durations() {
        let config = TestConfig::new(TestKind::Time(Duration::from_secs(60)), Difficulty::Normal);
        let mut menu = ConfigMenu::from_config(&config);
        assert_eq!(menu.apply().kind, TestKind::Time(Duration::from_secs(60)));
        menu.move_selection(1); // Mode -> Time
        menu.cycle(1); // 60s -> 120s
        assert_eq!(menu.apply().kind, TestKind::Time(Duration::from_secs(120)));
    }

    #[test]
    fn from_config_roundtrips_practice_and_expert() {
        let config = TestConfig::new(TestKind::Practice(50), Difficulty::Expert);
        let menu = ConfigMenu::from_config(&config);
        assert_eq!(menu.apply(), config);
    }

    #[test]
    fn quote_mode_hides_count_and_time_rows() {
        let mut menu = default_menu();
        menu.cycle(1); // Words -> Time
        menu.cycle(1); // Time -> Quote
        assert_eq!(menu.rows(), vec![Row::Mode, Row::Difficulty, Row::Start]);
    }

    #[test]
    fn switching_mode_clamps_selection_into_range() {
        let mut menu = default_menu();
        menu.select_start();
        assert_eq!(menu.selected, 3);
        menu.cycle(2); // Words -> Quote
        assert!(menu.selected < menu.rows().len());
    }

    #[test]
    fn selection_wraps_around() {
        let mut menu = default_menu();
        menu.move_selection(-1);
        assert_eq!(menu.selected, 3);
        menu.move_selection(1);
        assert_eq!(menu.selected, 0);
    }
}