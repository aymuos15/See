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
}

impl PreviewContentType {
    #[allow(dead_code)]
    pub const fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. })
    }
}

/// Reference-counted preview content for efficient sharing between panes
pub type SharedPreviewContent = std::rc::Rc<PreviewContentType>;
