use ratatui::text::Line;
use std::path::PathBuf;

/// Type of content being previewed
#[derive(Clone)]
pub enum PreviewContentType {
    /// Text content with syntax highlighting
    Text {
        lines: Vec<Line<'static>>,
        raw_lines: Vec<String>,
    },
    /// Image content for Kitty terminal
    Image {
        /// Path to image file
        path: PathBuf,
        /// Image dimensions in pixels
        dimensions: (u32, u32),
    },
    /// PDF content, rendered page by page into the continuous scroll view.
    /// Page state lives in `App::pdf_view`; this only marks the file's kind.
    Pdf {
        /// Path to PDF file
        path: PathBuf,
    },
}

/// Reference-counted preview content for efficient sharing between panes
pub type SharedPreviewContent = std::rc::Rc<PreviewContentType>;
