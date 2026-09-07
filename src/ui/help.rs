use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::input::command::Command;
use crate::ui::widgets::theme::Theme;

fn command_label(command: &Command) -> &'static str {
    match command {
        Command::Quit => "Quit",
        Command::Restart => "Restart test",
        Command::Pause => "Pause / resume",
        Command::NextTest => "Next test",
        Command::PreviousTest => "Previous view",
        Command::OpenSettings => "Open settings",
        Command::OpenHistory => "Open history",
        Command::OpenHelp => "Open this help",
        Command::ToggleStats => "Toggle live stats",
    }
}

fn command_lines(app: &App, theme: Theme) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        " commands ",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))];
    for command in Command::ALL {
        let keys: Vec<String> = app
            .keybindings
            .keymap()
            .entries()
            .filter(|(bound, _)| *bound == command)
            .map(|(_, key)| key.to_string())
            .collect();
        lines.push(Line::from(vec![
            Span::styled(
                format!("   {:<20}", command_label(&command)),
                Style::default().fg(theme.accent),
            ),
            Span::styled(keys.join("  or  "), Style::default().fg(theme.correct)),
        ]));
    }
    lines
}

fn screen_keys(theme: Theme) -> Vec<Line<'static>> {
    let key = |label: &'static str| {
        Span::styled(
            format!("   {label:<20}"),
            Style::default()
                .fg(theme.correct)
                .add_modifier(Modifier::BOLD),
        )
    };
    let detail = |text: &str| Span::styled(text.to_owned(), Style::default().fg(theme.muted));
    vec![
        Line::from(Span::styled(
            " screen input ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            key("a-z · 0-9 · chars"),
            detail("type the expected character"),
        ]),
        Line::from(vec![
            key("backspace"),
            detail("delete the previous character"),
        ]),
        Line::from(vec![key("↑/↓"), detail("move selection")]),
        Line::from(vec![key("←/→"), detail("change value / switch tab")]),
        Line::from(vec![key("enter"), detail("submit / start")]),
    ]
}

pub struct HelpScreen;

impl HelpScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let theme = app.theme;
        let mut lines = vec![Line::from("")];
        lines.extend(command_lines(app, theme));
        lines.push(Line::from(""));
        lines.extend(screen_keys(theme));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            app.config.label(),
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "   Esc back",
            Style::default().fg(theme.muted),
        )));

        let block = Block::default()
            .title(" dytype · help ")
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background));
        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_has_a_description() {
        for command in Command::ALL {
            let label = command_label(&command);
            assert!(!label.is_empty(), "{command:?} has no label");
        }
    }
}
