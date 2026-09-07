#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub position: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { position: 0 }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
