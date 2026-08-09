//! Git mode: a commit list on the left, and either the selected commit's
//! summary or its diff on the right.

use crate::app::git_mode::{GitLevel, GitMode};
use crate::app::App;
use crate::git::{Commit, CommitDetail, FileStat};
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Wrap};
use std::time::{SystemTime, UNIX_EPOCH};

/// Share of the width given to the commit list.
const COMMIT_LIST_PERCENT: u16 = 45;
/// Share of the width given to the changed-file list beside a diff.
const FILE_LIST_PERCENT: u16 = 32;
/// Width of the abbreviated hash column.
const HASH_WIDTH: usize = 7;
/// Width of the age column, enough for the longest label ("11mo").
const AGE_WIDTH: usize = 4;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.config.theme.clone();

    let background = Block::default().style(Style::default().bg(theme.bg_main));
    frame.render_widget(background, area);

    let [header_area, body_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(area);

    let Some(mode) = app.git_mode.as_mut() else {
        return;
    };

    render_header(frame, header_area, &theme, mode);

    if let Some(error) = mode.error.clone() {
        if mode.commits.is_empty() {
            let message = Paragraph::new(error)
                .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
                .alignment(Alignment::Center);
            frame.render_widget(message, body_area);
            return;
        }
    }

    let list_percent = match mode.level {
        GitLevel::Commits => COMMIT_LIST_PERCENT,
        GitLevel::Detail => FILE_LIST_PERCENT,
    };
    let [list_area, content_area] =
        Layout::horizontal([Constraint::Percentage(list_percent), Constraint::Min(10)])
            .areas(body_area);

    // The diff is what page keys scroll, so it owns the preview area.
    app.last_preview_area = Some(content_area);

    match mode.level {
        GitLevel::Commits => {
            render_commit_list(frame, list_area, &theme, mode);
            render_commit_summary(frame, content_area, &theme, mode);
        }
        GitLevel::Detail => {
            render_file_list(frame, list_area, &theme, mode);
            render_diff(frame, content_area, &theme, mode);
        }
    }
}

fn render_header(frame: &mut Frame, area: Rect, theme: &Theme, mode: &GitMode) {
    let repo = mode.repo.file_name().map_or_else(
        || mode.repo.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );

    let mut spans = vec![
        Span::styled(
            format!(" {repo} "),
            Style::default()
                .fg(theme.fg_selected)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("· {} commits ", mode.commits.len()),
            Style::default().fg(theme.fg_dim),
        ),
    ];

    if mode.level == GitLevel::Detail {
        if let Some(commit) = mode.selected_commit() {
            spans.push(Span::styled(
                format!("· {} {} ", commit.short_hash, commit.subject),
                Style::default().fg(theme.fg_text),
            ));
        }
    }

    let header = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.bg_darker));
    frame.render_widget(header, area);
}

fn render_commit_list(frame: &mut Frame, area: Rect, theme: &Theme, mode: &mut GitMode) {
    let height = area.height as usize;
    mode.scroll_list_into_view(height);
    let background = Block::default().style(Style::default().bg(theme.bg_darker));
    frame.render_widget(background, area);

    let width = area.width as usize;
    let now = unix_now();
    let lines: Vec<Line> = mode
        .commits
        .iter()
        .enumerate()
        .skip(mode.list_offset)
        .take(height)
        .map(|(index, commit)| commit_line(commit, now, width, index == mode.selected, theme))
        .collect();

    frame.render_widget(Paragraph::new(lines), area);
}

/// One row of the commit list: hash, subject, and age, each in a fixed column
/// so the ages stay in line however long a subject or label runs.
fn commit_line<'a>(
    commit: &Commit,
    now: i64,
    width: usize,
    selected: bool,
    theme: &Theme,
) -> Line<'a> {
    let age = truncate(&relative_time(now - commit.timestamp), AGE_WIDTH);
    let subject_width = width.saturating_sub(HASH_WIDTH + AGE_WIDTH + 4);
    let subject = truncate(&commit.subject, subject_width);

    let (subject_fg, bg) = if selected {
        (theme.fg_selected, theme.bg_selected)
    } else {
        (theme.fg_text, theme.bg_darker)
    };

    let subject_padding = subject_width.saturating_sub(display_width(&subject));

    Line::from(vec![
        Span::styled(
            format!(" {:<HASH_WIDTH$} ", commit.short_hash),
            Style::default().fg(theme.diff_hunk).bg(bg),
        ),
        Span::styled(subject, Style::default().fg(subject_fg).bg(bg)),
        Span::styled(" ".repeat(subject_padding), Style::default().bg(bg)),
        Span::styled(
            format!(" {age:>AGE_WIDTH$} "),
            Style::default().fg(theme.fg_dim).bg(bg),
        ),
    ])
}

