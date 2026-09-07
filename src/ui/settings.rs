use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::audio::settings::SoundPack;
use crate::config::settings::{Config, CursorAnimation, CursorStyle};
use crate::typing::generator::Difficulty;
use crate::typing::words::available_languages;
#[cfg(test)]
use crate::ui::widgets::theme::THEMES;

const PACKS: [SoundPack; 5] = [
    SoundPack::Mechanical,
    SoundPack::Typewriter,
    SoundPack::Soft,
    SoundPack::Retro,
    SoundPack::None,
];

const CURSOR_STYLES: [CursorStyle; 3] =
    [CursorStyle::Bar, CursorStyle::Block, CursorStyle::Underline];
const CURSOR_ANIMATIONS: [CursorAnimation; 2] = [CursorAnimation::Smooth, CursorAnimation::Off];
const DIFFICULTIES: [Difficulty; 4] = [
    Difficulty::Easy,
    Difficulty::Normal,
    Difficulty::Hard,
    Difficulty::Expert,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Theme,
    Cursor,
    CursorAnimation,
    Animations,
    Mouse,
    Language,
    Difficulty,
    Punctuation,
    Numbers,
    Sounds,
    Volume,
    Pack,
}

impl Row {
    const ALL: [Row; 12] = [
        Row::Theme,
        Row::Cursor,
        Row::CursorAnimation,
        Row::Animations,
        Row::Mouse,
        Row::Language,
        Row::Difficulty,
        Row::Punctuation,
        Row::Numbers,
        Row::Sounds,
        Row::Volume,
        Row::Pack,
    ];

