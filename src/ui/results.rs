use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;

pub struct ResultsScreen;

impl ResultsScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let ls = app.live_stats;
        let lines = vec![
            Line::from(Span::styled(
                "Test complete!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("{:.0} WPM", ls.wpm),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "Raw: {:.0} WPM    Accuracy: {:.1}%    Consistency: {:.0}%",
                ls.raw_wpm, ls.accuracy, ls.consistency
            )),
            Line::from(format!("Time: {:.1}s", ls.elapsed.as_secs_f64())),
            Line::from(""),
            Line::from("Enter: new test    Tab: config    Esc: quit"),
        ];
        let block = Block::default().title(" results ").borders(Borders::ALL);
        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}