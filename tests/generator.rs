use std::time::Duration;

use dytype::typing::generator::{Difficulty, Generator, TestConfig, TestKind, WordPools};
use dytype::typing::test::TestMode;

fn pools() -> WordPools {
    WordPools::default()
}

#[test]
fn embedded_pools_all_lowercase_ascii() {
    let pools = pools();
    for pool in [
        &pools.easy.list,
        &pools.normal.list,
        &pools.hard.list,
        &pools.expert.list,
    ] {
        assert!(!pool.is_empty());
        for word in pool {
            assert!(
                word.chars().all(|c| c.is_ascii_lowercase()),
                "token {word:?} is not plain lowercase"
            );
        }
    }
}

#[test]
fn embedding_pool_into_message_uses_generated_words() {
    let mut generator = Generator::with_seed(3);
    let config = TestConfig {
        kind: TestKind::Words(25),
        difficulty: Difficulty::Hard,
        punctuation: false,
        numbers: false,
    };
    let (mode, words) = generator.generate(&config, &pools(), &[]);
    assert_eq!(mode, TestMode::Words(25));
    assert_eq!(words.len(), 25);
}

#[test]
fn practice_recycles_previous_misses() {
    let mut generator = Generator::with_seed(3);
    let config = TestConfig::new(TestKind::Practice(4), Difficulty::Expert);
    let missed = vec!["misanthrope".to_owned(), "loquacious".to_owned()];
    let (mode, words) = generator.generate(&config, &pools(), &missed);
    assert_eq!(mode, TestMode::Words(4));
    assert_eq!(words.len(), 4);
    assert!(words.iter().all(|w| missed.contains(w)));
    for word in &words {
        assert!(word.chars().all(|c| c.is_ascii_lowercase()));
    }
}

#[test]
fn time_mode_always_builds_enough_words() {
    let mut generator = Generator::with_seed(3);
    for secs in [15u64, 30, 60, 120] {
        let config = TestConfig::new(TestKind::Time(Duration::from_secs(secs)), Difficulty::Easy);
        let (mode, words) = generator.generate(&config, &pools(), &[]);
        let TestMode::Time(duration) = mode else {
            panic!("expected time mode");
        };
        assert_eq!(duration, Duration::from_secs(secs));
        assert!(!words.is_empty());
    }
}
