use std::time::{Duration, Instant};

use crate::stats::accuracy::Accuracy;
use crate::stats::consistency::Consistency;
use crate::stats::wpm::Wpm;
use crate::typing::test::TypingTest;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveStats {
    pub elapsed: Duration,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
}

impl Default for LiveStats {
    fn default() -> Self {
        Self {
            elapsed: Duration::ZERO,
            wpm: 0.0,
            raw_wpm: 0.0,
            accuracy: 0.0,
            consistency: 0.0,
        }
    }
}

impl LiveStats {
    pub fn calculate(test: &TypingTest, now: Instant) -> Self {
        let elapsed = test.duration.unwrap_or_else(|| test.elapsed(now));
        let total_keys = test.correct_chars + test.incorrect_chars;
        let wpm = Wpm::calculate(test.correct_chars, elapsed.as_secs_f64());
        let raw_wpm = Wpm::calculate(total_keys, elapsed.as_secs_f64());
        let accuracy = Accuracy::calculate(test.correct_chars, total_keys);
        let consistency = Consistency::from_keystrokes(&test.keystrokes);
        Self {
            elapsed,
            wpm,
            raw_wpm,
            accuracy,
            consistency,
        }
    }
}
