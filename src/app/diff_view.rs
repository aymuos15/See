//! Diff view functionality for the app

use super::App;
use crate::git_mode::GitDiff;

impl App {
    /// Toggle diff view mode (for git changes)
    pub fn toggle_diff_view(&mut self) {
        if self.git_diff_mode {
            self.exit_diff_mode();
        } else {
            self.enter_diff_mode();
        }
    }

    /// Enter diff view mode
    pub fn enter_diff_mode(&mut self) {
        self.git_diff_mode = true;
        self.git_diff_selected_file = 0;
        self.git_diff_scroll = 0;
        self.load_git_diff();
    }

    /// Exit diff view mode
    pub const fn exit_diff_mode(&mut self) {
        self.git_diff_mode = false;
    }

    /// Load git diff data
    fn load_git_diff(&mut self) {
        self.git_diff = GitDiff::load(&self.current_dir).unwrap_or_default();
        self.git_diff_selected_file = 0;
        self.git_diff_scroll = 0;
    }

    /// Navigate up in diff file list
    pub fn diff_navigate_up(&mut self) {
        if self.git_diff_selected_file > 0 {
            self.git_diff_selected_file -= 1;
        } else if !self.git_diff.files().is_empty() {
            self.git_diff_selected_file = self.git_diff.files().len() - 1;
        }
        self.git_diff_scroll = 0;
    }

    /// Navigate down in diff file list
    pub fn diff_navigate_down(&mut self) {
        if !self.git_diff.files().is_empty() {
            if self.git_diff_selected_file < self.git_diff.files().len() - 1 {
                self.git_diff_selected_file += 1;
            } else {
                self.git_diff_selected_file = 0;
            }
        }
        self.git_diff_scroll = 0;
    }

    /// Scroll up in the diff content
    pub const fn diff_scroll_up(&mut self) {
        if self.git_diff_scroll > 0 {
            self.git_diff_scroll -= 1;
        }
    }

    /// Scroll down in the diff content
    pub const fn diff_scroll_down(&mut self) {
        self.git_diff_scroll += 1;
    }
}
