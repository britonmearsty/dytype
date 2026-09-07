pub struct Accuracy;

impl Accuracy {
    pub fn calculate(correct: usize, total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        correct as f64 / total as f64 * 100.0
    }
}
