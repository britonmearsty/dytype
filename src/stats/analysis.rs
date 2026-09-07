use std::collections::BTreeMap;

use crate::stats::calendar;
use crate::stats::result::{CharStat, TestResult};

/// Aggregate numbers across a collection of recorded tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub total_tests: usize,
    pub best_wpm: f64,
    pub avg_wpm: f64,
    pub avg_raw_wpm: f64,
    pub avg_accuracy: f64,
    pub avg_consistency: f64,
    pub avg_error_rate: f64,
    pub avg_cpm: f64,
    pub total_characters: usize,
    pub total_correct: usize,
    pub total_errors: usize,
    pub total_time_ms: u64,
    pub perfect_tests: usize,
}

impl Summary {
    pub fn total_time_hours(&self) -> f64 {
        self.total_time_ms as f64 / 3_600_000.0
    }
}

pub fn summarize(results: &[TestResult]) -> Summary {
    let total = results.len();
    let n = total.max(1) as f64;
    let mean = |pick: fn(&TestResult) -> f64| results.iter().map(pick).sum::<f64>() / n;
    Summary {
        total_tests: total,
        best_wpm: results.iter().fold(0.0, |best, r| best.max(r.wpm)),
        avg_wpm: mean(|r| r.wpm),
        avg_raw_wpm: mean(|r| r.raw_wpm),
        avg_accuracy: mean(|r| r.accuracy),
        avg_consistency: mean(|r| r.consistency),
        avg_error_rate: mean(|r| r.error_rate),
        avg_cpm: mean(|r| r.cpm),
        total_characters: results.iter().map(|r| r.characters).sum(),
        total_correct: results.iter().map(|r| r.correct_chars).sum(),
        total_errors: results.iter().map(|r| r.errors).sum(),
        total_time_ms: results.iter().map(|r| r.duration_ms).sum(),
        perfect_tests: results
            .iter()
            .filter(|r| r.errors == 0 && r.accuracy == 100.0)
            .count(),
    }
}

fn series(results: &[TestResult], pick: fn(&TestResult) -> f64) -> Vec<f64> {
    results.iter().map(pick).collect()
}

pub fn wpm_series(results: &[TestResult]) -> Vec<f64> {
    series(results, |r| r.wpm)
}

pub fn raw_wpm_series(results: &[TestResult]) -> Vec<f64> {
    series(results, |r| r.raw_wpm)
}

pub fn accuracy_series(results: &[TestResult]) -> Vec<f64> {
    series(results, |r| r.accuracy)
}

pub fn consistency_series(results: &[TestResult]) -> Vec<f64> {
    series(results, |r| r.consistency)
}

pub fn error_rate_series(results: &[TestResult]) -> Vec<f64> {
    series(results, |r| r.error_rate)
}

pub fn error_count_series(results: &[TestResult]) -> Vec<f64> {
    results.iter().map(|r| r.errors as f64).collect()
}

/// Running maximum of WPM, i.e. how the personal best improved over time.
pub fn best_progression(results: &[TestResult]) -> Vec<f64> {
    let mut best = 0.0f64;
    results
        .iter()
        .map(|r| {
            best = best.max(r.wpm);
            best
        })
        .collect()
}

/// One day of typing history, bucketed by UTC calendar day.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyPoint {
    pub day: i64,
    pub label: String,
    pub tests: usize,
    pub wpm: f64,
    pub accuracy: f64,
    pub errors: usize,
}

/// Aggregates results into daily points for the days that actually appear,
/// limited to the `limit` most recent distinct days (ascending order).
pub fn daily(results: &[TestResult], limit: usize) -> Vec<DailyPoint> {
    let mut buckets: BTreeMap<i64, (usize, f64, f64, usize, u64)> = BTreeMap::new();
    for r in results {
        let day = calendar::day_key(r.timestamp);
        let entry = buckets.entry(day).or_insert((0, 0.0, 0.0, 0, r.timestamp));
        entry.0 += 1;
        entry.1 += r.wpm;
        entry.2 += r.accuracy;
        entry.3 += r.errors;
    }
    buckets
        .into_iter()
        .rev()
        .take(limit)
        .map(
            |(day, (tests, wpm_sum, acc_sum, errors, first_ts))| DailyPoint {
                day,
                label: calendar::short_date(first_ts),
                tests,
                wpm: wpm_sum / tests as f64,
                accuracy: acc_sum / tests as f64,
                errors,
            },
        )
        .rev()
        .collect()
}

