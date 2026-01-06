# Viewer

A TUI file viewer built with Ratatui.

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run [FILE_PATH]
```

## Controls

- `q` - Quit
- `Shift+H` / `Shift+L` - Shrink/grow file list pane
- `/` - Open search (searches entire directory tree from root)
  - Type to filter files by name
  - `Up`/`Down` - Navigate results
  - `Enter` - Go to file and close search
  - `Esc` / `q` - Close search

## Config

Exclude files using glob patterns in `~/.config/viewer/config.toml`:

```toml
exclude = ["*.pyc", "target/**", "node_modules/**"]
```
