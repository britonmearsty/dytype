use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SoundPack {
    #[default]
    Mechanical,
    Typewriter,
    Soft,
    Retro,
    None,
}

impl SoundPack {
    pub fn label(self) -> &'static str {
        match self {
            SoundPack::Mechanical => "Mechanical",
            SoundPack::Typewriter => "Typewriter",
            SoundPack::Soft => "Soft",
            SoundPack::Retro => "Retro",
            SoundPack::None => "None",
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_volume() -> f64 {
    0.65
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 0.0 ..= 1.0 playback amplitude.
    #[serde(default = "default_volume")]
    pub volume: f64,
    #[serde(default)]
    pub sound_pack: SoundPack,
    #[serde(default = "default_true")]
    pub error: bool,
    #[serde(default = "default_true")]
    pub complete: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.65,
            sound_pack: SoundPack::Mechanical,
            error: true,
            complete: true,
        }
    }
}