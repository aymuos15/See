use crate::constants::{
    MAX_SPLIT_PERCENT, MIN_SPLIT_PERCENT, MOUSE_SCROLL_LINES, PREVIEW_PAGE_SCROLL_LINES,
    SPLIT_RESIZE_STEP,
};
use std::rc::Rc;

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

        // Update active pane if split layout exists
        self.sync_active_pane_content(next);
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

        // Update active pane if split layout exists
        self.sync_active_pane_content(prev);
    }

    /// Sync the active pane's content with the main preview content.
    /// Uses `Rc::clone` for O(1) sharing instead of deep cloning.
    fn sync_active_pane_content(&mut self, file_idx: usize) {
        if let Some(ref mut layout) = self.split_layout {
            if let Some(preview) = &self.shared_preview_content {
                if let Some(pane) = layout.get_active_pane_mut() {
                    // Rc::clone is O(1) - just increments reference count
                    pane.preview_content = Some(Rc::clone(preview));
                    pane.scroll = 0;
                    if let Some(entry) = self.files.get(file_idx) {
                        pane.file_path = Some(entry.path.clone());
                    }
                }
            }
        }
    }

    pub(super) fn scroll_preview_down(&mut self) {
        if let Some(ref mut layout) = self.split_layout {
            if let Some(pane) = layout.get_active_pane_mut() {
                if let Some(preview) = &pane.preview_content {
                    if !preview.lines.is_empty() {
                        let max_scroll = preview.lines.len().saturating_sub(1);
                        pane.scroll =
                            (pane.scroll + 1).min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
                    }
                }
            }
        } else if let Some(preview) = &self.shared_preview_content {
            if !preview.lines.is_empty() {
                let max_scroll = preview.lines.len().saturating_sub(1);
                self.preview_scroll =
                    (self.preview_scroll + 1).min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn scroll_preview_up(&mut self) {
        if let Some(ref mut layout) = self.split_layout {
            if let Some(pane) = layout.get_active_pane_mut() {
                pane.scroll = pane.scroll.saturating_sub(1);
            }
        } else {
            self.preview_scroll = self.preview_scroll.saturating_sub(1);
        }
    }

    pub(super) fn mouse_scroll_down(&mut self) {
        if let Some(ref mut layout) = self.split_layout {
            if let Some(pane) = layout.get_active_pane_mut() {
                if let Some(preview) = &pane.preview_content {
                    if !preview.lines.is_empty() {
                        let max_scroll = preview.lines.len().saturating_sub(1);
                        pane.scroll = (pane.scroll + MOUSE_SCROLL_LINES)
                            .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
                    }
                }
            }
        } else if let Some(preview) = &self.shared_preview_content {
            if !preview.lines.is_empty() {
                let max_scroll = preview.lines.len().saturating_sub(1);
                self.preview_scroll = (self.preview_scroll + MOUSE_SCROLL_LINES)
                    .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn mouse_scroll_up(&mut self) {
        if let Some(ref mut layout) = self.split_layout {
            if let Some(pane) = layout.get_active_pane_mut() {
                pane.scroll = pane.scroll.saturating_sub(MOUSE_SCROLL_LINES);
            }
        } else {
            self.preview_scroll = self.preview_scroll.saturating_sub(MOUSE_SCROLL_LINES);
        }
    }

    fn get_page_scroll_amount(&self) -> u16 {
        self.last_preview_area
            .map_or(PREVIEW_PAGE_SCROLL_LINES, |area| {
                area.height.saturating_sub(2).max(1)
            })
    }

    pub(super) fn scroll_preview_page_down(&mut self) {
        let scroll_amount = self.get_page_scroll_amount();
        if let Some(ref mut layout) = self.split_layout {
            if let Some(pane) = layout.get_active_pane_mut() {
                if let Some(preview) = &pane.preview_content {
                    if !preview.lines.is_empty() {
                        let max_scroll = preview.lines.len().saturating_sub(1);
                        pane.scroll = (pane.scroll + scroll_amount)
                            .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
                    }
                }
            }
        } else if let Some(preview) = &self.shared_preview_content {
            if !preview.lines.is_empty() {
                let max_scroll = preview.lines.len().saturating_sub(1);
                self.preview_scroll = (self.preview_scroll + scroll_amount)
                    .min(u16::try_from(max_scroll).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn scroll_preview_page_up(&mut self) {
        let scroll_amount = self.get_page_scroll_amount();
        if let Some(ref mut layout) = self.split_layout {
            if let Some(pane) = layout.get_active_pane_mut() {
                pane.scroll = pane.scroll.saturating_sub(scroll_amount);
            }
        } else {
            self.preview_scroll = self.preview_scroll.saturating_sub(scroll_amount);
        }
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
