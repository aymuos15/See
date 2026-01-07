use crate::files::{read_directory, read_file_content};

use super::{App, PreviewContent};

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

        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if entry.is_file {
                    if let Ok(content) = read_file_content(&entry.path) {
                        let lines = self.highlighter.highlight(&entry.path, &content);
                        let raw_lines: Vec<String> = content.lines().map(String::from).collect();
                        self.preview_content = Some(PreviewContent { lines, raw_lines });
                        // Watch this file for changes
                        let _ = self.file_watcher.watch_preview_file(Some(&entry.path));
                        // Clear selection when loading new file
                        self.selection = None;
                        return;
                    }
                }
            }
        }
        // No preview, stop watching preview file
        let _ = self.file_watcher.watch_preview_file(None);
        self.preview_content = None;
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
                    if let Ok(content) = read_file_content(&entry.path) {
                        let raw_lines: Vec<String> = content.lines().map(String::from).collect();

                        // Check if content actually changed
                        let content_changed = self
                            .preview_content
                            .as_ref()
                            .is_none_or(|prev| prev.raw_lines != raw_lines);

                        if content_changed {
                            let lines = self.highlighter.highlight(&entry.path, &content);
                            self.preview_content = Some(PreviewContent { lines, raw_lines });
                            // Only clear selection if content changed
                            self.selection = None;
                        }
                        return;
                    }
                }
            }
        }
        // No valid preview
        self.preview_content = None;
        self.selection = None;
    }
}
