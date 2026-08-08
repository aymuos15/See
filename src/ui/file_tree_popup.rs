use crate::app::App;
use crate::constants::{SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_WIDTH_PERCENT};
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::ListItem;

/// Renders the file tree popup showing all files recursively.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;

    let panel = popup::centered_popup(
        frame.area(),
        SEARCH_POPUP_WIDTH_PERCENT,
        SEARCH_POPUP_HEIGHT_PERCENT,
    );
    let inner = popup::render_panel(frame, panel, "File Tree", theme);

    if app.file_tree_popup_entries.is_empty() {
        popup::render_empty_message(frame, inner, "No files found", theme);
        return;
    }

    let items: Vec<ListItem> = app
        .file_tree_popup_entries
        .iter()
        .map(|entry| {
            let display_path = entry
                .path
                .strip_prefix(app.root_dir())
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or(&entry.name);

            // Directories carry a chevron and the folder color; files sit flush.
            let (marker, color) = if entry.is_file {
                ("  ", theme.fg_text)
            } else {
                ("▸ ", theme.fg_folder)
            };

            ListItem::new(format!("{marker}{display_path}"))
                .style(Style::default().fg(color).bg(theme.bg_search))
        })
        .collect();

    popup::render_list(frame, inner, items, app.file_tree_popup_selected, theme);
}
