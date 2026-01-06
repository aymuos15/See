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
    None,
}

pub fn poll_event(timeout: Duration) -> anyhow::Result<AppEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(AppEvent::None);
            }
            return Ok(handle_key(key.code));
        }
    }
    Ok(AppEvent::None)
}

fn handle_key(code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => AppEvent::Quit,
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
