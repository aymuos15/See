use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.config.theme;

    let block = Block::default().style(Style::default().bg(theme.bg_main));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if let Some(preview) = &app.preview_content {
        let horizontal = Layout::horizontal([Constraint::Length(5), Constraint::Min(1)]);
        let [line_num_area, content_area] = horizontal.areas(inner_area);

        let visible_height = content_area.height as usize;
        let start = app.preview_scroll as usize;
        let end = (start + visible_height).min(preview.lines.len());

        // Line numbers
        let line_numbers: Vec<Line> = (start + 1..=end)
            .map(|n| {
                Line::from(format!("{n:>4} "))
                    .style(Style::default().fg(theme.line_num).bg(theme.bg_main))
            })
            .collect();

        let line_num_paragraph =
            Paragraph::new(line_numbers).style(Style::default().bg(theme.bg_main));
        frame.render_widget(line_num_paragraph, line_num_area);

        // Content
        let visible_lines: Vec<Line> = preview.lines[start..end].to_vec();
        let content = Paragraph::new(visible_lines).style(Style::default().bg(theme.bg_main));
        frame.render_widget(content, content_area);
    } else {
        let placeholder = Paragraph::new("Select a file to preview")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner_area);
    }
}
