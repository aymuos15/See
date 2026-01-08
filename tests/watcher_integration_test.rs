use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Test that notify v7.0 works with mpsc::Sender directly
#[test]
fn test_notify_with_channel_sender() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (tx, rx) = mpsc::channel::<Result<NotifyEvent, notify::Error>>();

    // This is the pattern used in the viewer code
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    // Watch the directory
    watcher
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .unwrap();

    // Give watcher time to initialize
    thread::sleep(Duration::from_millis(100));

    // Create a file in the watched directory
    let test_file = watch_path.join("test.txt");
    fs::write(&test_file, "hello").unwrap();

    // Wait for events
    thread::sleep(Duration::from_millis(500));

    // Try to receive events
    let mut received_events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        received_events.push(event);
    }

    println!("Received {} events", received_events.len());
    for (i, event) in received_events.iter().enumerate() {
        println!("Event {}: {:?}", i, event);
    }

    // We should have received at least one event
    assert!(
        !received_events.is_empty(),
        "Should have received file system events"
    );
}

/// Test the exact watcher pattern from viewer codebase
#[test]
fn test_viewer_watcher_pattern() {
    use std::sync::mpsc::{self, Receiver, TryRecvError};

    struct TestFileWatcher {
        watcher: RecommendedWatcher,
        receiver: Receiver<Result<NotifyEvent, notify::Error>>,
        current_dir: PathBuf,
    }

    impl TestFileWatcher {
        fn new(current_dir: &Path) -> anyhow::Result<Self> {
            let (tx, rx) = mpsc::channel();
            let watcher = notify::recommended_watcher(tx)?;

            let mut fw = Self {
                watcher,
                receiver: rx,
                current_dir: PathBuf::new(),
            };

            fw.watch_directory(current_dir)?;
            Ok(fw)
        }

        fn watch_directory(&mut self, dir: &Path) -> anyhow::Result<()> {
            if self.current_dir.exists() {
                let _ = self.watcher.unwatch(&self.current_dir);
            }
            self.watcher.watch(dir, RecursiveMode::NonRecursive)?;
            self.current_dir = dir.to_path_buf();
            Ok(())
        }

        fn poll_events(&mut self) -> Vec<NotifyEvent> {
            let mut events = Vec::new();
            loop {
                match self.receiver.try_recv() {
                    Ok(Ok(event)) => events.push(event),
                    Ok(Err(e)) => {
                        eprintln!("Error event: {:?}", e);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("Channel disconnected!");
                        break;
                    }
                }
            }
            events
        }
    }

    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let mut watcher = TestFileWatcher::new(&watch_path).unwrap();

    // Give watcher time to initialize
    thread::sleep(Duration::from_millis(100));

    println!("Creating test file...");
    let test_file = watch_path.join("test.txt");
    fs::write(&test_file, "hello").unwrap();

    // Wait for events
    thread::sleep(Duration::from_millis(500));

    let events = watcher.poll_events();
    println!("Received {} events", events.len());
    for (i, event) in events.iter().enumerate() {
        println!(
            "Event {}: kind={:?}, paths={:?}",
            i, event.kind, event.paths
        );
    }

    assert!(
        !events.is_empty(),
        "Should have received events for file creation"
    );
}

/// Test event classification logic
#[test]
fn test_event_classification() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();
    let file_in_dir = current_dir.join("file.txt");

    // Check if parent matches
    if let Some(parent) = file_in_dir.parent() {
        println!("File parent: {:?}", parent);
        println!("Current dir: {:?}", current_dir);
        println!("Parents match: {}", parent == current_dir);
    }

    // Test different event kinds
    let test_kinds = vec![
        EventKind::Create(notify::event::CreateKind::File),
        EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Any,
        )),
        EventKind::Remove(notify::event::RemoveKind::File),
        EventKind::Access(notify::event::AccessKind::Close(
            notify::event::AccessMode::Write,
        )),
    ];

    for kind in test_kinds {
        let event = NotifyEvent {
            kind,
            paths: vec![file_in_dir.clone()],
            attrs: Default::default(),
        };

        let should_match = matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
        );

        println!(
            "EventKind: {:?}, should match: {}",
            event.kind, should_match
        );
    }
}

/// Test debouncing behavior
#[test]
fn test_debouncing() {
    use std::time::Instant;

    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let (tx, rx) = mpsc::channel::<Result<NotifyEvent, notify::Error>>();
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    watcher
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .unwrap();

    thread::sleep(Duration::from_millis(100));

    // Rapidly create multiple files
    println!("Creating files rapidly...");
    let start = Instant::now();
    for i in 0..5 {
        let test_file = watch_path.join(format!("test{}.txt", i));
        fs::write(&test_file, format!("content {}", i)).unwrap();
        println!("Created file {} at {:?}", i, start.elapsed());
    }

    // Wait for all events
    thread::sleep(Duration::from_millis(1000));

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Ok(e) = event {
            events.push(e);
        }
    }

    println!("Received {} events for 5 file creations", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("Event {}: {:?}", i, event);
    }

    // We should get multiple events since files are created with different names
    assert!(
        !events.is_empty(),
        "Should have received at least some events"
    );
}

/// Test watching file modifications
#[test]
fn test_file_modification() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();
    let test_file = watch_path.join("test.txt");

    // Create file first
    fs::write(&test_file, "initial content").unwrap();

    let (tx, rx) = mpsc::channel::<Result<NotifyEvent, notify::Error>>();
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    // Watch the specific file
    watcher
        .watch(&test_file, RecursiveMode::NonRecursive)
        .unwrap();

    thread::sleep(Duration::from_millis(100));

    println!("Modifying file...");
    fs::write(&test_file, "modified content").unwrap();

    thread::sleep(Duration::from_millis(500));

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Ok(e) = event {
            println!("Event: kind={:?}, paths={:?}", e.kind, e.paths);
            events.push(e);
        }
    }

    println!("Received {} events for file modification", events.len());

    assert!(
        !events.is_empty(),
        "Should have received events for file modification"
    );
}
