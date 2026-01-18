use crate::app::selection::TextSelection;
use crate::app::App;
use crate::app::PreviewContentType;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Wrap};
use std::path::Path;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.config.theme.clone();

    // Fill background without using Block wrapper (no implicit margins)
    let bg = Block::default().style(Style::default().bg(theme.bg_main));
    frame.render_widget(bg, area);

    // Store area for coordinate mapping (use full area, no inner)
    app.last_preview_area = Some(area);
    app.last_pane_areas = vec![(0, area)];

    // Use shared_preview_content (Rc) for better performance
    if let Some(content) = app.shared_preview_content.clone() {
        match content.as_ref() {
            PreviewContentType::Text { lines, raw_lines } => {
                render_text_content(frame, app, area, &theme, lines, raw_lines);
            }
            PreviewContentType::Image {
                path, dimensions, ..
            } => {
                // Look up protocol from cache (need mutable access for rendering)
                let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                let protocol = app.image_protocols.get_mut(&canonical_path);
                render_image_content(frame, area, *dimensions, protocol);
            }
            PreviewContentType::Pdf {
                path,
                current_page,
                total_pages,
            } => {
                render_pdf_content(frame, app, area, path, *current_page, *total_pages, &theme);
            }
        }
    } else {
        let placeholder = Paragraph::new("Select a file to preview")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, area);
    }
}

fn render_text_content(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    theme: &Theme,
    lines: &[Line<'static>],
    raw_lines: &[String],
) {
    let horizontal = Layout::horizontal([Constraint::Length(5), Constraint::Min(1)]);
    let [line_num_area, content_area] = horizontal.areas(area);

    let visible_height = content_area.height as usize;
    let start = app.preview_scroll as usize;
    let end = (start + visible_height).min(lines.len());

    // Line numbers
    let line_numbers: Vec<Line> = (start + 1..=end)
        .map(|n| {
            Line::from(format!("{n:>4} "))
                .style(Style::default().fg(theme.line_num).bg(theme.bg_main))
        })
        .collect();

    let line_num_paragraph = Paragraph::new(line_numbers).style(Style::default().bg(theme.bg_main));
    frame.render_widget(line_num_paragraph, line_num_area);

    // Content with selection highlighting
    let mut visible_lines: Vec<Line> = app.selection.as_ref().map_or_else(
        || lines[start..end].to_vec(),
        |selection| {
            apply_selection_to_lines(
                &lines[start..end],
                &raw_lines[start..end],
                selection,
                start,
                theme,
            )
        },
    );

    // Apply word highlights if active
    if let Some(word) = &app.highlighted_word {
        visible_lines = apply_word_highlights(&visible_lines, &raw_lines[start..end], word, theme);
    }

    let content = Paragraph::new(visible_lines).style(Style::default().bg(theme.bg_main));
    let content = if app.config.wrap {
        content.wrap(Wrap { trim: false })
    } else {
        content
    };
    frame.render_widget(content, content_area);
}

fn render_image_content(
    frame: &mut Frame,
    area: Rect,
    dimensions: (u32, u32),
    protocol: Option<&mut ratatui_image::protocol::StatefulProtocol>,
) {
    use ratatui_image::StatefulImage;

    if let Some(protocol) = protocol {
        let image_widget = StatefulImage::default();
        frame.render_stateful_widget(image_widget, area, protocol);
    } else {
        // Show placeholder with dimensions while image loads
        // or if graphics protocol is not available
        let dimensions_text = format!(
            "Image Preview\n\n{}x{} pixels\n\n[Graphics protocol unavailable]\n[Kitty graphics required]",
            dimensions.0, dimensions.1
        );
        let placeholder = Paragraph::new(dimensions_text)
            .style(Style::default().fg(Theme::default().fg_text))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, area);
    }
}