fn render_commit_summary(frame: &mut Frame, area: Rect, theme: &Theme, mode: &GitMode) {
    let Some(commit) = mode.selected_commit() else {
        return;
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!(" {}", commit.subject),
            Style::default()
                .fg(theme.fg_selected)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(" {} · {}", commit.author, commit.hash),
            Style::default().fg(theme.fg_dim),
        )),
        Line::from(""),
    ];

    let Some(detail) = mode.selected_detail() else {
        lines.push(Line::from(Span::styled(
            " Loading commit...",
            Style::default().fg(theme.fg_dim),
        )));
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(theme.bg_main)),
            area,
        );
        return;
    };

    // The body of the message, past the subject line already shown above.
    let body = detail
        .message
        .split_once('\n')
        .map_or("", |(_, rest)| rest)
        .trim();
    if !body.is_empty() {
        for line in body.lines() {
            lines.push(Line::from(Span::styled(
                format!(" {line}"),
                Style::default().fg(theme.fg_text),
            )));
        }
        lines.push(Line::from(""));
    }

    lines.push(change_summary(detail, theme, theme.bg_main));
    let columns = stat_columns(detail);
    for file in &detail.files {
        lines.push(file_line(file, theme, area.width as usize, false, columns));
    }

    let summary = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg_main))
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, area);
}

fn render_file_list(frame: &mut Frame, area: Rect, theme: &Theme, mode: &GitMode) {
    let background = Block::default().style(Style::default().bg(theme.bg_darker));
    frame.render_widget(background, area);

    let Some(detail) = mode.selected_detail() else {
        return;
    };

    let height = area.height as usize;
    let offset = mode.file_row_offset(height);
    let width = area.width as usize;

    let columns = stat_columns(detail);
    let mut lines = vec![change_summary(detail, theme, theme.bg_darker)];

    lines.extend(
        detail
            .files
            .iter()
            .enumerate()
            .skip(offset)
            .take(height.saturating_sub(1))
            .map(|(index, file)| {
                file_line(file, theme, width, index == mode.file_selected, columns)
            }),
    );

    frame.render_widget(Paragraph::new(lines), area);
}

/// Widths of the added and removed columns, so every row in one commit lines
/// its counts up under the same digits.
fn stat_columns(detail: &CommitDetail) -> (usize, usize) {
    let digits = |count: u32| count.to_string().len();
    let widest =
        |counts: &mut dyn Iterator<Item = u32>| counts.map(digits).max().unwrap_or(1).max(1);

    (
        widest(&mut detail.files.iter().filter_map(|file| file.added)),
        widest(&mut detail.files.iter().filter_map(|file| file.removed)),
    )
}

/// One row of the changed-file list: status letter, path, and line counts.
fn file_line<'a>(
    file: &FileStat,
    theme: &Theme,
    width: usize,
    selected: bool,
    columns: (usize, usize),
) -> Line<'a> {
    let (fg, bg) = if selected {
        (theme.fg_selected, theme.bg_selected)
    } else {
        (theme.fg_text, theme.bg_darker)
    };

    let (added_width, removed_width) = columns;
    // "+N" and "-N" plus the space between them.
    let stats_width = added_width + removed_width + 3;

    let kind_color = match file.kind {
        crate::git::ChangeKind::Added => theme.diff_add,
        crate::git::ChangeKind::Deleted => theme.diff_del,
        _ => theme.diff_hunk,
    };

    // Leading marker, its two spaces and the trailing space bracket the row;
    // one more space keeps the path off the counts.
    let path = truncate_start(&file.path, width.saturating_sub(stats_width + 5));
    let padding = width.saturating_sub(display_width(&path) + stats_width + 4);

    let mut spans = vec![
        Span::styled(
            format!(" {} ", file.kind.letter()),
            Style::default().fg(kind_color).bg(bg),
        ),
        Span::styled(path, Style::default().fg(fg).bg(bg)),
        Span::styled(" ".repeat(padding), Style::default().bg(bg)),
    ];

    if file.is_binary() {
        spans.push(Span::styled(
            format!("{:>stats_width$}", "bin"),
            Style::default().fg(theme.fg_dim).bg(bg),
        ));
    } else {
        let added = format!("+{}", file.added.unwrap_or(0));
        let removed = format!("-{}", file.removed.unwrap_or(0));
        spans.push(Span::styled(
            format!("{added:>width$}", width = added_width + 1),
            Style::default().fg(theme.diff_add).bg(bg),
        ));
        spans.push(Span::styled(
            format!(" {removed:>width$}", width = removed_width + 1),
            Style::default().fg(theme.diff_del).bg(bg),
        ));
    }
    spans.push(Span::styled(" ", Style::default().bg(bg)));

    Line::from(spans)
}

