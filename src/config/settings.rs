use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::audio::player::AudioSettings;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub audio: AudioSettings,
    pub theme: String,
    pub word_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio: AudioSettings::default(),
            theme: "default".to_owned(),
            word_path: None,
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