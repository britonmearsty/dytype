pub use crate::stats::result::TestResult;

/// In-memory store of every completed test, in insertion order.
#[derive(Debug, Clone, Default)]
pub struct History {
    pub results: Vec<TestResult>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// The id to assign to the next recorded test.
    pub fn next_id(&self) -> u64 {
        self.results.iter().map(|r| r.id).max().unwrap_or(0) + 1
    }
}
