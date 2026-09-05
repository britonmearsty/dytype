use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, _app: &App) {
        let block = Block::default().title(" settings ").borders(Borders::ALL);
        frame.render_widget(Paragraph::new("Coming soon").block(block), area);
    }
}