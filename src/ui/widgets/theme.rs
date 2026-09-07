use std::path::Path;
use std::sync::OnceLock;

use ratatui::style::Color;
use serde::Deserialize;

/// A color palette for the whole TUI. Screens never hard-code colors at
/// call sites; they read the fields here so themes are pure data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub foreground: Color,
    pub text: Color,
    pub correct: Color,
    pub incorrect: Color,
    pub cursor: Color,
    pub muted: Color,
    pub accent: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub const DEFAULT: Theme = Theme {
    name: "Default",
    background: Color::Black,
    foreground: Color::White,
    text: Color::White,
    correct: rgb(0x66, 0xD9, 0x9E),
    incorrect: rgb(0xFF, 0x6B, 0x6B),
    cursor: rgb(0xE9, 0xC4, 0x6A),
    muted: Color::Gray,
    accent: Color::Cyan,
};

pub const MONOKAI: Theme = Theme {
    name: "Monokai",
    background: rgb(0x27, 0x28, 0x22),
    foreground: rgb(0xF8, 0xF8, 0xF2),
    text: rgb(0xF8, 0xF8, 0xF2),
    correct: rgb(0xA6, 0xE2, 0x2E),
    incorrect: rgb(0xF9, 0x26, 0x72),
    cursor: rgb(0x66, 0xD9, 0xEF),
    muted: rgb(0x75, 0x71, 0x5E),
    accent: rgb(0xE6, 0xDB, 0x74),
};

pub const DRACULA: Theme = Theme {
    name: "Dracula",
    background: rgb(0x28, 0x2A, 0x36),
    foreground: rgb(0xF8, 0xF8, 0xF2),
    text: rgb(0xF8, 0xF8, 0xF2),
    correct: rgb(0x50, 0xFA, 0x7B),
    incorrect: rgb(0xFF, 0x55, 0x55),
    cursor: rgb(0xBD, 0x93, 0xF9),
    muted: rgb(0x62, 0x72, 0xA4),
    accent: rgb(0xFF, 0x79, 0xC6),
};

pub const GRUVBOX: Theme = Theme {
    name: "Gruvbox",
    background: rgb(0x28, 0x28, 0x28),
    foreground: rgb(0xEB, 0xDB, 0xB2),
    text: rgb(0xEB, 0xDB, 0xB2),
    correct: rgb(0xB8, 0xBB, 0x26),
    incorrect: rgb(0xFB, 0x49, 0x34),
    cursor: rgb(0xFA, 0xBD, 0x2F),
    muted: rgb(0x92, 0x83, 0x74),
    accent: rgb(0x83, 0xA5, 0x98),
};

pub const NORD: Theme = Theme {
    name: "Nord",
    background: rgb(0x2E, 0x34, 0x40),
    foreground: rgb(0xD8, 0xDE, 0xE9),
    text: rgb(0xEC, 0xEF, 0xF4),
    correct: rgb(0xA3, 0xBE, 0x8C),
    incorrect: rgb(0xBF, 0x61, 0x6A),
    cursor: rgb(0x88, 0xC0, 0xD0),
    muted: rgb(0x4C, 0x56, 0x6A),
    accent: rgb(0x81, 0xA1, 0xC1),
};

pub const CATPPUCCIN: Theme = Theme {
    name: "Catppuccin",
    background: rgb(0x1E, 0x1E, 0x2E),
    foreground: rgb(0xCD, 0xD6, 0xF4),
    text: rgb(0xCD, 0xD6, 0xF4),
    correct: rgb(0xA6, 0xE3, 0xA1),
    incorrect: rgb(0xF3, 0x8B, 0xA8),
    cursor: rgb(0xF5, 0xE0, 0xDC),
    muted: rgb(0x6C, 0x70, 0x86),
    accent: rgb(0x89, 0xB4, 0xFA),
};

/// Placeholder for user-defined themes. Until TOML theme files exist it
/// mirrors the Default palette.
pub const CUSTOM: Theme = Theme {
    name: "Custom",
    background: Color::Black,
    foreground: Color::White,
    text: Color::White,
    correct: rgb(0x66, 0xD9, 0x9E),
    incorrect: rgb(0xFF, 0x6B, 0x6B),
    cursor: rgb(0xE9, 0xC4, 0x6A),
    muted: Color::Gray,
    accent: Color::Cyan,
};

/// Every built-in theme, in menu order.
pub const THEMES: [Theme; 7] = [DEFAULT, MONOKAI, DRACULA, GRUVBOX, NORD, CATPPUCCIN, CUSTOM];

/// User theme files loaded from `~/.config/dytype/themes/*.toml`, resolved
/// once and reused for every lookup while the app runs.
static USER_THEMES: OnceLock<Vec<Theme>> = OnceLock::new();

