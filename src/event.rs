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
