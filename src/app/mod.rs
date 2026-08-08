mod content;
pub use content::{PreviewContentType, SharedPreviewContent};
mod directory;
mod event_handler;
mod help;
mod image;
mod mouse;
mod navigation;
pub mod pdf;
mod search;
pub mod selection;
pub mod split;
mod symbol_search;
mod theme_search;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::constants::INITIAL_SPLIT_PERCENT;
use crate::event::{FileWatcher, RefreshTimer};
use crate::files::{read_directory, FileEntry, Symbol};
use crate::theme::Theme;
use crate::worker::BackgroundWorker;
use ratatui::prelude::Rect;
use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use selection::TextSelection;
use split::SplitLayout;

/// Main application state for the TUI file viewer.
#[allow(clippy::struct_excessive_bools)]
pub struct App {
    root_dir: PathBuf,
    pub current_dir: PathBuf,
    pub files: Vec<FileEntry>,
    pub file_list_state: ListState,
    /// Shared preview content (text or image) for efficient pane sharing.
    pub shared_preview_content: Option<SharedPreviewContent>,
    pub preview_scroll: u16,
    pub should_quit: bool,
    pub split_percent: u16,
    pub config: Config,
    // Search mode state
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_selected: usize,
    // Find mode state (in-pane search)
    pub find_mode: bool,
    pub find_query: String,
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
    pub highlighted_word: Option<String>,
    pub last_preview_area: Option<Rect>,
    pub last_pane_areas: Vec<(usize, Rect)>,
    pub last_file_list_area: Option<Rect>,
    clipboard: ClipboardManager,
    // Theme state
    pub current_theme_name: String,
    pub available_themes: Vec<String>,
    pub theme_preview_mode: bool,
    // Help mode state
    pub help_mode: bool,
    // Split layout
    pub split_layout: Option<SplitLayout>,
    /// Whether the file list pane is shown alongside the preview
    pub file_list_visible: bool,
    // Background worker
    worker: BackgroundWorker,
    /// Highlighted lines already computed for a file, so revisiting one is
    /// instant instead of a fresh syntect pass
    highlight_cache: std::collections::HashMap<PathBuf, Rc<Vec<ratatui::text::Line<'static>>>>,
    /// Files whose highlighting is in flight, to avoid queueing duplicates
    highlight_pending: HashSet<PathBuf>,
    pub symbol_indexing_progress: Option<(usize, usize)>,
    // Image protocol cache (by canonical path)
    pub image_protocols:
        std::collections::HashMap<PathBuf, ratatui_image::protocol::StatefulProtocol>,
    // Image picker (initialized once at startup)
    pub(crate) image_picker: Option<ratatui_image::picker::Picker>,
    // Track which images have full quality loaded (vs thumbnail)
    full_quality_images: HashSet<PathBuf>,
    // Pending full-quality image load (path, timestamp when to load)
    pending_full_quality: Option<(PathBuf, Instant)>,
    // PDF loading error message (shown when PDFium fails)
    pub pdf_error: Option<String>,
    /// Continuous-scroll state for the PDF being previewed, if any
    pub pdf_view: Option<pdf::PdfView>,
    // File tree popup state
    /// Line counts shown beside file tree entries, filled in by the worker
    pub tree_line_counts: crate::worker::TreeLineCounts,
    pub file_tree_popup_mode: bool,
    pub file_tree_popup_entries: Vec<crate::files::TreeRow>,
    pub file_tree_popup_selected: usize,
}

