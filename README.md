# See: A TUI file viewer.

![Main view](assets/main.png)

*File browser with syntax-highlighted preview*

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

Press `?` in the app to see all keyboard shortcuts, or see [`config.example.toml`](config.example.toml) for the full list of configurable key bindings.

## Configuration

Configuration file location: `~/.config/viewer/config.toml`

See [`config.example.toml`](config.example.toml) for all available options including:
- File/directory exclusion patterns
- Theme configuration (Helix themes or custom colors)
- Customizable key bindings
- Preview options such as `wrap` and `indent_guides`

## Key Features

- Syntax highlighting for a wide range of languages, done on a background
  thread so navigation never stalls on a large file
- Symbol search powered by tree-sitter: rust, python, javascript, typescript,
  go, html, css, yaml, toml, c/c++, cuda, markdown and latex
- Indent guides in the preview, and markdown pipe tables aligned into columns
- Global file tree (`Ctrl+t`) showing the hierarchy with line counts per file
  and per directory
- Split panes, fuzzy file and symbol search, text selection and copy
- Image and PDF support through kitty protocol

## Auto-Refresh

The viewer automatically watches for file changes:

- **Current directory**: Refreshes file list when files are added/deleted
- **Preview file**: Refreshes preview when the viewed file is modified
- **Search index**: Refreshes every 30 seconds to pick up new files

## Development

### Pre-commit Hooks

Use prek! https://github.com/j178/prek
