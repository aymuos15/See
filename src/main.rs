use clap::Parser;
use std::path::PathBuf;

mod app;
mod event;
mod files;
mod highlight;
mod theme;
mod tui;
mod ui;

#[derive(Parser)]
#[command(name = "viewer")]
#[command(about = "A Helix-inspired file viewer with syntax highlighting")]
struct Cli {
    /// Directory to view (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut terminal = tui::init()?;

    let result = app::App::new(cli.path)?.run(&mut terminal);

    tui::restore()?;

    result
}
