use crate::app::content::PreviewContentType;
use crate::files::read_directory;
use std::rc::Rc;

use super::App;

impl App {
    pub(super) fn enter_directory(&mut self) {
        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if !entry.is_file {
                    if let Ok(files) = read_directory(&entry.path, &self.root_dir, &self.config) {
                        self.current_dir = entry.path.clone();
                        self.files = files;
                        self.file_list_state.select(Some(0));
                        self.preview_scroll = 0;
                        self.load_preview();
                        // Update watcher for new directory
                        let _ = self.file_watcher.watch_directory(&self.current_dir);
                    }
                }
            }
        }
    }

    pub(super) fn go_back(&mut self) {
        // Check if we're already at the root boundary
        if self.current_dir == self.root_dir {
            // Silent ignore: already at root, cannot go back further
            return;
        }

        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();

            // Ensure parent is within or equal to root_dir
            if parent_path.starts_with(&self.root_dir) {
                if let Ok(files) = read_directory(&parent_path, &self.root_dir, &self.config) {
                    self.current_dir = parent_path;
                    self.files = files;
                    self.file_list_state.select(Some(0));
                    self.preview_scroll = 0;
                    self.load_preview();
                    // Update watcher for new directory
                    let _ = self.file_watcher.watch_directory(&self.current_dir);
                }
            }
            // else: Silent ignore, parent is outside root boundary
        }
    }

    pub(super) fn load_preview(&mut self) {
        // Exit diff mode when loading new file
        self.diff_mode = false;
        self.original_preview_content = None;

        // Extract entry info before mutable borrows
        let entry_info = self.file_list_state.selected().and_then(|idx| {
            self.files.get(idx).and_then(|entry| {
                if entry.is_file {
                    Some(entry.path.clone())
                } else {
                    None
                }
            })
        });

        let Some(entry_path) = entry_info else {
            // No valid preview
            let _ = self.file_watcher.watch_preview_file(None);
            self.shared_preview_content = None;
            return;
        };

        let Ok(content) = crate::files::loader::load_preview_content(&entry_path) else {
            let _ = self.file_watcher.watch_preview_file(None);
            self.shared_preview_content = None;
            return;
        };

        // Handle text content with syntax highlighting
        let content = match &content {
            PreviewContentType::Text { raw_lines, .. } => {
                let text = raw_lines.join("\n");
                let lines = self.highlighter.highlight(&entry_path, &text);
                PreviewContentType::Text {
                    lines,
                    raw_lines: raw_lines.clone(),
                }
            }
            PreviewContentType::Image { path, dimensions } => {
                // Cancel any pending full quality load from previous image
                self.cancel_pending_full_quality();

                // Check if we already have full quality cached
                let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                if self.full_quality_images.contains(&canonical) {
                    // Already have full quality, no need to load
                } else if self.image_protocols.contains_key(&canonical) {
                    // Have thumbnail cached, schedule full quality
                    self.schedule_full_quality_load(&canonical);
                } else {
                    // No cache, request thumbnail (fast) first
                    self.worker.request_thumbnail_load(path);
                }

                PreviewContentType::Image {
                    path: path.clone(),
                    dimensions: *dimensions,
                }
            }
        };

        // Create shared reference for panes (Rc::clone is O(1))
        let shared = Rc::new(content);
        self.shared_preview_content = Some(shared);
        // Watch this file for changes
        let _ = self.file_watcher.watch_preview_file(Some(&entry_path));
        // Clear selection when loading new file
        self.selection = None;
        // Keep highlighted_word for cross-file search persistence
    }

    /// Refresh the file list for the current directory
    pub(super) fn refresh_current_directory(&mut self) {
        if let Ok(files) = read_directory(&self.current_dir, &self.root_dir, &self.config) {
            // Preserve selection if possible
            let selected_path = self
                .file_list_state
                .selected()
                .and_then(|idx| self.files.get(idx))
                .map(|e| e.path.clone());

            self.files = files;

            // Try to re-select the same file
            if let Some(path) = selected_path {
                if let Some(new_idx) = self.files.iter().position(|f| f.path == path) {
                    self.file_list_state.select(Some(new_idx));
                } else if !self.files.is_empty() {
                    // File was deleted, select first item
                    self.file_list_state.select(Some(0));
                } else {
                    self.file_list_state.select(None);
                }
            } else if !self.files.is_empty() {
                self.file_list_state.select(Some(0));
            }

            // Refresh preview for current selection
            self.load_preview();
        }
    }

    /// Refresh the preview content for the currently viewed file
    /// Preserves selection if content hasn't changed
    pub(super) fn refresh_preview(&mut self) {
        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if entry.is_file {
                    // Use load_preview_content to handle both images and text
                    if let Ok(content) = crate::files::loader::load_preview_content(&entry.path) {
                        match content {
                            PreviewContentType::Text { raw_lines, .. } => {
                                // Check if content actually changed
                                let content_changed = self
                                    .shared_preview_content
                                    .as_ref()
                                    .is_none_or(|prev| match prev.as_ref() {
                                        PreviewContentType::Text {
                                            raw_lines: prev_lines,
                                            ..
                                        } => prev_lines != &raw_lines,
                                        PreviewContentType::Image { .. } => true,
                                    });

                                if content_changed {
                                    let lines = self
                                        .highlighter
                                        .highlight(&entry.path, &raw_lines.join("\n"));
                                    self.shared_preview_content =
                                        Some(Rc::new(PreviewContentType::Text {
                                            lines,
                                            raw_lines,
                                        }));
                                    // Only clear selection if content changed
                                    self.selection = None;
                                    // Keep highlighted_word for cross-file search persistence
                                }
                            }
                            PreviewContentType::Image { path, dimensions } => {
                                // For images, just keep the existing preview
                                // Don't reload unless the file was actually modified
                                // (size/mtime check would be needed for proper detection)
                                if self.shared_preview_content.as_ref().is_none_or(|prev| {
                                    !matches!(prev.as_ref(), PreviewContentType::Image { .. })
                                }) {
                                    // Changed from non-image to image, reload
                                    self.worker.request_image_load(&path);
                                    self.shared_preview_content =
                                        Some(Rc::new(PreviewContentType::Image {
                                            path,
                                            dimensions,
                                        }));
                                }
                            }
                        }
                        return;
                    }
                }
            }
        }
        // No valid preview
        self.shared_preview_content = None;
        self.selection = None;
        // Keep highlighted_word for cross-file search persistence
    }
}