/// Main application state for the TUI file viewer.
#[allow(clippy::struct_excessive_bools)]
impl App {
    /// Returns the root directory being browsed.
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Initialize the image picker after TUI setup
    /// This must be called after entering alternate screen but before event loop
    pub fn init_image_picker(&mut self) {
        if self.image_picker.is_none() {
            match ratatui_image::picker::Picker::from_query_stdio() {
                Ok(picker) => {
                    self.image_picker = Some(picker);
                }
                Err(_e) => {
                    // Image picker failed - likely no TTY or graphics protocol unsupported
                    // Image files will show dimensions but not render
                    self.image_picker = None;
                }
            }
        }
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

        // Initialize file watcher
        let file_watcher = FileWatcher::new(&current_dir)?;
        let search_index_timer = RefreshTimer::new();

        // Spawn background worker
        let worker = BackgroundWorker::spawn();

        let mut app = Self {
            root_dir,
            current_dir,
            files,
            file_list_state: ListState::default(),
            shared_preview_content: None,
            preview_scroll: 0,
            should_quit: false,
            split_percent: INITIAL_SPLIT_PERCENT,
            config,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_index: Vec::new(),
            find_mode: false,
            find_query: String::new(),
            symbol_search_mode: false,
            symbol_search_query: String::new(),
            symbol_index: Vec::new(),
            symbol_search_results: Vec::new(),
            symbol_search_selected: 0,
            file_watcher,
            search_index_timer,
            selection: None,
            highlighted_word: None,
            last_preview_area: None,
            last_pane_areas: Vec::new(),
            last_file_list_area: None,
            clipboard: ClipboardManager::new(),
            current_theme_name: "jellybeans".to_string(),
            available_themes: Theme::list_builtins()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            theme_preview_mode: false,
            help_mode: false,
            split_layout: None,
            file_list_visible: true,
            worker,
            highlight_cache: std::collections::HashMap::new(),
            highlight_pending: HashSet::new(),
            symbol_indexing_progress: None,
            image_protocols: std::collections::HashMap::new(),
            image_picker: None, // Will be initialized after TUI setup
            full_quality_images: HashSet::new(),
            pending_full_quality: None,
            pdf_error: None,
            pdf_view: None,
            tree_line_counts: crate::worker::TreeLineCounts::default(),
            file_tree_popup_mode: false,
            file_tree_popup_entries: Vec::new(),
            file_tree_popup_selected: 0,
        };

        if !app.files.is_empty() {
            app.file_list_state.select(Some(0));
            app.load_preview();
        }

        Ok(app)
    }
}

// File tree popup management
impl App {
    /// Toggle the file tree popup
    pub fn toggle_file_tree_popup(&mut self) {
        self.file_tree_popup_mode = !self.file_tree_popup_mode;
        if self.file_tree_popup_mode {
            // Built fresh each time so the tree reflects the directory as it
            // is now, in hierarchy order rather than the search index's flat one.
            self.file_tree_popup_entries = crate::files::build_tree(&self.root_dir, &self.config);
            self.file_tree_popup_selected = 0;

            // Counting lines means reading every file, so it happens on the
            // worker; entries render without counts until it answers.
            self.worker
                .request_tree_line_counts(&self.root_dir, self.config.clone());
        }
    }

    /// Navigate up in the file tree popup
    pub const fn file_tree_popup_navigate_up(&mut self) {
        if self.file_tree_popup_selected > 0 {
            self.file_tree_popup_selected -= 1;
        } else if !self.file_tree_popup_entries.is_empty() {
            self.file_tree_popup_selected = self.file_tree_popup_entries.len() - 1;
        }
    }

    /// Navigate down in the file tree popup
    pub const fn file_tree_popup_navigate_down(&mut self) {
        if !self.file_tree_popup_entries.is_empty() {
            if self.file_tree_popup_selected < self.file_tree_popup_entries.len() - 1 {
                self.file_tree_popup_selected += 1;
            } else {
                self.file_tree_popup_selected = 0;
            }
        }
    }

