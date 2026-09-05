use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::typing::rng::{Rng, XorShift};

pub const DEFAULT_WORDS: &str = include_str!("../../assets/words/english.txt");
pub const EASY_WORDS: &str = include_str!("../../assets/words/easy.txt");
pub const NORMAL_WORDS: &str = include_str!("../../assets/words/normal.txt");
pub const HARD_WORDS: &str = include_str!("../../assets/words/hard.txt");
pub const EXPERT_WORDS: &str = include_str!("../../assets/words/expert.txt");
pub const QUOTES: &str = include_str!("../../assets/quotes/english.txt");

#[derive(Serialize, Deserialize)]
pub struct Words {
    pub list: Vec<String>,
}

impl Words {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self {
            list: parse_words(&content),
        })
    }

    pub fn from_text(content: &str) -> Self {
        Self {
            list: parse_words(content),
        }
    }

    pub fn builtin() -> Self {
        Self::from_text(DEFAULT_WORDS)
    }

    pub fn shuffled(&self) -> Self {
        let mut rng = XorShift::from_time();
        let mut list = self.list.clone();
        for i in (1..list.len()).rev() {
            let j = rng.next_below(i as u32 + 1) as usize;
            list.swap(i, j);
        }
        Self { list }
    }
}

impl Default for Words {
    fn default() -> Self {
        Self::builtin()
    }
}

fn parse_words(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}