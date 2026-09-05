use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::stats::analysis::{self, DailyPoint};
use crate::stats::calendar;
use crate::stats::chart;
use crate::stats::progress::{self, ACHIEVEMENTS};
use crate::stats::result::TestResult;
use crate::typing::test::TestMode;
use crate::ui::widgets::theme::Theme;

const TABS: [HistoryTab; 5] = [
    HistoryTab::Overview,
    HistoryTab::Wpm,
    HistoryTab::Accuracy,
    HistoryTab::Errors,
    HistoryTab::History,
];

const CHART_WIDTH_FLOOR: usize = 12;
const CHART_HEIGHT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryTab {
    #[default]
    Overview,
    Wpm,
    Accuracy,
    Errors,
    History,
}

impl HistoryTab {
    pub fn label(self) -> &'static str {
        match self {
            HistoryTab::Overview => "Overview",
            HistoryTab::Wpm => "WPM",
            HistoryTab::Accuracy => "Accuracy",
            HistoryTab::Errors => "Errors",
            HistoryTab::History => "History",
        }
    }

    pub fn cycle(self, dir: i8) -> Self {
        let next = (self as i8 + dir).rem_euclid(TABS.len() as i8) as usize;
        TABS[next]
    }
}

fn tab_bar(tab: HistoryTab, theme: Theme) -> Line<'static> {
    let mut spans = Vec::with_capacity(TABS.len());
    for (i, option) in TABS.iter().enumerate() {
        let pad = if i == 0 { "" } else { "  " };
        if *option == tab {
            spans.push(Span::styled(
                format!("{pad}[ {} ]", option.label()),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("{pad}  {} ", option.label()),
                Style::default().fg(theme.muted),
            ));
        }
    }
    Line::from(spans)
}

fn section(title: &str, theme: Theme) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {title} "),
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    ))
}

fn empty_state(theme: Theme) -> Line<'static> {
    Line::from(Span::styled(
        "No completed tests yet. Finish a test and it will show up here.",
        Style::default().fg(theme.muted),
    ))
}

pub fn mode_label(mode: TestMode) -> String {
    match mode {
        TestMode::Words(n) => format!("w{n}"),
        TestMode::Time(duration) => format!("{}s", duration.as_secs()),
    }
}

