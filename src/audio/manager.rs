use super::backend::{SilentBackend, SoundBackend, WaveBackend};
use super::event::SoundEvent;
use super::settings::{AudioSettings, SoundPack};

/// Owns the active sound backend and routes engine events to it, honoring the
/// user's audio settings. The engine and UI never touch backends directly.
pub struct AudioManager {
    backend: Box<dyn SoundBackend>,
}

impl AudioManager {
    pub fn new(settings: &AudioSettings) -> std::io::Result<Self> {
        Ok(Self {
            backend: Self::build_backend(settings),
        })
    }

    fn build_backend(settings: &AudioSettings) -> Box<dyn SoundBackend> {
        if settings.sound_pack == SoundPack::None {
            Box::new(SilentBackend)
        } else if let Some(backend) = WaveBackend::new(settings.sound_pack) {
            Box::new(backend)
        } else {
            Box::new(SilentBackend)
        }
    }

    pub fn reconfigure(&mut self, settings: &AudioSettings) {
        self.backend = Self::build_backend(settings);
    }

    pub fn emit(&mut self, event: SoundEvent, settings: &AudioSettings) {
        if !settings.enabled {
            return;
        }
        let allowed = match event {
            SoundEvent::Error => settings.error,
            SoundEvent::TestComplete => settings.complete,
            _ => true,
        };
        if !allowed {
            return;
        }
        self.backend.play(event, settings.volume.clamp(0.0, 1.0));
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::super::backend::SoundBackend;
    use super::*;

    type SoundLog = Rc<RefCell<Vec<(SoundEvent, f64)>>>;

    #[derive(Default)]
    struct Recording {
        log: SoundLog,
    }

    impl SoundBackend for Recording {
        fn play(&mut self, event: SoundEvent, volume: f64) {
            self.log.borrow_mut().push((event, volume));
        }
    }

    fn manager_with_recording() -> (AudioManager, SoundLog) {
        let log = Rc::new(RefCell::new(Vec::new()));
        let manager = AudioManager {
            backend: Box::new(Recording { log: log.clone() }),
        };
        (manager, log)
    }

    #[test]
    fn disabled_master_toggle_silences_everything() {
        let (mut manager, log) = manager_with_recording();
        let settings = AudioSettings {
            enabled: false,
            ..Default::default()
        };
        manager.emit(SoundEvent::Keypress, &settings);
        manager.emit(SoundEvent::Error, &settings);
        manager.emit(SoundEvent::TestComplete, &settings);
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn error_and_complete_toggles_gate_their_events() {
        let (mut manager, log) = manager_with_recording();
        let settings = AudioSettings {
            error: false,
            complete: false,
            ..Default::default()
        };
        manager.emit(SoundEvent::Error, &settings);
        manager.emit(SoundEvent::WordComplete, &settings);
        manager.emit(SoundEvent::TestComplete, &settings);
        manager.emit(SoundEvent::Keypress, &settings);
        let played = log.borrow();
        assert_eq!(
            played.as_slice(),
            &[
                (SoundEvent::WordComplete, 0.65),
                (SoundEvent::Keypress, 0.65)
            ]
        );
    }

    #[test]
    fn volume_is_clamped_to_unit_range() {
        let (mut manager, log) = manager_with_recording();
        let loud = AudioSettings {
            volume: 1.7,
            ..Default::default()
        };
        manager.emit(SoundEvent::Keypress, &loud);
        let muted = AudioSettings {
            volume: -0.3,
            ..Default::default()
        };
        manager.emit(SoundEvent::Keypress, &muted);
        let played = log.borrow();
        assert_eq!(
            played.as_slice(),
            &[(SoundEvent::Keypress, 1.0), (SoundEvent::Keypress, 0.0)]
        );
    }

    #[test]
    fn none_pack_builds_silent_backend() {
        let settings = AudioSettings {
            sound_pack: SoundPack::None,
            ..Default::default()
        };
        let mut manager = AudioManager::new(&settings).expect("manager");
        manager.emit(SoundEvent::TestComplete, &settings);
        manager.emit(SoundEvent::Keypress, &settings);
        // No crash on a temperamental backend; silent by construction.
    }
}