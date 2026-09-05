use crate::stats::wpm::Wpm;
use crate::typing::test::Keystroke;

pub struct Consistency;

impl Consistency {
    pub fn calculate(wpm_samples: &[f64]) -> f64 {
        if wpm_samples.is_empty() {
            return 0.0;
        }
        let mean = wpm_samples.iter().sum::<f64>() / wpm_samples.len() as f64;
        let variance = wpm_samples
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / wpm_samples.len() as f64;
        let std_dev = variance.sqrt();
        if mean == 0.0 {
            return 0.0;
        }
        (100.0 - (std_dev / mean * 100.0)).clamp(0.0, 100.0)
    }

    pub fn from_keystrokes(keystrokes: &[Keystroke]) -> f64 {
        if keystrokes.len() < 2 {
            return 100.0;
        }
        let first = keystrokes[0].timestamp;
        let last = keystrokes[keystrokes.len() - 1].timestamp;
        let total_secs = last.saturating_duration_since(first).as_secs();
        let mut buckets = vec![0usize; total_secs as usize + 1];
        for stroke in keystrokes {
            let index = stroke.timestamp.saturating_duration_since(first).as_secs() as usize;
            if let Some(bucket) = buckets.get_mut(index) {
                *bucket += 1;
            }
        }
        let samples: Vec<f64> = buckets.iter().map(|&count| Wpm::calculate(count, 1.0)).collect();
        Consistency::calculate(&samples)
    }
}