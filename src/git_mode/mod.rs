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
    /// Viewing diff of selected commit
    Diff,
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
            Self::Diff => "Diff",
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

/// Represents a single file's changes in a diff
#[derive(Debug, Clone)]
pub struct DiffFileStat {
    /// File path
    pub path: String,
    /// Number of insertions
    pub insertions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Whether the file is new (added)
    pub is_new: bool,
    /// Whether the file is deleted
    pub is_deleted: bool,
    /// The actual diff content (unified diff format)
    pub content: String,
}

impl DiffFileStat {
    /// Get the change indicator character
    #[must_use]
    pub const fn change_char(&self) -> char {
        if self.is_new {
            'A'
        } else if self.is_deleted {
            'D'
        } else {
            'M'
        }
    }
}

/// Git diff data manager
#[derive(Debug, Clone)]
pub struct GitDiff {
    files: Vec<DiffFileStat>,
    total_insertions: usize,
    total_deletions: usize,
}

impl GitDiff {
    /// Create a new empty git diff
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files: Vec::new(),
            total_insertions: 0,
            total_deletions: 0,
        }
    }

    /// Load git diff from a repository path (unstaged changes)
    ///
    /// # Errors
    /// Returns an error if the git repository cannot be opened or accessed
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let repo = git2::Repository::discover(path)?;
        let mut files = Vec::new();
        let mut total_insertions = 0;
        let mut total_deletions = 0;

        let diff = repo.diff_index_to_workdir(None, None)?;

        // Process each delta in the diff
        for delta in diff.deltas() {
            let path_str = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let is_new = matches!(delta.status(), git2::Delta::Added);
            let is_deleted = matches!(delta.status(), git2::Delta::Deleted);

            // For now, we'll collect minimal stats
            // Full diff content would require more complex git2 API usage
            files.push(DiffFileStat {
                path: path_str,
                insertions: 0,
                deletions: 0,
                is_new,
                is_deleted,
                content: String::new(),
            });
        }

        // Try to get actual diff content using git command
        // This is a fallback since git2's diff API is complex
        if let Ok(output) = std::process::Command::new("git")
            .args(&["diff", "--stat"])
            .current_dir(path)
            .output()
        {
            if let Ok(stat_output) = String::from_utf8(output.stdout) {
                // Parse stat output to get per-file insertions/deletions
                for line in stat_output.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let Some(file_index) =
                            files.iter().position(|f| f.path.ends_with(parts[0]))
                        {
                            // Extract insertion/deletion counts from stat line
                            let stat_part = parts[parts.len() - 1];
                            let changes: Vec<&str> =
                                stat_part.split(|c: char| c == '+' || c == '-').collect();

                            if changes.len() >= 3 {
                                if let Ok(adds) = changes[1].parse::<usize>() {
                                    files[file_index].insertions = adds;
                                }
                                if let Ok(subs) = changes[2].parse::<usize>() {
                                    files[file_index].deletions = subs;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Get full diff content
        if let Ok(output) = std::process::Command::new("git")
            .args(&["diff"])
            .current_dir(path)
            .output()
        {
            if let Ok(diff_content) = String::from_utf8(output.stdout) {
                // Parse the unified diff format and populate file contents
                let mut current_file_idx: Option<usize> = None;

                for line in diff_content.lines() {
                    if line.starts_with("diff --git") {
                        // Extract file path from "diff --git a/path b/path"
                        if let Some(b_start) = line.rfind(" b/") {
                            let file_path = &line[b_start + 3..];
                            if let Some(idx) =
                                files.iter().position(|f| f.path.ends_with(file_path))
                            {
                                current_file_idx = Some(idx);
                            }
                        }
                    } else if let Some(idx) = current_file_idx {
                        if line.starts_with("+++")
                            || line.starts_with("---")
                            || line.starts_with("@@")
                        {
                            files[idx].content.push_str(line);
                            files[idx].content.push('\n');
                        } else if line.starts_with('+') {
                            files[idx].content.push_str(line);
                            files[idx].content.push('\n');
                        } else if line.starts_with('-') {
                            files[idx].content.push_str(line);
                            files[idx].content.push('\n');
                        } else if line.starts_with(' ') {
                            files[idx].content.push_str(line);
                            files[idx].content.push('\n');
                        }
                    }
                }
            }
        }

        // Calculate totals
        for file in &files {
            total_insertions += file.insertions;
            total_deletions += file.deletions;
        }

        Ok(Self {
            files,
            total_insertions,
            total_deletions,
        })
    }

    /// Get the diff files
    #[must_use]
    pub fn files(&self) -> &[DiffFileStat] {
        &self.files
    }

    /// Get total insertions
    #[must_use]
    pub const fn total_insertions(&self) -> usize {
        self.total_insertions
    }

    /// Get total deletions
    #[must_use]
    pub const fn total_deletions(&self) -> usize {
        self.total_deletions
    }

    /// Load git diff for a specific commit hash
    ///
    /// # Errors
    /// Returns an error if the git repository cannot be opened or accessed
    pub fn load_for_commit(path: &Path, commit_hash: &str) -> anyhow::Result<Self> {
        let mut files = Vec::new();
        let mut total_insertions = 0;
        let mut total_deletions = 0;

        // Get full diff content for the commit
        if let Ok(output) = std::process::Command::new("git")
            .args(&["show", "--no-patch", "--format=%H", commit_hash])
            .current_dir(path)
            .output()
        {
            // First validate the commit exists
            if !output.status.success() {
                anyhow::bail!("Invalid commit hash: {}", commit_hash);
            }
        }

        // Get diff stats for the commit
        if let Ok(output) = std::process::Command::new("git")
            .args(&[
                "diff",
                &format!("{}^..{}", commit_hash, commit_hash),
                "--stat",
            ])
            .current_dir(path)
            .output()
        {
            if let Ok(stat_output) = String::from_utf8(output.stdout) {
                for line in stat_output.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }

                    let file_path = parts[0].to_string();
                    let mut insertions = 0;
                    let mut deletions = 0;

                    // Extract insertion/deletion counts
                    if parts.len() >= 3 {
                        let stat_part = parts[parts.len() - 1];
                        let changes: Vec<&str> =
                            stat_part.split(|c: char| c == '+' || c == '-').collect();

                        if changes.len() >= 3 {
                            if let Ok(adds) = changes[1].parse::<usize>() {
                                insertions = adds;
                            }
                            if let Ok(subs) = changes[2].parse::<usize>() {
                                deletions = subs;
                            }
                        }
                    }

                    total_insertions += insertions;
                    total_deletions += deletions;

                    files.push(DiffFileStat {
                        path: file_path,
                        insertions,
                        deletions,
                        is_new: false,
                        is_deleted: false,
                        content: String::new(),
                    });
                }
            }
        }

        // Get full diff content
        if let Ok(output) = std::process::Command::new("git")
            .args(&["diff", &format!("{}^..{}", commit_hash, commit_hash)])
            .current_dir(path)
            .output()
        {
            if let Ok(diff_content) = String::from_utf8(output.stdout) {
                // Parse the unified diff format and populate file contents
                let mut current_file: Option<usize> = None;

                for line in diff_content.lines() {
                    if line.starts_with("diff --git") {
                        // Extract file path from diff line (typically second path)
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            let file_name = parts[parts.len() - 1];
                            // Remove leading "b/" if present
                            let file_name = file_name.strip_prefix("b/").unwrap_or(file_name);
                            current_file = files.iter().position(|f| f.path.ends_with(file_name));
                        }
                    } else if let Some(idx) = current_file {
                        files[idx].content.push_str(line);
                        files[idx].content.push('\n');
                    }
                }
            }
        }

        Ok(Self {
            files,
            total_insertions,
            total_deletions,
        })
    }
}

impl Default for GitDiff {
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
