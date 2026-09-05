use std::io;

use crate::stats::history::{History, TestResult};

pub struct Database;

impl Database {
    pub fn open() -> io::Result<Self> {
        Ok(Self)
    }

    pub fn load_history(&self) -> io::Result<History> {
        Ok(History::new())
    }

    pub fn save_result(&self, _result: &TestResult) -> io::Result<()> {
        Ok(())
    }
}