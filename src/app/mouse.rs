use crate::ui::coordinates::screen_to_text_position;

use super::selection::TextSelection;
use super::App;

impl App {
    pub fn handle_mouse_down(&mut self, column: u16, row: u16) {
        // Check if click is in file list area
        if let Some(file_list_area) = self.last_file_list_area {
            if column >= file_list_area.x
                && column < file_list_area.x + file_list_area.width
                && row >= file_list_area.y
                && row < file_list_area.y + file_list_area.height
            {
                // Calculate which file was clicked
                let relative_row = row.saturating_sub(file_list_area.y) as usize;
                if relative_row < self.files.len() {
                    self.file_list_state.select(Some(relative_row));
                    self.preview_scroll = 0;
                    self.selection = None;
                    self.load_preview();
                }
                return;
            }
        }

        // Check if click is in preview area for text selection
        let Some(preview_area) = self.last_preview_area else {
            return;
        };
        let Some(preview) = &self.shared_preview_content else {
            return;
        };

        let pos = screen_to_text_position(
            column,
            row,
            preview_area,
            self.preview_scroll,
            preview.raw_lines.len(),
            &preview.raw_lines,
        );

        if let Some(pos) = pos {
            self.selection = Some(TextSelection::new(pos));
        } else {
            // Clicked outside content area, clear selection
            self.selection = None;
        }
    }

    pub fn handle_mouse_drag(&mut self, column: u16, row: u16) {
        let Some(selection) = self.selection.as_mut() else {
            return;
        };
        if !selection.active {
            return;
        }

        let Some(preview_area) = self.last_preview_area else {
            return;
        };
        let Some(preview) = &self.shared_preview_content else {
            return;
        };

        if let Some(pos) = screen_to_text_position(
            column,
            row,
            preview_area,
            self.preview_scroll,
            preview.raw_lines.len(),
            &preview.raw_lines,
        ) {
            selection.cursor = pos;
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn handle_mouse_up(&mut self, _column: u16, _row: u16) {
        if let Some(selection) = self.selection.as_mut() {
            selection.active = false;
        }
    }

    pub fn copy_selection(&mut self) -> bool {
        let Some(selection) = &self.selection else {
            return false;
        };
        if selection.is_empty() {
            return false;
        }

        let Some(preview) = &self.shared_preview_content else {
            return false;
        };

        let text = selection.extract_text(&preview.raw_lines);
        self.clipboard.copy_text(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::selection::TextPosition;
    use ratatui::prelude::Rect;

    fn create_test_app() -> App {
        // Create a minimal app for testing
        let temp_dir = std::env::temp_dir();
        App::new(temp_dir).expect("Failed to create test app")
    }

    #[test]
    fn test_handle_mouse_down_without_preview() {
        let mut app = create_test_app();
        app.shared_preview_content = None;
        app.last_preview_area = Some(Rect::new(0, 0, 80, 24));

        app.handle_mouse_down(10, 5);

        // Selection should remain None when no preview
        assert!(app.selection.is_none());
    }

    #[test]
    fn test_handle_mouse_down_without_area() {
        let mut app = create_test_app();
        app.last_preview_area = None;

        app.handle_mouse_down(10, 5);

        // Selection should remain None when no area
        assert!(app.selection.is_none());
    }

    #[test]
    fn test_handle_mouse_drag_without_active_selection() {
        let mut app = create_test_app();
        app.selection = Some(TextSelection {
            anchor: TextPosition::new(0, 0),
            cursor: TextPosition::new(0, 0),
            active: false, // Not active
        });

        let original_cursor = app.selection.as_ref().map(|s| s.cursor);
        app.handle_mouse_drag(20, 10);

        // Cursor should not change when selection is not active
        assert_eq!(app.selection.as_ref().map(|s| s.cursor), original_cursor);
    }

    #[test]
    fn test_handle_mouse_up_deactivates_selection() {
        let mut app = create_test_app();
        app.selection = Some(TextSelection {
            anchor: TextPosition::new(0, 0),
            cursor: TextPosition::new(1, 5),
            active: true,
        });

        app.handle_mouse_up(0, 0);

        // Selection should be deactivated
        assert!(!app.selection.as_ref().expect("selection exists").active);
    }

    #[test]
    fn test_copy_selection_without_selection() {
        let mut app = create_test_app();
        app.selection = None;

        let result = app.copy_selection();

        assert!(!result);
    }

    #[test]
    fn test_copy_selection_with_empty_selection() {
        let mut app = create_test_app();
        app.selection = Some(TextSelection {
            anchor: TextPosition::new(0, 5),
            cursor: TextPosition::new(0, 5), // Same position = empty
            active: false,
        });

        let result = app.copy_selection();

        assert!(!result);
    }

    #[test]
    fn test_copy_selection_without_preview() {
        let mut app = create_test_app();
        app.shared_preview_content = None;
        app.selection = Some(TextSelection {
            anchor: TextPosition::new(0, 0),
            cursor: TextPosition::new(0, 5),
            active: false,
        });

        let result = app.copy_selection();

        assert!(!result);
    }
}
