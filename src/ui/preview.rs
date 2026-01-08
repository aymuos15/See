use crate::app::selection::TextSelection;
use crate::app::App;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.config.theme;

    let block = Block::default().style(Style::default().bg(theme.bg_main));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Store area for coordinate mapping
    app.last_preview_area = Some(inner_area);

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

        // Content with selection highlighting
        let visible_lines: Vec<Line> = app.selection.as_ref().map_or_else(
            || preview.lines[start..end].to_vec(),
            |selection| {
                apply_selection_to_lines(
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
        let placeholder = Paragraph::new("Select a file to preview")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner_area);
    }
}

/// Apply selection highlighting to lines
pub fn apply_selection_to_lines<'a>(
    highlighted_lines: &[Line<'a>],
    raw_lines: &[String],
    selection: &TextSelection,
    start_line_idx: usize,
    theme: &Theme,
) -> Vec<Line<'a>> {
    let (sel_start, sel_end) = selection.ordered();
    let selection_style = Style::default().bg(theme.bg_selection);

    highlighted_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_idx = start_line_idx + i;

            // Check if this line is in selection range
            if line_idx < sel_start.line || line_idx > sel_end.line {
                return line.clone();
            }

            // Get the raw line for bounds checking
            let raw_line = raw_lines.get(i).map_or(0, String::len);

            // Calculate selection bounds for this line
            let start_col = if line_idx == sel_start.line {
                sel_start.column
            } else {
                0
            };
            let end_col = if line_idx == sel_end.line {
                sel_end.column
            } else {
                raw_line
            };

            // Apply selection style to spans
            apply_selection_to_line(line, start_col, end_col, selection_style)
        })
        .collect()
}

/// Apply selection style to a single line's spans
fn apply_selection_to_line(
    line: &Line<'_>,
    start_col: usize,
    end_col: usize,
    selection_style: Style,
) -> Line<'static> {
    let mut new_spans: Vec<Span<'static>> = Vec::new();
    let mut current_col = 0;

    for span in &line.spans {
        let span_len = span.content.chars().count();
        let span_start = current_col;
        let span_end = current_col + span_len;

        if span_end <= start_col || span_start >= end_col {
            // Span is entirely outside selection
            new_spans.push(Span::styled(span.content.to_string(), span.style));
        } else if span_start >= start_col && span_end <= end_col {
            // Span is entirely within selection
            new_spans.push(Span::styled(
                span.content.to_string(),
                span.style.patch(selection_style),
            ));
        } else {
            // Span partially overlaps selection - split it
            let content_chars: Vec<char> = span.content.chars().collect();
            let rel_start = start_col.saturating_sub(span_start);
            let rel_end = end_col.saturating_sub(span_start).min(span_len);

            // Before selection
            if rel_start > 0 {
                let before: String = content_chars[..rel_start].iter().collect();
                new_spans.push(Span::styled(before, span.style));
            }

            // Selected portion
            if rel_start < rel_end {
                let selected: String = content_chars[rel_start..rel_end].iter().collect();
                new_spans.push(Span::styled(selected, span.style.patch(selection_style)));
            }

            // After selection
            if rel_end < span_len {
                let after: String = content_chars[rel_end..].iter().collect();
                new_spans.push(Span::styled(after, span.style));
            }
        }

        current_col = span_end;
    }

    Line::from(new_spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::selection::TextPosition;
    use ratatui::style::Color;

    fn create_test_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn test_apply_selection_to_line_entire_span() {
        let line = Line::from(vec![Span::raw("Hello")]);
        let selection_style = Style::default().bg(Color::Yellow);

        let result = apply_selection_to_line(&line, 0, 5, selection_style);

        assert_eq!(result.spans.len(), 1);
        assert_eq!(result.spans[0].content, "Hello");
        assert_eq!(result.spans[0].style.bg, Some(Color::Yellow));
    }

    #[test]
    fn test_apply_selection_to_line_partial_span() {
        let line = Line::from(vec![Span::raw("Hello World")]);
        let selection_style = Style::default().bg(Color::Yellow);

        let result = apply_selection_to_line(&line, 0, 5, selection_style);

        // Should have 2 spans: "Hello" (selected) and " World" (not selected)
        assert_eq!(result.spans.len(), 2);
        assert_eq!(result.spans[0].content, "Hello");
        assert_eq!(result.spans[0].style.bg, Some(Color::Yellow));
        assert_eq!(result.spans[1].content, " World");
        assert_eq!(result.spans[1].style.bg, None);
    }

    #[test]
    fn test_apply_selection_to_line_middle_selection() {
        let line = Line::from(vec![Span::raw("Hello World")]);
        let selection_style = Style::default().bg(Color::Yellow);

        let result = apply_selection_to_line(&line, 2, 8, selection_style);

        // Should have 3 spans: "He" + "llo Wo" (selected) + "rld"
        assert_eq!(result.spans.len(), 3);
        assert_eq!(result.spans[0].content, "He");
        assert_eq!(result.spans[1].content, "llo Wo");
        assert_eq!(result.spans[1].style.bg, Some(Color::Yellow));
        assert_eq!(result.spans[2].content, "rld");
    }

    #[test]
    fn test_apply_selection_to_line_no_overlap() {
        let line = Line::from(vec![Span::raw("Hello")]);
        let selection_style = Style::default().bg(Color::Yellow);

        let result = apply_selection_to_line(&line, 10, 15, selection_style);

        // No overlap, should return unchanged
        assert_eq!(result.spans.len(), 1);
        assert_eq!(result.spans[0].content, "Hello");
        assert_eq!(result.spans[0].style.bg, None);
    }

    #[test]
    fn test_apply_selection_to_lines_single_line() {
        let lines = vec![Line::from(vec![Span::raw("Hello World")])];
        let raw_lines = vec!["Hello World".to_string()];
        let selection = TextSelection {
            anchor: TextPosition::new(0, 0),
            cursor: TextPosition::new(0, 5),
            active: false,
        };
        let theme = create_test_theme();

        let result = apply_selection_to_lines(&lines, &raw_lines, &selection, 0, &theme);

        assert_eq!(result.len(), 1);
        // First span should be selected
        assert_eq!(result[0].spans[0].content, "Hello");
    }

    #[test]
    fn test_apply_selection_to_lines_multi_line() {
        let lines = vec![
            Line::from(vec![Span::raw("Line 1")]),
            Line::from(vec![Span::raw("Line 2")]),
            Line::from(vec![Span::raw("Line 3")]),
        ];
        let raw_lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];
        let selection = TextSelection {
            anchor: TextPosition::new(0, 3),
            cursor: TextPosition::new(2, 3),
            active: false,
        };
        let theme = create_test_theme();

        let result = apply_selection_to_lines(&lines, &raw_lines, &selection, 0, &theme);

        assert_eq!(result.len(), 3);
        // All three lines should have selection applied
    }

    #[test]
    fn test_apply_selection_outside_visible_range() {
        let lines = vec![Line::from(vec![Span::raw("Visible line")])];
        let raw_lines = vec!["Visible line".to_string()];
        // Selection is on line 10, but we're showing line 0
        let selection = TextSelection {
            anchor: TextPosition::new(10, 0),
            cursor: TextPosition::new(10, 5),
            active: false,
        };
        let theme = create_test_theme();

        let result = apply_selection_to_lines(&lines, &raw_lines, &selection, 0, &theme);

        // Line should be unchanged since selection is outside visible range
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].spans[0].content, "Visible line");
    }
}
