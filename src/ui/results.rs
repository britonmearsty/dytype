use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::animation::{Rgb, mix};
use crate::app::App;

const GREEN: Rgb = (0x66, 0xD9, 0x9E);
const WHITE: Rgb = (0xFF, 0xFF, 0xFF);

fn rgb(color: Rgb) -> Color {
    Color::Rgb(color.0, color.1, color.2)
}

pub struct ResultsScreen;

impl ResultsScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let ls = app.live_stats;
        let now = app.anim.frame_now;
        let (wpm, raw, accuracy, consistency) = app.anim.results.values(now);
        let title_alpha = app.anim.results.title_alpha(now);
        let title_color = rgb(mix(GREEN, WHITE, 0.55 * title_alpha));
        let wpm_color = rgb(mix(GREEN, WHITE, 0.35 * title_alpha));
        let lines = vec![
            Line::from(Span::styled(
                "Test complete!",
                Style::default().fg(title_color).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("{wpm:.0} WPM"),
                Style::default().fg(wpm_color).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "Raw: {raw:.0} WPM    Accuracy: {accuracy:.1}%    Consistency: {consistency:.0}%"
            )),
            Line::from(format!("Time: {:.1}s", ls.elapsed.as_secs_f64())),
            Line::from(""),
            Line::from("Enter: new test    Tab: config    F2: settings    F3: history    Esc: quit"),
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