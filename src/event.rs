use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

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
    None,
}

pub fn poll_event(timeout: Duration, search_mode: bool) -> anyhow::Result<AppEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(AppEvent::None);
            }
            return Ok(handle_key(key.code, search_mode));
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
