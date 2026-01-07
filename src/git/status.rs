use git2::Repository;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct GitStatus {
    repo: Option<Repository>,
    modified_files: HashSet<PathBuf>,
    last_refresh: Instant,
}

const CACHE_DURATION: Duration = Duration::from_secs(2);

impl GitStatus {
    /// Create a new GitStatus, discovering the git repository if it exists
    pub fn new(path: &Path) -> Self {
        let repo = Repository::discover(path).ok();

        Self {
            repo,
            modified_files: HashSet::new(),
            // Initialize to past to ensure first refresh() call always executes
            last_refresh: Instant::now() - CACHE_DURATION,
        }
    }

    /// Check if a file is modified according to git status
    pub fn is_modified(&self, path: &Path) -> bool {
        self.modified_files.contains(path)
    }

    /// Check if we're in a git repository
    #[allow(dead_code)]
    pub fn is_in_git_repo(&self) -> bool {
        self.repo.is_some()
    }

    /// Refresh the modified files list from git status
    pub fn refresh(&mut self) {
        // Skip if cache is still fresh
        if self.last_refresh.elapsed() < CACHE_DURATION {
            return;
        }

        self.modified_files.clear();

        if let Some(ref repo) = self.repo {
            if let Ok(statuses) = repo.statuses(None) {
                for entry in statuses.iter() {
                    if let Some(path) = entry.path() {
                        if is_modified_status(entry.status()) {
                            // Convert to absolute path for comparison
                            let file_path = repo.workdir()
                                .map(|wd| {
                                    // Use canonicalize if possible to ensure consistent paths
                                    let joined = wd.join(path);
                                    joined.canonicalize().unwrap_or(joined)
                                })
                                .unwrap_or_else(|| PathBuf::from(path));

                            self.modified_files.insert(file_path.clone());

                            // Also mark all parent directories as modified
                            let mut parent = file_path.parent();
                            while let Some(p) = parent {
                                self.modified_files.insert(p.to_path_buf());
                                parent = p.parent();
                            }
                        }
                    }
                }
            }
        }

        self.last_refresh = Instant::now();
    }
}

/// Check if a git status represents a modified file
fn is_modified_status(status: git2::Status) -> bool {
    status.is_wt_modified()
        || status.is_index_modified()
        || status.is_wt_deleted()
        || status.is_index_deleted()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_new_non_git_dir() {
        let temp_dir = std::env::temp_dir();
        let git_status = GitStatus::new(&temp_dir);

        // Should gracefully handle non-git directory
        assert!(!git_status.is_in_git_repo());
        assert!(!git_status.is_modified(&temp_dir));
    }

    #[test]
    fn test_cache_duration() {
        let temp_dir = std::env::temp_dir();
        let mut git_status = GitStatus::new(&temp_dir);

        let start = git_status.last_refresh;
        git_status.refresh();

        // First refresh should execute because last_refresh is initialized to past
        assert!(git_status.last_refresh > start);

        let after_first_refresh = git_status.last_refresh;
        git_status.refresh();

        // Second refresh should be skipped due to cache
        assert_eq!(git_status.last_refresh, after_first_refresh);
    }
}
