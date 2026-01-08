mod watcher;

pub use watcher::{FileWatcher, RefreshTimer};

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use std::time::Duration;

/// Application events triggered by user input or system events.
#[derive(Debug)]
#[allow(dead_code)]
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
    OpenSymbolSearch,
    CloseSymbolSearch,
    SymbolSearchInput(char),
    SymbolSearchBackspace,
    SymbolSearchNavigateUp,
    SymbolSearchNavigateDown,
    SymbolSearchConfirm,
    ToggleGitHighlight,
    ToggleDiff,
    ToggleThemePreview,
    ToggleHelp,
    DirectoryChanged,
    PreviewFileChanged,
    SearchIndexRefreshTimer,
    // Mouse selection events
    MouseDown { column: u16, row: u16 },
    MouseDrag { column: u16, row: u16 },
    MouseUp { column: u16, row: u16 },
    CopySelection,
    SplitHorizontal,
    SplitVertical,
    SplitUp,
    SplitDown,
    SplitLeft,
    SplitRight,
    SwapSplitOrientation,
    CloseActivePane,
    CyclePane,
    ToggleFileList,
    ResizeSplitLeft,
    ResizeSplitRight,
    None,
}

/// Polls for the next application event with a timeout.
///
/// Checks file watcher events, refresh timer, and keyboard/mouse input.
pub fn poll_event(
    timeout: Duration,
    any_search_mode: bool,
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
                // Handle Ctrl+c for copy selection
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(AppEvent::CopySelection);
                }
                return Ok(handle_key(key.code, key.modifiers, any_search_mode));
            }
            Event::Mouse(mouse) => {
                return Ok(match mouse.kind {
                    MouseEventKind::ScrollDown => AppEvent::ScrollPreviewDown,
                    MouseEventKind::ScrollUp => AppEvent::ScrollPreviewUp,
                    MouseEventKind::Down(MouseButton::Left) => AppEvent::MouseDown {
                        column: mouse.column,
                        row: mouse.row,
                    },
                    MouseEventKind::Drag(MouseButton::Left) => AppEvent::MouseDrag {
                        column: mouse.column,
                        row: mouse.row,
                    },
                    MouseEventKind::Up(MouseButton::Left) => AppEvent::MouseUp {
                        column: mouse.column,
                        row: mouse.row,
                    },
                    _ => AppEvent::None,
                });
            }
            _ => {}
        }
    }
    Ok(AppEvent::None)
}

