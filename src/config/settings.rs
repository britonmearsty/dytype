use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::audio::player::AudioSettings;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub audio: AudioSettings,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub word_path: Option<PathBuf>,
    #[serde(default)]
    pub cursor: CursorSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio: AudioSettings::default(),
            theme: "default".to_owned(),
            word_path: None,
            cursor: CursorSettings::default(),
        }
    }
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
    #[serde(default)]
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

impl Settings {
    pub fn file_path() -> Option<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "dytype", "dytype")?;
        Some(dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> std::io::Result<Settings> {
        let path = Settings::file_path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config dir"))?;
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        } else {
            Ok(Settings::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_toml_yields_defaults() {
        let settings: Settings = toml::from_str("").expect("defaults from empty toml");
        assert_eq!(settings.cursor.style, CursorStyle::Bar);
        assert!(settings.cursor.blink);
        assert_eq!(settings.cursor.blink_speed, 500);
        assert!(settings.audio.enabled);
    }

    #[test]
    fn parses_cursor_style_string() {
        let settings: Settings = toml::from_str("cursor.style = \"block\"\n").expect("parses");
        assert_eq!(settings.cursor.style, CursorStyle::Block);
    }

    #[test]
    fn omitted_cursor_section_defaults() {
        let settings: Settings = toml::from_str("theme = \"dark\"\n").expect("parses");
        assert_eq!(settings.cursor, CursorSettings::default());
    }
}