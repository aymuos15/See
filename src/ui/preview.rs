use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let title = app
        .preview_content
        .as_ref()
        .map(|p| {
            p.path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        })
        .unwrap_or_else(|| "No file selected".to_string());

    let block = Block::default()
        .title(format!(" Preview: {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if let Some(preview) = &app.preview_content {
        let horizontal = Layout::horizontal([
            Constraint::Length(6),
            Constraint::Min(1),
        ]);
        let [line_num_area, content_area] = horizontal.areas(inner_area);

        let visible_height = content_area.height as usize;
        let start = app.preview_scroll as usize;
        let end = (start + visible_height).min(preview.lines.len());

        // Line numbers
        let line_numbers: Vec<Line> = (start + 1..=end)
            .map(|n| {
                Line::from(format!("{:>4} ", n)).style(Style::default().fg(Color::DarkGray))
            })
            .collect();

        let line_num_paragraph = Paragraph::new(line_numbers);
        frame.render_widget(line_num_paragraph, line_num_area);

        // Content
        let visible_lines: Vec<Line> = preview.lines[start..end].to_vec();
        let content = Paragraph::new(visible_lines);
        frame.render_widget(content, content_area);
    } else {
        let placeholder = Paragraph::new("Select a file to preview")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner_area);
    }
}
