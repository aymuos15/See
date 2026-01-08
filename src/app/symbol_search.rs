use crate::files::{extract_symbols, find_all_files_recursive};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use std::collections::HashMap;

use super::App;

impl App {
    pub fn enter_symbol_search_mode(&mut self) {
        self.symbol_search_mode = true;
        self.symbol_search_query.clear();
        self.symbol_search_selected = 0;

        // Build symbol index on first entry
        if self.symbol_index.is_empty() {
            if let Ok(all_files) = find_all_files_recursive(&self.root_dir, &self.config) {
                let mut symbols = Vec::new();
                for file_entry in all_files {
                    if file_entry.is_file {
                        if let Ok(content) = std::fs::read_to_string(&file_entry.path) {
                            let file_symbols = extract_symbols(&file_entry.path, &content);
                            symbols.extend(file_symbols);
                        }
                    }
                }
                self.symbol_index = symbols;
            }
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
            self.symbol_search_selected = if self.symbol_search_selected == 0 {
                self.symbol_search_results.len() - 1
            } else {
                self.symbol_search_selected - 1
            };
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn symbol_search_navigate_down(&mut self) {
        if !self.symbol_search_results.is_empty() {
            self.symbol_search_selected =
                (self.symbol_search_selected + 1) % self.symbol_search_results.len();
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

                    // Scroll to the symbol's line in the preview
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
            if line < content.lines.len() {
                self.preview_scroll = u16::try_from(line).unwrap_or(u16::MAX);
            }
        }
    }

    pub(super) fn apply_symbol_filter(&mut self) {
        if self.symbol_search_query.is_empty() {
            self.symbol_search_results = (0..self.symbol_index.len()).collect();
            return;
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(
            &self.symbol_search_query,
            CaseMatching::Ignore,
            Normalization::Smart,
        );

        let names: Vec<&str> = self.symbol_index.iter().map(|s| s.name.as_str()).collect();
        let matched_names = pattern.match_list(&names, &mut matcher);

        // Build index map for O(1) lookup
        let name_to_idx: HashMap<&str, usize> = self
            .symbol_index
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name.as_str(), i))
            .collect();

        // matched_names is Vec<(&str, u32)> sorted by score descending
        self.symbol_search_results = matched_names
            .into_iter()
            .filter_map(|(name, _score)| name_to_idx.get(name).copied())
            .collect();
    }
}
