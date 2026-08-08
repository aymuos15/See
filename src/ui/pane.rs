use crate::app::split::Pane;
use crate::app::PreviewContentType;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Wrap};
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut Frame,
    pane: &Pane,
    area: Rect,
    theme: &Theme,
    _is_active: bool,
    wrap: bool,
    indent_guides: bool,
    highlight_word: Option<&str>,
    image_protocols: &mut std::collections::HashMap<
        PathBuf,
        ratatui_image::protocol::StatefulProtocol,
    >,
) {
    // No borders - tab bar serves as top separator, divider line between panes drawn separately
    let block = Block::default().style(Style::default().bg(theme.bg_main));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if let Some(content) = &pane.preview_content {
        match content.as_ref() {
            PreviewContentType::Text { lines, raw_lines } => {
                render_text_pane(
                    frame,
                    pane,
                    inner_area,
                    theme,
                    lines,
                    raw_lines,
                    wrap,
                    indent_guides,
                    highlight_word,
                );
            }
            PreviewContentType::Image { path, dimensions } => {
                let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                let protocol = image_protocols.get_mut(&canonical_path);
                render_image_pane(frame, inner_area, *dimensions, protocol);
            }
            PreviewContentType::Pdf {
                path,
                current_page,
                total_pages,
            } => {
                render_pdf_pane(
                    frame,
                    inner_area,
                    theme,
                    path,
                    *current_page,
                    *total_pages,
                    image_protocols,
                );
            }
        }
    } else {
        let placeholder = Paragraph::new("Empty")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner_area);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_text_pane(
    frame: &mut Frame,
    pane: &Pane,
    inner_area: Rect,
    theme: &Theme,
    lines: &[Line<'static>],
    raw_lines: &[String],
    wrap: bool,
    indent_guides: bool,
    highlight_word: Option<&str>,
) {
    let horizontal = Layout::horizontal([Constraint::Length(5), Constraint::Min(0)]);
    let [line_num_area, content_area] = horizontal.areas(inner_area);

    let visible_height = content_area.height as usize;
    let start = pane.scroll as usize;
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
    let mut visible_lines: Vec<Line> = pane.selection.as_ref().map_or_else(
        || lines[start..end].to_vec(),
        |selection| {
            crate::ui::preview::apply_selection_to_lines(
                &lines[start..end],
                &raw_lines[start..end],
                selection,
                start,
                theme,
            )
        },
    );

    // Apply word highlights if active (carries find highlight to split panes)
    if let Some(word) = highlight_word {
        visible_lines = crate::ui::preview::apply_word_highlights(
            &visible_lines,
            &raw_lines[start..end],
            word,
            theme,
        );
    }

    if indent_guides {
        let width = crate::ui::indent::infer_width(raw_lines);
        visible_lines =
            crate::ui::indent::apply(&visible_lines, &raw_lines[start..end], width, theme);
    }

    let content = Paragraph::new(visible_lines).style(Style::default().bg(theme.bg_main));
    let content = if wrap {
        content.wrap(Wrap { trim: false })
    } else {
        content
    };
    frame.render_widget(content, content_area);
}

fn render_image_pane(
    frame: &mut Frame,
    inner_area: Rect,
    dimensions: (u32, u32),
    protocol: Option<&mut ratatui_image::protocol::StatefulProtocol>,
) {
    use ratatui_image::StatefulImage;

    if let Some(protocol) = protocol {
        let image_widget = StatefulImage::default();
        frame.render_stateful_widget(image_widget, inner_area, protocol);
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
        frame.render_widget(placeholder, inner_area);
    }
}

fn render_pdf_pane(
    frame: &mut Frame,
    inner_area: Rect,
    theme: &Theme,
    path: &Path,
    current_page: usize,
    total_pages: usize,
    image_protocols: &mut std::collections::HashMap<
        PathBuf,
        ratatui_image::protocol::StatefulProtocol,
    >,
) {
    use ratatui_image::StatefulImage;

    // Split area for page indicator and content
    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
    let [content_area, indicator_area] = layout.areas(inner_area);

    // Look up the rendered page from cache
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut page_key = canonical_path;
    page_key.set_extension(format!("pdf.page{current_page}"));

    let protocol = image_protocols.get_mut(&page_key);

    if let Some(protocol) = protocol {
        let image_widget = StatefulImage::default();
        frame.render_stateful_widget(image_widget, content_area, protocol);
    } else if total_pages == 0 {
        let loading_text = "Loading PDF...";
        let placeholder = Paragraph::new(loading_text)
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, content_area);
    } else {
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
    let page_text = if total_pages > 0 {
        format!(" Page {}/{} ", current_page + 1, total_pages)
    } else {
        " Loading... ".to_string()
    };

    let indicator = Paragraph::new(page_text)
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_darker))
        .alignment(Alignment::Center);
    frame.render_widget(indicator, indicator_area);
}
