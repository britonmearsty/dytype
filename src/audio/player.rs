use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AudioSettings {
    pub enabled: bool,
    pub volume: f64,
    pub sound_pack: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 1.0,
            sound_pack: "mechanical".to_owned(),
        }
    }
}

pub struct AudioPlayer;

impl AudioPlayer {
    pub fn new(_settings: &AudioSettings) -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn play_keypress(&self) {}
}