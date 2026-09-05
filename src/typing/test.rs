use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    Time(Duration),
    Words(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    NotStarted,
    Running,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    Unseen,
    Correct,
    Incorrect,
}

#[derive(Debug, Clone)]
pub struct Keystroke {
    pub key: char,
    pub position: usize,
    pub correct: bool,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypingError {
    pub position: usize,
    pub expected: Option<char>,
    pub actual: char,
}

pub struct TypingTest {
    pub text: String,
    pub chars: Vec<char>,
    pub cursor: usize,
    pub states: Vec<CharState>,
    pub status: TestStatus,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub errors: Vec<TypingError>,
    pub keystrokes: Vec<Keystroke>,
    pub mode: TestMode,
    pub duration: Option<Duration>,
    word_end: Vec<usize>,
}

const BACKSPACE: char = '\u{0008}';
const DELETE: char = '\u{007f}';

pub const BACKSPACE_KEY: char = BACKSPACE;

impl TypingTest {
    pub fn new(mode: TestMode, words: &[String]) -> Self {
        let text = words.join(" ");
        let chars: Vec<char> = text.chars().collect();
        let word_end = {
            let mut ends = Vec::new();
            for (index, &ch) in chars.iter().enumerate() {
                if ch == ' ' {
                    ends.push(index);
                }
            }
            ends.push(chars.len());
            ends
        };
        let states = vec![CharState::Unseen; chars.len()];
        Self {
            text,
            chars,
            cursor: 0,
            states,
            status: TestStatus::NotStarted,
            started_at: None,
            finished_at: None,
            correct_chars: 0,
            incorrect_chars: 0,
            errors: Vec::new(),
            keystrokes: Vec::new(),
            mode,
            duration: None,
            word_end,
        }
    }

    pub fn start(&mut self, now: Instant) {
        if self.status != TestStatus::NotStarted {
            return;
        }
        self.started_at = Some(now);
        self.status = TestStatus::Running;
    }

    pub fn char_at(&self, index: usize) -> Option<char> {
        self.chars.get(index).copied()
    }

    pub fn char_under_cursor(&self) -> Option<char> {
        self.char_at(self.cursor)
    }

    pub fn is_running(&self) -> bool {
        self.status == TestStatus::Running
    }

    pub fn is_finished(&self) -> bool {
        self.status == TestStatus::Finished
    }

    pub fn completed_words(&self) -> usize {
        let last = self.word_end.len().saturating_sub(1);
        (0..self.word_end.len())
            .filter(|&i| {
                if i == last {
                    self.cursor >= self.word_end[i]
                } else {
                    self.cursor > self.word_end[i]
                }
            })
            .count()
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        self.started_at
            .map(|start| now.saturating_duration_since(start))
            .unwrap_or_default()
    }

    pub fn handle_key(&mut self, key: char, now: Instant) {
        if self.status == TestStatus::Finished {
            return;
        }
        if key == BACKSPACE || key == DELETE {
            if self.status != TestStatus::NotStarted {
                self.backspace();
            }
            return;
        }
        if self.status == TestStatus::NotStarted {
            self.start(now);
        }
        if self.cursor < self.chars.len() && self.chars[self.cursor] == key {
            self.record_correct(key, now);
        } else {
            self.record_incorrect(key, now);
        }
        self.check_finish(now);
    }

    pub fn tick(&mut self, now: Instant) {
        if self.status == TestStatus::Running {
            self.check_finish(now);
        }
    }

    pub fn submit(&mut self, now: Instant) {
        self.finish(now);
    }

    fn record_correct(&mut self, key: char, now: Instant) {
        let position = self.cursor;
        self.states[position] = CharState::Correct;
        self.correct_chars += 1;
        self.keystrokes.push(Keystroke {
            key,
            position,
            correct: true,
            timestamp: now,
        });
        self.cursor += 1;
    }

    fn record_incorrect(&mut self, key: char, now: Instant) {
        let position = self.cursor;
        if position < self.chars.len() {
            self.states[position] = CharState::Incorrect;
        }
        let expected = self.char_at(position);
        self.incorrect_chars += 1;
        self.errors.push(TypingError {
            position,
            expected,
            actual: key,
        });
        self.keystrokes.push(Keystroke {
            key,
            position,
            correct: false,
            timestamp: now,
        });
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        if let Some(state) = self.states.get_mut(self.cursor)
            && *state != CharState::Unseen
        {
            *state = CharState::Unseen;
        }
    }

    fn check_finish(&mut self, now: Instant) {
        let finished = match self.mode {
            TestMode::Words(count) => self.completed_words() >= count,
            TestMode::Time(duration) => self.elapsed(now) >= duration,
        };
        if finished {
            self.finish(now);
        }
    }

