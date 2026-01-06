use arboard::Clipboard;

pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            clipboard: Clipboard::new().ok(),
        }
    }

    pub fn copy_text(&mut self, text: &str) -> bool {
        self.clipboard
            .as_mut()
            .map_or(false, |clipboard| clipboard.set_text(text).is_ok())
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