/// Aggregates per-character error counts across all recorded tests.
pub fn character_errors(results: &[TestResult], limit: usize) -> Vec<CharStat> {
    let mut counts: BTreeMap<char, (u32, u32)> = BTreeMap::new();
    for r in results {
        for stat in &r.char_stats {
            let entry = counts.entry(stat.ch).or_insert((0, 0));
            entry.0 += stat.typed;
            entry.1 += stat.errors;
        }
    }
    let mut stats: Vec<CharStat> = counts
        .into_iter()
        .map(|(ch, (typed, errors))| CharStat {
            ch,
            typed,
            errors,
            ..CharStat::default()
        })
        .filter(|s| s.errors > 0)
        .collect();
    stats.sort_by(|a, b| {
        b.errors.cmp(&a.errors).then(
            b.rate()
                .partial_cmp(&a.rate())
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    stats.truncate(limit);
    stats
}

/// An error-prone character sequence (bigram or trigram): what was expected
/// plus the `typed`/`errors` accumulated across history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextStat {
    pub ngram: String,
    pub typed: u32,
    pub errors: u32,
}

impl ContextStat {
    pub fn rate(&self) -> f64 {
        if self.typed == 0 {
            0.0
        } else {
            self.errors as f64 / self.typed as f64 * 100.0
        }
    }
}

fn mark(prefix: &mut String, c: Option<char>) {
    match c {
        Some(c) => prefix.push(c),
        // No expected character before this one: the start of the text/word.
        None => prefix.push('^'),
    }
}

/// The `depth` characters of context (the previous `depth` expected chars)
/// that precede a character, gathered with that character into a readable
/// ngram label.
fn context_label(prev2: Option<char>, prev: Option<char>, ch: char, depth: usize) -> String {
    let mut ngram = String::new();
    if depth >= 2 {
        mark(&mut ngram, prev2);
    }
    mark(&mut ngram, prev);
    ngram.push(ch);
    ngram
}

/// Weakest bigrams: the (previous char, current char) pairs with the most
/// errors, merged across history. `depth` 1 = bigrams, 2 = trigrams.
/// A context key is `(prev2, prev, ch)`; the tally is `(typed, errors)`.
type ContextKey = (Option<char>, Option<char>, char);
type ContextTally = (u32, u32);

pub fn context_errors(results: &[TestResult], depth: usize, limit: usize) -> Vec<ContextStat> {
    let mut counts: BTreeMap<ContextKey, ContextTally> = BTreeMap::new();
    for r in results {
        for stat in &r.char_stats {
            let (prev, prev2) = if depth >= 2 {
                (stat.prev, stat.prev2)
            } else {
                (stat.prev, None)
            };
            let entry = counts.entry((prev2, prev, stat.ch)).or_insert((0, 0));
            entry.0 += stat.typed;
            entry.1 += stat.errors;
        }
    }
    let mut stats: Vec<ContextStat> = counts
        .into_iter()
        .filter(|(_, (_, errors))| *errors > 0)
        .map(|((prev2, prev, ch), (typed, errors))| ContextStat {
            ngram: context_label(prev2, prev, ch, depth),
            typed,
            errors,
        })
        .collect();
    stats.sort_by(|a, b| {
        b.errors
            .cmp(&a.errors)
            .then(
                b.rate()
                    .partial_cmp(&a.rate())
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(a.ngram.cmp(&b.ngram))
    });
    stats.truncate(limit);
    stats
}

/// Convenience wrappers for the two useful depths.
pub fn bigram_errors(results: &[TestResult], limit: usize) -> Vec<ContextStat> {
    context_errors(results, 1, limit)
}

pub fn trigram_errors(results: &[TestResult], limit: usize) -> Vec<ContextStat> {
    context_errors(results, 2, limit)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationBucket {
    Under30,
    Under60,
    Under120,
    Under300,
    Over300,
}

impl DurationBucket {
    pub fn label(&self) -> &'static str {
        match self {
            DurationBucket::Under30 => "<30s",
            DurationBucket::Under60 => "30-60s",
            DurationBucket::Under120 => "1-2m",
            DurationBucket::Under300 => "2-5m",
            DurationBucket::Over300 => "5m+",
        }
    }

    pub fn from_millis(ms: u64) -> Self {
        let secs = ms / 1000;
        match secs {
            0..=29 => DurationBucket::Under30,
            30..=59 => DurationBucket::Under60,
            60..=119 => DurationBucket::Under120,
            120..=299 => DurationBucket::Under300,
            _ => DurationBucket::Over300,
        }
    }
}

/// Distribution of test durations across the five standard buckets, in order.
pub fn duration_distribution(results: &[TestResult]) -> Vec<(DurationBucket, usize)> {
    let mut counts = [0usize; 5];
    for r in results {
        let index = match DurationBucket::from_millis(r.duration_ms) {
            DurationBucket::Under30 => 0,
            DurationBucket::Under60 => 1,
            DurationBucket::Under120 => 2,
            DurationBucket::Under300 => 3,
            DurationBucket::Over300 => 4,
        };
        counts[index] += 1;
    }
    [
        (DurationBucket::Under30, counts[0]),
        (DurationBucket::Under60, counts[1]),
        (DurationBucket::Under120, counts[2]),
        (DurationBucket::Under300, counts[3]),
        (DurationBucket::Over300, counts[4]),
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::result::{CharStat, TestResult};
    use crate::typing::generator::Difficulty;
    use crate::typing::test::TestMode;

    fn sample(id: u64, wpm: f64, accuracy: f64, errors: usize, timestamp: u64) -> TestResult {
        TestResult {
            id,
            timestamp,
            wpm,
            raw_wpm: wpm + 10.0,
            accuracy,
            error_rate: errors as f64,
            consistency: 80.0,
            cpm: wpm * 5.0,
            characters: 30,
            correct_chars: 20,
            incorrect_chars: 2,
            errors,
            correct_keystrokes: 20,
            incorrect_keystrokes: 2,
            completed_words: 10,
            avg_key_interval_ms: 120,
            duration_ms: 30_000,
            mode: TestMode::Words(10),
            difficulty: Difficulty::Normal,
            char_stats: if errors > 0 {
                vec![CharStat {
                    ch: 'e',
                    typed: 10,
                    errors: errors as u32,
                    prev: Some('h'),
                    prev2: Some('t'),
                }]
            } else {
                Vec::new()
            },
        }
    }

    #[test]
    fn empty_history_summarizes_to_zeros() {
        let summary = summarize(&[]);
        assert_eq!(summary.total_tests, 0);
        assert_eq!(summary.best_wpm, 0.0);
        assert_eq!(summary.total_errors, 0);
    }

    #[test]
    fn summary_averages_and_aggregates() {
        let results = vec![sample(1, 50.0, 95.0, 2, 0), sample(2, 100.0, 100.0, 0, 0)];
        let summary = summarize(&results);
        assert_eq!(summary.total_tests, 2);
        assert_eq!(summary.best_wpm, 100.0);
        assert_eq!(summary.avg_wpm, 75.0);
        assert_eq!(summary.avg_accuracy, 97.5);
        assert_eq!(summary.total_errors, 2);
        assert_eq!(summary.perfect_tests, 1);
    }

    #[test]
    fn best_progression_is_running_max() {
        let results = vec![
            sample(1, 50.0, 90.0, 0, 0),
            sample(2, 40.0, 90.0, 0, 0),
            sample(3, 80.0, 95.0, 0, 0),
        ];
        assert_eq!(best_progression(&results), vec![50.0, 50.0, 80.0]);
    }

    #[test]
    fn daily_buckets_by_day_and_averages() {
        let day_a = 1_700_000_000;
        let day_b = day_a + 86_400;
        let results = vec![
            sample(1, 60.0, 95.0, 1, day_a + 100),
            sample(2, 80.0, 95.0, 1, day_a + 200),
            sample(3, 70.0, 90.0, 0, day_b),
            sample(4, 90.0, 95.0, 0, day_b + 500),
        ];
        let days = daily(&results, 10);
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].day, 19_675);
        assert_eq!(days[0].wpm, 70.0);
        assert_eq!(days[0].tests, 2);
        assert_eq!(days[1].day, days[0].day + 1);
        assert_eq!(days[1].wpm, 80.0);
        assert_eq!(days[1].errors, 0);
    }

    #[test]
    fn daily_limit_keeps_most_recent() {
        let mut results = Vec::new();
        for offset in 0..5u64 {
            results.push(sample(
                1 + offset,
                60.0,
                95.0,
                0,
                1_700_000_000 + offset * 86_400,
            ));
        }
        let days = daily(&results, 2);
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].day, results[3].timestamp.div_euclid(86_400) as i64);
    }

    #[test]
    fn date_math_matches_day_key() {
        let timestamp = 1_709_164_800;
        let day = calendar::day_key(timestamp);
        assert_eq!(calendar::civil_from_days(day), (2024, 2, 29));
    }

    #[test]
    fn character_errors_merge_across_tests() {
        let results = vec![sample(1, 50.0, 90.0, 3, 0), sample(2, 80.0, 95.0, 2, 0)];
        let errors = character_errors(&results, 5);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].ch, 'e');
        assert_eq!(errors[0].typed, 20);
        assert_eq!(errors[0].errors, 5);
    }

    #[test]
    fn bigram_errors_aggregate_by_context() {
        // Two tests erroring on 'e' — one after 'h', one after 't' —
        // land in separate bigrams.
        let mut a = sample(1, 50.0, 90.0, 1, 0);
        a.char_stats = vec![CharStat {
            ch: 'e',
            typed: 10,
            errors: 2,
            prev: Some('h'),
            prev2: Some('t'),
        }];
        let mut b = sample(2, 60.0, 95.0, 1, 0);
        b.char_stats = vec![CharStat {
            ch: 'e',
            typed: 8,
            errors: 1,
            prev: Some('t'),
            prev2: None,
        }];
        let bigrams = bigram_errors(&[a, b], 10);
        let he = bigrams.iter().find(|s| s.ngram == "he").expect("he bigram");
        assert_eq!((he.typed, he.errors), (10, 2));
        let te = bigrams.iter().find(|s| s.ngram == "te").expect("te bigram");
        assert_eq!((te.typed, te.errors), (8, 1));
    }

    #[test]
    fn trigram_errors_include_two_chars_of_context() {
        let mut result = sample(1, 50.0, 90.0, 1, 0);
        result.char_stats = vec![CharStat {
            ch: 'r',
            typed: 12,
            errors: 3,
            prev: Some('e'),
            prev2: Some('h'),
        }];
        let trigrams = trigram_errors(&[result], 10);
        assert_eq!(trigrams.len(), 1);
        assert_eq!(trigrams[0].ngram, "her");
        assert_eq!((trigrams[0].typed, trigrams[0].errors), (12, 3));
        assert!((24.0..=26.0).contains(&trigrams[0].rate()));
    }

    #[test]
    fn bigram_ordering_prefers_higher_rate_after_error_tie() {
        // Same error count (1) but "xy" was typed once (100% rate) vs "ab"
        // typed ten times (10% rate): the rarest strike wins the tie.
        let mut a = sample(1, 50.0, 90.0, 1, 0);
        a.char_stats = vec![CharStat {
            ch: 'y',
            typed: 1,
            errors: 1,
            prev: Some('x'),
            prev2: None,
        }];
        let mut b = sample(2, 50.0, 90.0, 1, 0);
        b.char_stats = vec![CharStat {
            ch: 'b',
            typed: 10,
            errors: 1,
            prev: Some('a'),
            prev2: None,
        }];
        let bigrams = bigram_errors(&[a, b], 10);
        assert_eq!(bigrams.first().map(|s| s.ngram.as_str()), Some("xy"));
    }

    #[test]
    fn duration_buckets_map_ranges() {
        assert_eq!(DurationBucket::from_millis(29_000), DurationBucket::Under30);
        assert_eq!(DurationBucket::from_millis(30_000), DurationBucket::Under60);
        assert_eq!(
            DurationBucket::from_millis(119_000),
            DurationBucket::Under120
        );
        assert_eq!(
            DurationBucket::from_millis(299_000),
            DurationBucket::Under300
        );
        assert_eq!(
            DurationBucket::from_millis(301_000),
            DurationBucket::Over300
        );
        let results = vec![sample(1, 60.0, 95.0, 0, 0), sample(2, 60.0, 95.0, 0, 0)];
        let distribution = duration_distribution(&results);
        assert_eq!(distribution[0], (DurationBucket::Under30, 0));
        assert_eq!(distribution[1], (DurationBucket::Under60, 2));
    }
}