    fn finish(&mut self, now: Instant) {
        if self.status != TestStatus::Running {
            return;
        }
        self.status = TestStatus::Finished;
        self.finished_at = Some(now);
        self.duration = self
            .started_at
            .map(|start| now.saturating_duration_since(start));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_words() -> Vec<String> {
        vec!["foo".to_owned(), "bar".to_owned()]
    }

    fn run_keys(test: &mut TypingTest, keys: &str, now: Instant) {
        for key in keys.chars() {
            test.handle_key(key, now);
        }
    }

    #[test]
    fn fresh_test_is_not_started() {
        let test = TypingTest::new(TestMode::Words(2), &test_words());
        assert_eq!(test.status, TestStatus::NotStarted);
        assert_eq!(test.cursor, 0);
        assert_eq!(test.completed_words(), 0);
    }

    #[test]
    fn correct_key_starts_test_and_advances() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        test.handle_key('f', Instant::now());
        assert_eq!(test.status, TestStatus::Running);
        assert_eq!(test.cursor, 1);
        assert_eq!(test.correct_chars, 1);
        assert_eq!(test.states[0], CharState::Correct);
    }

    #[test]
    fn incorrect_key_records_error_without_advancing() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        test.handle_key('z', Instant::now());
        assert_eq!(test.cursor, 0);
        assert_eq!(test.incorrect_chars, 1);
        assert_eq!(test.errors.len(), 1);
        assert_eq!(test.errors[0].expected, Some('f'));
        assert_eq!(test.errors[0].actual, 'z');
        assert_eq!(test.states[0], CharState::Incorrect);
    }

    #[test]
    fn space_advances_to_next_word() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        run_keys(&mut test, "foo", Instant::now());
        assert_eq!(test.cursor, 3);
        assert_eq!(test.completed_words(), 0);
        test.handle_key(' ', Instant::now());
        assert_eq!(test.cursor, 4);
        assert_eq!(test.completed_words(), 1);
    }

    #[test]
    fn stray_key_at_word_boundary_is_an_error() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        run_keys(&mut test, "fooz", Instant::now());
        assert_eq!(test.cursor, 3);
        assert_eq!(test.errors.len(), 1);
        assert_eq!(test.keystrokes.len(), 4);
        assert_eq!(test.errors[0].expected, Some(' '));
    }

    #[test]
    fn backspace_reverts_last_char() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        run_keys(&mut test, "fo", Instant::now());
        assert_eq!(test.cursor, 2);
        test.handle_key(DELETE, Instant::now());
        assert_eq!(test.cursor, 1);
        assert_eq!(test.states[1], CharState::Unseen);
    }

    #[test]
    fn backspace_before_start_does_not_start_test() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        test.handle_key(BACKSPACE, Instant::now());
        assert_eq!(test.status, TestStatus::NotStarted);
        assert_eq!(test.cursor, 0);
    }

    #[test]
    fn word_mode_finishes_after_completing_words() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        run_keys(&mut test, "foo bar", Instant::now());
        assert_eq!(test.status, TestStatus::Finished);
        assert!(test.finished_at.is_some());
        assert!(test.duration.is_some());
        assert_eq!(test.keystrokes.len(), 7);
    }

    #[test]
    fn mistake_can_be_fixed_and_still_finish() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        run_keys(&mut test, "fioo bar", Instant::now());
        assert_eq!(test.status, TestStatus::Finished);
        assert_eq!(test.incorrect_chars, 1);
        assert_eq!(test.errors.len(), 1);
        assert_eq!(test.correct_chars, 7);
    }

    #[test]
    fn time_mode_finishes_after_duration() {
        let start = Instant::now();
        let mut test = TypingTest::new(
            TestMode::Time(Duration::from_secs(1)),
            &test_words(),
        );
        test.handle_key('f', start);
        assert_eq!(test.status, TestStatus::Running);
        test.tick(start + Duration::from_millis(900));
        assert_eq!(test.status, TestStatus::Running);
        test.tick(start + Duration::from_millis(1200));
        assert_eq!(test.status, TestStatus::Finished);
        assert_eq!(test.duration, Some(Duration::from_millis(1200)));
    }

    #[test]
    fn finished_test_ignores_keys() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        run_keys(&mut test, "foo bar", Instant::now());
        let keystrokes = test.keystrokes.len();
        test.handle_key('x', Instant::now());
        assert_eq!(test.keystrokes.len(), keystrokes);
        assert_eq!(test.status, TestStatus::Finished);
    }
}