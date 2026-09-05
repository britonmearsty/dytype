use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::animation::{EffectKind, Rgb, mix};
use crate::app::App;
use crate::config::settings::{CursorAnimation, CursorStyle};
use crate::typing::test::{CharState, TestStatus, TypingTest};
use crate::ui::widgets::theme::Theme;

const BAR_RAMP: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorGlyph {
    Bar,
    Block,
    Underline,
}

pub struct TypingScreen;

impl TypingScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        if app.settings.display.compact_mode {
            render_prompt(frame, area, app);
            return;
        }
        let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(5)]).split(area);
        render_prompt(frame, chunks[0], app);
        render_stats(frame, chunks[1], app);
    }
}

fn render_prompt(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let test = &app.engine.test;
    let animations_on = app.settings.display.animations;
    let block = Block::default()
        .title(" dytype ")
        .borders(Borders::ALL)
        .style(Style::default().bg(app.theme.background));
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

    let text_width = app.settings.display.text_width;
    let wrap_max = if text_width == 0 {
        usize::from(inner.width)
    } else {
        usize::from(inner.width.min(text_width))
    };
    let wrapped = wrap_chars(test, wrap_max.max(1));
    let frame_now = app.anim.frame_now;
    let cursor_f = app.anim.cursor.current_x;
    let cursor_idx = (cursor_f.floor() as usize).min(test.chars.len());
    let (cursor_row, cursor_col) = cursor_position(&wrapped, cursor_idx);
    let frac = cursor_f - cursor_f.floor();

    let blink_alpha = if animations_on && app.settings.display.cursor.blink {
        app.anim.blink_alpha(frame_now, app.settings.display.cursor.blink_speed)
    } else {
        1.0
    };
    let pop = if animations_on {
        app.anim.word_landing(frame_now)
    } else {
        0.0
    };
    let snap = app.settings.display.cursor_animation == CursorAnimation::Off;
    let glyph = match app.settings.display.cursor.style {
        CursorStyle::Bar => CursorGlyph::Bar,
        CursorStyle::Block => CursorGlyph::Block,
        CursorStyle::Underline => CursorGlyph::Underline,
    };

    let mut lines = Vec::with_capacity(wrapped.len());
    for (row, indices) in wrapped.iter().enumerate() {
        let mut spans: Vec<Span> = indices
            .iter()
            .map(|&i| {
                let effect = if animations_on {
                    app.anim.char_effect(i, frame_now)
                } else {
                    None
                };
                Span::styled(
                    format!("{}", test.chars[i]),
                    char_style(test.states[i], effect, app.theme),
                )
            })
            .collect();
        if row == cursor_row {
            apply_cursor(&mut spans, cursor_col, inner.width as usize, glyph, frac, blink_alpha, pop, app.theme, snap);
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
    let theme = app.theme;
    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:.0} WPM", ls.wpm),
                Style::default().fg(theme.correct).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("   Raw: {:.0}", ls.raw_wpm), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled(format!("Accuracy: {:.1}%", ls.accuracy), Style::default().fg(theme.text)),
            Span::styled(
                format!("   Consistency: {:.0}%", ls.consistency),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                format!("   Time: {:.1}s", ls.elapsed.as_secs_f64()),
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(Span::styled(
            format!(
                "[{status}]   {}   Tab: new test   Enter: finish   Esc: quit",
                app.config.label()
            ),
            Style::default().fg(theme.muted),
        )),
    ];
    let block = Block::default()
        .title(" stats ")
        .borders(Borders::ALL)
        .style(Style::default().bg(theme.background));
    frame.render_widget(
        Paragraph::new(lines).block(block).alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

fn char_style(state: CharState, effect: Option<(EffectKind, f32)>, theme: Theme) -> Style {
    let base = match state {
        CharState::Correct => theme.rgb(theme.correct),
        CharState::Incorrect => theme.rgb(theme.incorrect),
        CharState::Unseen => theme.rgb(theme.muted),
    };
    let mut style = Style::default().fg(rgb(base));
    if let Some((kind, progress)) = effect {
        let boost = (1.0 - progress).clamp(0.0, 1.0);
        match kind {
            EffectKind::Emphasize => {
                style = style.fg(rgb(mix(base, theme.rgb(theme.text), 0.65 * boost)));
            }
            EffectKind::Error => {
                style = style
                    .bg(rgb(mix(theme.rgb(theme.background), theme.rgb(theme.incorrect), 0.55 * boost)))
                    .add_modifier(Modifier::BOLD);
            }
        }
    }
    style
}

#[allow(clippy::too_many_arguments)]
fn apply_cursor(
    spans: &mut Vec<Span<'_>>,
    col: usize,
    width: usize,
    glyph: CursorGlyph,
    frac: f32,
    alpha: f32,
    pop: f32,
    theme: Theme,
    snap: bool,
) {
    let fade = if snap { 1.0 } else { (alpha + pop * 0.6).min(1.0) };
    let cursor_color = rgb(mix(theme.rgb(theme.background), theme.rgb(theme.cursor), fade));
    let fallback = " ".to_owned();
    let content = spans.get(col).map(|s| s.content.to_string()).unwrap_or(fallback);
    if col < spans.len() {
        spans[col] = cursor_span(content, glyph, cursor_color, frac, theme);
    } else if col == spans.len() && col < width {
        spans.push(cursor_span(content, glyph, cursor_color, frac, theme));
    }
}

fn cursor_span(
    content: String,
    glyph: CursorGlyph,
    cursor: ratatui::style::Color,
    frac: f32,
    theme: Theme,
) -> Span<'static> {
    match glyph {
        CursorGlyph::Block => Span::styled(
            content,
            Style::default().fg(theme.background).bg(cursor),
        ),
        CursorGlyph::Bar => Span::styled(bar_ramp(frac).to_string(), Style::default().fg(cursor)),
        CursorGlyph::Underline => Span::styled(
            content,
            Style::default().fg(cursor).add_modifier(Modifier::UNDERLINED),
        ),
    }
}

fn bar_ramp(frac: f32) -> char {
    BAR_RAMP[((frac * 8.0) as usize).min(7)]
}

fn rgb(color: Rgb) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.0, color.1, color.2)
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

    #[test]
    fn bar_ramp_is_bounded() {
        assert_eq!(bar_ramp(0.0), '▏');
        assert_eq!(bar_ramp(1.0), '█');
        assert_eq!(bar_ramp(0.5), '▋');
    }
}