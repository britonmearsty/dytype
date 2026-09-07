use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::stats::history::History;
use crate::stats::result::{CharStat, TestResult};
use crate::typing::generator::Difficulty;
use crate::typing::test::TestMode;

/// Append-only tab-separated persistence for completed tests. Each test is a
/// single line; the explanation of every field lives in [`encode`]/[`decode`].
pub struct Database {
    path: PathBuf,
}

fn default_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "dytype", "dytype")
        .map(|dirs| dirs.config_dir().join("history.tsv"))
}

fn io_missing(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, message)
}

impl Database {
    pub fn open() -> std::io::Result<Self> {
        let path = default_path().ok_or_else(|| io_missing("no config directory available"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }

    /// Opens the database at a specific file (used by tests).
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> std::io::Result<Vec<TestResult>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.path)?;
        let mut results = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if let Some(result) = decode(&line) {
                results.push(result);
            }
        }
        results.sort_by_key(|r| r.id);
        Ok(results)
    }

    pub fn load_history(&self) -> std::io::Result<History> {
        Ok(History {
            results: self.load()?,
        })
    }

    pub fn append(&self, result: &TestResult) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", encode(result))
    }

    /// Rewrites the whole file (used to collapse/repair, and by tests).
    pub fn save_all(&self, results: &[TestResult]) -> std::io::Result<()> {
        let mut content = String::new();
        for result in results {
            content.push_str(&encode(result));
            content.push('\n');
        }
        fs::write(&self.path, content)
    }
}

fn encode(result: &TestResult) -> String {
    let mode = match result.mode {
        TestMode::Words(count) => format!("w{count}"),
        TestMode::Time(duration) => format!("t{}", duration.as_secs()),
    };
    let char_stats: String = result
        .char_stats
        .iter()
        .map(|stat| {
            let prev = stat.prev.map_or(String::new(), |c| (c as u32).to_string());
            let prev2 = stat.prev2.map_or(String::new(), |c| (c as u32).to_string());
            format!(
                "{}:{}:{}:{}:{};",
                stat.ch as u32, stat.typed, stat.errors, prev, prev2
            )
        })
        .collect();
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        result.id,
        result.timestamp,
        format_float(result.wpm),
        format_float(result.raw_wpm),
        format_float(result.accuracy),
        format_float(result.error_rate),
        format_float(result.consistency),
        format_float(result.cpm),
        result.characters,
        result.correct_chars,
        result.incorrect_chars,
        result.errors,
        result.correct_keystrokes,
        result.incorrect_keystrokes,
        result.completed_words,
        result.avg_key_interval_ms,
        result.duration_ms,
        mode,
        result.difficulty.label(),
        char_stats,
    )
}

fn format_float(value: f64) -> String {
    // Rust's default float Display is the shortest representation that
    // round-trips exactly, which keeps the history file lossless and compact.
    value.to_string()
}

fn decode(line: &str) -> Option<TestResult> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() != 20 {
        return None;
    }
    let index = |i: usize| fields[i];
    let number = |i: usize| index(i).parse::<f64>().ok();
    let integer = |i: usize| index(i).parse::<usize>().ok();
    let mode = decode_mode(index(17))?;
    let difficulty = match index(18) {
        "easy" => Difficulty::Easy,
        "hard" => Difficulty::Hard,
        "expert" => Difficulty::Expert,
        _ => Difficulty::Normal,
    };
    Some(TestResult {
        id: integer(0)? as u64,
        timestamp: integer(1)? as u64,
        wpm: number(2)?,
        raw_wpm: number(3)?,
        accuracy: number(4)?,
        error_rate: number(5)?,
        consistency: number(6)?,
        cpm: number(7)?,
        characters: integer(8)?,
        correct_chars: integer(9)?,
        incorrect_chars: integer(10)?,
        errors: integer(11)?,
        correct_keystrokes: integer(12)?,
        incorrect_keystrokes: integer(13)?,
        completed_words: integer(14)?,
        avg_key_interval_ms: integer(15)? as u64,
        duration_ms: integer(16)? as u64,
        mode,
        difficulty,
        char_stats: decode_char_stats(index(19)),
    })
}

fn decode_mode(field: &str) -> Option<TestMode> {
    if let Some(count) = field.strip_prefix('w') {
        count.parse().ok().map(TestMode::Words)
    } else if let Some(secs) = field.strip_prefix('t') {
        secs.parse()
            .ok()
            .map(|s| TestMode::Time(std::time::Duration::from_secs(s)))
    } else {
        None
    }
}