#[allow(clippy::missing_const_for_fn)]
fn handle_key(code: KeyCode, modifiers: KeyModifiers, any_search_mode: bool) -> AppEvent {
    if any_search_mode {
        match code {
            KeyCode::Esc => AppEvent::CloseSearch, // Works for both file and symbol search
            KeyCode::Enter => AppEvent::SearchConfirm, // Routes to correct handler in app
            KeyCode::Backspace => AppEvent::SearchBackspace, // Routes to correct handler in app
            KeyCode::Up => AppEvent::SearchNavigateUp, // Routes to correct handler in app
            KeyCode::Down => AppEvent::SearchNavigateDown, // Routes to correct handler in app
            KeyCode::Char(c) => AppEvent::SearchInput(c), // Routes to correct handler in app
            _ => AppEvent::None,
        }
    } else {
        // Alt-based split controls
        if modifiers.contains(KeyModifiers::ALT) {
            match code {
                KeyCode::Char('s') => return AppEvent::SwapSplitOrientation,
                KeyCode::Char('q') => return AppEvent::CloseActivePane,
                KeyCode::Char('p') => return AppEvent::ToggleFileList,
                KeyCode::Up => return AppEvent::SplitUp,
                KeyCode::Down => return AppEvent::SplitDown,
                KeyCode::Left => return AppEvent::SplitLeft,
                KeyCode::Right => return AppEvent::SplitRight,
                KeyCode::Char('h') => return AppEvent::ResizeSplitLeft,
                KeyCode::Char('l') => return AppEvent::ResizeSplitRight,
                _ => {}
            }
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => AppEvent::Quit,
            KeyCode::Tab => AppEvent::CyclePane,
            KeyCode::Char('/') => AppEvent::OpenSearch,
            KeyCode::Char('f') => AppEvent::OpenSymbolSearch,
            KeyCode::Char('g') => AppEvent::ToggleGitHighlight,
            KeyCode::Char('d') => AppEvent::ToggleDiff,
            KeyCode::Char('t') => AppEvent::ToggleThemePreview,
            KeyCode::Char('?') => AppEvent::ToggleHelp,
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

#[cfg(test)]
mod tests {
    use super::*;

    const NO_MODS: KeyModifiers = KeyModifiers::NONE;

    #[test]
    fn test_handle_key_normal_mode_quit() {
        assert!(matches!(
            handle_key(KeyCode::Char('q'), NO_MODS, false),
            AppEvent::Quit
        ));
        assert!(matches!(
            handle_key(KeyCode::Esc, NO_MODS, false),
            AppEvent::Quit
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_navigation() {
        assert!(matches!(
            handle_key(KeyCode::Down, NO_MODS, false),
            AppEvent::NavigateDown
        ));
        assert!(matches!(
            handle_key(KeyCode::Up, NO_MODS, false),
            AppEvent::NavigateUp
        ));
        assert!(matches!(
            handle_key(KeyCode::Enter, NO_MODS, false),
            AppEvent::Enter
        ));
        assert!(matches!(
            handle_key(KeyCode::Backspace, NO_MODS, false),
            AppEvent::GoBack
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_scroll() {
        assert!(matches!(
            handle_key(KeyCode::Char('j'), NO_MODS, false),
            AppEvent::ScrollPreviewDown
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('k'), NO_MODS, false),
            AppEvent::ScrollPreviewUp
        ));
        assert!(matches!(
            handle_key(KeyCode::PageDown, NO_MODS, false),
            AppEvent::ScrollPreviewPageDown
        ));
        assert!(matches!(
            handle_key(KeyCode::PageUp, NO_MODS, false),
            AppEvent::ScrollPreviewPageUp
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_resize() {
        assert!(matches!(
            handle_key(KeyCode::Char('H'), NO_MODS, false),
            AppEvent::ShrinkFileList
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('L'), NO_MODS, false),
            AppEvent::GrowFileList
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_search() {
        assert!(matches!(
            handle_key(KeyCode::Char('/'), NO_MODS, false),
            AppEvent::OpenSearch
        ));
    }

    #[test]
    fn test_handle_key_search_mode_input() {
        assert!(matches!(
            handle_key(KeyCode::Char('a'), NO_MODS, true),
            AppEvent::SearchInput('a')
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('z'), NO_MODS, true),
            AppEvent::SearchInput('z')
        ));
    }

    #[test]
    fn test_handle_key_search_mode_navigation() {
        assert!(matches!(
            handle_key(KeyCode::Up, NO_MODS, true),
            AppEvent::SearchNavigateUp
        ));
        assert!(matches!(
            handle_key(KeyCode::Down, NO_MODS, true),
            AppEvent::SearchNavigateDown
        ));
    }

    #[test]
    fn test_handle_key_search_mode_confirm() {
        assert!(matches!(
            handle_key(KeyCode::Enter, NO_MODS, true),
            AppEvent::SearchConfirm
        ));
    }

    #[test]
    fn test_handle_key_search_mode_close() {
        assert!(matches!(
            handle_key(KeyCode::Esc, NO_MODS, true),
            AppEvent::CloseSearch
        ));
    }

    #[test]
    fn test_handle_key_search_mode_backspace() {
        assert!(matches!(
            handle_key(KeyCode::Backspace, NO_MODS, true),
            AppEvent::SearchBackspace
        ));
    }

    #[test]
    fn test_handle_key_unknown() {
        // Tab now cycles panes, so it's not None
        assert!(matches!(
            handle_key(KeyCode::Tab, NO_MODS, false),
            AppEvent::CyclePane
        ));
        assert!(matches!(
            handle_key(KeyCode::Tab, NO_MODS, true),
            AppEvent::None
        ));
    }

    #[test]
    fn test_handle_key_arrows_in_search() {
        // Arrows navigate search, not open files
        assert!(matches!(
            handle_key(KeyCode::Left, NO_MODS, true),
            AppEvent::None
        ));
        assert!(matches!(
            handle_key(KeyCode::Right, NO_MODS, true),
            AppEvent::None
        ));
    }
}
