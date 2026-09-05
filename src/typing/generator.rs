use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::typing::rng::{Rng, XorShift};
use crate::typing::test::TestMode;
use crate::typing::words::{
    EASY_WORDS, EXPERT_WORDS, HARD_WORDS, NORMAL_WORDS, QUOTES, Words,
};

const PUNCTUATION: [char; 6] = ['.', ',', ';', ':', '!', '?'];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Expert,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
            Difficulty::Expert => "expert",
        }
    }

    fn pool(self, pools: &WordPools) -> &[String] {
        match self {
            Difficulty::Easy => &pools.easy.list,
            Difficulty::Normal => &pools.normal.list,
            Difficulty::Hard => &pools.hard.list,
            Difficulty::Expert => &pools.expert.list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestKind {
    Words(usize),
    Time(Duration),
    Quote,
    Custom(Vec<String>),
    Practice(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestConfig {
    pub kind: TestKind,
    pub difficulty: Difficulty,
    /// Sprinkle punctuation into lower difficulties.
    pub punctuation: bool,
    /// Replace some words with numbers on lower difficulties.
    pub numbers: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            kind: TestKind::Words(25),
            difficulty: Difficulty::Normal,
            punctuation: false,
            numbers: false,
        }
    }
}

impl TestConfig {
    pub fn new(kind: TestKind, difficulty: Difficulty) -> Self {
        Self {
            kind,
            difficulty,
            punctuation: false,
            numbers: false,
        }
    }

    pub fn label(&self) -> String {
        let kind = match &self.kind {
            TestKind::Words(n) => format!("{n} words"),
            TestKind::Time(duration) => format!("{}s", duration.as_secs()),
            TestKind::Quote => "quote".to_owned(),
            TestKind::Custom(_) => "custom".to_owned(),
            TestKind::Practice(n) => format!("practice {n}"),
        };
        format!("{kind} · {}", self.difficulty.label())
    }
}

pub struct WordPools {
    pub easy: Words,
    pub normal: Words,
    pub hard: Words,
    pub expert: Words,
}

impl Default for WordPools {
    fn default() -> Self {
        Self {
            easy: Words::from_text(EASY_WORDS),
            normal: Words::from_text(NORMAL_WORDS),
            hard: Words::from_text(HARD_WORDS),
            expert: Words::from_text(EXPERT_WORDS),
        }
    }
}

pub struct Generator {
    rng: XorShift,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            rng: XorShift::from_time(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: XorShift::new(seed),
        }
    }

    pub fn generate(
        &mut self,
        config: &TestConfig,
        pools: &WordPools,
        practice_words: &[String],
    ) -> (TestMode, Vec<String>) {
        match &config.kind {
            TestKind::Words(count) => {
                let words = self.generate_words(
                    config.difficulty,
                    config.punctuation,
                    config.numbers,
                    pools,
                    *count,
                );
                (TestMode::Words(*count), words)
            }
            TestKind::Time(duration) => {
                let count = words_for_duration(*duration);
                let words = self.generate_words(
                    config.difficulty,
                    config.punctuation,
                    config.numbers,
                    pools,
                    count,
                );
                (TestMode::Time(*duration), words)
            }
            TestKind::Quote => {
                let words = self.pick_quote();
                let count = words.len();
                (TestMode::Words(count), words)
            }
            TestKind::Custom(words) => {
                let count = words.len();
                (TestMode::Words(count), words.clone())
            }
            TestKind::Practice(count) => {
                let source = if practice_words.is_empty() {
                    Words {
                        list: config.difficulty.pool(pools).to_vec(),
                    }
                } else {
                    Words {
                        list: practice_words.to_vec(),
                    }
                };
                let words = self.sample(&source.list, *count);
                (TestMode::Words(*count), words)
            }
        }
    }

    fn generate_words(
        &mut self,
        difficulty: Difficulty,
        punctuation: bool,
        numbers: bool,
        pools: &WordPools,
        count: usize,
    ) -> Vec<String> {
        let words = self.sample(difficulty.pool(pools), count);
        let heavy = difficulty == Difficulty::Hard || difficulty == Difficulty::Expert;
        if !heavy && !punctuation && !numbers {
            return words;
        }
        words
            .into_iter()
            .map(|word| self.decorate(word, difficulty, punctuation, numbers))
            .collect()
    }

    fn sample(&mut self, pool: &[String], count: usize) -> Vec<String> {
        let mut deck = pool.to_vec();
        let mut words = Vec::with_capacity(count);
        for _ in 0..count {
            if deck.is_empty() {
                deck = pool.to_vec();
            }
            let index = self.rng.next_below(deck.len() as u32) as usize;
            words.push(deck.swap_remove(index));
        }
        words
    }

    fn pick_quote(&mut self) -> Vec<String> {
        let quotes: Vec<&str> = QUOTES
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        let quote = quotes[self.rng.next_below(quotes.len() as u32) as usize];
        quote.split_whitespace().map(str::to_owned).collect()
    }

    /// Decorates a sampled word according to its difficulty (Hard and Expert
    /// always style words; lower difficulties gain punctuation/numbers only
    /// when the corresponding flags are enabled).
    fn decorate(
        &mut self,
        mut word: String,
        difficulty: Difficulty,
        punctuation: bool,
        numbers: bool,
    ) -> String {
        let heavy = difficulty == Difficulty::Hard || difficulty == Difficulty::Expert;
        if heavy {
            match difficulty {
                Difficulty::Expert => {
                    if self.rng.chance(10) {
                        word = word.to_uppercase();
                    } else if self.rng.chance(8) {
                        capitalize(&mut word);
                    }
                }
                _ => {
                    if self.rng.chance(8) {
                        capitalize(&mut word);
                    }
                }
            }
        }
        let add_punct = match difficulty {
            Difficulty::Expert => self.rng.chance(8),
            _ => self.rng.chance(6),
        };
        if add_punct && (heavy || punctuation) {
            word.push(self.random_punctuation());
        }
        let add_number = match difficulty {
            Difficulty::Expert => self.rng.chance(5),
            _ => self.rng.chance(6),
        };
        if add_number && (difficulty == Difficulty::Expert || numbers) {
            word = (self.rng.next_below(999) + 1).to_string();
        }
        word
    }

    fn random_punctuation(&mut self) -> char {
        PUNCTUATION[self.rng.next_below(PUNCTUATION.len() as u32) as usize]
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

fn words_for_duration(duration: Duration) -> usize {
    let secs = duration.as_secs() as usize;
    ((secs * 100) / 30).max(25)
}

fn capitalize(word: &mut String) {
    if let Some(ch) = word.chars().next() {
        let upper = ch.to_uppercase().next().unwrap_or(ch);
        word.replace_range(0..ch.len_utf8(), &upper.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pools() -> WordPools {
        WordPools::default()
    }

    #[test]
    fn pools_are_non_empty() {
        let pools = pools();
        assert!(!pools.easy.list.is_empty());
        assert!(!pools.normal.list.is_empty());
        assert!(!pools.hard.list.is_empty());
        assert!(!pools.expert.list.is_empty());
    }

    #[test]
    fn pools_are_lowercase_without_punctuation() {
        let pools = pools();
        for pool in [&pools.easy.list, &pools.normal.list, &pools.hard.list, &pools.expert.list] {
            for word in pool {
                assert!(
                    word.chars().all(|c| c.is_ascii_lowercase()),
                    "unexpected token in pool: {word:?}"
                );
            }
        }
    }

    #[test]
    fn word_kind_generates_requested_count() {
        let mut generator = Generator::with_seed(5);
        let config = TestConfig::new(TestKind::Words(10), Difficulty::Normal);
        let (mode, words) = generator.generate(&config, &pools(), &[]);
        assert_eq!(mode, TestMode::Words(10));
        assert_eq!(words.len(), 10);
    }

    #[test]
    fn time_kind_generates_words_and_preserves_mode() {
        let mut generator = Generator::with_seed(5);
        let duration = Duration::from_secs(15);
        let config = TestConfig::new(TestKind::Time(duration), Difficulty::Normal);
        let (mode, words) = generator.generate(&config, &pools(), &[]);
        assert_eq!(mode, TestMode::Time(duration));
        assert!(!words.is_empty());
    }

    #[test]
    fn quote_kind_returns_quote_words() {
        let mut generator = Generator::with_seed(5);
        let config = TestConfig::new(TestKind::Quote, Difficulty::Normal);
        let (mode, words) = generator.generate(&config, &pools(), &[]);
        match mode {
            TestMode::Words(count) => assert_eq!(count, words.len()),
            TestMode::Time(_) => panic!("quote should be word mode"),
        }
        assert!(words.len() >= 5);
        let joined = words.join(" ");
        assert!(joined.chars().any(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn custom_kind_uses_words_as_given() {
        let mut generator = Generator::with_seed(5);
        let custom = vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()];
        let config = TestConfig::new(TestKind::Custom(custom.clone()), Difficulty::Expert);
        let (mode, words) = generator.generate(&config, &pools(), &[]);
        assert_eq!(mode, TestMode::Words(3));
        assert_eq!(words, custom);
    }

    #[test]
    fn practice_kind_uses_missed_words() {
        let mut generator = Generator::with_seed(5);
        let missed = vec!["apple".to_owned(), "banana".to_owned(), "cherry".to_owned()];
        let config = TestConfig::new(TestKind::Practice(2), Difficulty::Normal);
        let (mode, words) = generator.generate(&config, &pools(), &missed);
        assert_eq!(mode, TestMode::Words(2));
        assert_eq!(words.len(), 2);
        assert!(words.iter().all(|w| missed.contains(w)));
    }

    #[test]
    fn practice_kind_falls_back_to_pool_when_empty() {
        let mut generator = Generator::with_seed(5);
        let config = TestConfig::new(TestKind::Practice(3), Difficulty::Normal);
        let (_, words) = generator.generate(&config, &pools(), &[]);
        assert_eq!(words.len(), 3);
    }

    #[test]
    fn easy_words_are_never_decorated() {
        for seed in 0..20 {
            let mut generator = Generator::with_seed(seed);
            let config = TestConfig::new(TestKind::Words(50), Difficulty::Easy);
            let (_, words) = generator.generate(&config, &pools(), &[]);
            for word in &words {
                assert!(
                    word.chars().all(|c| c.is_ascii_lowercase()),
                    "easy word decorated across seeds: {word:?}"
                );
            }
        }
    }

    #[test]
    fn punctuation_flag_decorates_easy_words() {
        let mut saw_punct = false;
        let mut config = TestConfig::new(TestKind::Words(50), Difficulty::Easy);
        config.punctuation = true;
        for seed in 0..400 {
            let mut generator = Generator::with_seed(seed);
            let (_, words) = generator.generate(&config, &pools(), &[]);
            saw_punct |= words
                .iter()
                .any(|word| word.chars().any(|c| PUNCTUATION.contains(&c)));
        }
        assert!(saw_punct, "punctuation flag should decorate easy words across seeds");
    }

    #[test]
    fn numbers_flag_decorates_normal_words() {
        let mut saw_digit = false;
        let mut config = TestConfig::new(TestKind::Words(50), Difficulty::Normal);
        config.numbers = true;
        for seed in 0..400 {
            let mut generator = Generator::with_seed(seed);
            let (_, words) = generator.generate(&config, &pools(), &[]);
            saw_digit |= words.iter().any(|word| word.chars().any(|c| c.is_ascii_digit()));
        }
        assert!(saw_digit, "numbers flag should replace words with numbers across seeds");
    }

    #[test]
    fn expert_words_include_decoration_over_many_seeds() {
        let mut saw_upper = false;
        let mut saw_punct = false;
        let mut saw_digit = false;
        for seed in 0..200 {
            let mut generator = Generator::with_seed(seed);
            let config = TestConfig::new(TestKind::Words(50), Difficulty::Expert);
            let (_, words) = generator.generate(&config, &pools(), &[]);
            for word in &words {
                saw_upper |= word.chars().any(|c| c.is_ascii_uppercase());
                saw_punct |= word.chars().any(|c| PUNCTUATION.contains(&c));
                saw_digit |= word.chars().any(|c| c.is_ascii_digit());
            }
        }
        assert!(saw_upper, "expert pool should produce uppercase across seeds");
        assert!(saw_punct, "expert pool should produce punctuation across seeds");
        assert!(saw_digit, "expert pool should produce digits across seeds");
    }

    #[test]
    fn same_seed_produces_same_output() {
        let config = TestConfig::new(TestKind::Words(20), Difficulty::Hard);
        let pools = pools();
        let mut a = Generator::with_seed(99);
        let mut b = Generator::with_seed(99);
        assert_eq!(
            a.generate(&config, &pools, &[]),
            b.generate(&config, &pools, &[])
        );
    }

    #[test]
    fn labels_describe_config() {
        let config = TestConfig::new(TestKind::Words(25), Difficulty::Normal);
        assert_eq!(config.label(), "25 words · normal");
        let config = TestConfig::new(TestKind::Time(Duration::from_secs(30)), Difficulty::Expert);
        assert_eq!(config.label(), "30s · expert");
    }

    #[test]
    fn sample_respects_count_with_refill() {
        let mut generator = Generator::with_seed(11);
        let pool = Words {
            list: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        };
        let words = generator.sample(&pool.list, 10);
        assert_eq!(words.len(), 10);
    }
}