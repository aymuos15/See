use super::App;
use crate::git::GitStatus;
use std::path::Path;

impl App {
    /// Toggle git highlighting on/off
    pub fn toggle_git_highlight(&mut self) {
        self.git_highlight_enabled = !self.git_highlight_enabled;

        // Initialize git status on first enable
        if self.git_highlight_enabled && self.git_status.is_none() {
            self.git_status = Some(GitStatus::new(&self.current_dir));
        }

        // Refresh git status when enabled
        if self.git_highlight_enabled {
            if let Some(ref mut git_status) = self.git_status {
                git_status.refresh();
            }
        }
    }

    /// Check if a file is modified according to git
    pub fn is_file_modified(&self, path: &Path) -> bool {
        if !self.git_highlight_enabled {
            return false;
        }

        // Try to canonicalize the path for consistent comparison
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        self.git_status
            .as_ref()
            .is_some_and(|status| status.is_modified(&canonical_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_toggle_git_highlight() {
        let temp_dir = TempDir::new().expect("operation failed");
        let app = App::new(temp_dir.path().to_path_buf()).expect("operation failed");

        assert!(!app.git_highlight_enabled);
        assert!(app.git_status.is_none());
    }

    #[test]
    fn test_is_file_modified_when_disabled() {
        let temp_dir = TempDir::new().expect("operation failed");
        let app = App::new(temp_dir.path().to_path_buf()).expect("operation failed");

        assert!(!app.is_file_modified(temp_dir.path()));
    }
}
