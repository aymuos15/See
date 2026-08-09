//! Browsing a repository's history: a commit list, and the diff of whichever
//! commit is opened.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::git::{Commit, CommitDetail};

use super::App;

/// Commits fetched per request. One batch covers most sessions; scrolling near
/// the end pulls the next one.
const LOG_BATCH: usize = 500;
/// Rows before the end of the loaded list at which the next batch is fetched.
const LOG_PREFETCH_MARGIN: usize = 50;
/// Commit diffs held in memory at once.
const DETAIL_CACHE_ENTRIES: usize = 32;

/// Which of git mode's two levels is on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitLevel {
    /// The commit list, with the selected commit summarised beside it.
    Commits,
    /// One commit: its files on the left, its diff on the right.
    Detail,
}

/// State for the whole git-mode session, dropped when the mode is left.
pub struct GitMode {
    pub repo: PathBuf,
    pub level: GitLevel,
    pub commits: Vec<Commit>,
    pub selected: usize,
    /// First commit row on screen, kept so the selection can stay in view.
    pub list_offset: usize,
    /// Diffs already loaded, by commit hash.
    details: HashMap<String, CommitDetail>,
    /// Commits whose diff has been asked of the worker.
    requested: Vec<String>,
    /// Whether another batch of commits is on its way.
    loading_more: bool,
    /// Set once the log is exhausted, so we stop asking for more.
    exhausted: bool,
    pub file_selected: usize,
    pub diff_scroll: usize,
    /// Message shown in place of content when git itself failed.
    pub error: Option<String>,
}

impl GitMode {
    fn new(repo: PathBuf) -> Self {
        Self {
            repo,
            level: GitLevel::Commits,
            commits: Vec::new(),
            selected: 0,
            list_offset: 0,
            details: HashMap::new(),
            requested: Vec::new(),
            loading_more: false,
            exhausted: false,
            file_selected: 0,
            diff_scroll: 0,
            error: None,
        }
    }

    pub fn selected_commit(&self) -> Option<&Commit> {
        self.commits.get(self.selected)
    }

    /// The loaded diff for the selected commit, if it has arrived.
    pub fn selected_detail(&self) -> Option<&CommitDetail> {
        let hash = &self.selected_commit()?.hash;
        self.details.get(hash)
    }

    /// Keep the selected row inside a list `height` rows tall.
    pub const fn scroll_list_into_view(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.selected < self.list_offset {
            self.list_offset = self.selected;
        } else if self.selected >= self.list_offset + height {
            self.list_offset = self.selected + 1 - height;
        }
    }

    /// Keep the selected file row inside a list `height` rows tall.
    pub const fn file_row_offset(&self, height: usize) -> usize {
        if height == 0 || self.file_selected < height {
            return 0;
        }
        self.file_selected + 1 - height
    }
}

impl App {
    /// Enter git mode, or leave it if it is already open.
    pub(super) fn toggle_git_mode(&mut self) {
        if self.git_mode.is_some() {
            self.git_mode = None;
            return;
        }

        let repo = crate::git::repo_root(&self.current_dir);
        let mut mode = GitMode::new(repo.clone().unwrap_or_else(|| self.current_dir.clone()));

        if repo.is_none() {
            mode.error = Some("Not a git repository".to_string());
        } else {
            mode.loading_more = true;
            self.worker.request_git_log(&mode.repo, 0, LOG_BATCH);
        }

        self.git_mode = Some(mode);
    }

    pub const fn in_git_mode(&self) -> bool {
        self.git_mode.is_some()
    }

    /// Store a batch of commits as it arrives from the worker.
    pub(super) fn handle_git_log_loaded(
        &mut self,
        skip: usize,
        result: anyhow::Result<Vec<Commit>>,
    ) {
        let Some(mode) = self.git_mode.as_mut() else {
            return;
        };
        mode.loading_more = false;

        match result {
            Ok(commits) => {
                if commits.len() < LOG_BATCH {
                    mode.exhausted = true;
                }
                // A batch that no longer lines up with the list is from a
                // previous session in this mode; ignore it rather than
                // duplicating or tearing the history.
                if skip == mode.commits.len() {
                    mode.commits.extend(commits);
                }
                if mode.commits.is_empty() {
                    mode.error = Some("This repository has no commits yet".to_string());
                }
            }
            Err(e) => mode.error = Some(e.to_string()),
        }

        self.request_selected_commit();
    }

    /// Store one commit's diff as it arrives from the worker.
    pub(super) fn handle_git_commit_loaded(
        &mut self,
        hash: &str,
        result: anyhow::Result<CommitDetail>,
    ) {
        let Some(mode) = self.git_mode.as_mut() else {
            return;
        };

        match result {
            Ok(detail) => {
                // Bound the cache so a long browse cannot grow it without limit.
                if mode.details.len() >= DETAIL_CACHE_ENTRIES {
                    mode.details.clear();
                    mode.requested.clear();
                }
                mode.details.insert(hash.to_string(), detail);
            }
            Err(e) => mode.error = Some(e.to_string()),
        }
    }

