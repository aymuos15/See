//! Renders a PDF as one continuously scrollable column of pages, centered
//! horizontally in whatever space the preview pane has.

use crate::app::pdf::{PdfView, PAGE_GAP_ROWS};
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui_image::picker::Picker;
use ratatui_image::StatefulImage;

/// Geometry of one page as laid out on screen.
struct PageBox {
    cols: u16,
    rows: u32,
    x: u16,
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    view: Option<&mut PdfView>,
    picker: Option<&Picker>,
    error: Option<&str>,
) {
    let content_area = area;

    if let Some(error) = error {
        let text = format!(
            "PDF Error\n\n{error}\n\nTo view PDFs, install the PDFium library:\n\n  - Linux: Install libpdfium-dev or download from\n    https://github.com/nicbarker/pdfium-binaries/releases\n  - macOS: brew install nicbarker/pdfium-binaries/pdfium\n  - Windows: Download pdfium.dll from the releases page"
        );
        centered(frame, content_area, theme, &text);
        return;
    }

    let (Some(view), Some(picker)) = (view, picker) else {
        centered(frame, content_area, theme, "Loading PDF...");
        return;
    };

    let page_box = layout_page(view, picker, content_area);
    view.page_rows = page_box.rows;

    if view.total_pages == 0 {
        // The page count is unknown while the first page renders, so the
        // scroll position cannot be clamped yet without discarding it.
        view.want_page(0);
        centered(frame, content_area, theme, "Loading PDF...");
        return;
    }

    // The viewport may have grown since the last scroll, so re-clamp.
    view.scroll_by(0, content_area.height);

    view.begin_frame();
    draw_pages(frame, content_area, theme, view, picker, &page_box);
    view.end_frame();

    crate::ui::preview::render_scrollbar(
        frame,
        content_area,
        theme,
        view.document_rows() as usize,
        view.scroll as usize,
    );
}

/// Fit a page to the viewport height, falling back to its width when the pane
/// is too narrow, so pages keep their proportions and sit centered.
fn layout_page(view: &PdfView, picker: &Picker, area: Rect) -> PageBox {
    let (cell_w, cell_h) = picker.font_size();
    let aspect = view.aspect();

    let max_width = f32::from(area.width) * f32::from(cell_w);
    let mut height = f32::from(area.height) * f32::from(cell_h);
    let mut width = height / aspect;
    if width > max_width {
        width = max_width;
        height = width * aspect;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cols = ((width / f32::from(cell_w)).round() as u16).clamp(1, area.width.max(1));
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rows = ((height / f32::from(cell_h)).round() as u32).max(1);

    PageBox {
        cols,
        rows,
        x: area.x + (area.width.saturating_sub(cols)) / 2,
    }
}

fn draw_pages(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    view: &mut PdfView,
    picker: &Picker,
    page_box: &PageBox,
) {
    let stride = page_box.rows + PAGE_GAP_ROWS;
    let viewport_top = view.scroll;
    let viewport_bottom = viewport_top + u32::from(area.height);
    let first = (viewport_top / stride) as usize;

    for page in first..view.total_pages {
        let top = u32::try_from(page)
            .unwrap_or(u32::MAX)
            .saturating_mul(stride);
        if top >= viewport_bottom {
            // Fetch one page beyond the fold so scrolling stays ahead of the
            // worker.
            view.want_page(page);
            break;
        }
        let bottom = top + page_box.rows;
        if bottom <= viewport_top {
            continue;
        }

        let visible_top = top.max(viewport_top);
        let visible_bottom = bottom.min(viewport_bottom);
        let row_offset = visible_top - top;
        #[allow(clippy::cast_possible_truncation)]
        let slice_rows = (visible_bottom - visible_top) as u16;
        if slice_rows == 0 {
            continue;
        }

        #[allow(clippy::cast_possible_truncation)]
        let rect = Rect {
            x: page_box.x,
            y: area.y + (visible_top - viewport_top) as u16,
            width: page_box.cols,
            height: slice_rows,
        };

        let slice = view.slice(
            page,
            page_box.cols,
            page_box.rows,
            row_offset,
            slice_rows,
            picker,
        );

        if let Some(protocol) = slice {
            frame.render_stateful_widget(StatefulImage::default(), rect, protocol);
        } else {
            view.want_page(page);
            let text = format!("[Rendering page {}...]", page + 1);
            centered(frame, rect, theme, &text);
        }
    }
}

fn centered(frame: &mut Frame, area: Rect, theme: &Theme, text: &str) {
    let paragraph = Paragraph::new(text.to_string())
        .style(Style::default().fg(theme.fg_dim).bg(theme.bg_main))
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
