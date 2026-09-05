use std::time::{Duration, Instant};

use crate::typing::test::TypingTest;

pub type Rgb = (u8, u8, u8);

pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let part = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (part(a.0, b.0), part(a.1, b.1), part(a.2, b.2))
}

/// A one-shot animation that moves from `from` to `to` over `duration`,
/// starting at `start`, and holds `to` after completion.
#[derive(Debug, Clone, Copy)]
pub struct Tween {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
}

impl Tween {
    pub fn new(from: f32, to: f32, start: Instant, duration: Duration) -> Self {
        Self {
            from,
            to,
            start,
            duration,
        }
    }

    pub fn value(&self, now: Instant) -> f32 {
        if now <= self.start {
            return self.from;
        }
        let elapsed = now.duration_since(self.start).as_secs_f32();
        let total = self.duration.as_secs_f32().max(1e-6);
        let t = (elapsed / total).min(1.0);
        lerp(self.from, self.to, ease_out_cubic(t))
    }

    pub fn finished(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start) >= self.duration
    }
}

/// A cursor position that smooth-chases its target. Frame-rate independent.
#[derive(Debug, Clone, Copy)]
pub struct CursorAnimation {
    pub current_x: f32,
    pub target_x: f32,
}

impl CursorAnimation {
    pub fn new(at: f32) -> Self {
        Self {
            current_x: at,
            target_x: at,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target_x = target;
    }

    /// Advance toward the target by an exponential step derived from `dt`.
    pub fn update(&mut self, dt: Duration, tau: Duration) {
        let factor = 1.0 - (-dt.as_secs_f32() / tau.as_secs_f32().max(1e-3)).exp();
        self.current_x += (self.target_x - self.current_x) * factor;
        if (self.current_x - self.target_x).abs() < 0.03 {
            self.current_x = self.target_x;
        }
    }
}

/// Smooth sine-based blink anchored at an epoch.
#[derive(Debug, Clone, Copy)]
pub struct Blinker {
    epoch: Instant,
}

impl Blinker {
    pub fn new(epoch: Instant) -> Self {
        Self { epoch }
    }

