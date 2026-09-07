use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::animation::{Rgb, mix};
use crate::app::App;

fn rgb(color: Rgb) -> Color {
    Color::Rgb(color.0, color.1, color.2)
}

pub struct ResultsScreen;

impl ResultsScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let ls = app.live_stats;
        let now = app.anim.frame_now;
        let theme = app.theme;
        let (wpm, raw, accuracy, consistency) = app.anim.results.values(now);
        let title_alpha = app.anim.results.title_alpha(now);
        let green = theme.rgb(theme.correct);
        let white = theme.rgb(theme.text);
        let title_color = rgb(mix(green, white, 0.55 * title_alpha));
        let wpm_color = rgb(mix(green, white, 0.35 * title_alpha));
        let lines = vec![
            Line::from(Span::styled(
                "Test complete!",
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("{wpm:.0} WPM"),
                Style::default().fg(wpm_color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "Raw: {raw:.0} WPM    Accuracy: {accuracy:.1}%    Consistency: {consistency:.0}%"
                ),
                Style::default().fg(theme.text),
            )),
            Line::from(Span::styled(
                format!("Time: {:.1}s", ls.elapsed.as_secs_f64()),
                Style::default().fg(theme.muted),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Enter: new test    Tab: config    F1: help    F2: settings    F3: history    Esc: quit",
                Style::default().fg(theme.muted),
            )),
        ];
        let block = Block::default()
            .title(" results ")
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background));
        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}
