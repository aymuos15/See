//! UI and behavior constants

/// Initial split between file list and preview pane (percentage)
pub const INITIAL_SPLIT_PERCENT: u16 = 30;

/// Minimum split percentage for file list pane
pub const MIN_SPLIT_PERCENT: u16 = 10;

/// Maximum split percentage for file list pane
pub const MAX_SPLIT_PERCENT: u16 = 80;

/// Increment/decrement for split resize operations
pub const SPLIT_RESIZE_STEP: u16 = 5;

/// Number of lines to scroll per page down/up
pub const PREVIEW_PAGE_SCROLL_LINES: u16 = 10;

/// Maximum file size to preview (1 MB)
pub const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Number of bytes to check for binary file detection
pub const BINARY_DETECTION_BYTES: usize = 8000;
