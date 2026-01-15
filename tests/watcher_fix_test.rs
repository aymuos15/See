use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Test the fixed debouncing logic
#[test]
fn test_fixed_debouncing_logic() {
    use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Instant;

    const FILE_EVENT_DEBOUNCE_MS: u64 = 100;

    struct FixedWatcher {
        _watcher: RecommendedWatcher,
        receiver: mpsc::Receiver<Result<NotifyEvent, notify::Error>>,
        current_dir: PathBuf,
        last_event_time: Instant,
    }

    impl FixedWatcher {
        fn new(dir: &std::path::Path) -> anyhow::Result<Self> {
            let (tx, rx) = mpsc::channel();
            let mut watcher = notify::recommended_watcher(tx)?;
            watcher.watch(dir, RecursiveMode::NonRecursive)?;

            Ok(Self {
                _watcher: watcher,
                receiver: rx,
                current_dir: dir.to_path_buf(),
                last_event_time: Instant::now(),
            })
        }

        fn classify_event(&self, event: &NotifyEvent) -> Option<String> {
            // Filter to relevant event kinds
            match event.kind {
                EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {}
                _ => {
                    eprintln!("  [CLASSIFY] Event kind {:?} not relevant", event.kind);
                    return None;
                }
            }

            for path in &event.paths {
                if let Some(parent) = path.parent() {
                    if parent == self.current_dir {
                        eprintln!("  [CLASSIFY] ✓ Matched!");
                        return Some(format!("{:?}", event.kind));
                    }
                }
            }

            None
        }

        fn poll_with_fixed_logic(&mut self) -> Vec<String> {
            let mut results = Vec::new();

            loop {
                match self.receiver.try_recv() {
                    Ok(Ok(event)) => {
                        eprintln!("[EVENT] kind={:?}", event.kind);

                        // FIXED LOGIC: Classify first, then debounce
                        let classification = self.classify_event(&event);

                        if classification.is_none() {
                            eprintln!("  [SKIP] Not relevant, don't update debounce timer");
                            continue; // Don't update debounce timer for irrelevant events
                        }

                        // Event is relevant, now check debouncing
                        let now = Instant::now();
                        let elapsed = now.duration_since(self.last_event_time);

                        if elapsed < Duration::from_millis(FILE_EVENT_DEBOUNCE_MS) {
                            eprintln!(
                                "  [DEBOUNCED] elapsed={:?}ms < {FILE_EVENT_DEBOUNCE_MS}ms",
                                elapsed.as_millis(),
                            );
                            continue;
                        }

                        // Update debounce timer only for relevant, non-debounced events
                        self.last_event_time = now;
                        eprintln!("  [ACCEPTED] Event accepted!");
                        results.push(classification.expect("classification checked above"));
                    }
                    Ok(Err(_)) => {}
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }

            results
        }
    }

    // Create test directory
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let watch_path = temp_dir.path();

    eprintln!("\n=== Creating watcher for: {watch_path:?}");
    let mut watcher = FixedWatcher::new(watch_path).expect("Failed to create file watcher");

    thread::sleep(Duration::from_millis(200));

    eprintln!("\n=== TEST 1: Creating file (should get Create event)");
    let test_file = watch_path.join("test.txt");
    fs::write(&test_file, "hello world").expect("Failed to write test file");
    thread::sleep(Duration::from_millis(500));

    let results = watcher.poll_with_fixed_logic();
    eprintln!("Accepted events: {:?}\n", results);
    assert!(
        !results.is_empty(),
        "Should accept at least one relevant event"
    );
    assert!(
        results.iter().any(|e| e.contains("Create")),
        "Should have accepted Create event"
    );

    // Wait for debounce window to expire
    thread::sleep(Duration::from_millis(150));

    eprintln!("\n=== TEST 2: Modifying file (should get Modify event after debounce window)");
    fs::write(&test_file, "modified content").expect("Failed to modify test file");
    thread::sleep(Duration::from_millis(500));

    let results = watcher.poll_with_fixed_logic();
    eprintln!("Accepted events: {:?}\n", results);
    assert!(
        !results.is_empty(),
        "Should accept modification events after debounce window"
    );

    // Wait for debounce window to expire
    thread::sleep(Duration::from_millis(150));

    eprintln!("\n=== TEST 3: Deleting file (should get Remove event)");
    fs::remove_file(&test_file).expect("Failed to delete test file");
    thread::sleep(Duration::from_millis(500));

    let results = watcher.poll_with_fixed_logic();
    eprintln!("Accepted events: {:?}\n", results);
    assert!(!results.is_empty(), "Should accept delete event");
    assert!(
        results.iter().any(|e| e.contains("Remove")),
        "Should have accepted Remove event"
    );

    eprintln!("\n=== All tests passed!");
}