/// Raw representation of a theme TOML file. Every field is optional; missing
/// colors fall back to the Default palette.
#[derive(Debug, Clone, Default, Deserialize)]
struct ThemeFile {
    name: Option<String>,
    background: Option<String>,
    foreground: Option<String>,
    text: Option<String>,
    correct: Option<String>,
    incorrect: Option<String>,
    cursor: Option<String>,
    muted: Option<String>,
    accent: Option<String>,
}

/// Parses a color written as `#RRGGBB`, `#RGB` or one of the common named
/// colors, returning `None` for anything else so themes degrade gracefully.
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let digits: Vec<u8> = hex
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .filter_map(|c| c.to_digit(16).map(|d| d as u8))
            .collect();
        return match digits.len() {
            6 => Some(Color::Rgb(
                digits[0] << 4 | digits[1],
                digits[2] << 4 | digits[3],
                digits[4] << 4 | digits[5],
            )),
            3 => Some(Color::Rgb(
                digits[0] << 4 | digits[0],
                digits[1] << 4 | digits[1],
                digits[2] << 4 | digits[2],
            )),
            _ => None,
        };
    }
    match s.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "white" => Some(Color::White),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        _ => None,
    }
}

/// Leaks an owned string so themes can carry `&'static str` names while
/// staying `Copy`. A handful of one-time theme loads at startup is fine.
fn leak(name: String) -> &'static str {
    Box::leak(name.into_boxed_str())
}

impl ThemeFile {
    fn build(self) -> Option<Theme> {
        let raw = self.name?;
        let name = raw.trim();
        if name.is_empty() {
            return None;
        }
        let base = DEFAULT;
        let color = |value: Option<String>, fallback: Color| {
            value.as_deref().and_then(parse_color).unwrap_or(fallback)
        };
        Some(Theme {
            name: leak(name.to_owned()),
            background: color(self.background, base.background),
            foreground: color(self.foreground, base.foreground),
            text: color(self.text, base.text),
            correct: color(self.correct, base.correct),
            incorrect: color(self.incorrect, base.incorrect),
            cursor: color(self.cursor, base.cursor),
            muted: color(self.muted, base.muted),
            accent: color(self.accent, base.accent),
        })
    }
}

/// Loads every valid `*.toml` theme from `dir`, skipping unreadable or
/// malformed files. Returns an empty list when the directory is absent.
pub fn load_user_themes(dir: &Path) -> Vec<Theme> {
    let mut themes = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return themes;
    };
    let mut files: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    files.sort();
    for path in files {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Ok(file) = toml::from_str::<ThemeFile>(&content)
            && let Some(theme) = file.build()
        {
            themes.push(theme);
        }
    }
    themes
}

/// The directory user themes are read from.
pub fn user_theme_dir() -> Option<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "dytype", "dytype")?;
    Some(dirs.config_dir().join("themes"))
}

/// User themes, reading them from disk on first use.
pub fn user_themes() -> &'static [Theme] {
    USER_THEMES
        .get_or_init(|| match user_theme_dir() {
            Some(dir) => load_user_themes(&dir),
            None => Vec::new(),
        })
        .as_slice()
}

impl Theme {
    /// The theme whose `name` matches among the built-ins and `user` themes.
    pub fn by_name_in(user: &[Theme], name: &str) -> Option<Theme> {
        THEMES
            .iter()
            .chain(user)
            .copied()
            .find(|theme| theme.name.eq_ignore_ascii_case(name))
    }

    /// The theme whose `name` matches, if any.
    pub fn by_name(name: &str) -> Option<Theme> {
        Self::by_name_in(user_themes(), name)
    }

    /// Look up a theme by name among built-ins and `user` themes.
    pub fn resolve_in(user: &[Theme], name: &str) -> Option<Theme> {
        Self::by_name_in(user, name)
    }

    /// Look up a theme by name, falling back to Default.
    pub fn resolve(name: &str) -> Theme {
        Self::by_name(name).unwrap_or(DEFAULT)
    }

    /// The canonical spelling of a theme name (built-in or user), or the
    /// name unchanged.
    pub fn canonical(name: &str) -> String {
        match Self::by_name(name) {
            Some(theme) => theme.name.to_owned(),
            None => name.to_owned(),
        }
    }

    /// Every selectable theme name, built-ins first then user themes, in
    /// the order the settings picker cycles through.
    pub fn display_names() -> Vec<String> {
        THEMES
            .iter()
            .map(|theme| theme.name.to_owned())
            .chain(user_themes().iter().map(|theme| theme.name.to_owned()))
            .collect()
    }

    /// Numeric name for a theme, matching `by_name` (case-insensitive "Default").
    pub fn index(&self) -> usize {
        THEMES
            .iter()
            .position(|theme| theme.name == self.name)
            .unwrap_or(0)
    }