    /// Visibility in `0.0..=1.0` (never fully hidden).
    pub fn alpha(&self, now: Instant, period_ms: u32) -> f32 {
        let elapsed = now.saturating_duration_since(self.epoch).as_secs_f32();
        let period = (period_ms as f32 / 1000.0).max(1e-3);
        let wave = (elapsed / period * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        0.25 + 0.75 * wave
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Emphasize,
    Error,
}

#[derive(Debug, Clone, Copy)]
struct CharEffect {
    index: usize,
    kind: EffectKind,
    start: Instant,
}

const EFFECT_LIFETIME: Duration = Duration::from_millis(260);

pub struct CharEffects {
    effects: Vec<CharEffect>,
}

impl CharEffects {
    pub fn new() -> Self {
        Self {
            effects: Vec::with_capacity(24),
        }
    }

    pub fn trigger(&mut self, index: usize, kind: EffectKind, now: Instant) {
        if let Some(effect) = self.effects.iter_mut().find(|e| e.index == index) {
            *effect = CharEffect { index, kind, start: now };
            return;
        }
        self.effects.push(CharEffect { index, kind, start: now });
        if self.effects.len() > 128 {
            self.effects.remove(0);
        }
    }

    pub fn prune(&mut self, now: Instant) {
        self.effects
            .retain(|e| now.saturating_duration_since(e.start) < EFFECT_LIFETIME);
    }

    /// Returns `(kind, progress)` where progress is `0.0` just after
    /// triggering and approaches `1.0` as the effect expires.
    pub fn progress(&self, index: usize, now: Instant) -> Option<(EffectKind, f32)> {
        let effect = self.effects.iter().find(|e| e.index == index)?;
        let age = now.saturating_duration_since(effect.start).as_secs_f32();
        let total = EFFECT_LIFETIME.as_secs_f32();
        (age < total).then(|| (effect.kind, (age / total).clamp(0.0, 1.0)))
    }
}

impl Default for CharEffects {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ResultsAnimation {
    wpm: Option<Tween>,
    raw: Option<Tween>,
    accuracy: Option<Tween>,
    consistency: Option<Tween>,
    title: Option<Tween>,
}

impl ResultsAnimation {
    pub fn new() -> Self {
        Self {
            wpm: None,
            raw: None,
            accuracy: None,
            consistency: None,
            title: None,
        }
    }

    pub fn start(&mut self, wpm: f32, raw: f32, accuracy: f32, consistency: f32, now: Instant) {
        let duration = Duration::from_millis(900);
        let delay = Duration::from_millis(100);
        self.wpm = Some(Tween::new(0.0, wpm, now, duration));
        self.raw = Some(Tween::new(0.0, raw, now + delay, duration));
        self.accuracy = Some(Tween::new(0.0, accuracy, now + delay * 2, duration));
        self.consistency = Some(Tween::new(0.0, consistency, now + delay * 3, duration));
        self.title = Some(Tween::new(0.0, 1.0, now, Duration::from_millis(400)));
    }

    pub fn values(&self, now: Instant) -> (f32, f32, f32, f32) {
        let value = |tween: &Option<Tween>| tween.map_or(0.0, |t| t.value(now));
        (
            value(&self.wpm),
            value(&self.raw),
            value(&self.accuracy),
            value(&self.consistency),
        )
    }

    pub fn title_alpha(&self, now: Instant) -> f32 {
        self.title.map_or(0.0, |t| t.value(now))
    }
}

impl Default for ResultsAnimation {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-test animation state owned by the app.
pub struct Animations {
    pub frame_now: Instant,
    last_now: Instant,
    pub cursor: CursorAnimation,
    cursor_target: usize,
    char_effects: CharEffects,
    word_landing: Option<Tween>,
    blinker: Blinker,
    pub results: ResultsAnimation,
    keystroke_count: usize,
    last_completed_words: usize,
}

impl Animations {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            frame_now: now,
            last_now: now,
            cursor: CursorAnimation::new(0.0),
            cursor_target: 0,
            char_effects: CharEffects::new(),
            word_landing: None,
            blinker: Blinker::new(now),
            results: ResultsAnimation::new(),
            keystroke_count: 0,
            last_completed_words: 0,
        }
    }

    pub fn mark_test_start(&mut self, now: Instant) {
        self.frame_now = now;
        self.last_now = now;
        self.cursor = CursorAnimation::new(0.0);
        self.cursor_target = 0;
        self.char_effects = CharEffects::new();
        self.word_landing = None;
        self.blinker = Blinker::new(now);
        self.results = ResultsAnimation::new();
        self.keystroke_count = 0;
        self.last_completed_words = 0;
    }

    pub fn update(&mut self, now: Instant) {
        let dt = now.saturating_duration_since(self.last_now);
        self.last_now = now;
        self.frame_now = now;
        self.cursor.update(dt, Duration::from_millis(70));
        self.char_effects.prune(now);
        if self.word_landing.as_ref().is_some_and(|t| t.finished(now)) {
            self.word_landing = None;
        }
    }

    /// Inspects the typing test after a key press to drive cursor, flashes,
    /// and word-completion pop.
    pub fn observe(&mut self, test: &TypingTest, now: Instant) {
        let start = self.keystroke_count.min(test.keystrokes.len());
        for stroke in &test.keystrokes[start..] {
            let kind = if stroke.correct {
                EffectKind::Emphasize
            } else {
                EffectKind::Error
            };
            self.char_effects.trigger(stroke.position, kind, now);
        }
        self.keystroke_count = test.keystrokes.len();
        if test.completed_words() > self.last_completed_words {
            self.last_completed_words = test.completed_words();
            self.word_landing = Some(Tween::new(1.0, 0.0, now, Duration::from_millis(260)));
        }
        self.set_cursor_target(test.cursor);
    }

    pub fn set_cursor_target(&mut self, index: usize) {
        self.cursor_target = index;
        self.cursor.set_target(index as f32);
    }

    pub fn char_effect(&self, index: usize, now: Instant) -> Option<(EffectKind, f32)> {
        self.char_effects.progress(index, now)
    }

    pub fn blink_alpha(&self, now: Instant, period_ms: u32) -> f32 {
        self.blinker.alpha(now, period_ms)
    }

    pub fn word_landing(&self, now: Instant) -> f32 {
        self.word_landing.map_or(0.0, |t| t.value(now))
    }
}

impl Default for Animations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Instant {
        Instant::now()
    }

    #[test]
    fn ease_bounds_and_monotonic() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        let mut prev = 0.0;
        for i in 0..=100 {
            let value = ease_out_cubic(i as f32 / 100.0);
            assert!(value >= prev - 1e-6);
            prev = value;
        }
    }

