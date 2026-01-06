use crate::constants::{
    MAX_SPLIT_PERCENT, MIN_SPLIT_PERCENT, PREVIEW_PAGE_SCROLL_LINES, SPLIT_RESIZE_STEP,
};

use super::App;

impl App {
    pub(super) fn navigate_down(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current = self.file_list_state.selected().unwrap_or(0);
        let next = if current >= self.files.len() - 1 {
            0
        } else {
            current + 1
        };

        self.file_list_state.select(Some(next));
        self.preview_scroll = 0;
        self.load_preview();
    }

    pub(super) fn navigate_up(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current = self.file_list_state.selected().unwrap_or(0);
        let prev = if current == 0 {
            self.files.len() - 1
        } else {
            current - 1
        };

        self.file_list_state.select(Some(prev));
        self.preview_scroll = 0;
        self.load_preview();
    }

    pub(super) fn scroll_preview_down(&mut self) {
        if let Some(preview) = &self.preview_content {
            if !preview.lines.is_empty() {
                let max_scroll = preview.lines.len().saturating_sub(1);
                self.preview_scroll =
                    (self.preview_scroll + 1).min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    pub(super) fn scroll_preview_page_down(&mut self) {
        if let Some(preview) = &self.preview_content {
            if !preview.lines.is_empty() {
                let max_scroll = preview.lines.len().saturating_sub(1);
                self.preview_scroll = (self.preview_scroll + PREVIEW_PAGE_SCROLL_LINES)
                    .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn scroll_preview_page_up(&mut self) {
        self.preview_scroll = self
            .preview_scroll
            .saturating_sub(PREVIEW_PAGE_SCROLL_LINES);
    }

    pub(super) fn shrink_file_list(&mut self) {
        self.split_percent = self
            .split_percent
            .saturating_sub(SPLIT_RESIZE_STEP)
            .max(MIN_SPLIT_PERCENT);
    }

    pub(super) fn grow_file_list(&mut self) {
        self.split_percent = (self.split_percent + SPLIT_RESIZE_STEP).min(MAX_SPLIT_PERCENT);
    }
}
