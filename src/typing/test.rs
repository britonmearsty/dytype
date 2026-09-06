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
    pub position: usize,
    pub expected: char,
    pub actual: char,
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
    paused_at: Option<Instant>,
    paused_total: Duration,
    word_end: Vec<usize>,
}

const BACKSPACE: char = '\u{0008}';
const DELETE: char = '\u{007f}';

pub const BACKSPACE_KEY: char = BACKSPACE;

fn char_to_optional(ch: char) -> Option<char> {
    (ch != '\0').then_some(ch)
}

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
            paused_at: None,
            paused_total: Duration::ZERO,
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

    /// Returns the space-separated word token containing `position`, if any.
    pub fn word_at(&self, position: usize) -> Option<String> {
        if position >= self.chars.len() {
            return None;
        }
        let start = self.chars[..position]
            .iter()
            .rposition(|&c| c == ' ')
            .map_or(0, |i| i + 1);
        let end = self.chars[position..]
            .iter()
            .position(|&c| c == ' ')
            .map_or(self.chars.len(), |i| position + i);
        Some(self.chars[start..end].iter().collect())
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
        let Some(start) = self.started_at else {
            return Duration::ZERO;
        };
        let floor = self.paused_at.unwrap_or(now);
        floor
            .saturating_duration_since(start)
            .saturating_sub(self.paused_total)
    }

    pub fn deadline(&self) -> Option<Instant> {
        match self.mode {
            TestMode::Time(duration) => {
                self.started_at.map(|start| start + duration + self.paused_total)
            }
            TestMode::Words(_) => None,
        }
    }

    pub fn pause(&mut self, now: Instant) {
        if self.status == TestStatus::Running && self.paused_at.is_none() {
            self.paused_at = Some(now);
        }
    }

    pub fn resume(&mut self, now: Instant) {
        if let Some(at) = self.paused_at.take() {
            self.paused_total += now.saturating_duration_since(at);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
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
            position,
            expected: self.chars[position],
            actual: key,
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
        let expected = self.char_at(position).unwrap_or('\0');
        self.incorrect_chars += 1;
        self.errors.push(TypingError {
            position,
            expected: char_to_optional(expected),
            actual: key,
        });
        self.keystrokes.push(Keystroke {
            position,
            expected,
            actual: key,
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
        if !finished {
            return;
        }
        let finish_at = match self.mode {
            TestMode::Time(duration) => {
                self.started_at.map_or(now, |start| start + duration + self.paused_total)
            }
            TestMode::Words(_) => now,
        };
        self.finish(finish_at);
    }

    fn finish(&mut self, now: Instant) {
        if self.status != TestStatus::Running {
            return;
        }
        self.status = TestStatus::Finished;
        self.finished_at = Some(now);
        self.duration = self
            .started_at
            .map(|start| now.saturating_duration_since(start).saturating_sub(self.paused_total));
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
    fn keystrokes_record_expected_actual_and_timestamp() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        let now = Instant::now();
        test.handle_key('x', now);
        let stroke = &test.keystrokes[0];
        assert_eq!(stroke.position, 0);
        assert_eq!(stroke.expected, 'f');
        assert_eq!(stroke.actual, 'x');
        assert!(!stroke.correct);
        assert_eq!(stroke.timestamp, now);
    }

    #[test]
    fn time_mode_reports_deadline_after_start() {
        let now = Instant::now();
        let mut test = TypingTest::new(TestMode::Time(Duration::from_secs(30)), &test_words());
        assert_eq!(test.deadline(), None);
        test.handle_key('f', now);
        assert_eq!(test.deadline(), Some(now + Duration::from_secs(30)));
    }

    #[test]
    fn word_mode_has_no_deadline() {
        let test = TypingTest::new(TestMode::Words(2), &test_words());
        assert_eq!(test.deadline(), None);
    }

    #[test]
    fn pause_freezes_elapsed_and_deadline_until_resume() {
        let start = Instant::now();
        let mut test = TypingTest::new(
            TestMode::Time(Duration::from_secs(30)),
            &test_words(),
        );
        test.handle_key('f', start);
        assert_eq!(test.status, TestStatus::Running);
        assert_eq!(test.deadline(), Some(start + Duration::from_secs(30)));

        test.pause(start + Duration::from_secs(1));
        assert!(test.is_paused());
        let while_paused = start + Duration::from_secs(10);
        assert_eq!(test.elapsed(while_paused), Duration::from_secs(1));
        test.tick(while_paused);
        assert_eq!(test.status, TestStatus::Running);

        test.resume(while_paused);
        assert!(!test.is_paused());
        let after_resume = start + Duration::from_secs(15);
        let paused_window = while_paused - (start + Duration::from_secs(1));
        assert_eq!(
            test.elapsed(after_resume),
            Duration::from_secs(15) - paused_window
        );
        assert_eq!(
            test.deadline(),
            Some(start + Duration::from_secs(30) + paused_window)
        );
    }

    #[test]
    fn pause_before_start_is_a_noop() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        test.pause(Instant::now());
        assert!(!test.is_paused());
        assert_eq!(test.status, TestStatus::NotStarted);
    }

    #[test]
    fn resume_without_pause_is_a_noop() {
        let mut test = TypingTest::new(TestMode::Words(2), &test_words());
        test.resume(Instant::now());
        assert!(!test.is_paused());
    }

    #[test]
    fn time_mode_still_finishes_after_pausing() {
        let start = Instant::now();
        let mut test = TypingTest::new(
            TestMode::Time(Duration::from_secs(1)),
            &test_words(),
        );
        test.handle_key('f', start);
        test.pause(start + Duration::from_millis(200));
        test.resume(start + Duration::from_millis(900));
        assert_eq!(test.status, TestStatus::Running);
        test.tick(start + Duration::from_millis(1100));
        assert_eq!(test.status, TestStatus::Running);
        test.tick(start + Duration::from_millis(1800));
        assert_eq!(test.status, TestStatus::Finished);
        assert_eq!(test.duration, Some(Duration::from_secs(1)));
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
        assert_eq!(test.duration, Some(Duration::from_secs(1)));
        assert_eq!(test.finished_at, Some(start + Duration::from_secs(1)));
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

    #[test]
    fn word_at_resolves_token_for_position() {
        let test = TypingTest::new(TestMode::Words(2), &test_words());
        assert_eq!(test.word_at(0), Some("foo".to_owned()));
        assert_eq!(test.word_at(2), Some("foo".to_owned()));
        assert_eq!(test.word_at(3), Some("foo".to_owned()));
        assert_eq!(test.word_at(4), Some("bar".to_owned()));
        assert_eq!(test.word_at(6), Some("bar".to_owned()));
        assert_eq!(test.word_at(99), None);
    }
}