    /// Confirm selection in the file tree popup
    pub fn file_tree_popup_confirm(&mut self) {
        if let Some(entry) = self
            .file_tree_popup_entries
            .get(self.file_tree_popup_selected)
            .map(|row| &row.entry)
        {
            if entry.is_file {
                // Navigate to the file's parent directory
                if let Some(parent) = entry.path.parent() {
                    self.current_dir = parent.to_path_buf();
                    // Reload files for the new directory
                    if let Ok(files) =
                        read_directory(&self.current_dir, &self.root_dir, &self.config)
                    {
                        self.files = files;
                        // Select the file
                        if let Some(idx) = self.files.iter().position(|f| f.path == entry.path) {
                            self.file_list_state.select(Some(idx));
                            self.load_preview();
                        }
                    }
                }
            } else {
                // Navigate into the directory
                self.current_dir = entry.path.clone();
                if let Ok(files) = read_directory(&self.current_dir, &self.root_dir, &self.config) {
                    self.files = files;
                    self.file_list_state.select(Some(0));
                    if !self.files.is_empty() {
                        self.load_preview();
                    }
                }
            }
        }
        // Close the popup
        self.file_tree_popup_mode = false;
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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");

        let app = App::new(root_path.clone()).expect("test setup failed");

        assert_eq!(
            app.root_dir,
            root_path.canonicalize().expect("test setup failed"),
            "Root dir should be canonicalized path"
        );
        assert_eq!(
            app.current_dir,
            root_path.canonicalize().expect("test setup failed"),
            "Current dir should start at root"
        );
        assert!(!app.files.is_empty(), "Files should be loaded");
    }

    #[test]
    fn test_app_new_with_nonexistent_path() {
        let result = App::new(PathBuf::from("/nonexistent/path/that/does/not/exist"));

        assert!(result.is_err(), "Should fail with nonexistent path");
        let err_msg = result.err().expect("test setup failed").to_string();
        assert!(
            err_msg.contains("Path does not exist"),
            "Error message should mention path doesn't exist"
        );
    }

    #[test]
    fn test_app_new_with_file_path() {
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let file_path = temp_dir.path().join("root/file1.txt");

        let app = App::new(file_path).expect("test setup failed");

        // Should use parent directory (root) as the root_dir
        let expected_root = temp_dir
            .path()
            .join("root")
            .canonicalize()
            .expect("test setup failed");
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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");

        let mut app = App::new(root_path.clone()).expect("test setup failed");
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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");

        let mut app = App::new(root_path.clone()).expect("test setup failed");
        let root_canonical = root_path.canonicalize().expect("test setup failed");

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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");

        let mut app = App::new(root_path.clone()).expect("test setup failed");
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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");

        // Get the canonical path first
        let canonical_root = root_path.canonicalize().expect("test setup failed");

        // Create app with absolute path
        let app = App::new(root_path).expect("test setup failed");

        assert_eq!(
            app.root_dir, canonical_root,
            "Should canonicalize paths to absolute"
        );
    }

    #[test]
    fn test_starting_from_nested_directory() {
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let nested_path = temp_dir.path().join("root/subdir1/subdir2");

        let mut app = App::new(nested_path.clone()).expect("test setup failed");
        let nested_canonical = nested_path.canonicalize().expect("test setup failed");

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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

        assert!(!app.search_mode);
        assert!(app.search_query.is_empty());

        app.enter_search_mode();
        assert!(app.search_mode);

        app.exit_search_mode();
        assert!(!app.search_mode);
    }

    #[test]
    fn test_search_input() {
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

        app.enter_search_mode();
        app.search_input('f');
        app.search_input('i');
        app.search_input('l');
        app.search_input('e');

        assert_eq!(app.search_query, "file");
    }

    #[test]
    fn test_search_backspace() {
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

        // Start at first item
        app.file_list_state.select(Some(0));

        // Navigate up should wrap to last
        app.navigate_up();
        let selected = app.file_list_state.selected().unwrap_or(0);
        assert_eq!(selected, app.files.len() - 1);
    }

    #[test]
    fn test_navigate_down_wraps_around() {
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

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
        let temp_dir = create_test_dir_structure().expect("test setup failed");
        let root_path = temp_dir.path().join("root");
        let mut app = App::new(root_path.clone()).expect("test setup failed");

        // Select first file
        if let Some(first) = app.files.first() {
            if first.is_file {
                app.file_list_state.select(Some(0));
                app.load_preview();

                // Should have loaded preview content
                assert!(app.shared_preview_content.is_some());
                if let Some(content) = &app.shared_preview_content {
                    // For text files, lines should not be empty
                    if let PreviewContentType::Text { lines, .. } = content.as_ref() {
                        assert!(!lines.is_empty());
                    }
                }
            }
        }
    }
}
