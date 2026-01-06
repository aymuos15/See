# Viewer

A TUI file viewer built with Ratatui.

![Main view](assests/main.png)
*File browser with syntax-highlighted preview*

![Search](assests/search.png)
*Fuzzy search across entire directory tree*

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
- `j`/`k` or mouse scroll - Scroll preview (hold `Shift` to select text)
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

## Auto-Refresh

The viewer automatically watches for file changes:

- **Current directory**: Refreshes file list when files are added/deleted
- **Preview file**: Refreshes preview when the viewed file is modified
- **Search index**: Refreshes every 30 seconds to pick up new files
