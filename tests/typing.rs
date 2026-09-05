use dytype::typing::engine::Engine;
use dytype::typing::test::{TestMode, TestStatus};
use dytype::typing::words::Words;

fn builtin() -> Words {
    Words::builtin()
}

#[test]
fn engine_finishes_word_test() {
    let words = builtin();
    let mut engine = Engine::with_words(TestMode::Words(2), &words);
    for key in "the be".chars() {
        engine.handle_key(key);
    }
    assert_eq!(engine.test.status, TestStatus::Finished);
}

#[test]
fn engine_restart_builds_fresh_test() {
    let words = builtin();
    let mut engine = Engine::with_words(TestMode::Words(2), &words);
    for key in "the be".chars() {
        engine.handle_key(key);
    }
    assert!(engine.test.is_finished());
    engine.restart(&words);
    assert_eq!(engine.test.status, TestStatus::NotStarted);
    assert_eq!(engine.test.cursor, 0);
}

#[test]
fn engine_runs_time_test() {
    let words = builtin();
    let mode = TestMode::Time(std::time::Duration::from_secs(15));
    let mut engine = Engine::with_words(mode, &words);
    engine.handle_key('t');
    assert_eq!(engine.test.status, TestStatus::Running);
}