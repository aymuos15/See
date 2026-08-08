//! UI and behavior constants

/// Initial split between file list and preview pane (percentage)
pub const INITIAL_SPLIT_PERCENT: u16 = 10;

/// Minimum split percentage for file list pane
pub const MIN_SPLIT_PERCENT: u16 = 5;

/// Maximum split percentage for file list pane
pub const MAX_SPLIT_PERCENT: u16 = 80;

/// Increment/decrement for split resize operations
pub const SPLIT_RESIZE_STEP: u16 = 5;

/// Number of lines to scroll per mouse wheel tick
pub const MOUSE_SCROLL_LINES: u16 = 3;

/// Fallback number of lines to scroll per page down/up when viewport is unknown
pub const PREVIEW_PAGE_SCROLL_LINES: u16 = 10;

/// Maximum file size to preview (1 MB)
pub const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Maximum image file size to preview (10 MB)
pub const MAX_IMAGE_SIZE: u64 = 10 * 1024 * 1024;

/// Number of bytes to check for binary file detection
pub const BINARY_DETECTION_BYTES: usize = 8000;

/// Debounce duration for file watcher events (milliseconds)
pub const FILE_EVENT_DEBOUNCE_MS: u64 = 100;

/// Interval for search index refresh (seconds)
pub const SEARCH_INDEX_REFRESH_SECS: u64 = 30;

/// Search popup width as percentage of screen width
pub const SEARCH_POPUP_WIDTH_PERCENT: u16 = 60;

/// Search popup height as percentage of screen height
pub const SEARCH_POPUP_HEIGHT_PERCENT: u16 = 70;

/// How many files' highlighted lines to keep before clearing the cache
pub const HIGHLIGHT_CACHE_ENTRIES: usize = 48;

/// Default syntax highlighting theme name
pub const DEFAULT_SYNTAX_THEME: &str = "base16-eighties.dark";

/// Supported image file extensions
pub const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff", "ico", "tga",
];

/// Thumbnail size for fast preview (pixels)
pub const THUMBNAIL_SIZE: u32 = 256;

/// Delay before loading full-quality image after navigation stops (milliseconds)
pub const FULL_IMAGE_DELAY_MS: u64 = 150;

/// Maximum PDF file size to preview (50 MB)
pub const MAX_PDF_SIZE: u64 = 50 * 1024 * 1024;

/// Default PDF render resolution (pixels per point, 1.5 = 108 DPI)
pub const PDF_RENDER_SCALE: f32 = 2.0;
