//! Git mode functionality for the app

use super::App;
use crate::git_mode::{GitLog, GitModeState, GitStatusData};

impl App {
    /// Toggle git mode (enter/exit)
    pub fn toggle_git_mode(&mut self) {
        if self.git_mode_state.is_active() {
            self.exit_git_mode();
        } else {
            self.enter_git_mode();
        }
    }

    /// Enter git mode, defaulting to log view
    fn enter_git_mode(&mut self) {
        self.git_mode_state = GitModeState::Log;
        self.load_git_log();
    }

    /// Exit git mode
    const fn exit_git_mode(&mut self) {
        self.git_mode_state = GitModeState::None;
    }

    /// Load git log data
    fn load_git_log(&mut self) {
        self.git_log = GitLog::load(&self.current_dir, usize::MAX).unwrap_or_default();
        self.git_log_selected = 0;
        self.git_log_scroll = 0;
        self.git_log_list_scroll = 0;
    }

    /// Load git status data
    fn load_git_status(&mut self) {
        self.git_status_data = GitStatusData::load(&self.current_dir).unwrap_or_default();
        self.git_status_selected = 0;
    }

    /// Switch to git log view
    pub fn git_mode_show_log(&mut self) {
        if self.git_mode_state.is_active() {
            self.git_mode_state = GitModeState::Log;
            if self.git_log.entries().is_empty() {
                self.load_git_log();
            }
        }
    }

    /// Switch to git status view
    pub fn git_mode_show_status(&mut self) {
        if self.git_mode_state.is_active() {
            self.git_mode_state = GitModeState::Status;
            self.load_git_status();
        }
    }

    /// Navigate up in git mode
    pub fn git_mode_navigate_up(&mut self) {
        match self.git_mode_state {
            GitModeState::Log => {
                if self.git_log_selected > 0 {
                    self.git_log_selected -= 1;
                    // Scroll up if selected item would go above visible area
                    if self.git_log_selected < self.git_log_list_scroll {
                        self.git_log_list_scroll = self.git_log_selected;
                    }
                } else if !self.git_log.entries().is_empty() {
                    self.git_log_selected = self.git_log.entries().len() - 1;
                    // When wrapping to bottom, scroll to show the item
                    let visible_rows = 10; // Approximate visible rows in the table
                    if self.git_log_selected >= visible_rows {
                        self.git_log_list_scroll = self.git_log_selected - visible_rows + 1;
                    } else {
                        self.git_log_list_scroll = 0;
                    }
                }
                self.git_log_scroll = 0;
            }
            GitModeState::Status => {
                if self.git_status_selected > 0 {
                    self.git_status_selected -= 1;
                } else if !self.git_status_data.entries().is_empty() {
                    self.git_status_selected = self.git_status_data.entries().len() - 1;
                }
            }
            GitModeState::None => {}
        }
    }

    /// Navigate down in git mode
    pub fn git_mode_navigate_down(&mut self) {
        match self.git_mode_state {
            GitModeState::Log => {
                if !self.git_log.entries().is_empty() {
                    if self.git_log_selected < self.git_log.entries().len() - 1 {
                        self.git_log_selected += 1;
                    } else {
                        self.git_log_selected = 0;
                        self.git_log_list_scroll = 0;
                    }
                    // Scroll down if selected item would go below visible area
                    let visible_rows = 10; // Approximate visible rows in the table
                    if self.git_log_selected >= self.git_log_list_scroll + visible_rows {
                        self.git_log_list_scroll = self.git_log_selected - visible_rows + 1;
                    }
                }
                self.git_log_scroll = 0;
            }
            GitModeState::Status => {
                if !self.git_status_data.entries().is_empty() {
                    if self.git_status_selected < self.git_status_data.entries().len() - 1 {
                        self.git_status_selected += 1;
                    } else {
                        self.git_status_selected = 0;
                    }
                }
            }
            GitModeState::None => {}
        }
    }

    /// Scroll up within the current log entry (for long messages)
    pub fn git_mode_scroll_up(&mut self) {
        if self.git_mode_state == GitModeState::Log && self.git_log_scroll > 0 {
            self.git_log_scroll -= 1;
        }
    }

    /// Scroll down within the current log entry (for long messages)
    pub fn git_mode_scroll_down(&mut self) {
        if self.git_mode_state == GitModeState::Log {
            // Allow scrolling if there might be more content
            self.git_log_scroll += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_mode_toggle() {
        let temp_dir = TempDir::new().expect("test setup failed");
        let mut app = App::new(temp_dir.path().to_path_buf()).expect("test setup failed");

        // Initially not in git mode
        assert!(!app.git_mode_state.is_active());

        // Toggle on
        app.toggle_git_mode();
        assert!(app.git_mode_state.is_active());

        // Toggle off
        app.toggle_git_mode();
        assert!(!app.git_mode_state.is_active());
    }

    #[test]
    fn test_git_mode_navigation_log() {
        let temp_dir = TempDir::new().expect("test setup failed");
        let mut app = App::new(temp_dir.path().to_path_buf()).expect("test setup failed");

        // Enter git mode (should default to log)
        app.toggle_git_mode();
        assert_eq!(app.git_mode_state, GitModeState::Log);

        // Navigation should work even with empty log
        app.git_mode_navigate_down();
        app.git_mode_navigate_up();
    }

    #[test]
    fn test_git_mode_switch_views() {
        let temp_dir = TempDir::new().expect("test setup failed");
        let mut app = App::new(temp_dir.path().to_path_buf()).expect("test setup failed");

        // Enter git mode
        app.toggle_git_mode();
        assert_eq!(app.git_mode_state, GitModeState::Log);

        // Switch to status
        app.git_mode_show_status();
        assert_eq!(app.git_mode_state, GitModeState::Status);

        // Switch back to log
        app.git_mode_show_log();
        assert_eq!(app.git_mode_state, GitModeState::Log);
    }

    #[test]
    fn test_git_mode_switch_not_active() {
        let temp_dir = TempDir::new().expect("test setup failed");
        let mut app = App::new(temp_dir.path().to_path_buf()).expect("test setup failed");

        // Should not switch views when not in git mode
        app.git_mode_show_status();
        assert!(!app.git_mode_state.is_active());

        app.git_mode_show_log();
        assert!(!app.git_mode_state.is_active());
    }
}
