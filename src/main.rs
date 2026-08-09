use clap::Parser;
use std::path::PathBuf;

use viewer::{app::App, tui};

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

    // Create the app first (which validates the path) before initializing the terminal
    let mut app = App::new(cli.path)?;

    let mut terminal = tui::init()?;

    // Initialize image picker AFTER entering alternate screen
    app.init_image_picker();

    let result = app.run(&mut terminal);

    tui::restore()?;

    result
}
