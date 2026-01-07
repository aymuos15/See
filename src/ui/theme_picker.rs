use crate::app::App;
use crate::constants::{
    SEARCH_INPUT_HEIGHT, SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN,
    SEARCH_POPUP_WIDTH_PERCENT,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};

/// Renders the theme picker popup.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.config.theme;

    // Calculate centered popup size
    let popup_width = (area.width * SEARCH_POPUP_WIDTH_PERCENT) / 100;
    let popup_height = (area.height * SEARCH_POPUP_HEIGHT_PERCENT) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear area and render opaque popup block
    frame.render_widget(Clear, popup_area);
    let block = Block::default().style(Style::default().bg(theme.bg_search));
    frame.render_widget(block, popup_area);

    // Calculate inner areas
    let inner = popup_area.inner(Margin::new(SEARCH_POPUP_MARGIN, SEARCH_POPUP_MARGIN));
    let [header_area, themes_area] =
        Layout::vertical([Constraint::Length(SEARCH_INPUT_HEIGHT), Constraint::Min(0)])
            .areas(inner);

    // Render header
    let header =
        Paragraph::new("Themes").style(Style::default().fg(theme.fg_text).bg(theme.bg_search));
    frame.render_widget(header, header_area);

    // Render theme list
    let items: Vec<ListItem> = app
        .available_themes
        .iter()
        .map(|theme_name| {
            let is_current = theme_name == &app.current_theme_name;
            let label = if is_current {
                format!("✓ {theme_name}")
            } else {
                format!("  {theme_name}")
            };
            ListItem::new(label).style(
                Style::default()
                    .fg(app.config.theme.fg_text)
                    .bg(app.config.theme.bg_search),
            )
        })
        .collect();

    let current_idx = app
        .available_themes
        .iter()
        .position(|t| t == &app.current_theme_name)
        .unwrap_or(0);

    let themes_list = List::new(items)
        .style(Style::default().bg(theme.bg_search))
        .highlight_style(Style::default().bg(theme.bg_selected).fg(theme.fg_selected))
        .highlight_symbol("> ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(current_idx));

    frame.render_stateful_widget(themes_list, themes_area, &mut list_state);
}
