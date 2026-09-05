use std::path::Path;

use serde::{Deserialize, Serialize};

pub const DEFAULT_WORDS: &str = include_str!("../../assets/words/english.txt");

#[derive(Serialize, Deserialize)]
pub struct Words {
    pub list: Vec<String>,
}

impl Words {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let list = parse_words(&content);
        Ok(Self { list })
    }

    pub fn builtin() -> Self {
        Self {
            list: parse_words(DEFAULT_WORDS),
        }
    }

    pub fn shuffled(&self) -> Self {
        let mut rng = XorShift::from_time();
        let mut list = self.list.clone();
        for i in (1..list.len()).rev() {
            let j = rng.next() as usize % (i + 1);
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

struct XorShift(u64);

impl XorShift {
    fn from_time() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self(seed | 1)
    }

    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x & 0xFFFF_FFFF) as u32
    }
}