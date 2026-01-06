use crate::app::App;
use crate::constants::{
    SEARCH_INPUT_HEIGHT, SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN,
    SEARCH_POPUP_WIDTH_PERCENT,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

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
    let block = Block::default().style(Style::default().bg(app.config.theme.bg_search));

    frame.render_widget(block, popup_area);

    // Calculate inner areas (input + results list)
    let inner = popup_area.inner(Margin::new(SEARCH_POPUP_MARGIN, SEARCH_POPUP_MARGIN));
    let [input_area, results_area] =
        Layout::vertical([Constraint::Length(SEARCH_INPUT_HEIGHT), Constraint::Min(0)])
            .areas(inner);

    // Render search input
    let input_text = format!("/ {}", app.search_query);
    let input = Paragraph::new(input_text).style(
        Style::default()
            .fg(app.config.theme.fg_text)
            .bg(app.config.theme.bg_search),
    );
    frame.render_widget(input, input_area);

    // Render filtered results
    if app.search_results.is_empty() {
        let no_results = Paragraph::new("No matches")
            .style(
                Style::default()
                    .fg(app.config.theme.fg_dim)
                    .bg(app.config.theme.bg_search),
            )
            .alignment(Alignment::Center);
        frame.render_widget(no_results, results_area);
    } else {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .filter_map(|&idx| {
                app.search_index().get(idx).map(|file| {
                    // Display relative path from root
                    let display_path = file
                        .path
                        .strip_prefix(app.root_dir())
                        .ok()
                        .and_then(|p| p.to_str())
                        .unwrap_or(&file.name);
                    ListItem::new(display_path).style(
                        Style::default()
                            .fg(app.config.theme.fg_text)
                            .bg(app.config.theme.bg_search),
                    )
                })
            })
            .collect();

        let results_list = List::new(items)
            .style(Style::default().bg(app.config.theme.bg_search))
            .highlight_style(
                Style::default()
                    .bg(app.config.theme.bg_selected)
                    .fg(app.config.theme.fg_selected),
            )
            .highlight_symbol("> ");

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.search_selected));

        frame.render_stateful_widget(results_list, results_area, &mut list_state);
    }
}
