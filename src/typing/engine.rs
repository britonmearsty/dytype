use std::time::{Duration, Instant};

use crate::typing::test::{TestMode, TypingTest};
use crate::typing::words::Words;

pub struct Engine {
    pub test: TypingTest,
}

impl Engine {
    pub fn new(mode: TestMode) -> Self {
        Self {
            test: TypingTest::new(mode, &[]),
        }
    }

    pub fn with_words(mode: TestMode, words: &Words) -> Self {
        Self {
            test: TypingTest::new(mode, &selected_words(mode, words)),
        }
    }

    pub fn handle_key(&mut self, key: char, now: Instant) {
        self.test.handle_key(key, now);
    }

    pub fn restart(&mut self, words: &Words) {
        let mode = self.test.mode;
        self.test = TypingTest::new(mode, &selected_words(mode, words));
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(TestMode::Time(Duration::from_secs(15)))
    }
}

fn selected_words(mode: TestMode, words: &Words) -> Vec<String> {
    match mode {
        TestMode::Words(count) => words.list.iter().take(count).cloned().collect(),
        TestMode::Time(_) => words.list.clone(),
    }
}