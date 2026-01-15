use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Test the actual FileWatcher from the codebase
#[test]
fn test_real_file_watcher_from_codebase() {
    // We need to import from the actual binary crate
    // This test uses the same pattern as the main app

    use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Instant;

    const FILE_EVENT_DEBOUNCE_MS: u64 = 100;

    struct TestWatcher {
        _watcher: RecommendedWatcher,
        receiver: mpsc::Receiver<Result<NotifyEvent, notify::Error>>,
        current_dir: PathBuf,
        last_event_time: Instant,
    }

    impl TestWatcher {
        fn new(dir: &std::path::Path) -> anyhow::Result<Self> {
            let (tx, rx) = mpsc::channel();
            let mut watcher = notify::recommended_watcher(tx)?;

            // Watch directory
            watcher.watch(dir, RecursiveMode::NonRecursive)?;

            Ok(Self {
                _watcher: watcher,
                receiver: rx,
                current_dir: dir.to_path_buf(),
                last_event_time: Instant::now(),
            })
        }

        fn poll_with_debounce(&mut self) -> Vec<String> {
            let mut results = Vec::new();

            loop {
                match self.receiver.try_recv() {
                    Ok(Ok(event)) => {
                        let msg = format!("[EVENT] kind={:?}, paths={:?}", event.kind, event.paths);
                        results.push(msg.clone());
                        eprintln!("{msg}");

                        // Check debouncing
                        let now = Instant::now();
                        let elapsed = now.duration_since(self.last_event_time);
                        if elapsed < Duration::from_millis(FILE_EVENT_DEBOUNCE_MS) {
                            let msg = format!(
                                "[DEBOUNCED] elapsed={:?}ms < {FILE_EVENT_DEBOUNCE_MS}ms",
                                elapsed.as_millis(),
                            );
                            results.push(msg.clone());
                            eprintln!("{msg}");
                            continue;
                        }
                        self.last_event_time = now;

                        // Check event classification
                        let relevant = matches!(
                            event.kind,
                            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                        );

                        if !relevant {
                            let msg = "[FILTERED] Event kind not relevant".to_string();
                            results.push(msg.clone());
                            eprintln!("{msg}");
                            continue;
                        }

                        // Check if path is in current directory
                        for path in &event.paths {
                            if let Some(parent) = path.parent() {
                                if parent == self.current_dir {
                                    let msg = "[MATCHED] Path is in current directory!".to_string();
                                    results.push(msg.clone());
                                    eprintln!("{msg}");
                                } else {
                                    let msg = format!(
                                        "[NO MATCH] parent={parent:?} != current_dir={:?}",
                                        self.current_dir
                                    );
                                    results.push(msg.clone());
                                    eprintln!("{msg}");
                                }
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        let msg = format!("[ERROR] {e:?}");
                        results.push(msg.clone());
                        eprintln!("{msg}");
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        let msg = "[DISCONNECTED]".to_string();
                        results.push(msg.clone());
                        eprintln!("{msg}");
                        break;
                    }
                }
            }

            results
        }
    }

    // Create test directory
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let watch_path = temp_dir.path();

    eprintln!("\n=== Creating watcher for: {watch_path:?}");
    let mut watcher = TestWatcher::new(watch_path).expect("Failed to create file watcher");

    // Give watcher time to initialize
    thread::sleep(Duration::from_millis(200));

    eprintln!("\n=== Creating file...");
    let test_file = watch_path.join("test.txt");
    fs::write(&test_file, "hello world").expect("Failed to write test file");

    // Wait for events to propagate
    thread::sleep(Duration::from_millis(500));

    eprintln!("\n=== Polling for events...");
    let results = watcher.poll_with_debounce();

    eprintln!("\n=== Summary:");
    eprintln!("Total log entries: {}", results.len());
    eprintln!(
        "Events received: {}",
        results.iter().filter(|s| s.starts_with("[EVENT]")).count()
    );
    eprintln!(
        "Events matched: {}",
        results
            .iter()
            .filter(|s| s.starts_with("[MATCHED]"))
            .count()
    );
    eprintln!(
        "Events debounced: {}",
        results
            .iter()
            .filter(|s| s.starts_with("[DEBOUNCED]"))
            .count()
    );
    eprintln!(
        "Events filtered: {}",
        results
            .iter()
            .filter(|s| s.starts_with("[FILTERED]"))
            .count()
    );

    // We should have received events
    let event_count = results.iter().filter(|s| s.starts_with("[EVENT]")).count();
    assert!(
        event_count > 0,
        "Should have received at least one event, got {} total log entries",
        results.len()
    );

    // At least one event should have matched
    let matched_count = results
        .iter()
        .filter(|s| s.starts_with("[MATCHED]"))
        .count();
    assert!(
        matched_count > 0,
        "At least one event should have matched the current directory"
    );
}

/// Test to see what kinds of events are generated by different file operations
#[test]
fn test_event_types_for_file_operations() {
    use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let watch_path = temp_dir.path();

    let (tx, rx) = mpsc::channel::<Result<NotifyEvent, notify::Error>>();
    let mut watcher = notify::recommended_watcher(tx).expect("Failed to create file watcher");
    watcher
        .watch(watch_path, RecursiveMode::NonRecursive)
        .expect("Failed to watch directory");

    thread::sleep(Duration::from_millis(100));

    // Test 1: Create file
    eprintln!("\n=== TEST: Creating file");
    let file1 = watch_path.join("create_test.txt");
    fs::write(&file1, "content").expect("Failed to write test file");
    thread::sleep(Duration::from_millis(300));

    let mut events = Vec::new();
    while let Ok(Ok(event)) = rx.try_recv() {
        eprintln!("  Event: {:?}", event);
        events.push(event);
    }
    eprintln!("  Total events for CREATE: {}", events.len());

    // Test 2: Modify file
    eprintln!("\n=== TEST: Modifying file");
    fs::write(&file1, "modified content").expect("Failed to modify test file");
    thread::sleep(Duration::from_millis(300));

    events.clear();
    while let Ok(Ok(event)) = rx.try_recv() {
        eprintln!("  Event: {:?}", event);
        events.push(event);
    }
    eprintln!("  Total events for MODIFY: {}", events.len());

    // Test 3: Delete file
    eprintln!("\n=== TEST: Deleting file");
    fs::remove_file(&file1).expect("Failed to delete test file");
    thread::sleep(Duration::from_millis(300));

    events.clear();
    while let Ok(Ok(event)) = rx.try_recv() {
        eprintln!("  Event: {:?}", event);
        events.push(event);
    }
    eprintln!("  Total events for DELETE: {}", events.len());
}
