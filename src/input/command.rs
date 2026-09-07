use std::collections::BTreeMap;
use std::fmt;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// High-level application intents. Screen-specific input (typing a character,
/// backspace, menu navigation, confirm) stays outside this enum; anything a
/// user might want to rebind is a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Quit,
    Restart,
    Pause,
    NextTest,
    PreviousTest,
    OpenSettings,
    OpenHistory,
    OpenHelp,
    ToggleStats,
}

impl Command {
    pub const ALL: [Command; 9] = [
        Command::Quit,
        Command::Restart,
        Command::Pause,
        Command::NextTest,
        Command::PreviousTest,
        Command::OpenSettings,
        Command::OpenHistory,
        Command::OpenHelp,
        Command::ToggleStats,
    ];

    pub fn by_name(name: &str) -> Option<Command> {
        match name.to_ascii_lowercase().as_str() {
            "quit" => Some(Command::Quit),
            "restart" => Some(Command::Restart),
            "pause" => Some(Command::Pause),
            "next_test" => Some(Command::NextTest),
            "previous_test" => Some(Command::PreviousTest),
            "open_settings" => Some(Command::OpenSettings),
            "open_history" => Some(Command::OpenHistory),
            "open_help" => Some(Command::OpenHelp),
            "toggle_stats" => Some(Command::ToggleStats),
            _ => None,
        }
    }
}

/// A single key pairing, decoupled from crossterm so it can round-trip
/// through configuration as a string such as "ctrl+r" or "F2".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl Key {
    pub fn parse(spec: &str) -> Option<Key> {
        let spec = spec.trim();
        if spec.is_empty() {
            return None;
        }
        let mut modifiers = KeyModifiers::NONE;
        let mut rest = spec;
        loop {
            if let Some(tail) = rest.strip_prefix("ctrl+") {
                modifiers |= KeyModifiers::CONTROL;
                rest = tail;
            } else if let Some(tail) = rest.strip_prefix("shift+") {
                modifiers |= KeyModifiers::SHIFT;
                rest = tail;
            } else if let Some(tail) = rest.strip_prefix("alt+") {
                modifiers |= KeyModifiers::ALT;
                rest = tail;
            } else {
                break;
            }
        }
        if rest.is_empty() {
            return None;
        }
        let lowered = rest.to_ascii_lowercase();
        let code = match lowered.as_str() {
            "esc" | "escape" => KeyCode::Esc,
            "tab" => KeyCode::Tab,
            "enter" | "return" => KeyCode::Enter,
            "backspace" => KeyCode::Backspace,
            "space" => KeyCode::Char(' '),
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "delete" | "del" => KeyCode::Delete,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "insert" => KeyCode::Insert,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            _ if lowered.chars().count() == 1 => KeyCode::Char(lowered.chars().next()?),
            _ => {
                let digits = lowered.strip_prefix('f')?;
                let number: u8 = digits.parse().ok()?;
                if !(1..=12).contains(&number) {
                    return None;
                }
                KeyCode::F(number)
            }
        };
        Some(Key { code, modifiers })
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        if event.modifiers != self.modifiers {
            return false;
        }
        match (self.code, event.code) {
            (KeyCode::Char(a), KeyCode::Char(b)) => a.eq_ignore_ascii_case(&b),
            (a, b) => a == b,
        }
    }

    fn code_label(&self) -> String {
        match self.code {
            KeyCode::Char(' ') => "space".to_owned(),
            KeyCode::Char(c) => c.to_ascii_lowercase().to_string(),
            KeyCode::Esc => "esc".to_owned(),
            KeyCode::Tab => "tab".to_owned(),
            KeyCode::Enter => "enter".to_owned(),
            KeyCode::Backspace => "backspace".to_owned(),
            KeyCode::Up => "up".to_owned(),
            KeyCode::Down => "down".to_owned(),
            KeyCode::Left => "left".to_owned(),
            KeyCode::Right => "right".to_owned(),
            KeyCode::Delete => "delete".to_owned(),
            KeyCode::Home => "home".to_owned(),
            KeyCode::End => "end".to_owned(),
            KeyCode::Insert => "insert".to_owned(),
            KeyCode::PageUp => "pageup".to_owned(),
            KeyCode::PageDown => "pagedown".to_owned(),
            KeyCode::F(n) => format!("F{n}"),
            _ => format!("{self:?}"),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut modifiers: Vec<&str> = Vec::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            modifiers.push("ctrl");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            modifiers.push("shift");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            modifiers.push("alt");
        }
        let label = self.code_label();
        if modifiers.is_empty() {
            write!(f, "{label}")
        } else {
            write!(f, "{}+{label}", modifiers.join("+"))
        }
    }
}

fn default_bindings() -> Vec<(Command, Key)> {
    vec![
        (Command::Quit, Key::parse("ctrl+c").unwrap()),
        (Command::Quit, Key::parse("esc").unwrap()),
        (Command::Restart, Key::parse("ctrl+r").unwrap()),
        (Command::Restart, Key::parse("tab").unwrap()),
        (Command::Pause, Key::parse("ctrl+p").unwrap()),
        (Command::NextTest, Key::parse("F5").unwrap()),
        (Command::PreviousTest, Key::parse("F4").unwrap()),
        (Command::OpenSettings, Key::parse("F2").unwrap()),
        (Command::OpenHistory, Key::parse("F3").unwrap()),
        (Command::OpenHelp, Key::parse("F1").unwrap()),
        (Command::ToggleStats, Key::parse("ctrl+t").unwrap()),
    ]
}

