mod diff;
mod directory;
mod event_handler;
mod git;
mod help;
mod mouse;
mod navigation;
mod search;
pub mod selection;
mod symbol_search;
mod theme_search;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::constants::INITIAL_SPLIT_PERCENT;
use crate::event::{FileWatcher, RefreshTimer};
use crate::files::{read_directory, FileEntry, Symbol};
use crate::git::GitStatus;
use crate::highlight::SyntaxHighlighter;
use crate::theme::Theme;
use ratatui::prelude::Rect;
use ratatui::text::Line;
use ratatui::widgets::ListState;
use std::path::{Path, PathBuf};

use selection::TextSelection;

/// Content displayed in the preview pane with syntax highlighting.
#[derive(Clone)]
pub struct PreviewContent {
    /// Syntax-highlighted lines for rendering.
    pub lines: Vec<Line<'static>>,
    /// Raw text lines without highlighting (for copying).
    pub raw_lines: Vec<String>,
}

/// Main application state for the TUI file viewer.
#[allow(clippy::struct_excessive_bools)]
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
    pub config: Config,
    // Search mode state
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_selected: usize,
    // All files under root for searching
    search_index: Vec<FileEntry>,
    // Symbol search mode state
    pub symbol_search_mode: bool,
    pub symbol_search_query: String,
    pub symbol_index: Vec<Symbol>,
    pub symbol_search_results: Vec<usize>,
    pub symbol_search_selected: usize,
    // File watching
    file_watcher: FileWatcher,
    search_index_timer: RefreshTimer,
    // Text selection
    pub selection: Option<TextSelection>,
    pub last_preview_area: Option<Rect>,
    pub last_file_list_area: Option<Rect>,
    clipboard: ClipboardManager,
    // Git highlighting
    git_highlight_enabled: bool,
    git_status: Option<GitStatus>,
    // Diff mode state
    diff_mode: bool,
    original_preview_content: Option<PreviewContent>,
    // Theme state
    pub current_theme_name: String,
    pub available_themes: Vec<String>,
    pub theme_preview_mode: bool,
    // Help mode state
    pub help_mode: bool,
}

impl App {
    /// Returns the root directory being browsed.
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Returns the index of all files for searching.
    pub fn search_index(&self) -> &[FileEntry] {
        &self.search_index
    }

    /// Creates a new App instance for the given path.
    ///
    /// # Errors
    /// Returns an error if the path doesn't exist or is inaccessible.
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

        // Load config
        let config = Config::load();

        // Read directory contents
        let files = read_directory(&current_dir, &root_dir, &config)?;
        let highlighter = SyntaxHighlighter::new();

        // Initialize file watcher
        let file_watcher = FileWatcher::new(&current_dir)?;
        let search_index_timer = RefreshTimer::new();

        let mut app = Self {
            root_dir,
            current_dir,
            files,
            file_list_state: ListState::default(),
            preview_content: None,
            preview_scroll: 0,
            highlighter,
            should_quit: false,
            split_percent: INITIAL_SPLIT_PERCENT,
            config,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_index: Vec::new(),
            symbol_search_mode: false,
            symbol_search_query: String::new(),
            symbol_index: Vec::new(),
            symbol_search_results: Vec::new(),
            symbol_search_selected: 0,
            file_watcher,
            search_index_timer,
            selection: None,
            last_preview_area: None,
            last_file_list_area: None,
            clipboard: ClipboardManager::new(),
            git_highlight_enabled: false,
            git_status: None,
            diff_mode: false,
            original_preview_content: None,
            current_theme_name: "jellybeans".to_string(),
            available_themes: Theme::list_builtins()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            theme_preview_mode: false,
            help_mode: false,
        };

        if !app.files.is_empty() {
            app.file_list_state.select(Some(0));
            app.load_preview();
        }

        Ok(app)
    }
}

// Theme management
impl App {
    /// Switch to a built-in theme by name
    pub fn switch_theme(&mut self, name: &str) -> anyhow::Result<()> {
        if let Some(new_theme) = Theme::by_name(name) {
            self.config.theme = new_theme;
            self.current_theme_name = name.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme not found: {name}"))
        }
    }

    /// Toggle theme preview mode (shows theme picker popup)
    #[allow(clippy::missing_const_for_fn)]
    pub fn toggle_theme_preview(&mut self) {
        self.theme_preview_mode = !self.theme_preview_mode;
    }

    /// Cycle to the next available theme
    #[allow(dead_code)]
    pub fn cycle_theme(&mut self) {
        if self.available_themes.is_empty() {
            return;
        }

        let current_idx = self
            .available_themes
            .iter()
            .position(|t| t == &self.current_theme_name)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.available_themes.len();
        let next_theme = self.available_themes[next_idx].clone();
        let _ = self.switch_theme(&next_theme);
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

    #[test]
    fn test_search_mode_entry_exit() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        assert!(!app.search_mode);
        assert!(app.search_query.is_empty());

        app.enter_search_mode();
        assert!(app.search_mode);

        app.exit_search_mode();
        assert!(!app.search_mode);
    }

    #[test]
    fn test_search_input() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        app.enter_search_mode();
        app.search_input('f');
        app.search_input('i');
        app.search_input('l');
        app.search_input('e');

        assert_eq!(app.search_query, "file");
    }

    #[test]
    fn test_search_backspace() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        app.enter_search_mode();
        app.search_input('t');
        app.search_input('e');
        app.search_input('s');
        app.search_input('t');

        assert_eq!(app.search_query, "test");

        app.search_backspace();
        assert_eq!(app.search_query, "tes");

        app.search_backspace();
        assert_eq!(app.search_query, "te");
    }

    #[test]
    fn test_fuzzy_filter() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        // Build search index
        app.search_index = vec![
            FileEntry::new(temp_dir.path().join("root/file1.txt")),
            FileEntry::new(temp_dir.path().join("root/file2.txt")),
            FileEntry::new(temp_dir.path().join("root/config.toml")),
        ];

        app.search_query = "file".to_string();
        app.apply_fuzzy_filter();

        // Should match both file entries
        assert!(app.search_results.len() >= 2);
    }

    #[test]
    fn test_fuzzy_filter_no_matches() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        app.search_index = vec![
            FileEntry::new(temp_dir.path().join("root/file1.txt")),
            FileEntry::new(temp_dir.path().join("root/file2.txt")),
        ];

        app.search_query = "xyz123".to_string();
        app.apply_fuzzy_filter();

        // Should have no results
        assert!(app.search_results.is_empty());
    }

    #[test]
    fn test_navigate_up_wraps_around() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        // Start at first item
        app.file_list_state.select(Some(0));

        // Navigate up should wrap to last
        app.navigate_up();
        let selected = app.file_list_state.selected().unwrap_or(0);
        assert_eq!(selected, app.files.len() - 1);
    }

    #[test]
    fn test_navigate_down_wraps_around() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        // Set to last item
        let last_idx = app.files.len() - 1;
        app.file_list_state.select(Some(last_idx));

        // Navigate down should wrap to first
        app.navigate_down();
        let selected = app.file_list_state.selected().unwrap_or(0);
        assert_eq!(selected, 0);
    }

    #[test]
    fn test_load_preview_for_file() {
        let temp_dir = create_test_dir_structure().unwrap();
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path).unwrap();

        // Select first file
        if let Some(first) = app.files.first() {
            if first.is_file {
                app.file_list_state.select(Some(0));
                app.load_preview();

                // Should have loaded preview content
                assert!(app.preview_content.is_some());
                if let Some(content) = &app.preview_content {
                    assert!(!content.lines.is_empty());
                }
            }
        }
    }
}
