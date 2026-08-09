use ratatui::text::Line;
use std::cell::OnceCell;
use std::path::PathBuf;

/// Type of content being previewed
#[derive(Clone)]
pub enum PreviewContentType {
    /// Text content with syntax highlighting
    Text {
        lines: Vec<Line<'static>>,
        raw_lines: Vec<String>,
        /// Indent-guide width, inferred from the whole file on first use.
        /// Cached here because it is a pure function of `raw_lines` and the
        /// render loop would otherwise rescan the file every frame.
        indent_width: OnceCell<usize>,
    },
    /// Image content for Kitty terminal
    Image {
        /// Path to image file, canonicalized at load time so per-frame
        /// protocol lookups need no filesystem access
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

impl PreviewContentType {
    /// Text content; the indent width is inferred lazily on first render.
    pub const fn text(lines: Vec<Line<'static>>, raw_lines: Vec<String>) -> Self {
        Self::Text {
            lines,
            raw_lines,
            indent_width: OnceCell::new(),
        }
    }
}

/// Reference-counted preview content for efficient sharing between panes
pub type SharedPreviewContent = std::rc::Rc<PreviewContentType>;