    /// Convert a `Color` into animation-friendly RGB triples. Named colors
    /// map to close approximations; RGB colors pass through unchanged.
    pub fn rgb(&self, color: Color) -> (u8, u8, u8) {
        match color {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Black => (0, 0, 0),
            Color::Gray => (128, 128, 128),
            Color::DarkGray => (64, 64, 64),
            Color::White => (255, 255, 255),
            Color::Green => (0x66, 0xD9, 0x9E),
            Color::Cyan => (0x56, 0xD3, 0xE2),
            Color::Red => (0xFF, 0x6B, 0x6B),
            _ => match self.cursor {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (255, 255, 255),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_all_seven_builtin_themes() {
        assert_eq!(THEMES.len(), 7);
        let names: Vec<_> = THEMES.iter().map(|theme| theme.name).collect();
        assert_eq!(
            names,
            vec![
                "Default",
                "Monokai",
                "Dracula",
                "Gruvbox",
                "Nord",
                "Catppuccin",
                "Custom"
            ]
        );
        for theme in &THEMES {
            assert!(!theme.name.is_empty());
            assert_ne!(theme.background, theme.text);
        }
    }

    #[test]
    fn resolves_by_name_case_insensitively() {
        assert_eq!(Theme::resolve("monokai"), MONOKAI);
        assert_eq!(Theme::resolve("CATPPUCCIN"), CATPPUCCIN);
        assert_eq!(Theme::resolve("nope"), DEFAULT);
        assert_eq!(Theme::by_name("nope"), None);
    }

    #[test]
    fn palettes_really_differ() {
        assert_ne!(DEFAULT, MONOKAI);
        assert_ne!(MONOKAI, DRACULA);
        assert_ne!(DRACULA, GRUVBOX);
        assert_ne!(GRUVBOX, NORD);
        assert_ne!(NORD, CATPPUCCIN);
        assert_ne!(CATPPUCCIN, CUSTOM);
    }

    #[test]
    fn names_roundtrip_through_index() {
        for theme in &THEMES {
            assert_eq!(theme.name, THEMES[theme.index()].name);
        }
    }

    #[test]
    fn rgb_conversion_roundtrips_and_approximates() {
        assert_eq!(CATPPUCCIN.rgb(rgb(0x1E, 0x1E, 0x2E)), (0x1E, 0x1E, 0x2E));
        assert_eq!(CATPPUCCIN.rgb(Color::Black), (0, 0, 0));
        assert_eq!(CATPPUCCIN.rgb(Color::White), (255, 255, 255));
    }

    #[test]
    fn index_counts_are_correct() {
        assert_eq!(DEFAULT.index(), 0);
        assert_eq!(MONOKAI.index(), 1);
        assert_eq!(CATPPUCCIN.index(), 5);
        assert_eq!(CUSTOM.index(), 6);
    }

    #[test]
    fn parse_color_accepts_hex_and_named() {
        assert_eq!(parse_color("#66d99e"), Some(rgb(0x66, 0xD9, 0x9E)));
        assert_eq!(parse_color("#FFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(parse_color("black"), Some(Color::Black));
        assert_eq!(parse_color("GRAY"), Some(Color::Gray));
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("navy"), None);
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn theme_file_defaults_missing_colors_to_default_palette() {
        let file = ThemeFile {
            name: Some("Sunset".to_owned()),
            correct: Some("#ff0000".to_owned()),
            ..ThemeFile::default()
        };
        let theme = file.build().expect("theme from partial file");
        assert_eq!(theme.name, "Sunset");
        assert_eq!(theme.correct, rgb(0xFF, 0, 0));
        assert_eq!(theme.background, DEFAULT.background);
        assert_eq!(theme.accent, DEFAULT.accent);
    }

    #[test]
    fn theme_file_requires_a_name() {
        let none = ThemeFile {
            name: None,
            ..ThemeFile::default()
        };
        let blank = ThemeFile {
            name: Some("  ".to_owned()),
            ..ThemeFile::default()
        };
        assert!(none.build().is_none());
        assert!(blank.build().is_none());
    }

    #[test]
    fn user_themes_merge_after_builtins_and_resolve() {
        let dir = std::env::temp_dir().join(format!("dytype-themes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp themes dir");
        std::fs::write(
            dir.join("sunset.toml"),
            "name = \"Sunset\"\ncorrect = \"#ff0000\"\n",
        )
        .expect("write sunset");
        std::fs::write(dir.join("broken.toml"), "not = valid toml [").expect("write broken");

        let themes = load_user_themes(&dir);
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "Sunset");
        assert_eq!(themes[0].correct, rgb(0xFF, 0, 0));
        assert_eq!(themes[0].background, DEFAULT.background);

        assert_eq!(Theme::by_name_in(&themes, "sunset"), Some(themes[0]));
        assert_eq!(Theme::resolve_in(&themes, "Sunset"), Some(themes[0]));

        let builtins = THEMES.iter().map(|theme| theme.name.to_owned());
        let names: Vec<String> = builtins
            .chain(themes.iter().map(|theme| theme.name.to_owned()))
            .collect();
        assert!(names.starts_with(&["Default".to_owned(), "Monokai".to_owned()]));
        assert!(names.contains(&"Sunset".to_owned()));
    }
}
