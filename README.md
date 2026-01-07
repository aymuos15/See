A TUI file viewer built with Ratatui.

![Main view](assests/main.png)
*File browser with syntax-highlighted preview*

![Search](assests/search.png)
*Fuzzy search across entire directory tree*

![Select](assests/select.png)
*Text selection in preview pane*

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
- `j`/`k` or mouse scroll - Scroll preview
- Click and drag - Select text in preview
- `Ctrl+c` - Copy selected text
- `Shift+H` / `Shift+L` - Shrink/grow file list pane
- `g` - Toggle git highlighting (shows modified files and folders in red)
- `d` - Show git diff for modified file (unified format, toggle on/off)
- `/` - Open file search (searches entire directory tree from root)
  - Type to filter files by name
  - `Up`/`Down` - Navigate results
  - `Enter` - Go to file and close search
  - `Esc` / `q` - Close search
- `f` - Symbol search (tree-sitter powered, supports Rust/Python/JS/TS/Go/HTML/CSS)

## Config

Configure in `~/.config/viewer/config.toml`:

```toml
exclude = ["*.pyc", "target/**", "node_modules/**"]
```

## Auto-Refresh

The viewer automatically watches for file changes:

- **Current directory**: Refreshes file list when files are added/deleted
- **Preview file**: Refreshes preview when the viewed file is modified
- **Search index**: Refreshes every 30 seconds to pick up new files
