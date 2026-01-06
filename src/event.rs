use crate::constants::{FILE_EVENT_DEBOUNCE_MS, SEARCH_INDEX_REFRESH_SECS};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind};
use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

pub enum AppEvent {
    Quit,
    NavigateUp,
    NavigateDown,
    ScrollPreviewUp,
    ScrollPreviewDown,
    ScrollPreviewPageUp,
    ScrollPreviewPageDown,
    ShrinkFileList,
    GrowFileList,
    Enter,
    GoBack,
    OpenSearch,
    CloseSearch,
    SearchInput(char),
    SearchBackspace,
    SearchNavigateUp,
    SearchNavigateDown,
    SearchConfirm,
    DirectoryChanged,
    PreviewFileChanged,
    SearchIndexRefreshTimer,
    None,
}

pub fn poll_event(
    timeout: Duration,
    search_mode: bool,
    watcher: &mut FileWatcher,
    timer: &mut RefreshTimer,
) -> anyhow::Result<AppEvent> {
    // Check for file watcher events (non-blocking)
    if let Some(fs_event) = watcher.poll_events() {
        return Ok(fs_event);
    }

    // Check search index refresh timer
    if timer.check_and_reset() {
        return Ok(AppEvent::SearchIndexRefreshTimer);
    }

    // Check for keyboard and mouse events
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return Ok(AppEvent::None);
                }
                return Ok(handle_key(key.code, search_mode));
            }
            Event::Mouse(mouse) => {
                return Ok(match mouse.kind {
                    MouseEventKind::ScrollDown => AppEvent::ScrollPreviewDown,
                    MouseEventKind::ScrollUp => AppEvent::ScrollPreviewUp,
                    _ => AppEvent::None,
                });
            }
            _ => {}
        }
    }
    Ok(AppEvent::None)
}

const fn handle_key(code: KeyCode, search_mode: bool) -> AppEvent {
    if search_mode {
        match code {
            KeyCode::Esc => AppEvent::CloseSearch,
            KeyCode::Enter => AppEvent::SearchConfirm,
            KeyCode::Backspace => AppEvent::SearchBackspace,
            KeyCode::Up => AppEvent::SearchNavigateUp,
            KeyCode::Down => AppEvent::SearchNavigateDown,
            KeyCode::Char(c) => AppEvent::SearchInput(c),
            _ => AppEvent::None,
        }
    } else {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => AppEvent::Quit,
            KeyCode::Char('/') => AppEvent::OpenSearch,
            KeyCode::Char('j') => AppEvent::ScrollPreviewDown,
            KeyCode::Char('k') => AppEvent::ScrollPreviewUp,
            KeyCode::Down => AppEvent::NavigateDown,
            KeyCode::Up => AppEvent::NavigateUp,
            KeyCode::PageDown => AppEvent::ScrollPreviewPageDown,
            KeyCode::PageUp => AppEvent::ScrollPreviewPageUp,
            KeyCode::Char('H') => AppEvent::ShrinkFileList,
            KeyCode::Char('L') => AppEvent::GrowFileList,
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => AppEvent::Enter,
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => AppEvent::GoBack,
            _ => AppEvent::None,
        }
    }
}

/// File watcher for current directory and preview file
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<NotifyEvent, notify::Error>>,
    current_dir: PathBuf,
    preview_file: Option<PathBuf>,
    last_event_time: Instant,
}

impl FileWatcher {
    /// Create a new file watcher for the given directory
    pub fn new(current_dir: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let watcher = notify::recommended_watcher(tx)?;

        let mut fw = Self {
            watcher,
            receiver: rx,
            current_dir: PathBuf::new(),
            preview_file: None,
            last_event_time: Instant::now(),
        };

        fw.watch_directory(current_dir)?;

        Ok(fw)
    }

    /// Watch a new directory (non-recursive)
    pub fn watch_directory(&mut self, dir: &Path) -> anyhow::Result<()> {
        // Unwatch previous directory
        if self.current_dir.exists() {
            let _ = self.watcher.unwatch(&self.current_dir);
        }

        // Watch new directory (non-recursive for current dir)
        self.watcher.watch(dir, RecursiveMode::NonRecursive)?;
        self.current_dir = dir.to_path_buf();

        Ok(())
    }

    /// Watch a preview file for changes
    pub fn watch_preview_file(&mut self, file: Option<&PathBuf>) -> anyhow::Result<()> {
        // Unwatch previous preview file
        if let Some(prev) = &self.preview_file {
            let _ = self.watcher.unwatch(prev);
        }

        // Watch new preview file
        if let Some(path) = file {
            self.watcher.watch(path, RecursiveMode::NonRecursive)?;
            self.preview_file = Some(path.clone());
        } else {
            self.preview_file = None;
        }

        Ok(())
    }

