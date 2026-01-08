# See: A TUI file viewer.

![Main view](assets/main.png)

*File browser with syntax-highlighted preview*

![Git diff view](assets/diff_and_pane_git_notif.png)

*Git highlighting with visual diff - modified files marked with ● and colored text*

![Search](assets/search.png)

*Fuzzy search across entire directory tree*

![Select](assets/select.png)

*Text selection in preview pane*

![Split Panes](assets/tabs_and_panes.png)

*Split panes with tab bar for multiple files*

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run [FILE_PATH]
```

## Controls

- `q` or `Esc` - Quit
- `j`/`k` or mouse scroll - Scroll preview
- `PgUp`/`PgDn` - Page up/down in preview
- Click and drag - Select text in preview
- `Ctrl+c` - Copy selected text
- `h`/`l` or `Left`/`Right` - Navigate back/enter directory
- `Shift+H` / `Shift+L` - Shrink/grow file list pane
- `g` - Toggle git highlighting (shows modified files with ● dot indicator and colored text)
- `d` - Toggle unified git diff view for the current file
- `t` - Toggle theme preview (cycle through themes)
- `?` - Show keyboard shortcuts help overlay
- `/` - Open file search (searches entire directory tree from root)
  - Type to filter files by name
  - `Up`/`Down` - Navigate results
  - `Enter` - Go to file and close search
  - `Esc` - Close search
- `f` - Symbol search (tree-sitter powered, supports Rust/Python/JS/TS/Go/HTML/CSS)
  - Type to filter symbols
  - `Up`/`Down` - Navigate results
  - `Enter` - Go to symbol and close search
  - `Esc` - Close search

### Split Panes

- `Alt+Up/Down/Left/Right` - Split pane in that direction
- `Alt+q` - Close active pane
- `Alt+s` - Swap split orientation (horizontal/vertical)
- `Alt+p` - Toggle file list visibility in split mode
- `Alt+h` / `Alt+l` - Resize split (shrink/grow active pane)
- `Tab` - Cycle to next pane

## Configuration

Configuration file location: `~/.config/viewer/config.toml`

See [`config.example.toml`](config.example.toml) for a complete example with all options.

### General Options

```toml
# File/directory exclusion patterns (glob syntax)
exclude = ["*.pyc", "target/**", "node_modules/**", ".git"]

# Width of divider lines between split panes (1 = thin line, 2+ = solid block)
divider_width = 1
```

### Theme Configuration

```toml
[theme]
# Option 1: Use a Helix theme by name
helix_theme = "catppuccin_mocha"

# Option 2: Define custom colors (hex format)
# bg_main = "#1a1a1a"
# bg_darker = "#0f0f0f"
# bg_selected = "#2a2a2a"
# bg_search = "#ff6600"
# bg_selection = "#ffff00"
# fg_text = "#e0e0e0"
# fg_selected = "#00ff00"
# fg_dim = "#808080"
# fg_folder = "#00ccff"
# fg_modified = "#ff9900"
# border = "#666666"
# line_num = "#666666"
```

### Key Bindings

All key bindings are customizable. Keys can be specified as:
- Simple keys: `"q"`, `"j"`, `"/"`, `"?"`
- Special keys: `"enter"`, `"esc"`, `"backspace"`, `"tab"`, `"space"`
- Arrow/page keys: `"up"`, `"down"`, `"pageup"`, `"pagedown"`
- With modifiers: `"ctrl+c"`, `"alt+s"`, `"shift+h"`

```toml
[keys]
# Multiple keys can be bound to the same action
quit = ["q", "esc"]
navigate_up = ["up"]
navigate_down = ["down"]
enter = ["enter", "l", "right"]
go_back = ["backspace", "h", "left"]
scroll_preview_up = ["k"]
scroll_preview_down = ["j"]
open_search = ["/"]
toggle_help = ["?"]
copy_selection = ["ctrl+c"]

# Split pane controls
split_up = ["alt+up"]
split_down = ["alt+down"]
close_active_pane = ["alt+q"]
cycle_pane = ["tab"]
```

See [`config.example.toml`](config.example.toml) for all available key bindings.

## Auto-Refresh

The viewer automatically watches for file changes:

- **Current directory**: Refreshes file list when files are added/deleted
- **Preview file**: Refreshes preview when the viewed file is modified
- **Search index**: Refreshes every 30 seconds to pick up new files

## Development

### Pre-commit Hooks

Install pre-commit hooks to run formatting and linting checks before each commit:

```bash
pip install pre-commit
pre-commit install
```

This will run `cargo fmt`, `cargo clippy`, and other checks automatically on commit.
