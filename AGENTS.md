# AGENTS.md - Coding Agent Guidelines for viewer

This document provides guidelines for AI coding agents working on this Rust TUI file viewer project.

## Project Overview

A terminal-based file viewer built with Ratatui, featuring syntax highlighting, symbol search, split panes, a scrolling PDF view and a git history browser. Written in Rust (2021 edition).

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo run                      # Run with current directory
cargo run -- /path/to/dir      # Run with specific directory
cargo fmt                      # Format all code
cargo fmt --check              # Check formatting (CI)
cargo clippy                   # Run clippy lints
cargo clippy -- -D warnings    # Fail on warnings (CI)
```

## Test Commands

```bash
cargo test                           # Run all tests
cargo test test_config_default       # Run a single test by name
cargo test watcher                   # Run tests matching a pattern
cargo test --test watcher_integration_test  # Run specific integration test file
cargo test -- --nocapture            # Show stdout during tests
```

## Benchmark Commands

```bash
./benchmarks/run.sh                  # All benchmarks, compared with the baseline
./benchmarks/run.sh render           # One target: render, highlight or parse
./benchmarks/run.sh --quick          # Fewer samples, for a fast loop
./benchmarks/run.sh --save           # Also record the run in benchmarks/history/
python3 benchmarks/collect.py promote  # Make the last run the new baseline
```

See `benchmarks/README.md` for what is measured and how to read the output.
Anything under ±10% between runs is noise on a developer machine.

## Code Style Guidelines

### Formatting (rustfmt.toml)

- **Max line width**: 100 characters
- **Indentation**: 4 spaces (no tabs)
- **Imports**: Auto-reordered by rustfmt

Run `cargo fmt` before committing.

### Linting (Clippy)

This project uses **very strict** clippy configuration with `warn` on:
`clippy::all`, `clippy::pedantic`, `clippy::nursery`, `clippy::suspicious`, `clippy::complexity`, `clippy::style`, `clippy::perf`, `clippy::correctness`

**Prohibited patterns** (will trigger warnings):
- `unwrap()` - Use `?` operator or `expect("reason")`
- `panic!()` - Use `anyhow::bail!()` or return `Result`
- `todo!()` / `unimplemented!()` - Complete implementations

### Error Handling

```rust
// GOOD: Use anyhow::Result with ? operator
fn load_file(path: &Path) -> anyhow::Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

// GOOD: Use anyhow::bail! for custom errors
if !path.exists() {
    anyhow::bail!("Path does not exist: {}", path.display());
}

// BAD: Avoid unwrap()
let content = fs::read_to_string(path).unwrap(); // Don't do this
```

### Naming Conventions

- **Functions/variables**: `snake_case`
- **Types/structs/enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Import Style

```rust
// Standard library first
use std::fs;
use std::path::PathBuf;

// External crates second (blank line between)
use anyhow::Result;
use ratatui::prelude::Rect;

// Internal crates last (blank line between)
use crate::config::Config;
```

### Module Organization

- Each major feature has its own subdirectory with `mod.rs`
- Public re-exports in `mod.rs`: `pub use submodule::Type;`

```rust
// In mod.rs
mod watcher;
pub use watcher::{FileWatcher, RefreshTimer};
```

### Testing

- Unit tests go inline in source files within `#[cfg(test)]` modules
- Integration tests go in `tests/` directory
- Use `tempfile` crate for tests needing filesystem

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_behavior() {
        let config = Config::default();
        assert!(!config.is_excluded(Path::new("any/path")));
    }
}
```

## Project Structure

```
src/
  lib.rs            # Crate root: every module, so benches and tests can call in
  main.rs           # Entry point, CLI parsing
  tui.rs            # Terminal init/restore
  app/              # Core application logic (App struct, event handling, navigation)
    split.rs        # Split pane management (SplitLayout, Pane, SplitNode tree)
    pdf.rs          # Continuous PDF scroll state: page stack, slices, caching
    git_mode.rs     # Git mode state: commit list, selection, diff scroll
  config/           # Configuration loading
  event/            # Event handling, file watching
  files/            # File system operations, tree building, tree-sitter symbols
  git/              # Reading commits by shelling out to git, and parsing it
  highlight/        # Syntax highlighting
    markdown_table.rs # Aligns markdown pipe tables into columns
  theme/            # Theming system
  ui/               # UI rendering
    pane.rs         # Individual pane rendering
    pdf.rs          # Draws the PDF page column, centered and sliced to fit
    git_mode.rs     # Draws the commit list, commit summary and diff
    popup.rs        # Shared popup panel: surface, query line, list
    indent.rs       # Indent guides in the preview
    tab_bar.rs      # Tab bar for split panes
    layout.rs       # Layout calculation with dividers
  worker.rs         # Background thread: highlighting, symbols, images, PDF
                    # pages, git log and diffs, line counts
benches/            # Benchmark targets, with generated fixtures
benchmarks/         # Recorded results, the runner and the comparison tool
tests/              # Integration tests
```

The program lives in `src/lib.rs`, not `src/main.rs`: a binary crate exposes
nothing, so benchmarks could not call into it. `main.rs` is only argument
parsing and a call to `App::run`. Everything is `pub` for that reason, which is
also why `lib.rs` switches off the lints that police a published API — they
would demand `#[must_use]` and `# Errors` on ~80 internal functions.

## Performance Notes

Anything that reads or parses whole files belongs on the background worker,
not the UI thread. Syntax highlighting a mid-size file costs tens of
milliseconds, which is far too slow to run between keystrokes; the preview
shows plain text immediately and `WorkerResponse::FileHighlighted` swaps in
colors, cached per path. Counting lines for the file tree works the same way.
Rendering runs every frame, so per-frame work must stay proportional to what is
on screen rather than to file size. The `render/*` benchmarks exist to check
exactly that: each holds the viewport fixed and varies the subject, so a cost
that scales with the file or the directory shows up as a widening gap.

Some frames are expensive whatever we do: a PDF row of scroll re-encodes an
image for the terminal's graphics protocol. Input therefore arrives faster than
frames can be drawn, so `App::drain_pending_input` applies the whole queued
backlog before drawing again; without it a held key falls seconds behind.

## Key Dependencies

`ratatui` (TUI framework), `crossterm` (terminal), `syntect` (syntax highlighting), `anyhow` (errors), `serde`+`toml` (config), `tree-sitter` 0.26 (symbols), `notify` (file watching), `ratatui-image`+`pdfium-render` (images and PDFs)

Git support shells out to the `git` binary rather than linking a library: the
viewer only reads history, so `src/git/` runs `log` and `show` and parses their
output. Keep it that way unless something needs to write.

Grammars: rust, python, javascript, typescript, go, html, css, yaml, toml,
c/c++, cuda, markdown, latex. The runtime must stay at 0.25+ — several
grammars are ABI 15, which 0.24 rejects. Image and syntect features are
trimmed in `Cargo.toml` to keep build times down; check there before adding
a dependency's default features back.

## Common Patterns

### Creating new features

1. Add module in appropriate directory
2. Export public items in parent `mod.rs`
3. Add unit tests in `#[cfg(test)]` block
4. Run `cargo fmt && cargo clippy` before committing

### Handling optional values

```rust
let theme = config_file.theme.unwrap_or_default();
let config_path = dirs::config_dir()?.join("viewer/config.toml");
```

### Function Signatures

- Use `&self` for read-only methods, `&mut self` for modifying methods
- Use `const fn` when possible for simple getters
- Prefer references over owned values in parameters
- Add doc comments (`///`) to public items
