//! UI rendering for git mode

use crate::app::App;
use crate::constants::{
    SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN, SEARCH_POPUP_WIDTH_PERCENT,
};
use crate::git_mode::GitFileStatus;
use crate::git_mode::GitModeState;
use crate::theme::Theme;
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table};

/// Background color for git mode popup (distinguishes from search popup)
const GIT_BG_COLOR: Color = Color::Rgb(0x20, 0x24, 0x30); // Slightly blue-tinted dark background

/// Renders the git mode popup
pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = app.config.theme.clone();
    let area = frame.area();

    // Calculate centered popup size
    let popup_area = popup::centered_popup(
        area,
        SEARCH_POPUP_WIDTH_PERCENT,
        SEARCH_POPUP_HEIGHT_PERCENT,
    );
    // Use distinct background for git mode
    popup::render_popup_background(frame, popup_area, GIT_BG_COLOR);

    let inner = popup::popup_inner(popup_area, SEARCH_POPUP_MARGIN);

    // Split into header and content areas
    let header_height = 3u16;
    let content_area = Rect {
        x: inner.x,
        y: inner.y + header_height,
        width: inner.width,
        height: inner.height.saturating_sub(header_height),
    };
    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: header_height,
    };

    // Render header with mode indicator and help
    render_header(frame, app, header_area, &theme);

    // Render content based on current view
    match app.git_mode_state {
        GitModeState::Log => render_log(frame, app, content_area, &theme),
        GitModeState::Status => render_status(frame, app, content_area, &theme),
        GitModeState::Diff => render_diff(frame, app, content_area, &theme),
        GitModeState::None => {}
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let view_name = app.git_mode_state.view_name();
    let help_text = match app.git_mode_state {
        GitModeState::Log => "l: Log | s: Status | d: Diff | Shift+G: Exit",
        GitModeState::Status => "l: Log | s: Status | Shift+G: Exit",
        GitModeState::Diff => "↑/↓: Navigate files | PgUp/PgDn: Scroll | d: Back | Shift+G: Exit",
        GitModeState::None => "",
    };

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.fg_git_refs))
        .title(format!(" Git Mode - {view_name} "))
        .title_style(Style::default().fg(theme.fg_git_refs).bold());

    let header_text = Paragraph::new(help_text)
        .style(Style::default().fg(theme.fg_dim))
        .alignment(Alignment::Right)
        .block(header_block);

    frame.render_widget(header_text, area);
}

