# AGENTS.md - Coding Agent Guidelines for viewer

This document provides guidelines for AI coding agents working on this Rust TUI file viewer project.

## Project Overview

A terminal-based file viewer built with Ratatui, featuring syntax highlighting, git integration, and symbol search. Written in Rust (2021 edition).

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
  main.rs           # Entry point, CLI parsing
  tui.rs            # Terminal init/restore
  app/              # Core application logic (App struct, event handling, navigation)
  config/           # Configuration loading
  event/            # Event handling, file watching
  files/            # File system operations
  git/              # Git integration
  highlight/        # Syntax highlighting
  theme/            # Theming system
  ui/               # UI rendering
tests/              # Integration tests
```

## Key Dependencies

`ratatui` (TUI framework), `crossterm` (terminal), `syntect` (syntax highlighting), `anyhow` (errors), `serde`+`toml` (config), `git2` (git), `tree-sitter` (symbols), `notify` (file watching)

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
