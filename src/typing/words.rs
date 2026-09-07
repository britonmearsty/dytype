use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::typing::rng::{Rng, XorShift};

pub const DEFAULT_WORDS: &str = include_str!("../../assets/words/english.txt");
pub const EASY_WORDS: &str = include_str!("../../assets/words/easy.txt");
pub const NORMAL_WORDS: &str = include_str!("../../assets/words/normal.txt");
pub const HARD_WORDS: &str = include_str!("../../assets/words/hard.txt");
pub const EXPERT_WORDS: &str = include_str!("../../assets/words/expert.txt");
pub const QUOTES: &str = include_str!("../../assets/quotes/english.txt");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Directory bundled word lists ship from in a source checkout. Embedding
/// English keeps the app working after packaging; other languages load from
/// here at runtime and simply won't be available if the directory is absent.
fn words_asset_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/words")
}

fn quotes_asset_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/quotes")
}

/// Reads one asset file, if present.
fn read_asset(dir: &Path, file: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(file)).ok()
}

/// Every selectable language: "English" (always embedded) first, then the
/// plain (non-difficulty) word lists found under `assets/words/`.
pub fn available_languages() -> Vec<String> {
    let mut names = vec!["English".to_owned()];
    let dir = words_asset_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return names;
    };
    let mut found: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .filter_map(|path| {
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            if stem.contains('-') {
                return None;
            }
            Some(stem)
        })
        .filter(|stem| !stem.eq_ignore_ascii_case("english"))
        .collect();
    found.sort();
    names.extend(found.into_iter().filter(|name| !name.is_empty()));
    names
}

fn parse_words(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Word list for a language: difficulty-specific file first, then the plain
/// language file. Returns `None` when neither exists.
pub fn language_word_list(slug: &str, difficulty: &str) -> Option<String> {
    let dir = words_asset_dir();
    read_asset(&dir, &format!("{slug}-{difficulty}.txt"))
        .or_else(|| read_asset(&dir, &format!("{slug}.txt")))
}

/// Quote list for a language, falling back to the embedded English quotes.
pub fn language_quote_list(slug: &str) -> String {
    let dir = quotes_asset_dir();
    read_asset(&dir, &format!("{slug}.txt")).unwrap_or_else(|| QUOTES.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_always_available_first() {
        let languages = available_languages();
        assert_eq!(languages.first().map(String::as_str), Some("English"));
        assert!(languages.iter().any(|name| name == "English"));
    }

    #[test]
    fn language_word_asset_falls_back_to_plain_list() {
        // english.txt exists; english-easy.txt does not.
        let list = language_word_list("english", "easy").expect("english pool");
        assert!(!list.trim().is_empty());
    }

    #[test]
    fn unknown_language_has_no_asset() {
        assert!(language_word_list("klingon", "easy").is_none());
    }

    #[test]
    fn quotes_fall_back_to_english() {
        assert!(!language_quote_list("klingon").trim().is_empty());
    }
}
