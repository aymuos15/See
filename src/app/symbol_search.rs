use crate::util::fuzzy;

use super::App;

impl App {
    pub fn enter_symbol_search_mode(&mut self) {
        self.symbol_search_mode = true;
        self.symbol_search_query.clear();
        self.symbol_search_selected = 0;

        // Request background indexing if not already indexed
        if self.symbol_index.is_empty() && self.symbol_indexing_progress.is_none() {
            self.symbol_indexing_progress = Some((0, 0));
            self.worker
                .request_symbol_indexing(&self.root_dir, self.config.clone());
        }

        self.apply_symbol_filter();
    }

    pub fn exit_symbol_search_mode(&mut self) {
        self.symbol_search_mode = false;
        self.symbol_search_query.clear();
        self.symbol_search_results.clear();
        self.symbol_search_selected = 0;
    }

    pub fn symbol_search_input(&mut self, c: char) {
        self.symbol_search_query.push(c);
        self.symbol_search_selected = 0;
        self.apply_symbol_filter();
    }

    pub fn symbol_search_backspace(&mut self) {
        self.symbol_search_query.pop();
        self.symbol_search_selected = 0;
        self.apply_symbol_filter();
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn symbol_search_navigate_up(&mut self) {
        if !self.symbol_search_results.is_empty() {
            self.symbol_search_selected = fuzzy::move_selection(
                self.symbol_search_selected,
                self.symbol_search_results.len(),
                -1,
            );
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn symbol_search_navigate_down(&mut self) {
        if !self.symbol_search_results.is_empty() {
            self.symbol_search_selected = fuzzy::move_selection(
                self.symbol_search_selected,
                self.symbol_search_results.len(),
                1,
            );
        }
    }

    pub fn symbol_search_confirm(&mut self) {
        if !self.symbol_search_results.is_empty() {
            let symbol_idx = self.symbol_search_results[self.symbol_search_selected];
            if let Some(symbol) = self.symbol_index.get(symbol_idx).cloned() {
                // Navigate to the file containing the symbol
                let target_dir = symbol.file.parent().unwrap_or(&self.root_dir).to_path_buf();

                let symbol_line = symbol.line;
                let symbol_file = symbol.file;

                if let Ok(files) =
                    crate::files::read_directory(&target_dir, &self.root_dir, &self.config)
                {
                    self.current_dir = target_dir;
                    self.files = files;

                    // Select the file in the new listing
                    if let Some(pos) = self.files.iter().position(|f| f.path == symbol_file) {
                        self.file_list_state.select(Some(pos));
                    } else {
                        self.file_list_state.select(Some(0));
                    }

                    self.preview_scroll = 0;
                    self.load_preview();

                    // Scroll to the symbol's line in the preview (only for text content)
                    self.jump_to_line(symbol_line);

                    // Update watcher for new directory
                    let _ = self.file_watcher.watch_directory(&self.current_dir);
                }
            }
        }
        self.exit_symbol_search_mode();
    }

    pub fn jump_to_line(&mut self, line: usize) {
        if let Some(content) = &self.shared_preview_content {
            // Only jump to line for text content, not images
            if let crate::app::content::PreviewContentType::Text { lines, .. } = content.as_ref() {
                if line < lines.len() {
                    self.preview_scroll = u16::try_from(line).unwrap_or(u16::MAX);
                }
            }
        }
    }

    pub(super) fn apply_symbol_filter(&mut self) {
        self.symbol_search_results =
            fuzzy::fuzzy_filter_indices(&self.symbol_search_query, &self.symbol_index, |s| &s.name);
    }
}
