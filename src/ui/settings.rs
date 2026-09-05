use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::audio::settings::{AudioSettings, SoundPack};
use crate::config::settings::Settings;

const PACKS: [SoundPack; 5] = [
    SoundPack::Mechanical,
    SoundPack::Typewriter,
    SoundPack::Soft,
    SoundPack::Retro,
    SoundPack::None,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Sounds,
    Volume,
    Pack,
    Error,
    Complete,
}

impl Row {
    const ALL: [Row; 5] = [Row::Sounds, Row::Volume, Row::Pack, Row::Error, Row::Complete];

    fn label(self) -> &'static str {
        match self {
            Row::Sounds => "Typing sounds",
            Row::Volume => "Volume",
            Row::Pack => "Sound pack",
            Row::Error => "Error sound",
            Row::Complete => "Completion sound",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsMenu {
    sounds: bool,
    volume_pct: u8,
    pack: usize,
    error: bool,
    complete: bool,
    selected: usize,
}

impl SettingsMenu {
    pub fn from_settings(settings: &Settings) -> Self {
        let audio = &settings.audio;
        Self {
            sounds: audio.enabled,
            volume_pct: (audio.volume * 100.0).round().clamp(0.0, 100.0) as u8,
            pack: PACKS
                .iter()
                .position(|&pack| pack == audio.sound_pack)
                .unwrap_or(0),
            error: audio.error,
            complete: audio.complete,
            selected: 0,
        }
    }

    pub fn apply(&self) -> AudioSettings {
        AudioSettings {
            enabled: self.sounds,
            volume: f64::from(self.volume_pct) / 100.0,
            sound_pack: PACKS[self.pack],
            error: self.error,
            complete: self.complete,
        }
    }

    pub fn move_selection(&mut self, dir: i8) {
        self.selected =
            (self.selected as i8 + dir).rem_euclid(Row::ALL.len() as i8) as usize;
    }

    pub fn cycle(&mut self, dir: i8) {
        match Row::ALL[self.selected] {
            Row::Sounds => self.sounds = !self.sounds,
            Row::Volume => {
                self.volume_pct =
                    ((self.volume_pct as i16 + dir as i16 * 5).clamp(0, 100)) as u8;
            }
            Row::Pack => {
                self.pack = (self.pack as i8 + dir).rem_euclid(PACKS.len() as i8) as usize;
            }
            Row::Error => self.error = !self.error,
            Row::Complete => self.complete = !self.complete,
        }
    }

    fn on_off(value: bool) -> &'static str {
        if value {
            "ON"
        } else {
            "OFF"
        }
    }

    fn value_text(&self, row: Row) -> String {
        match row {
            Row::Sounds => Self::on_off(self.sounds).to_owned(),
            Row::Volume => format!("{}%", self.volume_pct),
            Row::Pack => PACKS[self.pack].label().to_owned(),
            Row::Error => Self::on_off(self.error).to_owned(),
            Row::Complete => Self::on_off(self.complete).to_owned(),
        }
    }
}

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let menu = &app.settings_menu;
        let mut lines = vec![Line::from("")];
        for (i, row) in Row::ALL.iter().enumerate() {
            let selected = i == menu.selected;
            let marker = if selected { "▸" } else { " " };
            let value_style = if selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("   {marker} {:<18}", row.label()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(menu.value_text(*row), value_style),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from("   ↑/↓ move   ←/→ change   Esc back"));
        let block = Block::default().title(" settings ").borders(Borders::ALL);
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_menu() -> SettingsMenu {
        SettingsMenu::from_settings(&Settings::default())
    }

    #[test]
    fn default_menu_applies_to_default_settings() {
        assert_eq!(default_menu().apply(), AudioSettings::default());
    }

    #[test]
    fn toggling_rows_flips_booleans() {
        let mut menu = default_menu();
        menu.cycle(1); // Sounds -> OFF
        assert!(!menu.apply().enabled);
        menu.move_selection(1); // -> Volume
        menu.move_selection(1); // -> Pack
        menu.move_selection(1); // -> Error
        menu.cycle(1);
        assert!(!menu.apply().error);
        menu.move_selection(1); // -> Complete
        menu.cycle(1);
        assert!(!menu.apply().complete);
    }

    #[test]
    fn volume_cycles_in_five_percent_steps_and_clamps() {
        let mut menu = default_menu();
        menu.move_selection(1); // Volume
        menu.cycle(1);
        assert_eq!(menu.apply().volume, 0.7);
        for _ in 0..60 {
            menu.cycle(-1);
        }
        assert_eq!(menu.apply().volume, 0.0);
        for _ in 0..60 {
            menu.cycle(1);
        }
        assert_eq!(menu.apply().volume, 1.0);
    }

    #[test]
    fn pack_cycles_through_all_packs() {
        let mut menu = default_menu();
        menu.move_selection(2); // Pack
        menu.cycle(4);
        assert_eq!(menu.apply().sound_pack, SoundPack::None);
        menu.cycle(1);
        assert_eq!(menu.apply().sound_pack, SoundPack::Mechanical);
    }

    #[test]
    fn from_settings_roundtrips_non_defaults() {
        let settings = Settings {
            audio: AudioSettings {
                enabled: false,
                volume: 0.4,
                sound_pack: SoundPack::Retro,
                error: false,
                complete: true,
            },
            ..Default::default()
        };
        assert_eq!(
            SettingsMenu::from_settings(&settings).apply(),
            settings.audio
        );
    }

    #[test]
    fn selection_wraps_around() {
        let mut menu = default_menu();
        menu.move_selection(-1);
        assert_eq!(menu.selected, 4);
    }
}