fn render_pdf_content(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    path: &Path,
    current_page: usize,
    total_pages: usize,
    theme: &Theme,
) {
    use ratatui_image::StatefulImage;

    // Split area for page indicator and content
    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
    let [content_area, indicator_area] = layout.areas(area);

    // Look up the rendered page from cache
    let mut page_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    page_key.set_extension(format!("pdf.page{current_page}"));

    let protocol = app.image_protocols.get_mut(&page_key);

    // Check for PDF loading error first
    if let Some(ref error) = app.pdf_error {
        let error_text = format!(
            "PDF Error\n\n{}\n\nTo view PDFs, install the PDFium library:\n\n  - Linux: Install libpdfium-dev or download from\n    https://github.com/nicbarker/pdfium-binaries/releases\n  - macOS: brew install nicbarker/pdfium-binaries/pdfium\n  - Windows: Download pdfium.dll from the releases page",
            error
        );
        let placeholder = Paragraph::new(error_text)
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, content_area);
    } else if let Some(protocol) = protocol {
        // Render the PDF page image
        let image_widget = StatefulImage::default();
        frame.render_stateful_widget(image_widget, content_area, protocol);
    } else if total_pages == 0 {
        // Still loading page count
        let loading_text = "Loading PDF...";
        let placeholder = Paragraph::new(loading_text)
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, content_area);
    } else {
        // Page is loading or graphics protocol unavailable
        let loading_text = format!(
            "PDF Preview\n\nPage {} of {}\n\n[Rendering page...]",
            current_page + 1,
            total_pages
        );
        let placeholder = Paragraph::new(loading_text)
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, content_area);
    }

    // Render page indicator
    let page_text = if app.pdf_error.is_some() {
        " PDF Error - PDFium library not available ".to_string()
    } else if total_pages > 0 {
        format!(" Page {}/{} | n:next p:prev Home:first End:last ", current_page + 1, total_pages)
    } else {
        " Loading... ".to_string()
    };

    let indicator = Paragraph::new(page_text)
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_darker))
        .alignment(Alignment::Center);
    frame.render_widget(indicator, indicator_area);
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

            // Get the raw line length for bounds checking
            let raw_line_len = raw_lines.get(i).map_or(0, String::len);

            // Calculate selection bounds for this line
            let start_col = if line_idx == sel_start.line {
                sel_start.column
            } else {
                0
            };
            let end_col = if line_idx == sel_end.line {
                sel_end.column
            } else {
                raw_line_len
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

/// Apply word highlights to lines
pub fn apply_word_highlights<'a>(
    lines: &[Line<'a>],
    raw_lines: &[String],
    word: &str,
    theme: &Theme,
) -> Vec<Line<'a>> {
    if word.is_empty() {
        return lines.to_vec();
    }

    let highlight_style = Style::default()
        .bg(theme.bg_selection)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let word_lower = word.to_lowercase();

    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let Some(raw_line) = raw_lines.get(i) else {
                return line.clone();
            };

            let mut current_line = line.clone();
            let mut start_search = 0;
            let raw_line_lower = raw_line.to_lowercase();

            while let Some(start_idx) = raw_line_lower[start_search..].find(&word_lower) {
                let absolute_start = start_search + start_idx;
                let absolute_end = absolute_start + word.len();

                // Check if it's a whole word match
                let before_char = absolute_start
                    .checked_sub(1)
                    .and_then(|i| raw_line.chars().nth(i));
                let after_char = raw_line.chars().nth(absolute_end);

                let is_whole_word = before_char.is_none_or(|c| !c.is_alphanumeric() && c != '_')
                    && after_char.is_none_or(|c| !c.is_alphanumeric() && c != '_');

                if is_whole_word {
                    current_line = apply_selection_to_line(
                        &current_line,
                        absolute_start,
                        absolute_end,
                        highlight_style,
                    );
                }

                start_search = absolute_start + word.len().max(1);
            }

            current_line
        })
        .collect()
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

    #[test]
    fn test_apply_word_highlights() {
        let lines = vec![Line::from(vec![Span::raw("let mut app = App::new();")])];
        let raw_lines = vec!["let mut app = App::new();".to_string()];
        let theme = create_test_theme();

        // Highlight "app" - case-insensitive, so both "app" and "App" should be highlighted
        let result = apply_word_highlights(&lines, &raw_lines, "app", &theme);
        // "let mut " + "app" (highlighted) + " = " + "App" (highlighted) + "::new();" -> 5 spans
        assert_eq!(result[0].spans[1].content, "app");
        assert!(result[0].spans[1].style.bg.is_some());

        // "App" should also be highlighted (case-insensitive)
        let app_capital_span = result[0].spans.iter().find(|s| s.content == "App");
        assert!(app_capital_span.is_some());
        assert!(app_capital_span.expect("operation failed").style.bg.is_some());
    }
}
