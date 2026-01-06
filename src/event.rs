use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub enum AppEvent {
    Quit,
    NavigateUp,
    NavigateDown,
    ScrollPreviewUp,
    ScrollPreviewDown,
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
        KeyCode::Char('j') | KeyCode::Down => AppEvent::NavigateDown,
        KeyCode::Char('k') | KeyCode::Up => AppEvent::NavigateUp,
        KeyCode::Char('J') | KeyCode::PageDown => AppEvent::ScrollPreviewDown,
        KeyCode::Char('K') | KeyCode::PageUp => AppEvent::ScrollPreviewUp,
        _ => AppEvent::None,
    }
}
