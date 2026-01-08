use crate::app::split::Pane;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

pub fn render(frame: &mut Frame, pane: &Pane, area: Rect, theme: &Theme, is_active: bool) {
    let border_style = if is_active {
        Style::default().fg(theme.fg_selected)
    } else {
        Style::default().fg(theme.fg_dim)
    };

    let block = Block::bordered()
        .border_style(border_style)
        .style(Style::default().bg(theme.bg_main));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if let Some(preview) = &pane.preview_content {
        let horizontal = Layout::horizontal([Constraint::Length(5), Constraint::Min(0)]);
        let [line_num_area, content_area] = horizontal.areas(inner_area);

        let visible_height = content_area.height as usize;
        let start = pane.scroll as usize;
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

        // Content with selection highlighting
        let visible_lines: Vec<Line> = pane.selection.as_ref().map_or_else(
            || preview.lines[start..end].to_vec(),
            |selection| {
                crate::ui::preview::apply_selection_to_lines(
                    &preview.lines[start..end],
                    &preview.raw_lines[start..end],
                    selection,
                    start,
                    theme,
                )
            },
        );

        let content = Paragraph::new(visible_lines).style(Style::default().bg(theme.bg_main));
        frame.render_widget(content, content_area);
    } else {
        let placeholder = Paragraph::new("Empty")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner_area);
    }
}
