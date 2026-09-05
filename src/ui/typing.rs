use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::typing::test::{CharState, TestStatus, TypingTest};

pub struct TypingScreen;

impl TypingScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(5)]).split(area);
        render_prompt(frame, chunks[0], &app.engine.test);
        render_stats(frame, chunks[1], app);
    }
}

fn render_prompt(frame: &mut Frame<'_>, area: Rect, test: &TypingTest) {
    let block = Block::default().title(" dytype ").borders(Borders::ALL);
    frame.render_widget(block, area);
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let wrapped = wrap_chars(test, inner.width as usize);
    let (cursor_row, cursor_col) = cursor_position(&wrapped, test.cursor);

    let mut lines = Vec::with_capacity(wrapped.len());
    for (row, indices) in wrapped.iter().enumerate() {
        let mut spans: Vec<Span> = indices
            .iter()
            .map(|&i| Span::styled(format!("{}", test.chars[i]), char_style(test.states[i])))
            .collect();
        if row == cursor_row {
            if cursor_col < spans.len() {
                spans[cursor_col].style =
                    spans[cursor_col].style.add_modifier(Modifier::REVERSED | Modifier::UNDERLINED);
            } else if cursor_col == spans.len() && cursor_col < inner.width as usize {
                spans.push(
                    Span::styled(
                        " ",
                        Style::default().fg(Color::Black).bg(Color::LightBlue),
                    ),
                );
            }
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_stats(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let ls = app.live_stats;
    let status = match app.engine.test.status {
        TestStatus::NotStarted => "type to start",
        TestStatus::Running => "running",
        TestStatus::Finished => "complete",
    };
    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:.0} WPM", ls.wpm),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   Raw: {:.0}", ls.raw_wpm),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("Accuracy: {:.1}%", ls.accuracy), Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("   Consistency: {:.0}%", ls.consistency),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(
                format!("   Time: {:.1}s", ls.elapsed.as_secs_f64()),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(format!(
            "[{status}]   {}   Tab: new test   Enter: finish   Esc: quit",
            app.config.label()
        )),
    ];
    let block = Block::default().title(" stats ").borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(lines).block(block).alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

fn char_style(state: CharState) -> Style {
    match state {
        CharState::Correct => Style::default().fg(Color::Green),
        CharState::Incorrect => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        CharState::Unseen => Style::default().fg(Color::Gray),
    }
}

fn wrap_chars(test: &TypingTest, max_chars: usize) -> Vec<Vec<usize>> {
    if max_chars == 0 {
        return vec![Vec::new()];
    }
    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut line: Vec<usize> = Vec::new();
    for index in 0..test.chars.len() {
        if line.len() == max_chars {
            lines.push(std::mem::take(&mut line));
        }
        line.push(index);
    }
    lines.push(line);
    if lines.last().is_some_and(|last| last.len() == max_chars) {
        lines.push(Vec::new());
    }
    lines
}

fn cursor_position(wrapped: &[Vec<usize>], cursor: usize) -> (usize, usize) {
    let mut prev_end = 0;
    let last_row = wrapped.len() - 1;
    for (row, line) in wrapped.iter().enumerate() {
        let end = prev_end + line.len();
        if cursor < end || (row == last_row && cursor == end) {
            return (row, cursor - prev_end);
        }
        prev_end = end;
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_of(text: &str) -> TypingTest {
        TypingTest::new(crate::typing::test::TestMode::Words(1), &[text.to_owned()])
    }

    #[test]
    fn wraps_short_text_into_single_line() {
        let test = test_of("abc");
        let wrapped = wrap_chars(&test, 10);
        assert_eq!(wrapped, vec![vec![0, 1, 2]]);
    }

    #[test]
    fn wraps_text_into_multiple_lines() {
        let test = test_of("abcdef");
        let wrapped = wrap_chars(&test, 3);
        assert_eq!(wrapped, vec![vec![0, 1, 2], vec![3, 4, 5], vec![]]);
    }

    #[test]
    fn wraps_full_line_and_adds_cursor_line() {
        let test = test_of("abc");
        let wrapped = wrap_chars(&test, 3);
        assert_eq!(wrapped, vec![vec![0, 1, 2], vec![]]);
    }

    #[test]
    fn empty_text_yields_one_empty_line() {
        let test = test_of("");
        let wrapped = wrap_chars(&test, 3);
        assert_eq!(wrapped, vec![vec![]]);
    }

    #[test]
    fn cursor_at_end_lands_on_last_line() {
        let test = test_of("abc");
        let wrapped = wrap_chars(&test, 3);
        let (row, col) = cursor_position(&wrapped, 3);
        assert_eq!((row, col), (1, 0));
    }

    #[test]
    fn cursor_mid_text_lands_on_its_line() {
        let test = test_of("abcdef");
        let wrapped = wrap_chars(&test, 3);
        let (row, col) = cursor_position(&wrapped, 4);
        assert_eq!((row, col), (1, 1));
    }
}