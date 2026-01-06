use crate::files::{find_all_files_recursive, read_directory};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

use super::App;

impl App {
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_selected = 0;

        // Build search index on first entry
        if self.search_index.is_empty() {
            if let Ok(all_files) = find_all_files_recursive(&self.root_dir, &self.config) {
                self.search_index = all_files;
            }
        }

        self.apply_fuzzy_filter();
    }

    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_results.clear();
        self.search_selected = 0;
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.search_selected = 0;
        self.apply_fuzzy_filter();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.search_selected = 0;
        self.apply_fuzzy_filter();
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn search_navigate_up(&mut self) {
        if !self.search_results.is_empty() {
            self.search_selected = if self.search_selected == 0 {
                self.search_results.len() - 1
            } else {
                self.search_selected - 1
            };
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn search_navigate_down(&mut self) {
        if !self.search_results.is_empty() {
            self.search_selected = (self.search_selected + 1) % self.search_results.len();
        }
    }

    pub fn search_confirm(&mut self) {
        if !self.search_results.is_empty() {
            let file_idx = self.search_results[self.search_selected];
            if let Some(entry) = self.search_index.get(file_idx) {
                let target_dir = if entry.is_file {
                    entry.path.parent().unwrap_or(&self.root_dir).to_path_buf()
                } else {
                    entry.path.clone()
                };

                // Navigate to the target directory
                if let Ok(files) = read_directory(&target_dir, &self.root_dir, &self.config) {
                    self.current_dir = target_dir;
                    self.files = files;

                    // Select the file/directory in the new listing
                    if let Some(pos) = self.files.iter().position(|f| f.path == entry.path) {
                        self.file_list_state.select(Some(pos));
                    } else {
                        self.file_list_state.select(Some(0));
                    }

                    self.preview_scroll = 0;
                    self.load_preview();
                    // Update watcher for new directory
                    let _ = self.file_watcher.watch_directory(&self.current_dir);
                }
            }
        }
        self.exit_search_mode();
    }

    pub(super) fn apply_fuzzy_filter(&mut self) {
        if self.search_query.is_empty() {
            self.search_results = (0..self.search_index.len()).collect();
            return;
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(
            &self.search_query,
            CaseMatching::Ignore,
            Normalization::Smart,
        );

        let names: Vec<&str> = self.search_index.iter().map(|f| f.name.as_str()).collect();
        let matched_names = pattern.match_list(&names, &mut matcher);

        // matched_names is Vec<(&str, u32)> sorted by score descending
        // We need to map back to indices
        self.search_results = matched_names
            .into_iter()
            .filter_map(|(name, _score)| self.search_index.iter().position(|f| f.name == *name))
            .collect();
    }

    /// Refresh the search index with all files under root
    pub(super) fn refresh_search_index(&mut self) {
        if let Ok(all_files) = find_all_files_recursive(&self.root_dir, &self.config) {
            self.search_index = all_files;

            // If currently in search mode, re-apply filter
            if self.search_mode {
                self.apply_fuzzy_filter();
            }
        }
    }
}