    #[test]
    fn lerp_interpolates() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(10.0, 20.0, 0.0), 10.0);
        assert_eq!(lerp(10.0, 20.0, 1.0), 20.0);
    }

    #[test]
    fn mix_blends_rgb() {
        assert_eq!(mix((0, 0, 0), (255, 255, 255), 0.5), (128, 128, 128));
        assert_eq!(mix((0, 0, 0), (255, 255, 255), 0.0), (0, 0, 0));
        assert_eq!(mix((0, 0, 0), (255, 255, 255), 1.0), (255, 255, 255));
    }

    #[test]
    fn tween_holds_after_completion() {
        let start = base();
        let tween = Tween::new(0.0, 100.0, start, Duration::from_millis(100));
        assert_eq!(tween.value(start), 0.0);
        assert!(!tween.finished(start));
        assert_eq!(tween.value(start + Duration::from_millis(50)), 87.5);
        assert_eq!(tween.value(start + Duration::from_millis(500)), 100.0);
        assert!(tween.finished(start + Duration::from_millis(500)));
    }

    #[test]
    fn tween_waiting_for_delayed_start_stays() {
        let start = base();
        let tween = Tween::new(0.0, 10.0, start + Duration::from_millis(100), Duration::from_secs(1));
        assert_eq!(tween.value(start), 0.0);
        assert_eq!(tween.value(start + Duration::from_millis(99)), 0.0);
    }

    #[test]
    fn cursor_chases_and_snaps() {
        let mut cursor = CursorAnimation::new(0.0);
        cursor.set_target(10.0);
        for _ in 0..200 {
            cursor.update(Duration::from_millis(16), Duration::from_millis(70));
        }
        assert_eq!(cursor.current_x, 10.0);
        assert_eq!(cursor.current_x, cursor.target_x);
    }

    #[test]
    fn cursor_moves_in_small_steps_without_overshoot() {
        let mut cursor = CursorAnimation::new(0.0);
        cursor.set_target(5.0);
        let mut last = 0.0;
        for _ in 0..20 {
            cursor.update(Duration::from_millis(16), Duration::from_millis(70));
            assert!(cursor.current_x > last - 1e-6);
            assert!(cursor.current_x <= 5.0 + 1e-6);
            last = cursor.current_x;
        }
    }

    #[test]
    fn blinker_stays_in_visible_range() {
        let blinker = Blinker::new(base());
        for ms in 0..2000 {
            let alpha = blinker.alpha(base() + Duration::from_millis(ms), 500);
            assert!((0.25..=1.0).contains(&alpha));
        }
    }

    #[test]
    fn char_effect_trigger_and_progress() {
        let start = base();
        let mut effects = CharEffects::new();
        effects.trigger(3, EffectKind::Emphasize, start);
        let (kind, progress) = effects.progress(3, start).unwrap();
        assert_eq!(kind, EffectKind::Emphasize);
        assert_eq!(progress, 0.0);
        let (_, progress) = effects
            .progress(3, start + Duration::from_millis(130))
            .unwrap();
        assert!((0.4..0.6).contains(&progress));
    }

    #[test]
    fn char_effect_retrigger_replaces_kind() {
        let start = base();
        let mut effects = CharEffects::new();
        effects.trigger(3, EffectKind::Emphasize, start);
        effects.trigger(3, EffectKind::Error, start + Duration::from_millis(50));
        let (kind, _) = effects.progress(3, start + Duration::from_millis(50)).unwrap();
        assert_eq!(kind, EffectKind::Error);
    }

    #[test]
    fn char_effect_expires() {
        let start = base();
        let mut effects = CharEffects::new();
        effects.trigger(3, EffectKind::Emphasize, start);
        effects.prune(start + Duration::from_millis(300));
        assert!(effects.progress(3, start + Duration::from_millis(300)).is_none());
    }

    #[test]
    fn results_count_toward_final_and_title_fades() {
        let start = base();
        let mut results = ResultsAnimation::new();
        assert_eq!(results.values(start), (0.0, 0.0, 0.0, 0.0));
        results.start(91.0, 102.0, 97.5, 88.0, start);
        let (wpm, raw, accuracy, consistency) = results.values(start + Duration::from_secs(5));
        assert_eq!(wpm, 91.0);
        assert_eq!(raw, 102.0);
        assert_eq!(accuracy, 97.5);
        assert_eq!(consistency, 88.0);
        assert_eq!(results.title_alpha(start + Duration::from_secs(1)), 1.0);
        assert_eq!(results.title_alpha(start), 0.0);
    }

    #[test]
    fn observe_drives_flash_and_word_landing() {
        let start = base();
        let mut anim = Animations::new();
        anim.mark_test_start(start);
        let words = vec!["foo".to_owned(), "bar".to_owned()];
        let mut test = TypingTest::new(crate::typing::test::TestMode::Words(2), &words);
        test.handle_key('f', start);
        anim.observe(&test, start);
        assert_eq!(anim.cursor.target_x, 1.0);
        assert!(anim.char_effect(0, start).is_some());
        assert_eq!(anim.word_landing(start), 0.0);

        for (key, index) in [('o', 1u8), ('o', 2u8), (' ', 3u8)] {
            test.handle_key(key, start);
            anim.observe(&test, start);
            let _ = index;
        }
        assert!(test.completed_words() == 1);
        assert!(anim.word_landing(start) > 0.9);
        assert!(anim.char_effect(3, start).is_some());
    }
}