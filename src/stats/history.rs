use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TestResult {
    pub wpm: f64,
    pub raw: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub mode: String,
    pub characters: usize,
    pub timestamp: u64,
}

pub struct History {
    pub results: Vec<TestResult>,
}

impl History {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn push(&mut self, result: TestResult) {
        self.results.push(result);
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}