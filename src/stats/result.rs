use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::stats::accuracy::Accuracy;
use crate::stats::consistency::Consistency;
use crate::stats::wpm::Wpm;
use crate::typing::generator::Difficulty;
use crate::typing::test::{Keystroke, TestMode, TypingTest};

/// Unix timestamp (seconds, UTC) for "now".
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

/// Per-character typing history within a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharStat {
    pub ch: char,
    pub typed: u32,
    pub errors: u32,
}

impl CharStat {
    pub fn rate(&self) -> f64 {
        if self.typed == 0 {
            0.0
        } else {
            self.errors as f64 / self.typed as f64 * 100.0
        }
    }
}

/// A fully recorded typing test. Produced once a test finishes and persisted
/// so history, graphs, error analysis and progression can be rebuilt.
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    pub id: u64,
    pub timestamp: u64,

    // Speed & accuracy (0-100 for percentages).
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub error_rate: f64,
    pub consistency: f64,
    pub cpm: f64,

    // Character/keystroke totals.
    pub characters: usize,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub errors: usize,
    pub correct_keystrokes: usize,
    pub incorrect_keystrokes: usize,
    pub completed_words: usize,

    pub avg_key_interval_ms: u64,
    pub duration_ms: u64,

    pub mode: TestMode,
    pub difficulty: Difficulty,

    pub char_stats: Vec<CharStat>,
}

impl TestResult {
    /// Computes every metric from a finished (or finishing) test.
    pub fn build(id: u64, test: &TypingTest, difficulty: Difficulty, now: Instant) -> Self {
        let elapsed = test.duration.unwrap_or_else(|| test.elapsed(now));
        let total_keys = test.correct_chars + test.incorrect_chars;
        let seconds = elapsed.as_secs_f64();
        let minutes = seconds / 60.0;
        let accuracy = Accuracy::calculate(test.correct_chars, total_keys);
        let error_rate = if total_keys == 0 {
            0.0
        } else {
            test.errors.len() as f64 / total_keys as f64 * 100.0
        };
        let cpm = if minutes > 0.0 {
            test.correct_chars as f64 / minutes
        } else {
            0.0
        };
        Self {
            id,
            timestamp: unix_timestamp(),
            wpm: Wpm::calculate(test.correct_chars, seconds),
            raw_wpm: Wpm::calculate(total_keys, seconds),
            accuracy,
            error_rate,
            consistency: Consistency::from_keystrokes(&test.keystrokes),
            cpm,
            characters: test.chars.len(),
            correct_chars: test.correct_chars,
            incorrect_chars: test.incorrect_chars,
            errors: test.errors.len(),
            correct_keystrokes: test.correct_chars,
            incorrect_keystrokes: test.incorrect_chars,
            completed_words: test.completed_words(),
            avg_key_interval_ms: avg_key_interval_ms(&test.keystrokes).round() as u64,
            duration_ms: elapsed.as_millis().min(u128::from(u64::MAX)) as u64,
            mode: test.mode,
            difficulty,
            char_stats: character_stats(&test.keystrokes),
        }
    }

    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.duration_ms)
    }
}

/// Mean time between consecutive keystrokes, in milliseconds.
fn avg_key_interval_ms(keystrokes: &[Keystroke]) -> f64 {
    if keystrokes.len() < 2 {
        return 0.0;
    }
    let total: u128 = keystrokes
        .windows(2)
        .map(|pair| {
            pair[1]
                .timestamp
                .saturating_duration_since(pair[0].timestamp)
                .as_micros()
        })
        .sum();
    total as f64 / (keystrokes.len() - 1) as f64 / 1000.0
}