    /// Ask for whatever the selected commit needs: its diff, and the next batch
    /// of commits once the selection nears the end of the list.
    fn request_selected_commit(&mut self) {
        let Some(mode) = self.git_mode.as_mut() else {
            return;
        };

        let mut wanted_hash = None;
        if let Some(commit) = mode.commits.get(mode.selected) {
            let hash = commit.hash.clone();
            if !mode.details.contains_key(&hash) && !mode.requested.contains(&hash) {
                mode.requested.push(hash.clone());
                wanted_hash = Some(hash);
            }
        }

        let want_more = !mode.exhausted
            && !mode.loading_more
            && mode.selected + LOG_PREFETCH_MARGIN >= mode.commits.len();
        let skip = mode.commits.len();
        if want_more {
            mode.loading_more = true;
        }

        let repo = mode.repo.clone();
        if let Some(hash) = wanted_hash {
            self.worker.request_git_commit(&repo, &hash);
        }
        if want_more {
            self.worker.request_git_log(&repo, skip, LOG_BATCH);
        }
    }

    /// Move the commit or file selection by `delta` rows.
    pub(super) fn git_select_by(&mut self, delta: isize) {
        let Some(mode) = self.git_mode.as_mut() else {
            return;
        };

        match mode.level {
            GitLevel::Commits => {
                let last = mode.commits.len().saturating_sub(1);
                mode.selected = step(mode.selected, delta, last);
                self.request_selected_commit();
            }
            GitLevel::Detail => {
                let last = mode
                    .selected_detail()
                    .map_or(0, |detail| detail.files.len().saturating_sub(1));
                mode.file_selected = step(mode.file_selected, delta, last);
                // Selecting a file scrolls the diff to where that file starts,
                // rather than reloading anything.
                if let Some(detail) = mode.selected_detail() {
                    if let Some(file) = detail.files.get(mode.file_selected) {
                        mode.diff_scroll = file.diff_line;
                    }
                }
            }
        }
    }

    /// Scroll the diff pane by `delta` rows. Only meaningful on the detail level.
    pub(super) fn git_scroll_diff(&mut self, delta: isize) {
        let Some(mode) = self.git_mode.as_mut() else {
            return;
        };
        if mode.level != GitLevel::Detail {
            return;
        }
        let last = mode
            .selected_detail()
            .map_or(0, |detail| detail.diff.len().saturating_sub(1));
        mode.diff_scroll = step(mode.diff_scroll, delta, last);
    }

    /// Open the selected commit's diff.
    pub(super) fn git_open_selected(&mut self) {
        if let Some(mode) = self.git_mode.as_mut() {
            if mode.level == GitLevel::Commits && mode.selected_commit().is_some() {
                mode.level = GitLevel::Detail;
                mode.file_selected = 0;
                mode.diff_scroll = 0;
            }
        }
    }

    /// Route an event while git mode is open. Returns false for events git
    /// mode does not claim, so they keep their normal meaning.
    pub(super) fn handle_git_mode_event(&mut self, event: &crate::event::AppEvent) -> bool {
        use crate::event::AppEvent as E;

        let page = self.last_preview_area.map_or(20, |area| {
            isize::from(i16::try_from(area.height.max(2)).unwrap_or(i16::MAX)) - 1
        });

        match event {
            E::ToggleGitMode => self.git_mode = None,
            E::Quit | E::GoBack => self.git_back(),
            E::Enter => self.git_open_selected(),
            E::NavigateUp => self.git_select_by(-1),
            E::NavigateDown => self.git_select_by(1),
            E::ScrollPreviewUp | E::MouseScrollUp => self.git_scroll_or_select(-1),
            E::ScrollPreviewDown | E::MouseScrollDown => self.git_scroll_or_select(1),
            E::ScrollPreviewPageUp => self.git_scroll_or_select(-page),
            E::ScrollPreviewPageDown => self.git_scroll_or_select(page),
            // Help opens over git mode, and the watcher and timer events keep
            // the file view behind it current.
            E::ToggleHelp
            | E::DirectoryChanged
            | E::PreviewFileChanged
            | E::SearchIndexRefreshTimer
            | E::None => return false,
            // Everything else belongs to the file view, whose panes are not on
            // screen: swallow it rather than acting invisibly.
            _ => {}
        }
        true
    }

    /// Scroll the diff when one is open, otherwise move through the commits.
    fn git_scroll_or_select(&mut self, delta: isize) {
        let level = self.git_mode.as_ref().map(|mode| mode.level);
        if level == Some(GitLevel::Detail) {
            self.git_scroll_diff(delta);
        } else {
            self.git_select_by(delta);
        }
    }

    /// Step back to the commit list, or leave git mode if already there.
    pub(super) fn git_back(&mut self) {
        let Some(mode) = self.git_mode.as_mut() else {
            return;
        };
        match mode.level {
            GitLevel::Detail => mode.level = GitLevel::Commits,
            GitLevel::Commits => self.git_mode = None,
        }
    }
}

/// Move an index by a signed delta, clamped to `0..=last`.
fn step(current: usize, delta: isize, last: usize) -> usize {
    let current = isize::try_from(current).unwrap_or(isize::MAX);
    let last = isize::try_from(last).unwrap_or(isize::MAX);
    let next = current.saturating_add(delta).clamp(0, last);
    usize::try_from(next).unwrap_or(0)
}
