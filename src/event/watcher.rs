use crate::constants::{FILE_EVENT_DEBOUNCE_MS, SEARCH_INDEX_REFRESH_SECS};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use super::AppEvent;

/// Minimum interval between emitting the same event type (ms)
const EVENT_COOLDOWN_MS: u64 = 250;

/// File watcher for current directory and preview file
pub struct FileWatcher {
    debouncer: Debouncer<RecommendedWatcher>,
    receiver: Receiver<Vec<DebouncedEvent>>,
    current_dir: PathBuf,
    preview_file: Option<PathBuf>,
    /// Last time we emitted a PreviewFileChanged event
    last_preview_event: Instant,
    /// Last time we emitted a DirectoryChanged event
    last_directory_event: Instant,
}

impl FileWatcher {
    /// Create a new file watcher for the given directory
    pub fn new(current_dir: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        // Create debouncer with our timeout
        let debouncer = new_debouncer(
            Duration::from_millis(FILE_EVENT_DEBOUNCE_MS),
            move |result| {
                if let Ok(events) = result {
                    let _ = tx.send(events);
                }
            },
        )?;

        // Use a past time so first event always fires
        let past = Instant::now() - Duration::from_secs(10);

        let mut fw = Self {
            debouncer,
            receiver: rx,
            current_dir: PathBuf::new(),
            preview_file: None,
            last_preview_event: past,
            last_directory_event: past,
        };

        fw.watch_directory(current_dir)?;

        Ok(fw)
    }

    /// Watch a new directory (non-recursive)
    pub fn watch_directory(&mut self, dir: &Path) -> anyhow::Result<()> {
        // Unwatch previous directory
        if self.current_dir.exists() {
            let _ = self.debouncer.watcher().unwatch(&self.current_dir);
        }

        // Watch new directory (non-recursive for current dir)
        self.debouncer
            .watcher()
            .watch(dir, RecursiveMode::NonRecursive)?;
        self.current_dir = dir.to_path_buf();

        Ok(())
    }

    /// Watch a preview file for changes
    pub fn watch_preview_file(&mut self, file: Option<&PathBuf>) -> anyhow::Result<()> {
        // Unwatch previous preview file
        if let Some(prev) = &self.preview_file {
            let _ = self.debouncer.watcher().unwatch(prev);
        }

        // Watch new preview file
        if let Some(path) = file {
            self.debouncer
                .watcher()
                .watch(path, RecursiveMode::NonRecursive)?;
            self.preview_file = Some(path.clone());
        } else {
            self.preview_file = None;
        }

        Ok(())
    }

    /// Non-blocking check for file events with cooldown to prevent event storms
    pub fn poll_events(&mut self) -> Option<AppEvent> {
        let cooldown = Duration::from_millis(EVENT_COOLDOWN_MS);
        let now = Instant::now();

        // Drain all pending batches
        let mut has_preview_event = false;
        let mut has_directory_event = false;

        loop {
            match self.receiver.try_recv() {
                Ok(events) => {
                    for event in events {
                        match self.classify_event(&event) {
                            Some(AppEvent::PreviewFileChanged) => has_preview_event = true,
                            Some(AppEvent::DirectoryChanged) => has_directory_event = true,
                            _ => {}
                        }
                    }
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }

        // Return at most one event, respecting cooldown
        if has_directory_event && now.duration_since(self.last_directory_event) >= cooldown {
            self.last_directory_event = now;
            return Some(AppEvent::DirectoryChanged);
        }

        if has_preview_event && now.duration_since(self.last_preview_event) >= cooldown {
            self.last_preview_event = now;
            return Some(AppEvent::PreviewFileChanged);
        }

        None
    }

    fn classify_event(&self, event: &DebouncedEvent) -> Option<AppEvent> {
        let path = &event.path;

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
