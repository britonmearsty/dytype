/// Pure ASCII chart renderers. No terminal dependencies; everything is a
/// plain `String` so callers can lay out charts however they like.
/// Samples `values` down (or up) to exactly `width` columns by nearest-index
/// selection, so charts always fill their allotted horizontal space.
fn sample_to_width(values: &[f64], width: usize) -> Vec<f64> {
    if values.is_empty() || width == 0 {
        return Vec::new();
    }
    (0..width)
        .map(|column| {
            let index =
                ((column as f64 + 0.5) * values.len() as f64 / width as f64).floor() as usize;
            values.get(index).copied().unwrap_or(0.0)
        })
        .collect()
}

fn glyph(previous: Option<i32>, current: i32, next: Option<i32>) -> char {
    match (previous, next) {
        (Some(p), Some(n)) => {
            if p == current && n == current {
                '─'
            } else if p == current {
                if n < current { '╭' } else { '╯' }
            } else if n == current {
                if p > current { '╰' } else { '╮' }
            } else if p > current && n > current {
                '╭'
            } else if p < current && n < current {
                '╯'
            } else {
                '│'
            }
        }
        (None, Some(n)) => {
            if n == current {
                '─'
            } else if n < current {
                '╭'
            } else {
                '╯'
            }
        }
        (Some(p), None) => {
            if p == current {
                '─'
            } else if p < current {
                '╲'
            } else {
                '╱'
            }
        }
        (None, None) => '─',
    }
}

/// A single-line sparkline using eight block heights, `▁▂▃▄▅▆▇█`.
pub fn spark(values: &[f64], width: usize) -> String {
    if values.is_empty() || width == 0 {
        return String::new();
    }
    const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = values.iter().cloned().fold(0.0f64, f64::max).max(1e-9);
    sample_to_width(values, width)
        .into_iter()
        .map(|value| {
            let t = (value / max).clamp(0.0, 1.0) as f32;
            let index = (t * 7.99).floor() as usize;
            BLOCKS[index]
        })
        .collect()
}

/// A multi-row line chart rendered into box-drawing glyphs. Values are scaled
/// against the maximum so taller values draw higher rows.
///
/// Returns `height` rows, each exactly `width` characters long.
pub fn line_chart(values: &[f64], width: usize, height: usize) -> Vec<String> {
    let height = height.max(2);
    let width = width.max(1);
    let sampled = sample_to_width(values, width);
    if sampled.is_empty() {
        return vec![" ".repeat(width); height];
    }
    let max = sampled.iter().cloned().fold(0.0f64, f64::max).max(1e-9);
    let rows: Vec<i32> = sampled
        .iter()
        .map(|&value| {
            let t = (value / max).clamp(0.0, 1.0);
            ((1.0 - t) * (height - 1) as f64).round() as i32
        })
        .collect();
    let mut grid = vec![vec![' '; width]; height];
    for column in 0..width {
        let current = rows[column];
        let previous = if column > 0 {
            Some(rows[column - 1])
        } else {
            None
        };
        let next = if column + 1 < width {
            Some(rows[column + 1])
        } else {
            None
        };
        if let Some(previous_row) = previous {
            for row in (current.min(previous_row) + 1)..current.max(previous_row) {
                grid[row as usize][column] = '│';
            }
        }
        grid[current as usize][column] = glyph(previous, current, next);
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

/// Fills a horizontal bar proportional to `value / max`, using `█` blocks.
/// The returned string is exactly `width` characters.
pub fn bar_row(value: f64, max: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let scale = max.max(1e-9);
    let filled = ((value / scale).clamp(0.0, 1.0) * width as f64).round() as usize;
    let mut bar: String = "█".repeat(filled);
    bar.extend(std::iter::repeat_n('░', width.saturating_sub(filled)));
    bar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_line_is_horizontal() {
        // A constant series sits at the maximum value, i.e. the top row.
        let rows = line_chart(&[5.0, 5.0, 5.0, 5.0], 4, 5);
        assert_eq!(rows, vec!["────", "    ", "    ", "    ", "    "]);
    }

    #[test]
    fn rising_line_climbs() {
        let rows = line_chart(&[1.0, 2.0, 3.0], 3, 5);
        // Larger values map to smaller row indexes (higher on screen).
        assert!(rows[0].contains('╱'));
        assert!(rows[1].contains('│'));
        assert!(rows[3].contains('╭'));
    }

    #[test]
    fn peak_is_rendered_on_top() {
        let rows = line_chart(&[1.0, 5.0, 1.0], 3, 5);
        assert!(rows[0].contains('╭'));
        assert!(!rows[4].contains('╭'));
    }

    #[test]
    fn empty_values_render_blank_grid() {
        let rows = line_chart(&[], 4, 3);
        assert_eq!(rows, vec!["    ", "    ", "    "]);
    }

    #[test]
    fn sampling_keeps_constant_width() {
        let rows = line_chart(&[1.0, 2.0, 3.0], 10, 3);
        assert!(rows.iter().all(|row| row.chars().count() == 10));
    }

    #[test]
    fn zero_values_draw_bottomed_line() {
        let rows = line_chart(&[0.0, 0.0], 2, 4);
        assert_eq!(rows[3], "──");
    }

    #[test]
    fn spark_is_bounded_blocks() {
        let out = spark(&[1.0, 2.0, 3.0, 4.0], 4);
        assert_eq!(out.chars().count(), 4);
        assert!(out.chars().all(|c| "▁▂▃▄▅▆▇█".contains(c)));
        assert_eq!(spark(&[0.0; 4], 4), "▁▁▁▁");
        assert_eq!(spark(&[10.0; 4], 4), "████");
    }

    #[test]
    fn bar_row_scales_proportionally() {
        assert_eq!(bar_row(5.0, 10.0, 10), "█████░░░░░");
        assert_eq!(bar_row(10.0, 10.0, 10), "██████████");
        assert_eq!(bar_row(0.0, 10.0, 10), "░░░░░░░░░░");
        assert_eq!(bar_row(3.0, 10.0, 5), "██░░░");
    }
}