    fn label(self) -> &'static str {
        match self {
            Row::Theme => "Theme",
            Row::Cursor => "Cursor",
            Row::CursorAnimation => "Cursor animation",
            Row::Animations => "Animations",
            Row::Mouse => "Mouse",
            Row::Language => "Language",
            Row::Difficulty => "Difficulty",
            Row::Punctuation => "Punctuation",
            Row::Numbers => "Numbers",
            Row::Sounds => "Enabled",
            Row::Volume => "Volume",
            Row::Pack => "Sound pack",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Group {
    title: &'static str,
    first: usize,
}

const GROUPS: [Group; 3] = [
    Group {
        title: "Appearance",
        first: 0,
    },
    Group {
        title: "Typing",
        first: 5,
    },
    Group {
        title: "Sounds",
        first: 9,
    },
];

fn group_for(index: usize) -> &'static Group {
    GROUPS
        .iter()
        .rev()
        .find(|group| index >= group.first)
        .unwrap_or(&GROUPS[0])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsMenu {
    theme_names: Vec<String>,
    theme: usize,
    languages: Vec<String>,
    language: usize,
    cursor: usize,
    cursor_animation: usize,
    animations: bool,
    mouse: bool,
    difficulty: usize,
    punctuation: bool,
    numbers: bool,
    sounds: bool,
    volume_pct: u8,
    pack: usize,
    selected: usize,
}

impl SettingsMenu {
    pub fn from_config(config: &Config, theme_names: &[String]) -> Self {
        let languages = available_languages();
        Self {
            theme_names: theme_names.to_vec(),
            theme: theme_names
                .iter()
                .position(|name| name.eq_ignore_ascii_case(&config.theme.name))
                .unwrap_or(0),
            languages: languages.clone(),
            language: languages
                .iter()
                .position(|name| name.eq_ignore_ascii_case(&config.typing.language))
                .unwrap_or(0),
            cursor: CURSOR_STYLES
                .iter()
                .position(|style| *style == config.display.cursor.style)
                .unwrap_or(0),
            cursor_animation: CURSOR_ANIMATIONS
                .iter()
                .position(|style| *style == config.display.cursor_animation)
                .unwrap_or(0),
            animations: config.display.animations,
            mouse: config.display.mouse,
            difficulty: DIFFICULTIES
                .iter()
                .position(|difficulty| *difficulty == config.typing.difficulty)
                .unwrap_or(1),
            punctuation: config.typing.punctuation,
            numbers: config.typing.numbers,
            sounds: config.sounds.enabled,
            volume_pct: (config.sounds.volume * 100.0).round().clamp(0.0, 100.0) as u8,
            pack: PACKS
                .iter()
                .position(|pack| *pack == config.sounds.sound_pack)
                .unwrap_or(0),
            selected: 0,
        }
    }

    /// Merges every managed option into a fresh copy of `base`.
    pub fn apply(&self, base: &Config) -> Config {
        let mut config = base.clone();
        config.theme.name = self.theme_names[self.theme].clone();
        config.display.cursor.style = CURSOR_STYLES[self.cursor];
        config.display.cursor_animation = CURSOR_ANIMATIONS[self.cursor_animation];
        config.display.animations = self.animations;
        config.display.mouse = self.mouse;
        config.typing.language = self.languages[self.language].clone();
        config.typing.difficulty = DIFFICULTIES[self.difficulty];
        config.typing.punctuation = self.punctuation;
        config.typing.numbers = self.numbers;
        config.sounds.enabled = self.sounds;
        config.sounds.volume = f64::from(self.volume_pct) / 100.0;
        config.sounds.sound_pack = PACKS[self.pack];
        config
    }

    pub fn move_selection(&mut self, dir: i8) {
        self.selected = (self.selected as i8 + dir).rem_euclid(Row::ALL.len() as i8) as usize;
    }

    /// Selects the row whose rendered line sits at content row `y` inside
    /// `area`, accounting for the group header lines. Used by mouse clicks.
    pub fn row_at(&self, area: Rect, y: u16) -> Option<usize> {
        let first = area.y.saturating_add(2);
        if y < first {
            return None;
        }
        let mut row_y = first;
        for (i, _) in Row::ALL.iter().enumerate() {
            if group_for(i).first == i {
                row_y = row_y.saturating_add(2);
            }
            if y == row_y {
                return Some(i);
            }
            row_y = row_y.saturating_add(1);
        }
        None
    }

    pub fn select_row(&mut self, index: usize) {
        if index < Row::ALL.len() {
            self.selected = index;
        }
    }

    pub fn cycle(&mut self, dir: i8) {
        match Row::ALL[self.selected] {
            Row::Theme => {
                self.theme =
                    (self.theme as i8 + dir).rem_euclid(self.theme_names.len() as i8) as usize;
            }
            Row::Cursor => {
                self.cursor =
                    (self.cursor as i8 + dir).rem_euclid(CURSOR_STYLES.len() as i8) as usize;
            }
            Row::CursorAnimation => {
                self.cursor_animation = (self.cursor_animation as i8 + dir)
                    .rem_euclid(CURSOR_ANIMATIONS.len() as i8)
                    as usize;
            }
            Row::Animations => self.animations = !self.animations,
            Row::Mouse => self.mouse = !self.mouse,
            Row::Language => {
                self.language =
                    (self.language as i8 + dir).rem_euclid(self.languages.len() as i8) as usize;
            }
            Row::Difficulty => {
                self.difficulty =
                    (self.difficulty as i8 + dir).rem_euclid(DIFFICULTIES.len() as i8) as usize;
            }
            Row::Punctuation => self.punctuation = !self.punctuation,
            Row::Numbers => self.numbers = !self.numbers,
            Row::Sounds => self.sounds = !self.sounds,
            Row::Volume => {
                self.volume_pct = ((self.volume_pct as i16 + dir as i16 * 5).clamp(0, 100)) as u8;
            }
            Row::Pack => {
                self.pack = (self.pack as i8 + dir).rem_euclid(PACKS.len() as i8) as usize;
            }
        }
    }

    fn on_off(value: bool) -> &'static str {
        if value { "On" } else { "Off" }
    }

    fn value_text(&self, row: Row) -> String {
        match row {
            Row::Theme => self.theme_names[self.theme].clone(),
            Row::Cursor => cursor_label(CURSOR_STYLES[self.cursor]).to_owned(),
            Row::CursorAnimation => {
                cursor_animation_label(CURSOR_ANIMATIONS[self.cursor_animation]).to_owned()
            }
            Row::Animations => Self::on_off(self.animations).to_owned(),
            Row::Mouse => Self::on_off(self.mouse).to_owned(),
            Row::Language => self.languages[self.language].clone(),
            Row::Difficulty => title_case(DIFFICULTIES[self.difficulty].label()).to_owned(),
            Row::Punctuation => Self::on_off(self.punctuation).to_owned(),
            Row::Numbers => Self::on_off(self.numbers).to_owned(),
            Row::Sounds => Self::on_off(self.sounds).to_owned(),
            Row::Volume => {
                let bar_width = 10;
                let filled = usize::from(self.volume_pct) * bar_width / 100;
                let bar: String =
                    "█".repeat(filled) + &"░".repeat(bar_width.saturating_sub(filled));
                format!("{bar} {}%", self.volume_pct)
            }
            Row::Pack => PACKS[self.pack].label().to_owned(),
        }
    }
}

fn cursor_label(style: CursorStyle) -> &'static str {
    match style {
        CursorStyle::Bar => "Bar",
        CursorStyle::Block => "Block",
        CursorStyle::Underline => "Underline",
    }
}

