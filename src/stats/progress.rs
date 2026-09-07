use crate::stats::calendar;
use crate::stats::result::TestResult;

/// XP earned for a single test: effective WPM (speed weighted by accuracy).
pub fn xp_for_test(wpm: f64, accuracy: f64) -> u64 {
    (wpm * accuracy / 100.0).round().max(0.0) as u64
}

/// Cost, in XP, to cross from `level` to `level + 1`.
fn xp_cost(level: u32) -> u64 {
    300 * u64::from(level)
}

/// Cumulative XP needed to *reach* `level` (level 1 is free).
fn cumulative_xp(level: u32) -> u64 {
    300 * u64::from(level) * u64::from(level.saturating_sub(1)) / 2
}

/// Level derived from total lifetime XP, plus progress inside the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelInfo {
    pub level: u32,
    pub xp_into_level: u64,
    pub xp_for_next_level: u64,
    pub total_xp: u64,
}

impl LevelInfo {
    /// Progress through the current level, 0.0 ..= 1.0.
    pub fn progress(&self) -> f64 {
        if self.xp_for_next_level == 0 {
            0.0
        } else {
            self.xp_into_level as f64 / self.xp_for_next_level as f64
        }
    }
}

const MAX_LEVEL: u32 = 100;

pub fn level_from_xp(total_xp: u64) -> LevelInfo {
    let mut level = 1;
    while level < MAX_LEVEL && cumulative_xp(level + 1) <= total_xp {
        level += 1;
    }
    let xp_into_level = total_xp.saturating_sub(cumulative_xp(level));
    LevelInfo {
        level,
        xp_into_level,
        xp_for_next_level: xp_cost(level),
        total_xp,
    }
}

pub fn total_xp(results: &[TestResult]) -> u64 {
    results.iter().map(|r| xp_for_test(r.wpm, r.accuracy)).sum()
}

