use dytype::typing::words::Words;
use std::path::Path;

#[test]
fn default_words_are_not_empty() {
    let words = Words::builtin();
    assert!(!words.list.is_empty());
}

#[test]
fn loads_words_from_file() {
    let words = Words::load(Path::new("assets/words/english.txt")).expect("word list loads");
    assert!(!words.list.is_empty());
}

#[test]
fn shuffled_preserves_word_multiset() {
    let words = Words::builtin();
    let shuffled = words.shuffled();
    assert_eq!(shuffled.list.len(), words.list.len());
    let mut original = words.list.clone();
    original.sort();
    let mut result = shuffled.list.clone();
    result.sort();
    assert_eq!(original, result);
}