fn decode_char_stats(field: &str) -> Vec<CharStat> {
    let mut stats = Vec::new();
    for entry in field.split(';') {
        if entry.is_empty() {
            continue;
        }
        let parts: Vec<&str> = entry.split(':').collect();
        if parts.len() != 3 && parts.len() != 5 {
            continue;
        }
        let (Some(code), Some(typed), Some(errors)) = (parts.first(), parts.get(1), parts.get(2))
        else {
            continue;
        };
        if let (Some(ch), Ok(typed), Ok(errors)) = (
            code.parse::<u32>().ok().and_then(char::from_u32),
            typed.parse::<u32>(),
            errors.parse::<u32>(),
        ) {
            let (prev, prev2) = if parts.len() == 5 {
                (
                    parts[3].parse().ok().and_then(char::from_u32),
                    parts[4].parse().ok().and_then(char::from_u32),
                )
            } else {
                (None, None)
            };
            stats.push(CharStat {
                ch,
                typed,
                errors,
                prev,
                prev2,
            });
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing::test::TypingTest;
    use std::time::{Duration, Instant};

    fn sample_result(id: u64, wpm: f64, chars: usize) -> TestResult {
        let words: Vec<String> = vec!["foo".to_owned(), "bar".to_owned()];
        let mut test = TypingTest::new(TestMode::Words(2), &words);
        let start = Instant::now();
        for (i, key) in "foo bar".chars().enumerate() {
            test.handle_key(key, start + Duration::from_millis(i as u64 * 90));
        }
        test.submit(start + Duration::from_millis(1500));
        let mut result = TestResult::build(id, &test, Difficulty::Hard, start);
        result.wpm = wpm;
        result.characters = chars;
        result
    }

    fn temp_path(unique: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "dytype-history-test-{}-{unique}.tsv",
            std::process::id()
        ))
    }

    #[test]
    fn encode_decode_roundtrips() {
        let mut result = sample_result(42, 77.5, 90);
        result.char_stats = vec![
            CharStat {
                ch: 'e',
                typed: 30,
                errors: 4,
                prev: Some('h'),
                prev2: None,
            },
            CharStat {
                ch: ' ',
                typed: 10,
                errors: 0,
                prev: None,
                prev2: None,
            },
        ];
        let decoded = decode(&encode(&result)).expect("decodes");
        assert_eq!(decoded, result);
    }

    #[test]
    fn decode_char_stats_accepts_legacy_three_part_entries() {
        // Pre-bigram records stored only `code:typed:errors;`; decoding must
        // fall back to no context rather than dropping the entry.
        let field = "114:30:4;32:10:0;";
        let stats = decode_char_stats(field);
        assert_eq!(
            stats,
            vec![
                CharStat {
                    ch: 'r',
                    typed: 30,
                    errors: 4,
                    prev: None,
                    prev2: None
                },
                CharStat {
                    ch: ' ',
                    typed: 10,
                    errors: 0,
                    prev: None,
                    prev2: None
                },
            ]
        );
    }

    #[test]
    fn decode_char_stats_skips_garbage_entries() {
        let stats = decode_char_stats("114:30:4;garbage;:1:;114:x:4;");
        assert_eq!(
            stats,
            vec![CharStat {
                ch: 'r',
                typed: 30,
                errors: 4,
                prev: None,
                prev2: None
            }]
        );
    }

    #[test]
    fn decode_char_stats_reads_context_columns() {
        let field = "114:30:4:104:115;111:5:0::;";
        let stats = decode_char_stats(field);
        assert_eq!(
            stats,
            vec![
                CharStat {
                    ch: 'r',
                    typed: 30,
                    errors: 4,
                    prev: Some('h'),
                    prev2: Some('s')
                },
                CharStat {
                    ch: 'o',
                    typed: 5,
                    errors: 0,
                    prev: None,
                    prev2: None
                },
            ]
        );
    }

    #[test]
    fn float_encoding_roundtrips() {
        assert_eq!(format_float(77.5), "77.5");
        assert_eq!(format_float(100.0), "100");
        assert_eq!(format_float(87.125), "87.125");
        assert_eq!(
            decode(&encode(&sample_result(1, 87.125, 10))).unwrap().wpm,
            87.125
        );
    }

    #[test]
    fn append_and_load_persist_everything() {
        let path = temp_path("append");
        let _ = fs::remove_file(&path);
        let db = Database::at(&path);
        db.append(&sample_result(1, 50.0, 10)).expect("append");
        db.append(&sample_result(2, 60.0, 20)).expect("append");
        let loaded = db.load().expect("load");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, 1);
        assert_eq!(loaded[1].id, 2);
        assert_eq!(loaded[1].wpm, 60.0);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_is_empty() {
        let path = temp_path("missing");
        let _ = fs::remove_file(&path);
        let db = Database::at(&path);
        assert!(db.load().expect("load missing").is_empty());
    }

    #[test]
    fn corrupt_lines_are_skipped() {
        let path = temp_path("corrupt");
        let _ = fs::remove_file(&path);
        let db = Database::at(&path);
        db.save_all(&[sample_result(1, 50.0, 5)]).expect("save");
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open")
            .write_all(b"garbage\tline\n")
            .expect("write");
        let loaded = db.load().expect("load");
        assert_eq!(loaded.len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn mode_decode_is_robust() {
        assert_eq!(decode_mode("w25"), Some(TestMode::Words(25)));
        assert_eq!(
            decode_mode("t30"),
            Some(TestMode::Time(Duration::from_secs(30)))
        );
        assert_eq!(decode_mode("x"), None);
    }
}
