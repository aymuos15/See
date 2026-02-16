//! Git mode module for viewing git log and status

pub mod ui;

use std::path::Path;

/// The current state of git mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitModeState {
    /// Git mode is not active
    #[default]
    None,
    /// Viewing git log
    Log,
    /// Viewing git status
    Status,
}

impl GitModeState {
    /// Check if git mode is active
    #[must_use]
    pub const fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Get the name of the current view for display
    #[must_use]
    pub const fn view_name(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Log => "Log",
            Self::Status => "Status",
        }
    }
}

/// Represents a single commit in git log
#[derive(Debug, Clone)]
pub struct GitLogEntry {
    /// Full commit hash
    pub hash: String,
    /// Short commit hash (7 chars)
    pub short_hash: String,
    /// Author name
    pub author: String,
    /// Commit date as Unix timestamp
    pub timestamp: i64,
    /// Commit message (first line)
    pub message: String,
    /// Full commit message
    pub full_message: String,
    /// Branch/tag refs (e.g., "main, origin/main")
    #[allow(dead_code)]
    pub refs: Option<String>,
}

/// Represents a file entry in git status
#[derive(Debug, Clone)]
pub struct GitStatusEntry {
    /// File path relative to repo root
    pub path: String,
    /// Status in index (staged)
    pub index_status: GitFileStatus,
    /// Status in working tree (unstaged)
    pub worktree_status: GitFileStatus,
    /// Whether the file is renamed (original path)
    #[allow(dead_code)]
    pub original_path: Option<String>,
}

/// Git file status for a single file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatus {
    /// Unmodified
    Unmodified,
    /// Added
    Added,
    /// Modified
    Modified,
    /// Deleted
    Deleted,
    /// Renamed
    Renamed,
    /// Copied
    #[allow(dead_code)]
    Copied,
    /// Updated but unmerged
    #[allow(dead_code)]
    Unmerged,
    /// Untracked
    Untracked,
    /// Ignored
    Ignored,
    /// Conflict
    Conflict,
}

impl GitFileStatus {
    /// Get the single character representation
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Unmodified => ' ',
            Self::Added => 'A',
            Self::Modified => 'M',
            Self::Deleted => 'D',
            Self::Renamed => 'R',
            Self::Copied => 'C',
            Self::Unmerged => 'U',
            Self::Untracked => '?',
            Self::Ignored => '!',
            Self::Conflict => 'X',
        }
    }

    /// Parse from git2 status char
    #[must_use]
    pub fn from_git2_status(status: git2::Status) -> (Self, Self) {
        let index = if status.is_index_new() {
            Self::Added
        } else if status.is_index_modified() {
            Self::Modified
        } else if status.is_index_deleted() {
            Self::Deleted
        } else if status.is_index_renamed() {
            Self::Renamed
        } else if status.is_index_typechange() {
            Self::Modified
        } else {
            Self::Unmodified
        };

        let worktree = if status.is_wt_new() {
            Self::Untracked
        } else if status.is_wt_modified() {
            Self::Modified
        } else if status.is_wt_deleted() {
            Self::Deleted
        } else if status.is_wt_renamed() {
            Self::Renamed
        } else if status.is_wt_typechange() {
            Self::Modified
        } else if status.is_conflicted() {
            Self::Conflict
        } else if status.is_ignored() {
            Self::Ignored
        } else {
            Self::Unmodified
        };

        (index, worktree)
    }
}

/// Git log data manager
pub struct GitLog {
    entries: Vec<GitLogEntry>,
}

impl GitLog {
    /// Create a new empty git log
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load git log from a repository path
    ///
    /// # Errors
    /// Returns an error if the git repository cannot be opened or accessed
    pub fn load(path: &Path, limit: usize) -> anyhow::Result<Self> {
        let repo = git2::Repository::discover(path)?;
        let mut entries = Vec::new();

        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for (idx, oid_result) in revwalk.enumerate() {
            if idx >= limit {
                break;
            }

            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;

            let hash = oid.to_string();
            let short_hash = hash.chars().take(7).collect();

            let author = commit
                .author()
                .name()
                .map_or_else(|| "Unknown".to_string(), std::string::ToString::to_string);

            let timestamp = commit.time().seconds();

            let message = commit
                .message()
                .map_or_else(String::new, |m| m.lines().next().unwrap_or(m).to_string());

            let full_message = commit
                .message()
                .map_or_else(String::new, std::string::ToString::to_string);

            entries.push(GitLogEntry {
                hash,
                short_hash,
                author,
                timestamp,
                message,
                full_message,
                refs: None, // TODO: Get branch/tag refs
            });
        }

        Ok(Self { entries })
    }

    /// Get the log entries
    #[must_use]
    pub fn entries(&self) -> &[GitLogEntry] {
        &self.entries
    }
}

impl Default for GitLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Git status data manager
pub struct GitStatusData {
    entries: Vec<GitStatusEntry>,
    branch: Option<String>,
}

impl GitStatusData {
    /// Create a new empty git status
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            branch: None,
        }
    }

    /// Load git status from a repository path
    ///
    /// # Errors
    /// Returns an error if the git repository cannot be opened or accessed
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let repo = git2::Repository::discover(path)?;

        // Get current branch
        let branch = repo
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(std::string::ToString::to_string));

        // Get status
        let statuses = repo.statuses(None)?;
        let mut entries = Vec::new();

        for entry in statuses.iter() {
            if let Some(path_str) = entry.path() {
                let (index_status, worktree_status) =
                    GitFileStatus::from_git2_status(entry.status());

                // Skip unmodified entries
                if index_status == GitFileStatus::Unmodified
                    && worktree_status == GitFileStatus::Unmodified
                {
                    continue;
                }

                entries.push(GitStatusEntry {
                    path: path_str.to_string(),
                    index_status,
                    worktree_status,
                    original_path: None, // TODO: Handle renames
                });
            }
        }

        Ok(Self { entries, branch })
    }

    /// Get the status entries
    #[must_use]
    pub fn entries(&self) -> &[GitStatusEntry] {
        &self.entries
    }

    /// Get the current branch name
    #[must_use]
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

impl Default for GitStatusData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_mode_state_is_active() {
        assert!(!GitModeState::None.is_active());
        assert!(GitModeState::Log.is_active());
        assert!(GitModeState::Status.is_active());
    }

    #[test]
    fn test_git_file_status_as_char() {
        assert_eq!(GitFileStatus::Unmodified.as_char(), ' ');
        assert_eq!(GitFileStatus::Added.as_char(), 'A');
        assert_eq!(GitFileStatus::Modified.as_char(), 'M');
        assert_eq!(GitFileStatus::Deleted.as_char(), 'D');
        assert_eq!(GitFileStatus::Untracked.as_char(), '?');
    }
}