fn render_diff(frame: &mut Frame, area: Rect, theme: &Theme, mode: &GitMode) {
    let Some(detail) = mode.selected_detail() else {
        let loading = Paragraph::new("Loading diff...")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    };

    if detail.diff.is_empty() {
        let empty = Paragraph::new("This commit changes no files")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let lines: Vec<Line> = detail
        .diff
        .iter()
        .skip(mode.diff_scroll)
        .take(area.height as usize)
        .map(|line| Line::from(Span::styled(format!(" {line}"), diff_style(line, theme))))
        .collect();

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme.bg_main)),
        area,
    );
}

fn diff_style(line: &str, theme: &Theme) -> Style {
    if line.starts_with("diff --git ") {
        return Style::default()
            .fg(theme.diff_hunk)
            .add_modifier(Modifier::BOLD);
    }
    if line.starts_with("@@") {
        return Style::default().fg(theme.diff_hunk);
    }
    // File markers start with +++/--- and must be checked before the +/- lines.
    if line.starts_with("+++") || line.starts_with("---") || line.starts_with("index ") {
        return Style::default().fg(theme.fg_dim);
    }
    if line.starts_with('+') {
        return Style::default().fg(theme.diff_add);
    }
    if line.starts_with('-') {
        return Style::default().fg(theme.diff_del);
    }
    Style::default().fg(theme.fg_text)
}

