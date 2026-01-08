use crate::app::App;
use crate::constants::{
    SEARCH_INPUT_HEIGHT, SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN,
    SEARCH_POPUP_WIDTH_PERCENT,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

/// Renders the appropriate search popup based on current mode.
pub fn render(frame: &mut Frame, app: &App) {
    if app.symbol_search_mode {
        render_symbol_search(frame, app);
    } else if app.find_mode {
        render_find_search(frame, app);
    } else {
        render_file_search(frame, app);
    }
}

/// Creates a centered popup area and clears it for rendering.
fn create_popup_area(frame: &mut Frame, bg_color: Color) -> (Rect, Rect) {
    let area = frame.area();

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

    frame.render_widget(Clear, popup_area);
    let block = Block::default().style(Style::default().bg(bg_color));
    frame.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin::new(SEARCH_POPUP_MARGIN, SEARCH_POPUP_MARGIN));
    let [input_area, results_area] =
        Layout::vertical([Constraint::Length(SEARCH_INPUT_HEIGHT), Constraint::Min(0)])
            .areas(inner);

    [input_area, results_area].into()
}

/// Renders the search input field.
fn render_input(
    frame: &mut Frame,
    area: Rect,
    prefix: &str,
    query: &str,
    theme: &crate::theme::Theme,
) {
    let input_text = format!("{prefix} {query}");
    let input =
        Paragraph::new(input_text).style(Style::default().fg(theme.fg_text).bg(theme.bg_search));
    frame.render_widget(input, area);
}

/// Renders a "No matches" message.
fn render_no_results(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let no_results = Paragraph::new("No matches")
        .style(Style::default().fg(theme.fg_dim).bg(theme.bg_search))
        .alignment(Alignment::Center);
    frame.render_widget(no_results, area);
}

/// Renders a list of results with selection highlighting.
fn render_results_list(
    frame: &mut Frame,
    area: Rect,
    items: Vec<ListItem>,
    selected: usize,
    theme: &crate::theme::Theme,
) {
    let results_list = List::new(items)
        .style(Style::default().bg(theme.bg_search))
        .highlight_style(Style::default().bg(theme.bg_selected).fg(theme.fg_selected))
        .highlight_symbol("> ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(selected));

    frame.render_stateful_widget(results_list, area, &mut list_state);
}

fn render_file_search(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let (input_area, results_area) = create_popup_area(frame, theme.bg_search);

    render_input(frame, input_area, "/", &app.search_query, theme);

    if app.search_results.is_empty() {
        render_no_results(frame, results_area, theme);
    } else {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .filter_map(|&idx| {
                app.search_index().get(idx).map(|file| {
                    let display_path = file
                        .path
                        .strip_prefix(app.root_dir())
                        .ok()
                        .and_then(|p| p.to_str())
                        .unwrap_or(&file.name);
                    ListItem::new(display_path)
                        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search))
                })
            })
            .collect();

        render_results_list(frame, results_area, items, app.search_selected, theme);
    }
}

fn render_symbol_search(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let (input_area, results_area) = create_popup_area(frame, theme.bg_search);

    render_input(frame, input_area, "f", &app.symbol_search_query, theme);

    if app.symbol_search_results.is_empty() {
        render_no_results(frame, results_area, theme);
    } else {
        let items: Vec<ListItem> = app
            .symbol_search_results
            .iter()
            .filter_map(|&idx| {
                app.symbol_index.get(idx).map(|symbol| {
                    let location = format!(
                        "{}:{} [{}]",
                        symbol
                            .file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?"),
                        symbol.line + 1,
                        symbol.kind.icon()
                    );
                    let display = format!("{:<30} {}", symbol.name, location);
                    ListItem::new(display)
                        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search))
                })
            })
            .collect();

        render_results_list(
            frame,
            results_area,
            items,
            app.symbol_search_selected,
            theme,
        );
    }
}

fn render_find_search(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let area = frame.area();

    // Position in top right
    let popup_width = (area.width * 30).min(40);
    let popup_height = 3;
    let margin = 1;
    let popup_x = area.width.saturating_sub(popup_width + margin);
    let popup_y = margin;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.fg_dim))
        .style(Style::default().bg(theme.bg_search));
    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    render_input(frame, inner_area, "\\", &app.find_query, theme);
}
