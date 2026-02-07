use crate::app::App;
use crate::constants::{
    SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN, SEARCH_POPUP_WIDTH_PERCENT,
};
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, Paragraph};

/// Renders the file tree popup showing all files recursively.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let area = frame.area();

    // Calculate centered popup size
    let popup_area = popup::centered_popup(
        area,
        SEARCH_POPUP_WIDTH_PERCENT,
        SEARCH_POPUP_HEIGHT_PERCENT,
    );
    popup::render_popup_background(frame, popup_area, theme.bg_search);

    let inner = popup::popup_inner(popup_area, SEARCH_POPUP_MARGIN);

    // Split into header and list areas
    let header_height = 1;
    let list_area = Rect {
        x: inner.x,
        y: inner.y + header_height + 1,
        width: inner.width,
        height: inner.height.saturating_sub(header_height + 1),
    };
    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: header_height,
    };

    // Render header
    let header = Paragraph::new("File Tree (Global)")
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search));
    frame.render_widget(header, header_area);

    // Render file list
    if app.file_tree_popup_entries.is_empty() {
        let no_files = Paragraph::new("No files found")
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_search))
            .alignment(Alignment::Center);
        frame.render_widget(no_files, list_area);
    } else {
        let items: Vec<ListItem> = app
            .file_tree_popup_entries
            .iter()
            .map(|entry| {
                // Get relative path from root
                let display_path = entry
                    .path
                    .strip_prefix(app.root_dir())
                    .ok()
                    .and_then(|p| p.to_str())
                    .unwrap_or(&entry.name);

                // Add folder/file indicator and use different colors
                let (prefix, fg_color) = if entry.is_file {
                    ("  ", theme.fg_text)
                } else {
                    ("▸ ", theme.fg_folder)
                };
                let text = format!("{prefix}{display_path}");

                ListItem::new(text).style(Style::default().fg(fg_color).bg(theme.bg_search))
            })
            .collect();

        let file_list = List::new(items)
            .style(Style::default().bg(theme.bg_search))
            .highlight_style(Style::default().bg(theme.bg_selected).fg(theme.fg_selected))
            .highlight_symbol("> ");

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.file_tree_popup_selected));

        frame.render_stateful_widget(file_list, list_area, &mut list_state);
    }
}
