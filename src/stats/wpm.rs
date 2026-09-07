pub struct Wpm {
    pub wpm: f64,
    pub raw: f64,
}

impl Wpm {
    pub fn calculate(correct_chars: usize, seconds: f64) -> f64 {
        if seconds <= 0.0 {
            return 0.0;
        }
        correct_chars as f64 / 5.0 / (seconds / 60.0)
    }
}