fn render_log(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let entries = app.git_log.entries();

    if entries.is_empty() {
        let no_commits = Paragraph::new("No commits found or not in a git repository")
            .style(Style::default().fg(theme.fg_dim).bg(GIT_BG_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(no_commits, area);
        return;
    }

    // Calculate column widths
    let hash_width = 9u16; // 7 chars + margin
    let author_width = 16u16;
    let date_width = 12u16;
    let message_width = area
        .width
        .saturating_sub(hash_width + author_width + date_width + 8);

    // Build table rows with colored cells like git log --pretty
    // Skip rows based on scroll offset
    let scroll_offset = app.git_log_list_scroll;
    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .map(|(idx, entry)| {
            let is_selected = idx == app.git_log_selected;

            // Format date as relative time
            let date_str = format_relative_time(entry.timestamp);

            // Truncate fields to fit
            let author = truncate_string(&entry.author, author_width as usize - 1);
            let message = truncate_string(&entry.message, message_width as usize);

            let cells = vec![
                // Hash - yellow color (like git log)
                Cell::from(entry.short_hash.clone()).style(Style::default().fg(theme.fg_git_hash)),
                // Author - cyan/blue color
                Cell::from(author).style(Style::default().fg(theme.fg_git_author)),
                // Date - green color
                Cell::from(date_str).style(Style::default().fg(theme.fg_git_date)),
                // Message - default text color
                Cell::from(message).style(Style::default().fg(theme.fg_text)),
            ];

            let row_style = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default().bg(GIT_BG_COLOR)
            };

            Row::new(cells).style(row_style)
        })
        .collect();

    let widths = vec![
        Constraint::Length(hash_width),
        Constraint::Length(author_width),
        Constraint::Length(date_width),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Hash", "Author", "Date", "Message"])
                .style(Style::default().fg(theme.fg_git_refs).bold().underlined())
                .bottom_margin(1),
        )
        .style(Style::default().bg(GIT_BG_COLOR));

    frame.render_widget(table, area);

    // Show full message for selected commit in a detail area if there's room
    #[allow(clippy::cast_possible_truncation)]
    let entries_count = entries.len() as u16;
    if area.height > entries_count + 5 {
        let detail_area = Rect {
            x: area.x,
            y: area.y + entries_count + 3,
            width: area.width,
            height: area.height.saturating_sub(entries_count + 3),
        };

        if let Some(entry) = entries.get(app.git_log_selected) {
            render_log_detail(frame, entry, detail_area, theme, app.git_log_scroll);
        }
    }
}

fn render_log_detail(
    frame: &mut Frame,
    entry: &crate::git_mode::GitLogEntry,
    area: Rect,
    theme: &Theme,
    scroll: u16,
) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.fg_dim))
        .title(" Commit Details ");

    // Create colored lines for commit details
    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Commit: ", Style::default().fg(theme.fg_dim)),
            Span::styled(&entry.hash, Style::default().fg(theme.fg_git_hash)),
        ]),
        Line::from(vec![
            Span::styled("Author: ", Style::default().fg(theme.fg_dim)),
            Span::styled(&entry.author, Style::default().fg(theme.fg_git_author)),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().fg(theme.fg_dim)),
            Span::styled(
                format_timestamp(entry.timestamp),
                Style::default().fg(theme.fg_git_date),
            ),
        ]),
        Line::from(""),
    ];

    // Add message lines
    for msg_line in entry.full_message.lines().skip(scroll as usize) {
        lines.push(Line::from(msg_line.to_string()).style(Style::default().fg(theme.fg_text)));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(GIT_BG_COLOR));

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let entries = app.git_status_data.entries();

    let branch_height = 2u16;
    let branch_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: branch_height,
    };
    let list_area = Rect {
        x: area.x,
        y: area.y + branch_height,
        width: area.width,
        height: area.height.saturating_sub(branch_height),
    };

    let branch_text = Paragraph::new(Line::from(vec![
        Span::styled("On branch: ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            app.git_status_data.branch().unwrap_or("unknown"),
            Style::default().fg(theme.fg_git_refs).bold(),
        ),
    ]))
    .style(Style::default().bg(GIT_BG_COLOR))
    .alignment(Alignment::Left);
    frame.render_widget(branch_text, branch_area);

    if entries.is_empty() {
        let clean = Paragraph::new("Working tree clean")
            .style(Style::default().fg(theme.fg_git_date).bg(GIT_BG_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(clean, list_area);
        return;
    }

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = idx == app.git_status_selected;

            let index_char = entry.index_status.as_char();
            let worktree_char = entry.worktree_status.as_char();

            // Color the status characters
            let index_style = status_color(entry.index_status, theme);
            let worktree_style = status_color(entry.worktree_status, theme);

            let line = Line::from(vec![
                Span::styled(index_char.to_string(), index_style),
                Span::styled(worktree_char.to_string(), worktree_style),
                Span::styled(" ", Style::default()),
                Span::styled(&entry.path, Style::default().fg(theme.fg_text)),
            ]);

            let style = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default().bg(GIT_BG_COLOR)
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .style(Style::default().bg(GIT_BG_COLOR))
        .highlight_symbol("> ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.git_status_selected));

    frame.render_stateful_widget(list, list_area, &mut list_state);
}

/// Get color for a file status
fn status_color(status: GitFileStatus, theme: &Theme) -> Style {
    let color = match status {
        GitFileStatus::Added => theme.fg_git_date, // Green for added
        GitFileStatus::Modified => theme.fg_git_hash, // Yellow for modified
        GitFileStatus::Deleted | GitFileStatus::Conflict => theme.fg_modified, // Red for deleted/conflict
        GitFileStatus::Renamed => theme.fg_git_refs, // Magenta for renamed
        _ => theme.fg_dim,                           // Gray for untracked and others
    };
    Style::default().fg(color)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[allow(clippy::cast_possible_wrap)]
fn format_relative_time(timestamp: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3_600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86_400 {
        format!("{}h ago", diff / 3_600)
    } else if diff < 604_800 {
        format!("{}d ago", diff / 86_400)
    } else if diff < 2_592_000 {
        format!("{}w ago", diff / 604_800)
    } else if diff < 31_536_000 {
        format!("{}mo ago", diff / 2_592_000)
    } else {
        format!("{}y ago", diff / 31_536_000)
    }
}

#[allow(clippy::cast_sign_loss)]
fn format_timestamp(timestamp: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);

    // Simple formatting - in a real app you might use chrono
    datetime.duration_since(UNIX_EPOCH).map_or_else(
        |_| "unknown".to_string(),
        |time| {
            let secs = time.as_secs();
            let mins = (secs / 60) % 60;
            let hours = (secs / 3600) % 24;
            let days = secs / 86400;
            format!("{days} days, {hours:02}:{mins:02}")
        },
    )
}

fn render_diff(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    let files = app.git_diff.files();

    if files.is_empty() {
        let no_diff = Paragraph::new("No changes in this commit")
            .style(Style::default().fg(theme.fg_dim).bg(GIT_BG_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(no_diff, area);
        return;
    }

    // Split area into files list (left) and diff content (right)
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    let files_width = (f32::from(area.width) * 0.3) as u16;
    let files_area = Rect {
        x: area.x,
        y: area.y,
        width: files_width,
        height: area.height,
    };

    let diff_area = Rect {
        x: area.x + files_width,
        y: area.y,
        width: area.width - files_width,
        height: area.height,
    };

    // Store areas for mouse support
    app.last_diff_files_area = Some(files_area);
    app.last_diff_content_area = Some(diff_area);

    // Render files list
    render_diff_files_list(frame, app, files_area, theme);

    // Render diff content
    render_diff_content(frame, app, diff_area, theme);
}

fn render_diff_files_list(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let files = app.git_diff.files();

    let items: Vec<ListItem> = files
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let is_selected = idx == app.git_diff_selected_file;
            let status_char = if file.is_new {
                'A'
            } else if file.is_deleted {
                'D'
            } else {
                'M'
            };

            let file_name = file.path.split('/').next_back().unwrap_or(&file.path);

            let label = format!(" {status_char} {file_name}");
            let style = if is_selected {
                Style::default().bg(theme.bg_selected).fg(theme.fg_text)
            } else {
                Style::default().fg(theme.fg_text)
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.fg_git_refs))
            .title(" Files ")
            .title_style(Style::default().fg(theme.fg_git_refs)),
    );

    frame.render_widget(list, area);
}

fn render_diff_content(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let files = app.git_diff.files();

    if files.is_empty() {
        return;
    }

    let selected_file = &files[app.git_diff_selected_file];
    let total_insertions = app.git_diff.total_insertions();
    let total_deletions = app.git_diff.total_deletions();

    let mut content = vec![
        Line::from(vec![Span::styled(
            format!(
                " {} (+{} -{}) ",
                selected_file.path, selected_file.insertions, selected_file.deletions
            ),
            Style::default().fg(theme.fg_text).bold(),
        )]),
        Line::from(""),
    ];

    // Parse and color the diff content
    let diff_lines: Vec<&str> = selected_file.content.lines().collect();
    let start = (app.git_diff_scroll as usize).min(diff_lines.len());
    let end = (start + area.height.saturating_sub(2) as usize).min(diff_lines.len());

    // Account for borders (1 char on each side)
    let available_width = area.width.saturating_sub(4) as usize;

    for line in &diff_lines[start..end] {
        // Truncate line to fit within available width
        let truncated_line = if line.len() > available_width {
            format!("{}…", &line[..available_width.saturating_sub(1)])
        } else {
            (*line).to_string()
        };

        let styled_line = if line.starts_with('+') && !line.starts_with("+++") {
            Line::from(Span::styled(
                truncated_line,
                Style::default().fg(Color::Green).bg(Color::Rgb(40, 50, 40)),
            ))
        } else if line.starts_with('-') && !line.starts_with("---") {
            Line::from(Span::styled(
                truncated_line,
                Style::default().fg(Color::Red).bg(Color::Rgb(50, 40, 40)),
            ))
        } else if line.starts_with('@') {
            Line::from(Span::styled(
                truncated_line,
                Style::default().fg(Color::Cyan).bg(Color::Rgb(30, 40, 50)),
            ))
        } else {
            Line::from(truncated_line)
        };
        content.push(styled_line);
    }

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(theme.fg_text).bg(GIT_BG_COLOR))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.fg_git_refs))
                .title(format!(" Diff (+{total_insertions} -{total_deletions}) "))
                .title_style(Style::default().fg(theme.fg_git_refs)),
        );

    frame.render_widget(paragraph, area);
}