/// Aggregates keystrokes by the expected character, tracking how often each
/// one was typed and how often it was missed. Sorted by misses first.
fn character_stats(keystrokes: &[Keystroke]) -> Vec<CharStat> {
    let mut counts: Vec<(char, u32, u32)> = Vec::new();
    for stroke in keystrokes {
        match counts.iter_mut().find(|(ch, _, _)| *ch == stroke.expected) {
            Some((_, typed, errors)) => {
                *typed += 1;
                if !stroke.correct {
                    *errors += 1;
                }
            }
            None => counts.push((
                stroke.expected,
                1,
                if stroke.correct { 0 } else { 1 },
            )),
        }
    }
    let mut stats: Vec<CharStat> = counts
        .into_iter()
        .map(|(ch, typed, errors)| CharStat { ch, typed, errors })
        .collect();
    stats.sort_by(|a, b| {
        b.errors
            .cmp(&a.errors)
            .then(b.typed.cmp(&a.typed))
            .then(a.ch.cmp(&b.ch))
    });
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing::test::{TestStatus, TypingTest};

    /// Types `text` with ~90ms between keys and submits at 2s. Returns the
    /// finished test with full keystroke timing data.
    fn typed_test(text: &str, mode: TestMode) -> TypingTest {
        let words: Vec<String> = vec!["foo".to_owned(), "bar".to_owned()];
        let mut test = TypingTest::new(mode, &words);
        let start = Instant::now();
        for (i, key) in text.chars().enumerate() {
            test.handle_key(key, start + Duration::from_millis(i as u64 * 90));
        }
        test.submit(start + Duration::from_millis(2000));
        test
    }

    #[test]
    fn mistake_metrics_are_reflected_in_result() {
        // Time mode keeps the test open so the stray trailing key is counted.
        let test = typed_test(
            "foo barx",
            TestMode::Time(Duration::from_secs(3600)),
        );
        let result = TestResult::build(7, &test, Difficulty::Normal, Instant::now());
        assert_eq!(result.id, 7);
        assert_eq!(result.mode, TestMode::Time(Duration::from_secs(3600)));
        assert_eq!(result.difficulty, Difficulty::Normal);
        assert_ne!(result.timestamp, 0);
        assert_eq!(result.characters, 7);
        assert_eq!(result.correct_chars, 7);
        assert_eq!(result.incorrect_chars, 1);
        assert_eq!(result.errors, 1);
        assert_eq!(result.correct_keystrokes, 7);
        assert_eq!(result.incorrect_keystrokes, 1);
        assert_eq!(result.completed_words, 2);
        assert_eq!(result.accuracy, 87.5);
        assert_eq!(result.error_rate, 12.5);
        assert!(result.avg_key_interval_ms > 80 && result.avg_key_interval_ms < 95);
        assert!(result.wpm > 0.0);
        assert!(result.cpm > 0.0);
        assert!(result.duration_ms >= 600);
        assert!(result.consistency > 0.0);
    }

    #[test]
    fn helxo_example_reaches_exactly_four_of_five() {
        let words = vec!["hello".to_owned()];
        let mut test = TypingTest::new(TestMode::Words(1), &words);
        let start = Instant::now();
        for key in "helxo".chars() {
            test.handle_key(key, start);
        }
        assert_eq!(test.status, TestStatus::Finished);
        let result = TestResult::build(42, &test, Difficulty::Normal, start);
        assert_eq!(result.correct_chars, 4);
        assert_eq!(result.incorrect_chars, 1);
        assert_eq!(result.errors, 1);
        assert_eq!(result.characters, 5);
        assert_eq!(result.completed_words, 1);
        assert_eq!(result.accuracy, 80.0);
    }

    #[test]
    fn flawless_run_yields_full_accuracy() {
        let test = typed_test("foo bar", TestMode::Words(2));
        let result = TestResult::build(1, &test, Difficulty::Easy, Instant::now());
        assert_eq!(result.accuracy, 100.0);
        assert_eq!(result.error_rate, 0.0);
        assert_eq!(result.errors, 0);
        assert_eq!(result.completed_words, 2);
    }

    #[test]
    fn character_stats_count_per_position() {
        let test = typed_test("fioo bar", TestMode::Words(2));
        let stats = character_stats(&test.keystrokes);
        // 'o' is expected at positions 1 and 2; position 1 was missed with 'i'.
        let o = stats.iter().copied().find(|s| s.ch == 'o').expect("o expected");
        assert_eq!(o.typed, 2);
        assert_eq!(o.errors, 1);
        assert!((49.0..=51.0).contains(&o.rate()));
        // 'a' is expected once and was missed in the misalignment.
        let a = stats.iter().copied().find(|s| s.ch == 'a').expect("a expected");
        assert_eq!(a.typed, 1);
        assert_eq!(a.errors, 1);
        assert_eq!(a.rate(), 100.0);
        // Sorted by errors descending.
        for pair in stats.windows(2) {
            assert!(pair[0].errors >= pair[1].errors);
        }
    }

    #[test]
    fn empty_test_has_zero_metrics() {
        let words: Vec<String> = vec!["hi".to_owned()];
        let mut test = TypingTest::new(TestMode::Words(1), &words);
        test.submit(Instant::now());
        let result = TestResult::build(2, &test, Difficulty::Normal, Instant::now());
        assert_eq!(result.wpm, 0.0);
        assert_eq!(result.accuracy, 0.0);
        assert_eq!(result.errors, 0);
        assert!(result.char_stats.is_empty());
        assert_eq!(result.avg_key_interval_ms, 0);
    }
}