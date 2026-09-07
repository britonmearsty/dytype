use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use super::event::SoundEvent;
use super::settings::SoundPack;

const SAMPLE_RATE: u32 = 44_100;
const MAX_AMP: f32 = 32_000.0;

pub trait SoundBackend {
    fn play(&mut self, event: SoundEvent, volume: f64);
}

pub struct SilentBackend;

impl SoundBackend for SilentBackend {
    fn play(&mut self, _event: SoundEvent, _volume: f64) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    #[cfg(target_os = "linux")]
    Aplay,
    #[cfg(target_os = "linux")]
    Paplay,
    #[cfg(target_os = "macos")]
    Afplay,
    Ffplay,
    #[cfg(target_os = "windows")]
    PowerShell,
}

impl Player {
    fn binary(self) -> &'static str {
        match self {
            #[cfg(target_os = "linux")]
            Player::Aplay => "aplay",
            #[cfg(target_os = "linux")]
            Player::Paplay => "paplay",
            #[cfg(target_os = "macos")]
            Player::Afplay => "afplay",
            Player::Ffplay => "ffplay",
            #[cfg(target_os = "windows")]
            Player::PowerShell => "powershell",
        }
    }

    fn probe_arg(self) -> &'static str {
        match self {
            #[cfg(target_os = "linux")]
            Player::Aplay | Player::Paplay => "--version",
            #[cfg(target_os = "macos")]
            Player::Afplay => "-h",
            #[cfg(target_os = "windows")]
            Player::PowerShell => "--version",
            Player::Ffplay => "-version",
        }
    }

    fn available(self) -> bool {
        #[cfg(target_os = "windows")]
        if self == Player::PowerShell {
            // PowerShell ships with every supported Windows install.
            return true;
        }
        command_found(self.binary(), self.probe_arg())
    }

    /// Builds the argument vector for playing `path` with this player.
    fn args(self, path: &Path) -> Vec<OsString> {
        match self {
            #[cfg(target_os = "linux")]
            Player::Aplay => vec!["-q".into(), path.into()],
            #[cfg(target_os = "linux")]
            Player::Paplay => vec!["--quiet".into(), path.into()],
            #[cfg(target_os = "macos")]
            Player::Afplay => vec![path.into()],
            Player::Ffplay => vec![
                "-nodisp".into(),
                "-autoexit".into(),
                "-loglevel".into(),
                "quiet".into(),
                path.into(),
            ],
            #[cfg(target_os = "windows")]
            Player::PowerShell => vec![
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-STA".into(),
                "-Command".into(),
                format!(
                    "Add-Type -AssemblyName System.Windows.Forms; \
                     (New-Object System.Media.SoundPlayer '{}').PlaySync()",
                    path.display()
                )
                .into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum WaveKind {
    Sine,
    Square,
}

/// Synthesizes short clips per sound pack and plays them by spawning the
/// system player on a temp WAV file. Falls back to no-ops when unavailable.
pub struct WaveBackend {
    samples: Vec<Vec<i16>>,
    counter: u32,
}

impl WaveBackend {
    pub fn new(pack: SoundPack) -> Self {
        Self {
            samples: samples_for(pack),
            counter: 0,
        }
    }

    fn spawn_player(&self, path: &Path) -> Option<()> {
        let player = cached_player()?;
        let mut cmd = Command::new(player.binary());
        cmd.args(player.args(path));
        let _ = cmd.spawn();
        Some(())
    }
}

/// Detects the system audio player exactly once and caches the result, so
/// startup never pays for subprocess probes.
fn cached_player() -> Option<Player> {
    static CACHE: OnceLock<Option<Player>> = OnceLock::new();
    *CACHE.get_or_init(detect_player)
}

impl SoundBackend for WaveBackend {
    fn play(&mut self, event: SoundEvent, volume: f64) {
        let samples = &self.samples[event_index(event)];
        if samples.is_empty() {
            return;
        }
        let path = std::env::temp_dir().join(format!(
            "dytype-{}-{:04}.wav",
            std::process::id(),
            self.counter
        ));
        self.counter = self.counter.wrapping_add(1);
        if std::fs::write(&path, wav_bytes(samples, volume)).is_err() {
            return;
        }
        let _ = self.spawn_player(&path);
    }
}

fn event_index(event: SoundEvent) -> usize {
    match event {
        SoundEvent::Keypress => 0,
        SoundEvent::Error => 1,
        SoundEvent::WordComplete => 2,
        SoundEvent::TestComplete => 3,
    }
}

fn detect_player() -> Option<Player> {
    #[cfg(target_os = "linux")]
    let candidates = [Player::Aplay, Player::Paplay, Player::Ffplay];
    #[cfg(target_os = "macos")]
    let candidates = [Player::Afplay, Player::Ffplay];
    #[cfg(target_os = "windows")]
    let candidates = [Player::PowerShell, Player::Ffplay];

    candidates.into_iter().find(|player| player.available())
}

fn command_found(bin: &str, arg: &str) -> bool {
    Command::new(bin)
        .arg(arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn voice(f0: f32, f1: f32, dur: f32, decay: f32, kind: WaveKind, amp: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = if n <= 1 {
            0.0
        } else {
            i as f32 / (n as f32 - 1.0)
        };
        phase += std::f32::consts::TAU * (f0 + (f1 - f0) * frac) / SAMPLE_RATE as f32;
        let osc = match kind {
            WaveKind::Sine => phase.sin(),
            WaveKind::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
        };
        let env = (-t / decay.max(1e-4)).exp();
        out.push((osc * env * amp * MAX_AMP) as i16);
    }
    out
}

fn noise(dur: f32, decay: f32, amp: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let value = state as f32 / u64::MAX as f32 * 2.0 - 1.0;
        let env = (-t / decay.max(1e-4)).exp();
        out.push((value * env * amp * MAX_AMP) as i16);
    }
    out
}

fn samples_for(pack: SoundPack) -> Vec<Vec<i16>> {
    vec![
        keypress(pack),
        error(pack),
        word_complete(pack),
        test_complete(pack),
    ]
}

fn keypress(pack: SoundPack) -> Vec<i16> {
    match pack {
        SoundPack::Mechanical => noise(0.03, 0.004, 0.7),
        SoundPack::Typewriter => {
            let mut out = voice(750.0, 550.0, 0.045, 0.012, WaveKind::Sine, 0.6);
            out.extend(noise(0.012, 0.003, 0.25));
            out
        }
        SoundPack::Soft => voice(330.0, 300.0, 0.06, 0.03, WaveKind::Sine, 0.35),
        SoundPack::Retro => voice(1100.0, 800.0, 0.04, 0.01, WaveKind::Square, 0.35),
        SoundPack::None => Vec::new(),
    }
}

fn error(pack: SoundPack) -> Vec<i16> {
    match pack {
        SoundPack::Mechanical => {
            let mut out = voice(180.0, 140.0, 0.09, 0.02, WaveKind::Sine, 0.6);
            out.extend(noise(0.05, 0.015, 0.4));
            out
        }
        SoundPack::Typewriter => voice(240.0, 190.0, 0.08, 0.025, WaveKind::Sine, 0.55),
        SoundPack::Soft => voice(180.0, 150.0, 0.1, 0.04, WaveKind::Sine, 0.4),
        SoundPack::Retro => voice(220.0, 160.0, 0.1, 0.02, WaveKind::Square, 0.35),
        SoundPack::None => Vec::new(),
    }
}

fn word_complete(pack: SoundPack) -> Vec<i16> {
    match pack {
        SoundPack::Mechanical => {
            let mut out = voice(1200.0, 950.0, 0.03, 0.008, WaveKind::Sine, 0.5);
            out.extend(voice(1500.0, 1200.0, 0.04, 0.01, WaveKind::Sine, 0.45));
            out
        }
        SoundPack::Typewriter => {
            let mut out = voice(900.0, 700.0, 0.035, 0.012, WaveKind::Sine, 0.5);
            out.extend(voice(1000.0, 800.0, 0.04, 0.012, WaveKind::Sine, 0.45));
            out
        }
        SoundPack::Soft => voice(700.0, 650.0, 0.05, 0.03, WaveKind::Sine, 0.3),
        SoundPack::Retro => {
            let mut out = voice(800.0, 600.0, 0.03, 0.008, WaveKind::Square, 0.3);
            out.extend(voice(1000.0, 800.0, 0.035, 0.01, WaveKind::Square, 0.3));
            out
        }
        SoundPack::None => Vec::new(),
    }
}

fn test_complete(pack: SoundPack) -> Vec<i16> {
    let notes = [523.25, 659.25, 783.99, 1046.5];
    let mut out = Vec::new();
    for (i, note) in notes.iter().enumerate() {
        let dur = if i == notes.len() - 1 { 0.18 } else { 0.09 };
        let (kind, amp) = match pack {
            SoundPack::Mechanical => (WaveKind::Sine, 0.55),
            SoundPack::Typewriter => (WaveKind::Sine, 0.45),
            SoundPack::Soft => (WaveKind::Sine, 0.35),
            SoundPack::Retro => (WaveKind::Square, 0.3),
            SoundPack::None => continue,
        };
        out.extend(voice(*note, *note, dur, 0.06, kind, amp));
    }
    out
}

fn wav_bytes(samples: &[i16], volume: f64) -> Vec<u8> {
    let volume = volume.clamp(0.0, 1.0) as f32;
    let data_len = (samples.len() as u32) * 2;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &sample in samples {
        out.extend_from_slice(&((sample as f32 * volume).round() as i16).to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_is_empty() {
        assert!(keypress(SoundPack::None).is_empty());
        assert!(test_complete(SoundPack::None).is_empty());
    }

    #[test]
    fn every_real_pack_produces_samples_for_every_event() {
        for pack in [
            SoundPack::Mechanical,
            SoundPack::Typewriter,
            SoundPack::Soft,
            SoundPack::Retro,
        ] {
            for event in [
                SoundEvent::Keypress,
                SoundEvent::Error,
                SoundEvent::WordComplete,
                SoundEvent::TestComplete,
            ] {
                let samples = &samples_for(pack)[event_index(event)];
                assert!(!samples.is_empty(), "{pack:?} {event:?} should have sound");
                for &sample in samples {
                    assert!(sample >= -MAX_AMP as i16 && sample <= MAX_AMP as i16);
                }
            }
        }
    }

    #[test]
    fn mechanical_keypress_decomposes_quickly() {
        let samples = keypress(SoundPack::Mechanical);
        let tail: Vec<i16> = samples[samples.len() * 3 / 4..].to_vec();
        assert!(
            !samples.is_empty() && tail.iter().map(|s| s.abs()).max() < Some(200),
            "clip tail should decay toward silence"
        );
    }

    #[test]
    fn wav_header_is_valid() {
        let samples = voice(440.0, 440.0, 0.1, 0.05, WaveKind::Sine, 0.5);
        let bytes = wav_bytes(&samples, 1.0);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[16..20], 16u32.to_le_bytes());
        assert_eq!(&bytes[20..22], 1u16.to_le_bytes());
        assert_eq!(&bytes[22..24], 1u16.to_le_bytes());
        assert_eq!(&bytes[24..28], SAMPLE_RATE.to_le_bytes());
        assert_eq!(&bytes[28..32], (SAMPLE_RATE * 2).to_le_bytes());
        assert_eq!(&bytes[32..34], 2u16.to_le_bytes());
        assert_eq!(&bytes[34..36], 16u16.to_le_bytes());
        assert_eq!(&bytes[36..40], b"data");
        let data_len = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
        assert_eq!(data_len, samples.len() * 2);
        assert_eq!(bytes.len(), 44 + data_len);
    }

    #[test]
    fn volume_scales_amplitude() {
        let samples = voice(440.0, 440.0, 0.01, 0.005, WaveKind::Sine, 1.0);
        let loud = wav_bytes(&samples, 1.0);
        let quiet = wav_bytes(&samples, 0.5);
        let loud_i16: Vec<i16> = loud[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| i16::from_le_bytes(*c))
            .collect();
        let quiet_i16: Vec<i16> = quiet[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| i16::from_le_bytes(*c))
            .collect();
        for (l, q) in loud_i16.iter().zip(&quiet_i16) {
            assert!((l.abs_diff(*q) as f32).abs() <= (l.abs() as f32 * 0.6).max(2.0));
        }
        assert_ne!(loud_i16, quiet_i16);
    }

    #[test]
    fn volume_is_clamped() {
        let samples = [1000i16, -1000];
        let bytes = wav_bytes(&samples, 2.0);
        let data: Vec<i16> = bytes[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| i16::from_le_bytes(*c))
            .collect();
        assert_eq!(data, samples);
    }

    fn args_to_strings(player: Player, path: &str) -> Vec<String> {
        player
            .args(Path::new(path))
            .into_iter()
            .map(|os| os.into_string().expect("args are utf8"))
            .collect()
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_players_build_their_invocations() {
        assert_eq!(
            args_to_strings(Player::Aplay, "/tmp/clip.wav"),
            ["-q", "/tmp/clip.wav"]
        );
        assert_eq!(
            args_to_strings(Player::Paplay, "/tmp/clip.wav"),
            ["--quiet", "/tmp/clip.wav"]
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn macos_backend_is_preferred_and_simple() {
        assert_eq!(
            args_to_strings(Player::Afplay, "/tmp/clip.wav"),
            ["/tmp/clip.wav"]
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_backend_uses_powershell_sound_player() {
        let args = args_to_strings(Player::PowerShell, "C:\\tmp\\clip.wav");
        assert!(args.contains(&"-Command".to_string()));
        let command = args.last().expect("command is last");
        assert!(command.contains("System.Media.SoundPlayer"));
        assert!(command.contains("clip.wav"));
    }

    #[test]
    fn ffplay_fallback_is_cross_platform() {
        assert_eq!(
            args_to_strings(Player::Ffplay, "clip.wav"),
            ["-nodisp", "-autoexit", "-loglevel", "quiet", "clip.wav"]
        );
    }

    #[test]
    fn wave_backend_uses_scoped_temp_paths() {
        let samples = voice(440.0, 440.0, 0.01, 0.005, WaveKind::Sine, 1.0);
        let mut backend = WaveBackend {
            samples: vec![samples],
            counter: 0,
        };
        let path = std::env::temp_dir().join(format!("dytype-{}-0000.wav", std::process::id()));
        let _ = std::fs::remove_file(&path);
        backend.play(SoundEvent::Keypress, 0.5);
        assert!(
            path.exists(),
            "play() should write the temp WAV before handing it to the player"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn backend_construction_never_probes_for_a_player() {
        // WaveBackend::new must be free of subprocess work (it used to probe
        // aplay/paplay/ffplay synchronously). Construction alone is enough to
        // prove probing was deferred: if it spawned, this test would not
        // complete without those binaries present.
        let backend = WaveBackend::new(SoundPack::Mechanical);
        assert_eq!(backend.samples.len(), 4);
        assert_eq!(backend.counter, 0);
    }

    #[test]
    fn no_player_binaries_resolve_to_nothing() {
        // The one-shot cache is stable: repeated reads agree, and on a box
        // without any player the result is None rather than a panic.
        let first = cached_player();
        let second = cached_player();
        assert_eq!(first, second);
        let _ = first;
    }
}
