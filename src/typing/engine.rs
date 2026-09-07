use std::time::Instant;

use crate::typing::test::{TestMode, TypingTest};

pub struct Engine {
    pub test: TypingTest,
}

impl Engine {
    pub fn with_test(mode: TestMode, words: &[String]) -> Self {
        Self {
            test: TypingTest::new(mode, words),
        }
    }

    /// Underscores the raw given text (code snippets, custom layouts) instead
    /// of space-joining tokens.
    pub fn with_text(mode: TestMode, text: &str) -> Self {
        Self {
            test: TypingTest::with_text(mode, text),
        }
    }

    pub fn handle_key(&mut self, key: char, now: Instant) {
        self.test.handle_key(key, now);
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::with_test(TestMode::Time(std::time::Duration::from_secs(15)), &[])
    }
}
