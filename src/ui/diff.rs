//! Diff viewer UI for git changes

use crate::app::App;
use crate::git_mode::DiffFileStat;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table};

/// Renders the diff viewer split view (files list on left, diff content on right)
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.config.theme;

    // Split into header, file list (left), and diff content (right)
    let header_height = 3u16;
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: header_height,
    };

    let content_area = Rect {
        x: area.x,
        y: area.y + header_height,
        width: area.width,
        height: area.height.saturating_sub(header_height),
    };

    // Render header with statistics
    render_header(frame, app, header_area, theme);

    // Split content area: files list (left ~30%) and diff (right ~70%)
    let files_width = (content_area.width as f32 * 0.3) as u16;
    let files_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: files_width,
        height: content_area.height,
    };

    let diff_area = Rect {
        x: content_area.x + files_width + 1,
        y: content_area.y,
        width: content_area.width.saturating_sub(files_width + 1),
        height: content_area.height,
    };

    // Render file list
    render_file_list(frame, app, files_area, theme);

    // Render divider
    render_vertical_divider(
        frame,
        Rect {
            x: content_area.x + files_width,
            y: content_area.y,
            width: 1,
            height: content_area.height,
        },
        theme,
    );

    // Render diff content for selected file
    render_diff_content(frame, app, diff_area, theme);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let total_ins = app.git_diff.total_insertions();
    let total_dels = app.git_diff.total_deletions();
    let file_count = app.git_diff.files().len();

    let stats_text = format!(
        " {} files changed | {} insertions | {} deletions ",
        file_count, total_ins, total_dels
    );

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.fg_dim))
        .title(" Diff View ")
        .title_style(Style::default().fg(theme.fg_text).bold());

    let header_widget = Paragraph::new(stats_text)
        .style(Style::default().fg(theme.fg_dim))
        .alignment(Alignment::Center)
        .block(header_block);

    frame.render_widget(header_widget, area);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let files = app.git_diff.files();

    if files.is_empty() {
        let no_changes = Paragraph::new("No changes")
            .style(Style::default().fg(theme.fg_dim))
            .alignment(Alignment::Center);
        frame.render_widget(no_changes, area);
        return;
    }

    let items: Vec<ListItem> = files
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let is_selected = idx == app.git_diff_selected_file;

            let change_char = file.change_char();
            let change_color = match change_char {
                'A' => theme.fg_git_date, // Green for added
                'D' => theme.fg_modified, // Red for deleted
                'M' => theme.fg_git_hash, // Yellow for modified
                _ => theme.fg_text,
            };

            let file_name = file.path.split('/').last().unwrap_or(&file.path);

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", change_char),
                    Style::default().fg(change_color),
                ),
                Span::raw(format!("{} ", file_name)),
                Span::styled(
                    format!("+{} -{}", file.insertions, file.deletions),
                    Style::default().fg(theme.fg_dim),
                ),
            ]);

            let style = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Files ")
            .border_style(Style::default().fg(theme.fg_dim)),
    );

    frame.render_widget(list, area);
}

fn render_vertical_divider(frame: &mut Frame, area: Rect, theme: &Theme) {
    let divider = Block::default().style(Style::default().fg(theme.fg_dim).bg(theme.bg_main));

    frame.render_widget(divider, area);
}

fn render_diff_content(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let files = app.git_diff.files();

    if files.is_empty() {
        return;
    }

    if app.git_diff_selected_file >= files.len() {
        return;
    }

    let selected_file = &files[app.git_diff_selected_file];

    // Split into header and content
    let header_height = 2u16;
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: header_height,
    };

    let content_area = Rect {
        x: area.x,
        y: area.y + header_height,
        width: area.width,
        height: area.height.saturating_sub(header_height),
    };

    // Render file header
    let file_header = Paragraph::new(format!(
        "{} {}",
        selected_file.change_char(),
        selected_file.path
    ))
    .style(Style::default().fg(theme.fg_text))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.fg_dim)),
    );

    frame.render_widget(file_header, header_area);

    // Render diff content with color coding
    let lines: Vec<Line> = selected_file
        .content
        .lines()
        .skip(app.git_diff_scroll as usize)
        .take(content_area.height as usize)
        .map(|line| {
            if line.starts_with('+') && !line.starts_with("+++") {
                // Addition (green)
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(theme.fg_git_date),
                ))
            } else if line.starts_with('-') && !line.starts_with("---") {
                // Deletion (red)
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(theme.fg_modified),
                ))
            } else if line.starts_with('@') {
                // Hunk header (cyan)
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(theme.fg_git_author),
                ))
            } else {
                // Context line or header
                Line::from(Span::raw(line.to_string()))
            }
        })
        .collect();

    let diff_widget = Paragraph::new(lines)
        .style(Style::default().fg(theme.fg_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Changes ")
                .border_style(Style::default().fg(theme.fg_dim)),
        );

    frame.render_widget(diff_widget, content_area);
}
