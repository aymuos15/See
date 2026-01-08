mod watcher;

pub use watcher::{FileWatcher, RefreshTimer};

use crate::config::{Action, KeyBindings};
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
    ToggleWrap,
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
    keys: &KeyBindings,
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
                return Ok(handle_key(key.code, key.modifiers, any_search_mode, keys));
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

fn handle_key(
    code: KeyCode,
    modifiers: KeyModifiers,
    any_search_mode: bool,
    keys: &KeyBindings,
) -> AppEvent {
    if any_search_mode {
        // Check configurable search mode bindings first
        if let Some(action) = keys.lookup_search(code, modifiers) {
            return action_to_app_event(action);
        }
        // Fall through to text input for unbound keys
        if let KeyCode::Char(c) = code {
            return AppEvent::SearchInput(c);
        }
    } else {
        // Look up the action in the keybindings
        if let Some(action) = keys.lookup_normal(code, modifiers) {
            return action_to_app_event(action);
        }
    }
    AppEvent::None
}

/// Convert a keybinding Action to an `AppEvent`
const fn action_to_app_event(action: Action) -> AppEvent {
    match action {
        Action::Quit => AppEvent::Quit,
        Action::NavigateUp => AppEvent::NavigateUp,
        Action::NavigateDown => AppEvent::NavigateDown,
        Action::Enter => AppEvent::Enter,
        Action::GoBack => AppEvent::GoBack,
        Action::ScrollPreviewUp => AppEvent::ScrollPreviewUp,
        Action::ScrollPreviewDown => AppEvent::ScrollPreviewDown,
        Action::ScrollPreviewPageUp => AppEvent::ScrollPreviewPageUp,
        Action::ScrollPreviewPageDown => AppEvent::ScrollPreviewPageDown,
        Action::ShrinkFileList => AppEvent::ShrinkFileList,
        Action::GrowFileList => AppEvent::GrowFileList,
        Action::OpenSearch => AppEvent::OpenSearch,
        Action::OpenSymbolSearch => AppEvent::OpenSymbolSearch,
        Action::ToggleGitHighlight => AppEvent::ToggleGitHighlight,
        Action::ToggleDiff => AppEvent::ToggleDiff,
        Action::ToggleThemePreview => AppEvent::ToggleThemePreview,
        Action::ToggleHelp => AppEvent::ToggleHelp,
        Action::CyclePane => AppEvent::CyclePane,
        Action::CopySelection => AppEvent::CopySelection,
        Action::SwapSplitOrientation => AppEvent::SwapSplitOrientation,
        Action::CloseActivePane => AppEvent::CloseActivePane,
        Action::ToggleFileList => AppEvent::ToggleFileList,
        Action::SplitUp => AppEvent::SplitUp,
        Action::SplitDown => AppEvent::SplitDown,
        Action::SplitLeft => AppEvent::SplitLeft,
        Action::SplitRight => AppEvent::SplitRight,
        Action::ResizeSplitLeft => AppEvent::ResizeSplitLeft,
        Action::ResizeSplitRight => AppEvent::ResizeSplitRight,
        Action::ToggleWrap => AppEvent::ToggleWrap,
        Action::SearchClose => AppEvent::CloseSearch,
        Action::SearchConfirm => AppEvent::SearchConfirm,
        Action::SearchBackspace => AppEvent::SearchBackspace,
        Action::SearchNavigateUp => AppEvent::SearchNavigateUp,
        Action::SearchNavigateDown => AppEvent::SearchNavigateDown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_MODS: KeyModifiers = KeyModifiers::NONE;

    fn default_keys() -> KeyBindings {
        KeyBindings::default()
    }

    #[test]
    fn test_handle_key_normal_mode_quit() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Char('q'), NO_MODS, false, &keys),
            AppEvent::Quit
        ));
        assert!(matches!(
            handle_key(KeyCode::Esc, NO_MODS, false, &keys),
            AppEvent::Quit
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_navigation() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Down, NO_MODS, false, &keys),
            AppEvent::NavigateDown
        ));
        assert!(matches!(
            handle_key(KeyCode::Up, NO_MODS, false, &keys),
            AppEvent::NavigateUp
        ));
        assert!(matches!(
            handle_key(KeyCode::Enter, NO_MODS, false, &keys),
            AppEvent::Enter
        ));
        assert!(matches!(
            handle_key(KeyCode::Backspace, NO_MODS, false, &keys),
            AppEvent::GoBack
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_scroll() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Char('j'), NO_MODS, false, &keys),
            AppEvent::ScrollPreviewDown
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('k'), NO_MODS, false, &keys),
            AppEvent::ScrollPreviewUp
        ));
        assert!(matches!(
            handle_key(KeyCode::PageDown, NO_MODS, false, &keys),
            AppEvent::ScrollPreviewPageDown
        ));
        assert!(matches!(
            handle_key(KeyCode::PageUp, NO_MODS, false, &keys),
            AppEvent::ScrollPreviewPageUp
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_resize() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Char('H'), NO_MODS, false, &keys),
            AppEvent::ShrinkFileList
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('L'), NO_MODS, false, &keys),
            AppEvent::GrowFileList
        ));
    }

    #[test]
    fn test_handle_key_normal_mode_search() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Char('/'), NO_MODS, false, &keys),
            AppEvent::OpenSearch
        ));
    }

    #[test]
    fn test_handle_key_search_mode_input() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Char('a'), NO_MODS, true, &keys),
            AppEvent::SearchInput('a')
        ));
        assert!(matches!(
            handle_key(KeyCode::Char('z'), NO_MODS, true, &keys),
            AppEvent::SearchInput('z')
        ));
    }

    #[test]
    fn test_handle_key_search_mode_navigation() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Up, NO_MODS, true, &keys),
            AppEvent::SearchNavigateUp
        ));
        assert!(matches!(
            handle_key(KeyCode::Down, NO_MODS, true, &keys),
            AppEvent::SearchNavigateDown
        ));
    }

    #[test]
    fn test_handle_key_search_mode_confirm() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Enter, NO_MODS, true, &keys),
            AppEvent::SearchConfirm
        ));
    }

    #[test]
    fn test_handle_key_search_mode_close() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Esc, NO_MODS, true, &keys),
            AppEvent::CloseSearch
        ));
    }

    #[test]
    fn test_handle_key_search_mode_backspace() {
        let keys = default_keys();
        assert!(matches!(
            handle_key(KeyCode::Backspace, NO_MODS, true, &keys),
            AppEvent::SearchBackspace
        ));
    }

    #[test]
    fn test_handle_key_unknown() {
        let keys = default_keys();
        // Tab now cycles panes, so it's not None
        assert!(matches!(
            handle_key(KeyCode::Tab, NO_MODS, false, &keys),
            AppEvent::CyclePane
        ));
        assert!(matches!(
            handle_key(KeyCode::Tab, NO_MODS, true, &keys),
            AppEvent::None
        ));
    }

    #[test]
    fn test_handle_key_arrows_in_search() {
        let keys = default_keys();
        // Arrows navigate search, not open files
        assert!(matches!(
            handle_key(KeyCode::Left, NO_MODS, true, &keys),
            AppEvent::None
        ));
        assert!(matches!(
            handle_key(KeyCode::Right, NO_MODS, true, &keys),
            AppEvent::None
        ));
    }

    #[test]
    fn test_custom_keybindings() {
        use crate::config::KeyBindingsConfig;

        // Test that custom keybindings override defaults
        let config = KeyBindingsConfig {
            quit: Some(vec!["ctrl+q".to_string()]),
            ..Default::default()
        };
        let keys = KeyBindings::from_config(Some(config));

        // Old 'q' binding should not trigger quit
        assert!(matches!(
            handle_key(KeyCode::Char('q'), NO_MODS, false, &keys),
            AppEvent::None
        ));
        // New Ctrl+q binding should trigger quit
        assert!(matches!(
            handle_key(KeyCode::Char('q'), KeyModifiers::CONTROL, false, &keys),
            AppEvent::Quit
        ));
    }
}