/// "3 files · +42 -17", the line shown above a commit's changed files, with
/// its totals in the same colours as the rows beneath.
fn change_summary<'a>(detail: &CommitDetail, theme: &Theme, bg: Color) -> Line<'a> {
    let added: u32 = detail.files.iter().filter_map(|f| f.added).sum();
    let removed: u32 = detail.files.iter().filter_map(|f| f.removed).sum();
    let count = detail.files.len();
    let plural = if count == 1 { "" } else { "s" };

    Line::from(vec![
        Span::styled(
            format!(" {count} file{plural} · "),
            Style::default().fg(theme.fg_dim).bg(bg),
        ),
        Span::styled(
            format!("+{added}"),
            Style::default().fg(theme.diff_add).bg(bg),
        ),
        Span::styled(
            format!(" -{removed}"),
            Style::default().fg(theme.diff_del).bg(bg),
        ),
    ])
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Compact age, as git log's `--relative-date` would read it.
fn relative_time(seconds: i64) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const WEEK: i64 = 7 * DAY;
    const MONTH: i64 = 30 * DAY;
    const YEAR: i64 = 365 * DAY;

    let seconds = seconds.max(0);
    match seconds {
        s if s < MINUTE => "now".to_string(),
        s if s < HOUR => format!("{}m", s / MINUTE),
        s if s < DAY => format!("{}h", s / HOUR),
        s if s < WEEK => format!("{}d", s / DAY),
        s if s < MONTH => format!("{}w", s / WEEK),
        s if s < YEAR => format!("{}mo", s / MONTH),
        s => format!("{}y", s / YEAR),
    }
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

/// Keep the head of a string, marking any cut with an ellipsis.
fn truncate(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width <= 1 {
        return String::new();
    }
    let kept: String = text.chars().take(width - 1).collect();
    format!("{kept}…")
}

/// Keep the tail of a string, which is the informative end of a file path.
fn truncate_start(text: &str, width: usize) -> String {
    let length = display_width(text);
    if length <= width {
        return text.to_string();
    }
    if width <= 1 {
        return String::new();
    }
    let dropped = length - (width - 1);
    let kept: String = text.chars().skip(dropped).collect();
    format!("…{kept}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_ages_at_each_scale() {
        assert_eq!(relative_time(30), "now");
        assert_eq!(relative_time(60 * 5), "5m");
        assert_eq!(relative_time(60 * 60 * 3), "3h");
        assert_eq!(relative_time(60 * 60 * 24 * 2), "2d");
        assert_eq!(relative_time(60 * 60 * 24 * 10), "1w");
        assert_eq!(relative_time(60 * 60 * 24 * 60), "2mo");
        assert_eq!(relative_time(60 * 60 * 24 * 400), "1y");
    }

    fn commit(subject: &str, age_seconds: i64) -> Commit {
        Commit {
            hash: "0123456789abcdef".to_string(),
            short_hash: "0123456".to_string(),
            author: "Test".to_string(),
            timestamp: 1_000_000 - age_seconds,
            subject: subject.to_string(),
        }
    }

    /// Column where the age starts, counted from the left of the row.
    fn age_column(line: &Line) -> usize {
        line.spans
            .iter()
            .take(line.spans.len() - 1)
            .map(|span| display_width(&span.content))
            .sum()
    }

    #[test]
    fn commit_rows_are_exactly_the_pane_width() {
        let theme = Theme::default();
        let subjects = ["short", "a subject long enough to be cut off by any pane"];

        for subject in subjects {
            for age in [30, 60 * 60 * 24 * 60, 60 * 60 * 24 * 400] {
                let line = commit_line(&commit(subject, age), 1_000_000, 40, false, &theme);
                let width: usize = line
                    .spans
                    .iter()
                    .map(|span| display_width(&span.content))
                    .sum();
                assert_eq!(width, 40, "row for {subject:?} at age {age}");
            }
        }
    }

    #[test]
    fn ages_share_one_column_whatever_their_length() {
        let theme = Theme::default();
        let now = 1_000_000;
        let rows = [
            commit_line(&commit("short", 30), now, 48, false, &theme),
            commit_line(
                &commit("a much longer subject line", 60 * 60 * 24 * 60),
                now,
                48,
                false,
                &theme,
            ),
            commit_line(&commit("mid", 60 * 60 * 24 * 400), now, 48, true, &theme),
        ];

        let columns: Vec<usize> = rows.iter().map(age_column).collect();
        assert!(
            columns.windows(2).all(|pair| pair[0] == pair[1]),
            "ages should start at one column, got {columns:?}"
        );
    }

    fn detail_with(files: &[(&str, Option<u32>, Option<u32>)]) -> CommitDetail {
        CommitDetail {
            message: "Subject".to_string(),
            files: files
                .iter()
                .map(|(path, added, removed)| FileStat {
                    path: (*path).to_string(),
                    added: *added,
                    removed: *removed,
                    kind: crate::git::ChangeKind::Modified,
                    diff_line: 0,
                })
                .collect(),
            diff: Vec::new(),
        }
    }

    #[test]
    fn file_counts_line_up_under_each_other() {
        let theme = Theme::default();
        let detail = detail_with(&[
            ("AGENTS.md", Some(22), Some(2)),
            ("src/highlight/markdown_table.rs", Some(360), Some(0)),
            ("assets/logo.png", None, None),
        ]);
        let columns = stat_columns(&detail);
        assert_eq!(columns, (3, 1), "widest counts are 360 and 2");

        let rows: Vec<Line> = detail
            .files
            .iter()
            .map(|file| file_line(file, &theme, 60, false, columns))
            .collect();

        for row in &rows {
            let width: usize = row
                .spans
                .iter()
                .map(|span| display_width(&span.content))
                .sum();
            assert_eq!(width, 60);
        }

        // The counts begin at one column on every row: the marker, path and
        // padding before them always add up the same.
        let stat_starts: Vec<usize> = rows
            .iter()
            .map(|row| {
                row.spans
                    .iter()
                    .take(3)
                    .map(|span| display_width(&span.content))
                    .sum()
            })
            .collect();
        assert!(
            stat_starts.iter().all(|start| *start == stat_starts[0]),
            "counts should share a column, got {stat_starts:?}"
        );
    }

    #[test]
    fn added_and_removed_carry_their_own_colours() {
        let theme = Theme::default();
        let detail = detail_with(&[("src/main.rs", Some(4), Some(9))]);
        let row = file_line(&detail.files[0], &theme, 40, false, stat_columns(&detail));

        let added = row
            .spans
            .iter()
            .find(|span| span.content.contains("+4"))
            .expect("added span");
        let removed = row
            .spans
            .iter()
            .find(|span| span.content.contains("-9"))
            .expect("removed span");

        assert_eq!(added.style.fg, Some(theme.diff_add));
        assert_eq!(removed.style.fg, Some(theme.diff_del));
    }

    #[test]
    fn truncation_keeps_within_the_given_width() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(display_width(&truncate("a very long subject", 8)), 8);
        assert_eq!(truncate_start("src/app/git_mode.rs", 10), "…t_mode.rs");
        assert_eq!(truncate_start("short", 10), "short");
    }
}
