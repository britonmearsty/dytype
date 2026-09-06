use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::audio::settings::SoundSettings;
use crate::typing::generator::Difficulty;

fn default_true() -> bool {
    true
}

fn default_theme_name() -> String {
    "Default".to_owned()
}

fn default_language() -> String {
    "English".to_owned()
}

fn default_fps() -> u16 {
    60
}

fn canonical_theme_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    Ok(crate::ui::widgets::theme::Theme::canonical(&name))
}

/// Centralized, persisted configuration. The fields mirror the settings UI:
/// appearance + behavior live in `display`, the test itself in `typing`,
/// and the audio out in `sounds`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeSettings,
    #[serde(default)]
    pub typing: TypingSettings,
    #[serde(default)]
    pub sounds: SoundSettings,
    #[serde(default)]
    pub keybindings: KeybindingSettings,
    #[serde(default)]
    pub display: DisplaySettings,
    #[serde(default)]
    pub word_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeSettings {
    #[serde(default = "default_theme_name", deserialize_with = "canonical_theme_name")]
    pub name: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
        }
    }
}

/// Options that shape the test itself (shared with the test-setup menu).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypingSettings {
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub punctuation: bool,
    #[serde(default)]
    pub numbers: bool,
    #[serde(default = "default_true")]
    pub backspace: bool,
}

impl Default for TypingSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            language: default_language(),
            punctuation: false,
            numbers: false,
            backspace: true,
        }
    }
}

/// Per-command key overrides, as `command = "key"` pairs. Unknown commands
/// and unparseable keys are ignored; the built-in defaults fill the gaps.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeybindingSettings {
    #[serde(default)]
    pub overrides: BTreeMap<String, String>,
}

/// Everything about how the UI looks and animates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplaySettings {
    #[serde(default)]
    pub cursor: CursorSettings,
    #[serde(default)]
    pub cursor_animation: CursorAnimation,
    #[serde(default = "default_true")]
    pub animations: bool,
    #[serde(default = "default_fps")]
    pub fps: u16,
    /// Maximum prompt width in cells; 0 means "fill the terminal".
    #[serde(default)]
    pub text_width: u16,
    #[serde(default)]
    pub compact_mode: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            cursor: CursorSettings::default(),
            cursor_animation: CursorAnimation::Smooth,
            animations: true,
            fps: default_fps(),
            text_width: 0,
            compact_mode: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorAnimation {
    #[default]
    Smooth,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    #[default]
    Bar,
    Block,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorSettings {
    #[serde(default)]
    pub style: CursorStyle,
    #[serde(default = "default_true")]
    pub blink: bool,
    #[serde(default)]
    pub blink_speed: u32,
}

impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            style: CursorStyle::Bar,
            blink: true,
            blink_speed: 500,
        }
    }
}

impl Config {
    pub fn file_path() -> Option<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "dytype", "dytype")?;
        Some(dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> std::io::Result<Config> {
        let path = Config::file_path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config dir"))?;
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Config::file_path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config dir"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string(self)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::settings::SoundPack;

    #[test]
    fn empty_toml_yields_defaults() {
        let config: Config = toml::from_str("").expect("defaults from empty toml");
        assert_eq!(config.theme.name, "Default");
        assert_eq!(
            config.typing.difficulty,
            Difficulty::Normal
        );
        assert!(config.sounds.enabled);
        assert_eq!(config.display.cursor.style, CursorStyle::Bar);
        assert!(config.display.cursor.blink);
        assert_eq!(config.display.cursor.blink_speed, 500);
        assert_eq!(config.display.fps, 60);
        assert!(config.display.animations);
    }

    #[test]
    fn parses_cursor_style_string() {
        let config: Config = toml::from_str("display.cursor.style = \"block\"\n").expect("parses");
        assert_eq!(config.display.cursor.style, CursorStyle::Block);
    }

    #[test]
    fn parses_nested_theme_and_typing() {
        let config: Config = toml::from_str(
            "[theme]\nname = \"monokai\"\n[typing]\ndifficulty = \"hard\"\npunctuation = true\n",
        )
        .expect("parses nested");
        assert_eq!(config.theme.name, "Monokai");
        assert_eq!(config.typing.difficulty, Difficulty::Hard);
        assert!(config.typing.punctuation);
        assert!(!config.typing.numbers);
    }

    #[test]
    fn omitted_cursor_section_defaults() {
        let config: Config = toml::from_str("theme.name = \"dark\"\n").expect("parses");
        assert_eq!(config.display.cursor, CursorSettings::default());
    }

    #[test]
    fn old_sound_config_gains_new_fields_by_default() {
        let config: Config = toml::from_str(
            "[sounds]\nenabled = true\nvolume = 1.0\nsound_pack = \"mechanical\"\n",
        )
        .expect("old sound config parses");
        assert!(config.sounds.enabled);
        assert_eq!(config.sounds.volume, 1.0);
        assert_eq!(config.sounds.sound_pack, SoundPack::Mechanical);
        assert!(config.sounds.error);
        assert!(config.sounds.complete);
    }

    #[test]
    fn sound_pack_parses_lowercase() {
        for (pack, text) in [
            (SoundPack::Mechanical, "mechanical"),
            (SoundPack::Typewriter, "typewriter"),
            (SoundPack::Soft, "soft"),
            (SoundPack::Retro, "retro"),
            (SoundPack::None, "none"),
        ] {
            let config: Config =
                toml::from_str(&format!("[sounds]\nsound_pack = \"{text}\"\n")).expect("parses");
            assert_eq!(config.sounds.sound_pack, pack);
        }
    }

    #[test]
    fn config_roundtrip_through_toml() {
        let config = Config::default();
        let mut with_theme = config.clone();
        with_theme.theme.name = "Nord".to_owned();
        with_theme.display.cursor_animation = CursorAnimation::Off;
        with_theme.typing.numbers = true;
        let encoded = toml::to_string(&with_theme).expect("serializes");
        let decoded: Config = toml::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded, with_theme);

        let recanonical: Config =
            toml::from_str(&encoded.replace("\"Nord\"", "\"nord\"")).expect("recanonicalizes");
        assert_eq!(recanonical.theme.name, "Nord");
    }

    #[test]
    fn keybinding_overrides_parse_and_roundtrip() {
        let config: Config = toml::from_str(
            "[keybindings.overrides]\nrestart = \"ctrl+k\"\ntoggle_stats = \"F8\"\n",
        )
        .expect("parses keybindings");
        assert_eq!(
            config.keybindings.overrides.get("restart"),
            Some(&"ctrl+k".to_owned())
        );
        assert_eq!(config.keybindings.overrides.len(), 2);

        let encoded = toml::to_string(&config).expect("serializes keybindings");
        let decoded: Config = toml::from_str(&encoded).expect("roundtrips keybindings");
        assert_eq!(decoded.keybindings, config.keybindings);
    }

    #[test]
    fn missing_keybindings_section_is_empty() {
        let config: Config = toml::from_str("theme.name = \"Dracula\"\n").expect("parses");
        assert!(config.keybindings.overrides.is_empty());
    }
}