    /// Non-blocking check for file events with debouncing
    pub fn poll_events(&mut self) -> Option<AppEvent> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => {
                // Debounce: ignore events too close together
                let now = Instant::now();
                if now.duration_since(self.last_event_time)
                    < Duration::from_millis(FILE_EVENT_DEBOUNCE_MS)
                {
                    // Drain any additional pending events during debounce window
                    while self.receiver.try_recv().is_ok() {}
                    return None;
                }
                self.last_event_time = now;

                self.classify_event(&event)
            }
            Ok(Err(_)) | Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }

    fn classify_event(&self, event: &NotifyEvent) -> Option<AppEvent> {
        // Filter to relevant event kinds
        match event.kind {
            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {}
            _ => return None,
        }

        for path in &event.paths {
            // Check if preview file changed
            if let Some(preview) = &self.preview_file {
                if path == preview {
                    return Some(AppEvent::PreviewFileChanged);
                }
            }

            // Check if it's in current directory
            if let Some(parent) = path.parent() {
                if parent == self.current_dir {
                    return Some(AppEvent::DirectoryChanged);
                }
            }
        }

        None
    }
}

/// Timer for periodic search index refresh
pub struct RefreshTimer {
    last_refresh: Instant,
    interval: Duration,
}

impl RefreshTimer {
    /// Create a new refresh timer
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_refresh: Instant::now(),
            interval: Duration::from_secs(SEARCH_INDEX_REFRESH_SECS),
        }
    }

    /// Check if the interval has elapsed and reset if so
    pub fn check_and_reset(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.interval {
            self.last_refresh = now;
            true
        } else {
            false
        }
    }
}

impl Default for RefreshTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_key_normal_mode_quit() {
        assert!(matches!(handle_key(KeyCode::Char('q'), false), AppEvent::Quit));
        assert!(matches!(handle_key(KeyCode::Esc, false), AppEvent::Quit));
    }

    #[test]
    fn test_handle_key_normal_mode_navigation() {
        assert!(matches!(
            handle_key(KeyCode::Down, false),
            AppEvent::NavigateDown
        ));
        assert!(matches!(handle_key(KeyCode::Up, false), AppEvent::NavigateUp));
        assert!(matches!(handle_key(KeyCode::Enter, false), AppEvent::Enter));
        assert!(matches!(
            handle_key(KeyCode::Backspace, false),
            AppEvent::GoBack
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_scroll() {
        assert!(matches!(
            handle_key(KeyCode::Char('j'), false),
            AppEvent::ScrollPreviewDown
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('k'), false),
            AppEvent::ScrollPreviewUp
        ));
        assert!(matches!(
            handle_key(KeyCode::PageDown, false),
            AppEvent::ScrollPreviewPageDown
        ));
        assert!(matches!(
            handle_key(KeyCode::PageUp, false),
            AppEvent::ScrollPreviewPageUp
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_resize() {
        assert!(matches!(
            handle_key(KeyCode::Char('H'), false),
            AppEvent::ShrinkFileList
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('L'), false),
            AppEvent::GrowFileList
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_search() {
        assert!(matches!(handle_key(KeyCode::Char('/'), false), AppEvent::OpenSearch));
    }

    #[test]
    fn test_handle_key_search_mode_input() {
        assert!(matches!(
            handle_key(KeyCode::Char('a'), true),
            AppEvent::SearchInput('a')
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('z'), true),
            AppEvent::SearchInput('z')
        ));
    }

    #[test]
    fn test_handle_key_search_mode_navigation() {
        assert!(matches!(
            handle_key(KeyCode::Up, true),
            AppEvent::SearchNavigateUp
        ));
        assert!(matches!(
            handle_key(KeyCode::Down, true),
            AppEvent::SearchNavigateDown
        ));
    }

    #[test]
    fn test_handle_key_search_mode_confirm() {
        assert!(matches!(handle_key(KeyCode::Enter, true), AppEvent::SearchConfirm));
    }

    #[test]
    fn test_handle_key_search_mode_close() {
        assert!(matches!(handle_key(KeyCode::Esc, true), AppEvent::CloseSearch));
    }

    #[test]
    fn test_handle_key_search_mode_backspace() {
        assert!(matches!(handle_key(KeyCode::Backspace, true), AppEvent::SearchBackspace));
    }

    #[test]
    fn test_handle_key_unknown() {
        assert!(matches!(handle_key(KeyCode::Tab, false), AppEvent::None));
        assert!(matches!(handle_key(KeyCode::Tab, true), AppEvent::None));
    }

    #[test]
    fn test_handle_key_arrows_in_search() {
        // Arrows navigate search, not open files
        assert!(matches!(handle_key(KeyCode::Left, true), AppEvent::None));
        assert!(matches!(handle_key(KeyCode::Right, true), AppEvent::None));
    }
}
