use crossterm::{
    cursor::{Hide, Show, MoveTo},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use ratatui::prelude::*;
use std::io::{stdout, Stdout};
use std::panic;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> anyhow::Result<Tui> {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Hide
    )?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    Ok(terminal)
}

pub fn restore() -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), Show, LeaveAlternateScreen)?;
    Ok(())
}
