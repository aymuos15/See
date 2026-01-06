use crate::event::{poll_event, AppEvent};
use crate::files::{read_directory, read_file_content, find_all_files_recursive, FileEntry};
use crate::highlight::SyntaxHighlighter;
use crate::theme::Theme;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::text::Line;
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::time::Duration;

pub struct PreviewContent {
    pub lines: Vec<Line<'static>>,
}

pub struct App {
    root_dir: PathBuf,
    pub current_dir: PathBuf,
    pub files: Vec<FileEntry>,
    pub file_list_state: ListState,
    pub preview_content: Option<PreviewContent>,
    pub preview_scroll: u16,
    pub highlighter: SyntaxHighlighter,
    pub should_quit: bool,
    pub split_percent: u16,
    pub theme: Theme,
    // Search mode state
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_selected: usize,
    // All files under root for searching
    search_index: Vec<FileEntry>,
}

impl App {
    pub fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }

    pub fn search_index(&self) -> &Vec<FileEntry> {
        &self.search_index
    }

    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        // Validate the path exists
        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path.display());
        }

        // Determine the initial directory
        let initial_dir = if path.is_dir() {
            path
        } else {
            path.parent()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine parent directory"))?
                .to_path_buf()
        };

        // Canonicalize to get absolute, resolved path (symlinks resolved)
        let root_dir = initial_dir
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("Cannot access directory: {e}"))?;

        // Use the canonicalized path as current_dir
        let current_dir = root_dir.clone();

        // Read directory contents
        let files = read_directory(&current_dir)?;
        let highlighter = SyntaxHighlighter::new();

        let mut app = Self {
            root_dir,
            current_dir,
            files,
            file_list_state: ListState::default(),
            preview_content: None,
            preview_scroll: 0,
            highlighter,
            should_quit: false,
            split_percent: 30,
            theme: Theme::load(),
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_index: Vec::new(),
        };

        if !app.files.is_empty() {
            app.file_list_state.select(Some(0));
            app.load_preview();
        }

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut crate::tui::Tui) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::render(frame, self))?;

            match poll_event(Duration::from_millis(16), self.search_mode)? {
                AppEvent::Quit => {
                    if self.search_mode {
                        self.exit_search_mode();
                    } else {
                        self.should_quit = true;
                    }
                }
                AppEvent::OpenSearch => self.enter_search_mode(),
                AppEvent::CloseSearch => self.exit_search_mode(),
                AppEvent::SearchInput(c) => self.search_input(c),
                AppEvent::SearchBackspace => self.search_backspace(),
                AppEvent::SearchNavigateUp => self.search_navigate_up(),
                AppEvent::SearchNavigateDown => self.search_navigate_down(),
                AppEvent::SearchConfirm => self.search_confirm(),
                AppEvent::NavigateDown => {
                    if !self.search_mode {
                        self.navigate_down();
                    }
                }
                AppEvent::NavigateUp => {
                    if !self.search_mode {
                        self.navigate_up();
                    }
                }
                AppEvent::ScrollPreviewDown => {
                    if !self.search_mode {
                        self.scroll_preview_down();
                    }
                }
                AppEvent::ScrollPreviewUp => {
                    if !self.search_mode {
                        self.scroll_preview_up();
                    }
                }
                AppEvent::ScrollPreviewPageDown => {
                    if !self.search_mode {
                        self.scroll_preview_page_down();
                    }
                }
                AppEvent::ScrollPreviewPageUp => {
                    if !self.search_mode {
                        self.scroll_preview_page_up();
                    }
                }
                AppEvent::ShrinkFileList => {
                    if !self.search_mode {
                        self.shrink_file_list();
                    }
                }
                AppEvent::GrowFileList => {
                    if !self.search_mode {
                        self.grow_file_list();
                    }
                }
                AppEvent::Enter => {
                    if !self.search_mode {
                        self.enter_directory();
                    }
                }
                AppEvent::GoBack => {
                    if !self.search_mode {
                        self.go_back();
                    }
                }
                AppEvent::None => {}
            }
        }

        Ok(())
    }

    fn navigate_down(&mut self) {
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

    fn navigate_up(&mut self) {
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

    fn scroll_preview_down(&mut self) {
        if let Some(preview) = &self.preview_content {
            if !preview.lines.is_empty() {
                self.preview_scroll = (self.preview_scroll + 1)
                    .min(u16::try_from(preview.lines.len() - 1).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    fn scroll_preview_page_down(&mut self) {
        if let Some(preview) = &self.preview_content {
            if !preview.lines.is_empty() {
                self.preview_scroll = (self.preview_scroll + 10)
                    .min(u16::try_from(preview.lines.len() - 1).unwrap_or(u16::MAX));
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn scroll_preview_page_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(10);
    }

    fn shrink_file_list(&mut self) {
        self.split_percent = self.split_percent.saturating_sub(5).max(10);
    }

    fn grow_file_list(&mut self) {
        self.split_percent = (self.split_percent + 5).min(80);
    }

    fn enter_directory(&mut self) {
        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if !entry.is_file {
                    if let Ok(files) = read_directory(&entry.path) {
                        self.current_dir = entry.path.clone();
                        self.files = files;
                        self.file_list_state.select(Some(0));
                        self.preview_scroll = 0;
                        self.load_preview();
                    }
                }
            }
        }
    }

    fn go_back(&mut self) {
        // Check if we're already at the root boundary
        if self.current_dir == self.root_dir {
            // Silent ignore: already at root, cannot go back further
            return;
        }

        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();

            // Ensure parent is within or equal to root_dir
            if parent_path.starts_with(&self.root_dir) {
                if let Ok(files) = read_directory(&parent_path) {
                    self.current_dir = parent_path;
                    self.files = files;
                    self.file_list_state.select(Some(0));
                    self.preview_scroll = 0;
                    self.load_preview();
                }
            }
            // else: Silent ignore, parent is outside root boundary
        }
    }

    fn load_preview(&mut self) {
        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if entry.is_file {
                    if let Ok(content) = read_file_content(&entry.path) {
                        let lines = self.highlighter.highlight(&entry.path, &content);
                        self.preview_content = Some(PreviewContent { lines });
                        return;
                    }
                }
            }
        }
        self.preview_content = None;
    }

    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_selected = 0;
        
        // Build search index on first entry
        if self.search_index.is_empty() {
            if let Ok(all_files) = find_all_files_recursive(&self.root_dir) {
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
                if let Ok(files) = read_directory(&target_dir) {
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
                }
            }
        }
        self.exit_search_mode();
    }

    fn apply_fuzzy_filter(&mut self) {
        if self.search_query.is_empty() {
            self.search_results = (0..self.search_index.len()).collect();
            return;
        }

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(usize, i64)> = self
            .search_index
            .iter()
            .enumerate()
            .filter_map(|(idx, file)| {
                matcher
                    .fuzzy_match(&file.name, &self.search_query)
                    .map(|score| (idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        self.search_results = scored.into_iter().map(|(idx, _)| idx).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a test directory structure
    fn create_test_dir_structure() -> anyhow::Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let base = temp_dir.path();

        // Create directory structure:
        // temp_dir/
        //   ├── root/
        //   │   ├── file1.txt
        //   │   ├── subdir1/
        //   │   │   ├── file2.txt
        //   │   │   └── subdir2/
        //   │   │       └── file3.txt
        //   │   └── subdir3/
        //   │       └── file4.txt

        fs::create_dir(base.join("root"))?;
        fs::write(base.join("root/file1.txt"), "content1")?;

        fs::create_dir(base.join("root/subdir1"))?;
        fs::write(base.join("root/subdir1/file2.txt"), "content2")?;

        fs::create_dir(base.join("root/subdir1/subdir2"))?;
        fs::write(base.join("root/subdir1/subdir2/file3.txt"), "content3")?;

        fs::create_dir(base.join("root/subdir3"))?;
        fs::write(base.join("root/subdir3/file4.txt"), "content4")?;

        Ok(temp_dir)
    }

    #[test]
    fn test_app_new_with_valid_directory() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");

        let app = App::new(root_path.clone()).unwrap();

        assert_eq!(
            app.root_dir,
            root_path.canonicalize().unwrap(),
            "Root dir should be canonicalized path"
        );
        assert_eq!(
            app.current_dir,
            root_path.canonicalize().unwrap(),
            "Current dir should start at root"
        );
        assert!(!app.files.is_empty(), "Files should be loaded");
    }

    #[test]
    fn test_app_new_with_nonexistent_path() {
        let result = App::new(PathBuf::from("/nonexistent/path/that/does/not/exist"));

        assert!(result.is_err(), "Should fail with nonexistent path");
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("Path does not exist"),
            "Error message should mention path doesn't exist"
        );
    }

    #[test]
    fn test_app_new_with_file_path() {
        let temp_dir = create_test_dir_structure().unwrap();
        let file_path = temp_dir.path().join("root/file1.txt");

        let app = App::new(file_path).unwrap();

        // Should use parent directory (root) as the root_dir
        let expected_root = temp_dir.path().join("root").canonicalize().unwrap();
        assert_eq!(
            app.root_dir, expected_root,
            "Should use parent directory as root"
        );
        assert_eq!(
            app.current_dir, expected_root,
            "Should start at parent directory"
        );
    }

    #[test]
    fn test_go_back_at_root_boundary() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");

        let mut app = App::new(root_path.clone()).unwrap();
        let initial_dir = app.current_dir.clone();

        // Try to go back when already at root
        app.go_back();

        assert_eq!(
            app.current_dir, initial_dir,
            "Should stay at root when trying to go back"
        );
    }

    #[test]
    fn test_navigation_within_root_boundary() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");

        let mut app = App::new(root_path.clone()).unwrap();
        let root_canonical = root_path.canonicalize().unwrap();

        // Navigate into subdir1
        let subdir1_idx = app
            .files
            .iter()
            .position(|f| f.name == "subdir1")
            .expect("subdir1 should exist");

        app.file_list_state.select(Some(subdir1_idx));
        app.enter_directory();

        let expected_subdir1 = root_canonical.join("subdir1");
        assert_eq!(
            app.current_dir, expected_subdir1,
            "Should navigate into subdir1"
        );

        // Navigate into subdir2
        let subdir2_idx = app
            .files
            .iter()
            .position(|f| f.name == "subdir2")
            .expect("subdir2 should exist");

        app.file_list_state.select(Some(subdir2_idx));
        app.enter_directory();

        let expected_subdir2 = expected_subdir1.join("subdir2");
        assert_eq!(
            app.current_dir, expected_subdir2,
            "Should navigate into subdir2"
        );

        // Go back to subdir1
        app.go_back();
        assert_eq!(
            app.current_dir, expected_subdir1,
            "Should go back to subdir1"
        );

        // Go back to root
        app.go_back();
        assert_eq!(app.current_dir, root_canonical, "Should go back to root");

        // Try to go back past root - should stay at root
        app.go_back();
        assert_eq!(
            app.current_dir, root_canonical,
            "Should not go past root boundary"
        );
    }

    #[test]
    fn test_root_dir_immutable() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");

        let mut app = App::new(root_path.clone()).unwrap();
        let initial_root = app.root_dir.clone();

        // Navigate around
        let subdir1_idx = app
            .files
            .iter()
            .position(|f| f.name == "subdir1")
            .expect("subdir1 should exist");

        app.file_list_state.select(Some(subdir1_idx));
        app.enter_directory();

        // Root should never change
        assert_eq!(
            app.root_dir, initial_root,
            "Root dir should remain unchanged after navigation"
        );

        app.go_back();

        assert_eq!(
            app.root_dir, initial_root,
            "Root dir should remain unchanged after going back"
        );
    }

    #[test]
    fn test_relative_path_canonicalization() {
        // Create temp dir and navigate to it
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");

        // Get the canonical path first
        let canonical_root = root_path.canonicalize().unwrap();

        // Create app with absolute path
        let app = App::new(root_path).unwrap();

        assert_eq!(
            app.root_dir, canonical_root,
            "Should canonicalize paths to absolute"
        );
    }

    #[test]
    fn test_starting_from_nested_directory() {
        let temp_dir = create_test_dir_structure().unwrap();
        let nested_path = temp_dir.path().join("root/subdir1/subdir2");

        let mut app = App::new(nested_path.clone()).unwrap();
        let nested_canonical = nested_path.canonicalize().unwrap();

        // Root should be the nested directory, not the top-level root
        assert_eq!(
            app.root_dir, nested_canonical,
            "Root should be the specified nested directory"
        );

        // Try to go back - should not be able to
        app.go_back();
        assert_eq!(
            app.current_dir, nested_canonical,
            "Should not be able to navigate above the specified root"
        );
    }
}
