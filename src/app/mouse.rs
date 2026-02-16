use crate::app::content::PreviewContentType;
use crate::ui::coordinates::screen_to_text_position;

use super::selection::{get_word_at, TextSelection};
use super::App;

/// Get raw lines from content if it's text, otherwise return None
fn get_raw_lines(content: &PreviewContentType) -> Option<&[String]> {
    match content {
        PreviewContentType::Text { raw_lines, .. } => Some(raw_lines),
        PreviewContentType::Image { .. } | PreviewContentType::Pdf { .. } => None,
    }
}

impl App {
    pub fn handle_mouse_down(&mut self, column: u16, row: u16) {
        // Check if click is in git diff view
        if let Some(files_area) = self.last_diff_files_area {
            if column >= files_area.x
                && column < files_area.x + files_area.width
                && row >= files_area.y
                && row < files_area.y + files_area.height
            {
                // Calculate which file was clicked
                // Account for borders and header
                let relative_row = row.saturating_sub(files_area.y).saturating_sub(1) as usize;
                if relative_row < self.git_diff.files().len() {
                    self.git_diff_selected_file = relative_row;
                    self.git_diff_scroll = 0;
                }
                return;
            }
        }

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
                    if let Some(split_layout) = &mut self.split_layout {
                        if let Some(pane) = split_layout.get_active_pane_mut() {
                            pane.scroll = 0;
                            pane.selection = None;
                        }
                    } else {
                        self.preview_scroll = 0;
                        self.selection = None;
                    }
                    self.load_preview();
                }
                return;
            }
        }

        // Check if click is in any pane area
        let mut clicked_pane_id = None;
        let mut clicked_area = None;

        for (id, area) in &self.last_pane_areas {
            if column >= area.x
                && column < area.x + area.width
                && row >= area.y
                && row < area.y + area.height
            {
                clicked_pane_id = Some(*id);
                clicked_area = Some(*area);
                break;
            }
        }

        if let Some(pane_id) = clicked_pane_id {
            let area = clicked_area.expect("Area must exist if pane_id exists");

            // Update active pane if split layout exists
            if let Some(split_layout) = &mut self.split_layout {
                split_layout.active_pane_index = pane_id;

                if let Some(pane) = split_layout.get_active_pane_mut() {
                    if let Some(content) = &pane.preview_content {
                        if let Some(raw_lines) = get_raw_lines(content) {
                            let pos = screen_to_text_position(
                                column,
                                row,
                                area,
                                pane.scroll,
                                raw_lines.len(),
                                raw_lines,
                            );

                            if let Some(pos) = pos {
                                pane.selection = Some(TextSelection::new(pos));
                                self.highlighted_word = get_word_at(raw_lines, pos);
                            } else {
                                pane.selection = None;
                                self.highlighted_word = None;
                            }
                        }
                    }
                }
            } else if let Some(content) = &self.shared_preview_content {
                // Single pane mode
                if let Some(raw_lines) = get_raw_lines(content) {
                    let pos = screen_to_text_position(
                        column,
                        row,
                        area,
                        self.preview_scroll,
                        raw_lines.len(),
                        raw_lines,
                    );

                    if let Some(pos) = pos {
                        self.selection = Some(TextSelection::new(pos));
                        self.highlighted_word = get_word_at(raw_lines, pos);
                    } else {
                        self.selection = None;
                        self.highlighted_word = None;
                    }
                }
            }
        }
    }

    pub fn handle_mouse_drag(&mut self, column: u16, row: u16) {
        if let Some(split_layout) = &mut self.split_layout {
            let active_id = split_layout.active_pane_index;
            if let Some(pane) = split_layout.get_active_pane_mut() {
                let Some(selection) = pane.selection.as_mut() else {
                    return;
                };
                if !selection.active {
                    return;
                }

                // Find the area for the active pane
                let area = self
                    .last_pane_areas
                    .iter()
                    .find(|(id, _)| *id == active_id)
                    .map(|(_, area)| *area);

                let Some(area) = area else {
                    return;
                };

                let Some(content) = &pane.preview_content else {
                    return;
                };

                if let Some(raw_lines) = get_raw_lines(content) {
                    if let Some(pos) = screen_to_text_position(
                        column,
                        row,
                        area,
                        pane.scroll,
                        raw_lines.len(),
                        raw_lines,
                    ) {
                        selection.cursor = pos;
                    }
                }
            }
        } else {
            let Some(selection) = self.selection.as_mut() else {
                return;
            };
            if !selection.active {
                return;
            }

            let Some(preview_area) = self.last_preview_area else {
                return;
            };
            let Some(content) = &self.shared_preview_content else {
                return;
            };

            if let Some(raw_lines) = get_raw_lines(content) {
                if let Some(pos) = screen_to_text_position(
                    column,
                    row,
                    preview_area,
                    self.preview_scroll,
                    raw_lines.len(),
                    raw_lines,
                ) {
                    selection.cursor = pos;
                }
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn handle_mouse_up(&mut self, _column: u16, _row: u16) {
        if let Some(split_layout) = &mut self.split_layout {
            if let Some(pane) = split_layout.get_active_pane_mut() {
                if let Some(selection) = pane.selection.as_mut() {
                    selection.active = false;
                }
            }
        } else if let Some(selection) = self.selection.as_mut() {
            selection.active = false;
        }
    }

    pub fn copy_selection(&mut self) -> bool {
        if let Some(split_layout) = &mut self.split_layout {
            if let Some(pane) = split_layout.get_active_pane() {
                let Some(selection) = &pane.selection else {
                    return false;
                };
                if selection.is_empty() {
                    return false;
                }

                let Some(content) = &pane.preview_content else {
                    return false;
                };

                if let Some(raw_lines) = get_raw_lines(content) {
                    let text = selection.extract_text(raw_lines);
                    return self.clipboard.copy_text(&text);
                }
                return false;
            }
            return false;
        }

        let Some(selection) = &self.selection else {
            return false;
        };
        if selection.is_empty() {
            return false;
        }

        let Some(content) = &self.shared_preview_content else {
            return false;
        };

        if let Some(raw_lines) = get_raw_lines(content) {
            let text = selection.extract_text(raw_lines);
            return self.clipboard.copy_text(&text);
        }
        false
    }

    pub fn select_all(&mut self) {
        if let Some(split_layout) = &mut self.split_layout {
            if let Some(pane) = split_layout.get_active_pane_mut() {
                let Some(content) = &pane.preview_content else {
                    return;
                };

                let Some(raw_lines) = get_raw_lines(content) else {
                    return;
                };

                if raw_lines.is_empty() {
                    return;
                }

                let last_line = raw_lines.len() - 1;
                let last_col = raw_lines.last().map_or(0, String::len);

                pane.selection = Some(TextSelection {
                    anchor: crate::app::selection::TextPosition::new(0, 0),
                    cursor: crate::app::selection::TextPosition::new(last_line, last_col),
                    active: false,
                });
            }
            return;
        }

        let Some(content) = &self.shared_preview_content else {
            return;
        };

        let Some(raw_lines) = get_raw_lines(content) else {
            return;
        };

        if raw_lines.is_empty() {
            return;
        }

        let last_line = raw_lines.len() - 1;
        let last_col = raw_lines.last().map_or(0, String::len);

        self.selection = Some(TextSelection {
            anchor: crate::app::selection::TextPosition::new(0, 0),
            cursor: crate::app::selection::TextPosition::new(last_line, last_col),
            active: false,
        });
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