/// Key -> command table. Defaults live here (and only here); user overrides
/// come from the config file, so rebinding never touches application code.
#[derive(Debug, Clone)]
pub struct Keymap {
    bindings: Vec<(Command, Key)>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            bindings: default_bindings(),
        }
    }
}

impl Keymap {
    pub fn resolve(&self, event: &KeyEvent) -> Option<Command> {
        self.bindings
            .iter()
            .find(|(_, key)| key.matches(event))
            .map(|(command, _)| *command)
    }

    pub fn binding(&self, command: Command) -> Option<Key> {
        self.bindings
            .iter()
            .find(|(candidate, _)| *candidate == command)
            .map(|(_, key)| *key)
    }

    /// Every effective binding, including secondary keys for a command
    /// (e.g. Quit is on both ctrl+c and esc). Used by the help screen.
    pub fn entries(&self) -> impl Iterator<Item = (Command, Key)> + '_ {
        self.bindings.iter().copied()
    }

    /// Applies per-command overrides from configuration. Unrecognized command
    /// names and unparseable key specs are ignored, keeping the built-in map.
    pub fn with_overrides(&self, overrides: &BTreeMap<String, String>) -> Self {
        let mut bindings = self.bindings.clone();
        for (name, spec) in overrides {
            let Some(command) = Command::by_name(name) else {
                continue;
            };
            let Some(key) = Key::parse(spec) else {
                continue;
            };
            bindings.retain(|(candidate, _)| *candidate != command);
            bindings.push((command, key));
        }
        Self { bindings }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn parses_common_key_specs() {
        assert_eq!(
            Key::parse("ctrl+c"),
            Some(Key {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            })
        );
        assert_eq!(
            Key::parse("F2"),
            Some(Key {
                code: KeyCode::F(2),
                modifiers: KeyModifiers::NONE,
            })
        );
        assert_eq!(
            Key::parse("f12"),
            Some(Key {
                code: KeyCode::F(12),
                modifiers: KeyModifiers::NONE,
            })
        );
        assert_eq!(
            Key::parse("esc"),
            Some(Key {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            })
        );
        assert_eq!(
            Key::parse("space"),
            Some(Key {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
            })
        );
        assert_eq!(
            Key::parse("shift+tab"),
            Some(Key {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::SHIFT,
            })
        );
    }

    #[test]
    fn rejects_invalid_specs() {
        assert_eq!(Key::parse(""), None);
        assert_eq!(Key::parse("ctrl+"), None);
        assert_eq!(Key::parse("ctrl++c"), None);
        assert_eq!(Key::parse("f13"), None);
        assert_eq!(Key::parse("super+meta"), None);
    }

    #[test]
    fn display_roundtrips_through_parse() {
        for spec in ["ctrl+c", "F2", "esc", "shift+tab", "ctrl+shift+p", "space"] {
            let key = Key::parse(spec).unwrap();
            assert_eq!(Key::parse(&key.to_string()), Some(key));
        }
    }

    #[test]
    fn matches_ignores_char_case_with_shift() {
        let key = Key::parse("shift+a").unwrap();
        assert!(key.matches(&event(KeyCode::Char('A'), KeyModifiers::SHIFT)));
        assert!(!key.matches(&event(KeyCode::Char('a'), KeyModifiers::NONE)));
    }

    #[test]
    fn default_map_ties_commands_to_keys() {
        let keymap = Keymap::default();
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Command::Quit)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Command::Quit)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('r'), KeyModifiers::CONTROL)),
            Some(Command::Restart)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Tab, KeyModifiers::NONE)),
            Some(Command::Restart)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('p'), KeyModifiers::CONTROL)),
            Some(Command::Pause)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::F(2), KeyModifiers::NONE)),
            Some(Command::OpenSettings)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::F(1), KeyModifiers::NONE)),
            Some(Command::OpenHelp)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::F(3), KeyModifiers::NONE)),
            Some(Command::OpenHistory)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::F(5), KeyModifiers::NONE)),
            Some(Command::NextTest)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::F(4), KeyModifiers::NONE)),
            Some(Command::PreviousTest)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('t'), KeyModifiers::CONTROL)),
            Some(Command::ToggleStats)
        );
    }

    #[test]
    fn every_command_has_an_effective_binding() {
        let keymap = Keymap::default();
        for command in Command::ALL {
            assert!(keymap.binding(command).is_some(), "{command:?} has no key");
        }
    }

    #[test]
    fn overrides_rebind_a_command_and_replace_secondary_mappings() {
        let mut overrides = BTreeMap::new();
        overrides.insert("restart".to_owned(), "ctrl+k".to_owned());
        let keymap = Keymap::default().with_overrides(&overrides);
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('k'), KeyModifiers::CONTROL)),
            Some(Command::Restart)
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Tab, KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('r'), KeyModifiers::CONTROL)),
            None
        );
        assert_eq!(keymap.binding(Command::Restart), Key::parse("ctrl+k"));
    }

    #[test]
    fn invalid_overrides_fall_back_to_builtin_defaults() {
        let mut overrides = BTreeMap::new();
        overrides.insert("pause".to_owned(), "not-a-key".to_owned());
        overrides.insert("not_a_command".to_owned(), "ctrl+q".to_owned());
        let keymap = Keymap::default().with_overrides(&overrides);
        assert_eq!(
            keymap.resolve(&event(KeyCode::Char('p'), KeyModifiers::CONTROL)),
            Some(Command::Pause)
        );
    }
}
