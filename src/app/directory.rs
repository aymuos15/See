use crate::app::content::PreviewContentType;
use crate::constants::HIGHLIGHT_CACHE_ENTRIES;
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

    /// Returns cached highlighted lines when we have them, otherwise plain
    /// lines now plus a background request to highlight the file.
    fn highlighted_or_plain(
        &mut self,
        path: &std::path::Path,
        raw_lines: &[String],
    ) -> Vec<ratatui::text::Line<'static>> {
        if let Some(cached) = self.highlight_cache.get(path) {
            return cached.as_ref().clone();
        }

        if self.highlight_pending.insert(path.to_path_buf()) {
            self.worker.request_highlight(path, raw_lines.join("\n"));
        }

        raw_lines
            .iter()
            .map(|line| ratatui::text::Line::from(line.clone()))
            .collect()
    }

    /// Stores freshly highlighted lines and shows them if that file is still
    /// the one on screen.
    pub(super) fn apply_highlighted(
        &mut self,
        path: &std::path::Path,
        lines: Vec<ratatui::text::Line<'static>>,
    ) {
        self.highlight_pending.remove(path);

        // Bound the cache so browsing a large tree cannot grow it without limit.
        if self.highlight_cache.len() >= HIGHLIGHT_CACHE_ENTRIES {
            self.highlight_cache.clear();
        }
        let lines = std::rc::Rc::new(lines);
        self.highlight_cache
            .insert(path.to_path_buf(), std::rc::Rc::clone(&lines));

        self.replace_preview_lines(path, &lines);
    }

    /// Swaps highlighted lines into the main preview and any pane showing the
    /// same file, keeping each one's raw lines and scroll position.
    fn replace_preview_lines(
        &mut self,
        path: &std::path::Path,
        lines: &std::rc::Rc<Vec<ratatui::text::Line<'static>>>,
    ) {
        let selected_path = self
            .file_list_state
            .selected()
            .and_then(|idx| self.files.get(idx))
            .map(|entry| entry.path.clone());

        if selected_path.as_deref() == Some(path) {
            if let Some(content) = &self.shared_preview_content {
                if let PreviewContentType::Text { raw_lines, .. } = content.as_ref() {
                    self.shared_preview_content = Some(std::rc::Rc::new(PreviewContentType::text(
                        lines.as_ref().clone(),
                        raw_lines.clone(),
                    )));
                }
            }
        }

        if let Some(layout) = &mut self.split_layout {
            for pane in &mut layout.panes {
                if pane.file_path.as_deref() != Some(path) {
                    continue;
                }
                if let Some(content) = &pane.preview_content {
                    if let PreviewContentType::Text { raw_lines, .. } = content.as_ref() {
                        pane.preview_content = Some(std::rc::Rc::new(PreviewContentType::text(
                            lines.as_ref().clone(),
                            raw_lines.clone(),
                        )));
                    }
                }
            }
        }
    }

    pub(super) fn load_preview(&mut self) {
        // Clear any previous PDF error
        self.pdf_error = None;

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
                // Highlighting a large file costs tens of milliseconds, which is
                // far too slow to do between keystrokes. Show the text straight
                // away and let the worker colour it in.
                let lines = self.highlighted_or_plain(&entry_path, raw_lines);
                PreviewContentType::text(lines, raw_lines.clone())
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
            PreviewContentType::Pdf { path, .. } => {
                // Start (or keep) the continuous view, which pulls in pages as
                // they scroll into sight.
                self.begin_pdf_view(path);

                PreviewContentType::Pdf { path: path.clone() }
            }
        };

        if !matches!(content, PreviewContentType::Pdf { .. }) {
            self.pdf_view = None;
        }

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

    /// Path of the file the preview is showing, if it still exists.
    fn watched_preview_path(&self) -> Option<std::path::PathBuf> {
        let entry = self.files.get(self.file_list_state.selected()?)?;
        entry.is_file.then(|| entry.path.clone())
    }

    /// Refresh the preview content for the currently viewed file
    /// Preserves selection if content hasn't changed
    pub(super) fn refresh_preview(&mut self) {
        // A rebuilt file is usually a new file in the same place, and the watch
        // followed the old one, so re-arm it before doing anything else.
        if let Some(path) = self.watched_preview_path() {
            let _ = self.file_watcher.watch_preview_file(Some(&path));
        }

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
                                        PreviewContentType::Image { .. }
                                        | PreviewContentType::Pdf { .. } => true,
                                    });

                                if content_changed {
                                    let path = entry.path.clone();
                                    self.highlight_cache.remove(&path);
                                    self.highlight_pending.remove(&path);
                                    let lines = self.highlighted_or_plain(&path, &raw_lines);
                                    self.shared_preview_content =
                                        Some(Rc::new(PreviewContentType::text(lines, raw_lines)));
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
                            PreviewContentType::Pdf { path, .. } => {
                                if self.shared_preview_content.as_ref().is_none_or(|prev| {
                                    !matches!(prev.as_ref(), PreviewContentType::Pdf { .. })
                                }) {
                                    // Changed from non-PDF to PDF, start viewing it
                                    self.begin_pdf_view(&path);
                                    self.shared_preview_content =
                                        Some(Rc::new(PreviewContentType::Pdf { path }));
                                } else {
                                    // Same PDF: re-renders only if the file was
                                    // rebuilt, keeping the scroll position.
                                    self.begin_pdf_view(&path);
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