pub fn duration_label(ms: u64) -> String {
    let secs = ms / 1000;
    if secs >= 60 {
        format!("{}:{:0>2}", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

fn chart_width(area: Rect) -> usize {
    usize::from(area.width.saturating_sub(10)).max(CHART_WIDTH_FLOOR)
}

fn max_wpm(points: &[DailyPoint]) -> f64 {
    points.iter().map(|p| p.wpm).fold(0.0_f64, f64::max).max(1e-9)
}

pub struct HistoryScreen;

impl HistoryScreen {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let tab = app.history_tab;
        let results = &app.history.results;
        let theme = app.theme;
        let mut lines = Vec::new();

        lines.push(Line::from(""));
        lines.push(tab_bar(tab, theme));
        lines.push(Line::from(""));

        if results.is_empty() {
            lines.push(empty_state(theme));
        } else {
            match tab {
                HistoryTab::Overview => self.render_overview(&mut lines, results, theme),
                HistoryTab::Wpm => self.render_wpm(&mut lines, results, area, theme),
                HistoryTab::Accuracy => self.render_accuracy(&mut lines, results, area, theme),
                HistoryTab::Errors => self.render_errors(&mut lines, results, area, theme),
                HistoryTab::History => self.render_table(&mut lines, results, theme),
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "←/→ switch view   F2 settings   Enter new test   Esc back",
            Style::default().fg(theme.muted),
        )));

        let block = Block::default()
            .title(format!(" history · {} ", tab.label()))
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background));
        frame.render_widget(
            Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
            area,
        );
    }

    fn render_overview(&self, lines: &mut Vec<Line<'static>>, results: &[TestResult], theme: Theme) {
        let summary = analysis::summarize(results);
        let info = progress::level_from_xp(progress::total_xp(results));
        let earned = progress::earned(results);
        let streak = progress::longest_streak(results);

        lines.push(section("summary", theme));
        lines.push(Line::from(format!(
            "  Tests: {:<6} Best WPM: {:<6.0} Avg WPM: {:.0}",
            summary.total_tests, summary.best_wpm, summary.avg_wpm
        )));
        lines.push(Line::from(format!(
            "  Raw WPM: {:.0}   Consistency: {:.0}%   Error rate: {:.1}%",
            summary.avg_raw_wpm, summary.avg_consistency, summary.avg_error_rate
        )));
        lines.push(Line::from(format!(
            "  Accuracy: {:.1}%   Characters: {}   Perfect tests: {}",
            summary.avg_accuracy, summary.total_characters, summary.perfect_tests
        )));
        lines.push(Line::from(format!(
            "  Time spent: {:.0}m ({:.2}h)",
            summary.total_time_ms as f64 / 60_000.0,
            summary.total_time_hours()
        )));

        lines.push(Line::from(""));
        lines.push(section("progression", theme));
        let bar_width: usize = 24;
        let filled = (info.progress() * bar_width as f64).round() as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width.saturating_sub(filled));
        lines.push(Line::from(Span::styled(
            format!(
                "  Level {}   XP {}/{}   total {}",
                info.level, info.xp_into_level, info.xp_for_next_level, info.total_xp
            ),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!(
            "  [{bar}]  {:.0}%        streak {} day{}",
            info.progress() * 100.0,
            streak,
            if streak == 1 { "" } else { "s" },
        )));

        lines.push(Line::from(""));
        lines.push(section("achievements", theme));
        for (achievement, achieved) in ACHIEVEMENTS.iter().zip(earned.iter()) {
            let mark = if *achieved { "✓" } else { "·" };
            let style = if *achieved {
                Style::default().fg(theme.correct)
            } else {
                Style::default().fg(theme.muted)
            };
            lines.push(Line::from(Span::styled(
                format!(
                    "  {mark} {:<20}{}",
                    achievement.name, achievement.description
                ),
                style,
            )));
        }
    }

    fn render_wpm(&self, lines: &mut Vec<Line<'static>>, results: &[TestResult], area: Rect, theme: Theme) {
        let width = chart_width(area);

        lines.push(section("wpm over time", theme));
        let series = analysis::wpm_series(results);
        for row in chart::line_chart(&series, width, CHART_HEIGHT) {
            lines.push(Line::from(format!("  {row}")));
        }
        lines.push(Line::from(format!(
            "  Latest: {:.0}   Best: {:.0}   Avg: {:.0}",
            series.last().copied().unwrap_or(0.0),
            series.iter().cloned().fold(0.0_f64, f64::max),
            analysis::summarize(results).avg_wpm,
        )));

        lines.push(Line::from(""));
        lines.push(section("daily average wpm (last 14 days)", theme));
        let days = analysis::daily(results, 14);
        let weeks_max = max_wpm(&days);
        for point in &days {
            let bar = chart::bar_row(point.wpm, weeks_max, 10);
            lines.push(Line::from(format!(
                "  {}  {bar}  {:>5.0} wpm",
                point.label, point.wpm
            )));
        }

        lines.push(Line::from(""));
        lines.push(section("personal best progression", theme));
        let best = analysis::best_progression(results);
        lines.push(Line::from(format!(
            "  {}  {:.0} wpm",
            chart::spark(&best, width),
            best.last().copied().unwrap_or(0.0)
        )));
    }

    fn render_accuracy(&self, lines: &mut Vec<Line<'static>>, results: &[TestResult], area: Rect, theme: Theme) {
        let width = chart_width(area);

        lines.push(section("accuracy over time", theme));
        let accuracy = analysis::accuracy_series(results);
        for row in chart::line_chart(&accuracy, width, CHART_HEIGHT) {
            lines.push(Line::from(format!("  {row}")));
        }
        let summary = analysis::summarize(results);
        lines.push(Line::from(format!(
            "  Latest: {:.1}%   Best: {:.1}%   Avg: {:.1}%",
            accuracy.last().copied().unwrap_or(0.0),
            accuracy.iter().cloned().fold(0.0_f64, f64::max),
            summary.avg_accuracy,
        )));

        lines.push(Line::from(""));
        lines.push(section("consistency over time", theme));
        let consistency = analysis::consistency_series(results);
        for row in chart::line_chart(&consistency, width, CHART_HEIGHT) {
            lines.push(Line::from(format!("  {row}")));
        }
        lines.push(Line::from(format!(
            "  Latest: {:.0}%   Avg: {:.0}%",
            consistency.last().copied().unwrap_or(0.0),
            summary.avg_consistency,
        )));
    }

    fn render_errors(&self, lines: &mut Vec<Line<'static>>, results: &[TestResult], area: Rect, theme: Theme) {
        let width = chart_width(area);

        lines.push(section("error rate over time", theme));
        let rate = analysis::error_rate_series(results);
        for row in chart::line_chart(&rate, width, CHART_HEIGHT) {
            lines.push(Line::from(format!("  {row}")));
        }
        lines.push(Line::from(format!(
            "  Latest: {:.1}%   Avg: {:.1}%",
            rate.last().copied().unwrap_or(0.0),
            analysis::summarize(results).avg_error_rate,
        )));

        lines.push(Line::from(""));
        lines.push(section("most mistyped characters", theme));
        let stats = analysis::character_errors(results, 10);
        if stats.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No recorded mistakes. Flawless!",
                Style::default().fg(theme.correct),
            )));
        } else {
            let max_errors = stats.iter().map(|s| s.errors).max().unwrap_or(1) as f64;
            let bar_len = (width / 3).clamp(4, 20);
            for stat in &stats {
                let bar = chart::bar_row(stat.errors as f64, max_errors, bar_len);
                lines.push(Line::from(format!(
                    "  {:>1} {}  {:>3} errors  ({:.0}%)",
                    stat.ch, bar, stat.errors, stat.rate(),
                )));
            }
        }

        lines.push(Line::from(""));
        lines.push(section("duration distribution", theme));
        let total = results.len() as f64;
        for (bucket, count) in analysis::duration_distribution(results) {
            let bar = chart::bar_row(count as f64, total, width.saturating_sub(20).max(4));
            lines.push(Line::from(format!(
                "  {:<10} {:<3} {bar}",
                bucket.label(),
                count,
            )));
        }
    }

    fn render_table(&self, lines: &mut Vec<Line<'static>>, results: &[TestResult], theme: Theme) {
        lines.push(section("recent tests", theme));
        lines.push(Line::from(Span::styled(
            format!(
                "  {:<4} {:<6} {:<6} {:<7} {:>4} {:>6} {:>5} {:>6}",
                "id", "date", "mode", "difficulty", "wpm", "accuracy", "errors", "time"
            ),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )));
        for result in results.iter().rev().take(12) {
            lines.push(Line::from(format!(
                "  {:<4} {:<6} {:<6} {:<7} {:>4.0} {:>6.1} {:>5} {:>6}",
                result.id,
                calendar::short_date(result.timestamp),
                mode_label(result.mode),
                result.difficulty.label(),
                result.wpm,
                result.accuracy,
                result.errors,
                duration_label(result.duration_ms),
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_cycle_wrap_around() {
        assert_eq!(HistoryTab::default(), HistoryTab::Overview);
        assert_eq!(HistoryTab::Overview.cycle(1), HistoryTab::Wpm);
        assert_eq!(HistoryTab::Overview.cycle(-1), HistoryTab::History);
        assert_eq!(HistoryTab::History.cycle(1), HistoryTab::Overview);
    }

    #[test]
    fn tab_bar_highlights_active_tab() {
        let bar = tab_bar(HistoryTab::Wpm, crate::ui::widgets::theme::DEFAULT);
        let text = bar.to_string();
        assert!(text.contains("[ WPM ]"));
        assert!(text.contains("Overview"));
        assert!(!text.contains("[ Accuracy ]"));
    }

    #[test]
    fn mode_labels_match_display() {
        assert_eq!(mode_label(TestMode::Words(25)), "w25");
        assert_eq!(
            mode_label(TestMode::Time(std::time::Duration::from_secs(60))),
            "60s"
        );
    }

    #[test]
    fn duration_labels_format_minutes_and_seconds() {
        assert_eq!(duration_label(0), "0s");
        assert_eq!(duration_label(54_000), "54s");
        assert_eq!(duration_label(65_000), "1:05");
        assert_eq!(duration_label(3_600_000), "60:00");
    }

    #[test]
    fn tab_bar_default_marks_overview_active() {
        let bar = tab_bar(HistoryTab::Overview, crate::ui::widgets::theme::DEFAULT).to_string();
        assert!(bar.starts_with("[ Overview ]"));
    }
}