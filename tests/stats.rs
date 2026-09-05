use std::time::{Duration, Instant};

use dytype::stats::accuracy::Accuracy;
use dytype::stats::consistency::Consistency;
use dytype::stats::history::History;
use dytype::stats::live::LiveStats;
use dytype::stats::wpm::Wpm;
use dytype::typing::test::{Keystroke, TestMode, TypingTest};

#[test]
fn wpm_calculates_from_correct_chars() {
    let wpm = Wpm::calculate(100, 60.0);
    assert!((wpm - 20.0).abs() < 1e-9);
}

#[test]
fn wpm_returns_zero_for_zero_seconds() {
    assert_eq!(Wpm::calculate(100, 0.0), 0.0);
}

#[test]
fn accuracy_calculates_percent() {
    assert_eq!(Accuracy::calculate(9, 10), 90.0);
}

#[test]
fn accuracy_returns_zero_for_empty() {
    assert_eq!(Accuracy::calculate(0, 0), 0.0);
}

#[test]
fn consistency_detects_perfect_consistency() {
    let consistency = Consistency::calculate(&[50.0, 50.0, 50.0]);
    assert!((consistency - 100.0).abs() < 1e-9);
}

#[test]
fn consistency_drops_with_unstable_pace() {
    let start = Instant::now();
    let mut strokes = Vec::new();
    for i in 0..10 {
        strokes.push(Keystroke {
            position: i,
            expected: 'a',
            actual: 'a',
            correct: true,
            timestamp: start + Duration::from_millis((100 * i) as u64),
        });
    }
    for i in 0..5 {
        strokes.push(Keystroke {
            position: 10 + i,
            expected: 'a',
            actual: 'a',
            correct: true,
            timestamp: start + Duration::from_secs(10) + Duration::from_millis((100 * i) as u64),
        });
    }
    let consistency = Consistency::from_keystrokes(&strokes);
    assert!(consistency < 100.0);
}

#[test]
fn live_stats_reports_wpm_for_completed_test() {
    let words = vec!["ab cd ef".to_owned()];
    let mut test = TypingTest::new(TestMode::Time(Duration::from_secs(3600)), &words);
    let start = Instant::now();
    for (i, key) in "ab cd".chars().enumerate() {
        test.handle_key(key, start + Duration::from_secs(i as u64));
    }
    test.submit(start + Duration::from_secs(5));

    let stats = LiveStats::calculate(&test, start);
    assert_eq!(stats.elapsed, Duration::from_secs(5));
    assert!((stats.wpm - 12.0).abs() < 1e-9);
    assert!((stats.raw_wpm - 12.0).abs() < 1e-9);
    assert_eq!(stats.accuracy, 100.0);
    assert_eq!(stats.consistency, 100.0);
}

#[test]
fn live_stats_tracks_partial_progress() {
    let words = vec!["ab".to_owned(), "cd".to_owned()];
    let mut test = TypingTest::new(TestMode::Time(Duration::from_secs(60)), &words);
    let start = Instant::now();
    test.handle_key('a', start);
    test.handle_key('b', start + Duration::from_secs(30));
    test.handle_key('z', start + Duration::from_secs(30));

    let stats = LiveStats::calculate(&test, start + Duration::from_secs(60));
    assert_eq!(stats.elapsed, Duration::from_secs(60));
    assert!((stats.wpm - 0.4).abs() < 1e-9);
    assert!((stats.raw_wpm - 0.6).abs() < 1e-9);
    assert!((stats.accuracy - 66.666).abs() < 0.01);
}

#[test]
fn history_pushes_results() {
    let mut history = History::new();
    let result = sample::sample_result();
    history.push(result);
    assert_eq!(history.results.len(), 1);
}

#[cfg(test)]
mod sample {
    use dytype::stats::history::TestResult;
    use dytype::typing::generator::Difficulty;
    use dytype::typing::test::TestMode;

    pub fn sample_result() -> TestResult {
        TestResult {
            id: 1,
            timestamp: 1,
            wpm: 60.0,
            raw_wpm: 70.0,
            accuracy: 95.0,
            error_rate: 5.0,
            consistency: 90.0,
            cpm: 300.0,
            characters: 60,
            correct_chars: 57,
            incorrect_chars: 3,
            errors: 3,
            correct_keystrokes: 57,
            incorrect_keystrokes: 3,
            completed_words: 10,
            avg_key_interval_ms: 100,
            duration_ms: 5000,
            mode: TestMode::Time(std::time::Duration::from_secs(60)),
            difficulty: Difficulty::Normal,
            char_stats: Vec::new(),
        }
    }
}