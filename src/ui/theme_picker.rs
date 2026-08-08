use crate::app::App;
use crate::constants::{SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_WIDTH_PERCENT};
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::ListItem;

/// Renders the theme picker popup.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;

    let panel = popup::centered_popup(
        frame.area(),
        SEARCH_POPUP_WIDTH_PERCENT,
        SEARCH_POPUP_HEIGHT_PERCENT,
    );
    let inner = popup::render_panel(frame, panel, "Themes", theme);

    let items: Vec<ListItem> = app
        .available_themes
        .iter()
        .map(|name| {
            // A check marks the active theme; others align beneath it.
            let is_current = name == &app.current_theme_name;
            let (marker, color) = if is_current {
                ("✓ ", theme.fg_selected)
            } else {
                ("  ", theme.fg_text)
            };

            ListItem::new(format!("{marker}{name}"))
                .style(Style::default().fg(color).bg(theme.bg_search))
        })
        .collect();

    let current_idx = app
        .available_themes
        .iter()
        .position(|t| t == &app.current_theme_name)
        .unwrap_or(0);

    popup::render_list(frame, inner, items, current_idx, theme);
}
