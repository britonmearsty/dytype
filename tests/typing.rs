use std::time::{Duration, Instant};

use dytype::typing::engine::Engine;
use dytype::typing::test::{TestMode, TestStatus};
use dytype::typing::words::Words;

fn first(capacity: usize) -> Vec<String> {
    let words = Words::builtin();
    words.list.into_iter().take(capacity).collect()
}

#[test]
fn engine_finishes_word_test() {
    let words = first(2);
    let mut engine = Engine::with_test(TestMode::Words(2), &words);
    let start = Instant::now();
    for key in "the be".chars() {
        engine.handle_key(key, start);
    }
    assert_eq!(engine.test.status, TestStatus::Finished);
}

#[test]
fn engine_runs_time_test() {
    let mode = TestMode::Time(Duration::from_secs(15));
    let mut engine = Engine::with_test(mode, &first(50));
    engine.handle_key('t', Instant::now());
    assert_eq!(engine.test.status, TestStatus::Running);
    engine.handle_key('h', Instant::now());
    engine.handle_key('e', Instant::now());
    assert_eq!(engine.test.cursor, 3);
}