fn cursor_animation_label(animation: CursorAnimation) -> &'static str {
    match animation {
        CursorAnimation::Smooth => "Smooth",
        CursorAnimation::Off => "Off",
    }
}

fn title_case(label: &str) -> String {
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let menu = &app.settings_menu;
        let theme = app.theme;
        let mut lines = vec![Line::from("")];

        for (i, row) in Row::ALL.iter().enumerate() {
            let group = group_for(i);
            if group.first == i {
                lines.push(Line::from(Span::styled(
                    format!("  {}", group.title),
                    Style::default()
                        .fg(theme.muted)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )));
                lines.push(Line::from(""));
            }
            let selected = i == menu.selected;
            let marker = if selected { "▸" } else { " " };
            let value_style = if selected {
                Style::default()
                    .fg(theme.correct)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("   {marker} {:<20}", row.label()),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(menu.value_text(*row), value_style),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "   ↑/↓ move   ←/→ change   Esc back",
            Style::default().fg(theme.muted),
        )));

        let block = Block::default()
            .title(" settings ")
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background));
        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin_theme_names() -> Vec<String> {
        THEMES.iter().map(|theme| theme.name.to_owned()).collect()
    }

    fn default_menu() -> SettingsMenu {
        SettingsMenu::from_config(&Config::default(), &builtin_theme_names())
    }

    #[test]
    fn appearance_rows_cycle_and_apply() {
        let mut menu = default_menu();
        menu.cycle(1); // Theme -> Monokai
        let base = Config::default();
        let applied = menu.apply(&base);
        assert_eq!(applied.theme.name, "Monokai");

        menu.move_selection(1); // Cursor
        menu.cycle(2); // Bar -> Underline
        let applied = menu.apply(&Config::default());
        assert_eq!(applied.display.cursor.style, CursorStyle::Underline);

        menu.move_selection(1); // Cursor animation
        menu.cycle(1); // Smooth -> Off
        let applied = menu.apply(&Config::default());
        assert_eq!(applied.display.cursor_animation, CursorAnimation::Off);

        menu.move_selection(1); // Animations
        menu.cycle(1);
        let applied = menu.apply(&Config::default());
        assert!(!applied.display.animations);

        menu.move_selection(1); // Mouse
        menu.cycle(1);
        let applied = menu.apply(&Config::default());
        assert!(!applied.display.mouse);
    }

    #[test]
    fn typing_rows_cycle_and_apply() {
        let mut menu = default_menu();
        menu.move_selection(5); // Language
        menu.cycle(1); // English -> next available language
        let applied_language = menu.apply(&Config::default()).typing.language;
        assert_ne!(applied_language, "English");
        assert!(available_languages().contains(&applied_language));
        let mut menu = default_menu();
        menu.move_selection(6); // Difficulty
        menu.cycle(1); // Normal -> Hard
        menu.move_selection(1); // Punctuation
        menu.cycle(1);
        menu.move_selection(1); // Numbers
        menu.cycle(1);
        let applied = menu.apply(&Config::default());
        assert_eq!(applied.typing.difficulty, Difficulty::Hard);
        assert!(applied.typing.punctuation);
        assert!(applied.typing.numbers);
        assert_eq!(applied.typing.language, "English");
    }

    #[test]
    fn volume_cycles_in_five_percent_steps_and_clamps() {
        let mut menu = default_menu();
        menu.move_selection(10); // Volume
        menu.cycle(1);
        assert_eq!(menu.apply(&Config::default()).sounds.volume, 0.7);
        for _ in 0..60 {
            menu.cycle(-1);
        }
        assert_eq!(menu.apply(&Config::default()).sounds.volume, 0.0);
        for _ in 0..60 {
            menu.cycle(1);
        }
        assert_eq!(menu.apply(&Config::default()).sounds.volume, 1.0);
    }

    #[test]
    fn sound_rows_cycle_and_apply() {
        let mut menu = default_menu();
        menu.move_selection(9); // Sounds
        menu.cycle(1); // On -> Off
        assert!(!menu.apply(&Config::default()).sounds.enabled);
        menu.move_selection(2); // Pack
        menu.cycle(4);
        assert_eq!(
            menu.apply(&Config::default()).sounds.sound_pack,
            SoundPack::None
        );
        menu.cycle(1);
        assert_eq!(
            menu.apply(&Config::default()).sounds.sound_pack,
            SoundPack::Mechanical
        );
    }

    #[test]
    fn apply_preserves_unmanaged_settings() {
        let base = Config::default();
        let mut with_fields = base.clone();
        with_fields.sounds.error = false;
        with_fields.sounds.complete = false;
        with_fields.typing.backspace = false;
        let menu = SettingsMenu::from_config(&with_fields, &builtin_theme_names());
        let applied = menu.apply(&with_fields);
        assert!(!applied.sounds.error);
        assert!(!applied.sounds.complete);
        assert!(!applied.typing.backspace);
    }

    #[test]
    fn from_config_roundtrips_non_defaults() {
        let mut config = Config::default();
        config.theme.name = "Nord".to_owned();
        config.display.cursor.style = CursorStyle::Block;
        config.display.cursor_animation = CursorAnimation::Off;
        config.display.animations = false;
        config.display.mouse = false;
        config.typing.difficulty = Difficulty::Expert;
        config.typing.numbers = true;
        config.sounds.enabled = false;
        config.sounds.sound_pack = SoundPack::Retro;
        config.sounds.volume = 0.4;
        assert_eq!(
            SettingsMenu::from_config(&config, &builtin_theme_names()).apply(&config),
            config
        );
    }

    #[test]
    fn custom_theme_names_cycle_and_apply() {
        let names = vec![
            "Default".to_owned(),
            "Monokai".to_owned(),
            "Sunset".to_owned(),
        ];
        let mut config = Config::default();
        config.theme.name = "Monokai".to_owned();
        let mut menu = SettingsMenu::from_config(&config, &names);
        menu.cycle(1); // Monokai -> Sunset
        assert_eq!(menu.value_text(Row::Theme), "Sunset");
        assert_eq!(menu.apply(&Config::default()).theme.name, "Sunset");
        menu.cycle(-2); // back through the start of the custom list
        assert_eq!(menu.value_text(Row::Theme), "Default");
    }

    #[test]
    fn selection_wraps_around() {
        let mut menu = default_menu();
        menu.move_selection(-1);
        assert_eq!(menu.selected, Row::ALL.len() - 1);
    }

    #[test]
    fn title_case_pretty_prints() {
        assert_eq!(title_case("normal"), "Normal");
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn row_at_maps_click_y_to_row_skipping_group_headers() {
        let menu = default_menu();
        let area = Rect::new(0, 0, 80, 30);
        // Border (y 0) then content: blank, header + blank, so the first
        // appearance row "Theme" renders at y=4.
        assert_eq!(menu.row_at(area, 4), Some(0)); // Theme
        assert_eq!(menu.row_at(area, 5), Some(1)); // Cursor
        assert_eq!(menu.row_at(area, 8), Some(4)); // Mouse
        // After Mouse, "Typing" group adds header + blank before Language.
        assert_eq!(menu.row_at(area, 11), Some(5)); // Language
        assert_eq!(menu.row_at(area, 18), Some(10)); // Volume
        assert_eq!(menu.row_at(area, 2), None); // inside a header
        assert_eq!(menu.row_at(area, 20), None); // beyond the last row
        assert_eq!(menu.row_at(area, 0), None); // on the top border
    }

    #[test]
    fn select_row_clamps_out_of_range() {
        let mut menu = default_menu();
        menu.select_row(99);
        assert_eq!(menu.selected, 0);
        menu.select_row(4); // Mouse
        assert_eq!(menu.selected, 4);
    }
}