/// Longest run of consecutive calendar days that contain at least one test.
pub fn longest_streak(results: &[TestResult]) -> u32 {
    let mut days: Vec<i64> = results
        .iter()
        .map(|r| calendar::day_key(r.timestamp))
        .collect();
    days.sort_unstable();
    days.dedup();
    let mut best = 0u32;
    let mut current = 0u32;
    let mut previous: Option<i64> = None;
    for day in days {
        if previous == Some(day - 1) {
            current += 1;
        } else {
            current = 1;
        }
        best = best.max(current);
        previous = Some(day);
    }
    best
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const ACHIEVEMENTS: [Achievement; 8] = [
    Achievement {
        id: "first_test",
        name: "First test",
        description: "Complete your first test",
    },
    Achievement {
        id: "wpm_50",
        name: "Getting fast",
        description: "Reach 50 WPM in a test",
    },
    Achievement {
        id: "wpm_75",
        name: "Speed demon",
        description: "Reach 75 WPM in a test",
    },
    Achievement {
        id: "wpm_100",
        name: "Century",
        description: "Reach 100 WPM in a test",
    },
    Achievement {
        id: "accuracy_98",
        name: "Sharpshooter",
        description: "Score 98%+ accuracy in a test",
    },
    Achievement {
        id: "perfect_test",
        name: "Flawless",
        description: "Complete a test with 100% accuracy and no errors",
    },
    Achievement {
        id: "tests_100",
        name: "Veteran",
        description: "Complete 100 tests",
    },
    Achievement {
        id: "streak_7",
        name: "On a roll",
        description: "Type on 7 consecutive days",
    },
];

/// Which achievements have been earned, aligned with [`ACHIEVEMENTS`].
pub fn earned(results: &[TestResult]) -> Vec<bool> {
    let best_wpm = results.iter().fold(0.0f64, |best, r| best.max(r.wpm));
    let any = |pick: fn(&TestResult) -> bool| results.iter().any(pick);
    vec![
        !results.is_empty(),
        best_wpm >= 50.0,
        best_wpm >= 75.0,
        best_wpm >= 100.0,
        any(|r| r.accuracy >= 98.0),
        any(|r| r.errors == 0 && r.accuracy == 100.0),
        results.len() >= 100,
        longest_streak(results) >= 7,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::result::CharStat;
    use crate::typing::generator::Difficulty;
    use crate::typing::test::TestMode;

    fn sample(id: u64, wpm: f64, accuracy: f64, errors: u32, timestamp: u64) -> TestResult {
        TestResult {
            id,
            timestamp,
            wpm,
            raw_wpm: wpm + 10.0,
            accuracy,
            error_rate: f64::from(errors),
            consistency: 80.0,
            cpm: wpm * 5.0,
            characters: 30,
            correct_chars: 20,
            incorrect_chars: errors as usize,
            errors: errors as usize,
            correct_keystrokes: 20,
            incorrect_keystrokes: errors as usize,
            completed_words: 10,
            avg_key_interval_ms: 120,
            duration_ms: 30_000,
            mode: TestMode::Words(10),
            difficulty: Difficulty::Normal,
            char_stats: if errors > 0 {
                vec![CharStat {
                    ch: 'e',
                    typed: 10,
                    errors,
                    prev: Some('h'),
                    prev2: None,
                }]
            } else {
                Vec::new()
            },
        }
    }

    #[test]
    fn xp_uses_effective_wpm() {
        assert_eq!(xp_for_test(100.0, 95.0), 95);
        assert_eq!(xp_for_test(50.0, 100.0), 50);
        assert_eq!(xp_for_test(0.0, 100.0), 0);
    }

    #[test]
    fn level_boundaries() {
        assert_eq!(level_from_xp(0).level, 1);
        assert_eq!(level_from_xp(0).xp_into_level, 0);
        assert_eq!(level_from_xp(299).level, 1);
        assert_eq!(level_from_xp(299).xp_into_level, 299);
        assert_eq!(level_from_xp(300).level, 2);
        assert_eq!(level_from_xp(300).xp_into_level, 0);
        assert_eq!(level_from_xp(899).level, 2);
        assert_eq!(level_from_xp(900).level, 3);
        assert_eq!(level_from_xp(900).xp_into_level, 0);
        assert_eq!(level_from_xp(1000).xp_into_level, 100);
    }

    #[test]
    fn level_progress_is_unit_sized() {
        assert!((0.0..=1.0).contains(&level_from_xp(450).progress()));
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let base = 1_700_000_000;
        let r = |id: u64, offset: u64| sample(id, 60.0, 95.0, 1, base + offset * 86_400);
        assert_eq!(longest_streak(&[]), 0);
        assert_eq!(longest_streak(&[r(1, 0)]), 1);
        assert_eq!(longest_streak(&[r(1, 0), r(2, 1), r(3, 2)]), 3);
        assert_eq!(
            longest_streak(&[r(1, 0), r(2, 2), r(3, 3), r(4, 6), r(5, 7)]),
            2
        );
        // Duplicate days don't inflate the streak.
        assert_eq!(longest_streak(&[r(1, 0), r(2, 0), r(3, 1)]), 2);
    }

    #[test]
    fn achievements_track_progression() {
        assert!(earned(&[]).iter().all(|e| !e));
        let results = vec![
            sample(1, 80.0, 98.5, 1, 0),
            sample(2, 101.0, 100.0, 0, 86_400),
        ];
        let earned = earned(&results);
        assert!(earned[0]); // first test
        assert!(earned[1]); // 50
        assert!(earned[2]); // 75
        assert!(earned[3]); // 100
        assert!(earned[4]); // 98%
        assert!(earned[5]); // flawless
        assert!(!earned[6]); // 100 tests
        assert!(!earned[7]); // streak 7
    }

    #[test]
    fn achievement_ids_are_unique() {
        let mut ids: Vec<&str> = ACHIEVEMENTS.iter().map(|a| a.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ACHIEVEMENTS.len());
    }
}
