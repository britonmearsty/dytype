use ratatui::style::Color;

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
pub const THEMES: [Theme; 7] = [
    DEFAULT, MONOKAI, DRACULA, GRUVBOX, NORD, CATPPUCCIN, CUSTOM,
];

impl Theme {
    /// The theme whose `name` matches, if any.
    pub fn by_name(name: &str) -> Option<Theme> {
        THEMES
            .iter()
            .copied()
            .find(|theme| theme.name.eq_ignore_ascii_case(name))
    }

    /// Look up a theme by name, falling back to Default.
    pub fn resolve(name: &str) -> Theme {
        Self::by_name(name).unwrap_or(DEFAULT)
    }

    /// The canonical spelling of a built-in theme name, or the name unchanged.
    pub fn canonical(name: &str) -> String {
        match Self::by_name(name) {
            Some(theme) => theme.name.to_owned(),
            None => name.to_owned(),
        }
    }

    /// Numeric name for a theme, matching `by_name` (case-insensitive "Default").
    pub fn index(&self) -> usize {
        THEMES.iter().position(|theme| theme.name == self.name).unwrap_or(0)
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
            vec!["Default", "Monokai", "Dracula", "Gruvbox", "Nord", "Catppuccin", "Custom"]
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
}