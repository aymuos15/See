use crate::app::App;
use crate::constants::{SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_WIDTH_PERCENT};
use crate::theme::Theme;
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::ListItem;

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

/// Opens a search panel and splits it into a query line and a results area.
fn search_panel(frame: &mut Frame, title: &str, theme: &Theme) -> (Rect, Rect) {
    let area = frame.area();
    let panel = popup::centered_popup(
        area,
        SEARCH_POPUP_WIDTH_PERCENT,
        SEARCH_POPUP_HEIGHT_PERCENT,
    );
    let inner = popup::render_panel(frame, panel, title, theme);

    popup::split_query(inner)
}

/// Styles a result row. Ratatui pads unselected rows to the width of the
/// selection marker, so rows carry no indent of their own.
fn row(text: &str, color: Color, theme: &Theme) -> ListItem<'static> {
    ListItem::new(text.to_string()).style(Style::default().fg(color).bg(theme.bg_search))
}

fn render_file_search(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let (query_area, results_area) = search_panel(frame, "Find File", theme);

    popup::render_query(frame, query_area, "▸", &app.search_query, theme);

    if app.search_results.is_empty() {
        popup::render_empty_message(frame, results_area, "No matches", theme);
        return;
    }

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
                row(display_path, theme.fg_text, theme)
            })
        })
        .collect();

    popup::render_list(frame, results_area, items, app.search_selected, theme);
}

fn render_symbol_search(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let (query_area, results_area) = search_panel(frame, "Find Symbol", theme);

    popup::render_query(frame, query_area, "▸", &app.symbol_search_query, theme);

    if app.symbol_search_results.is_empty() {
        popup::render_empty_message(frame, results_area, "No matches", theme);
        return;
    }

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
                row(
                    &format!("{:<30} {location}", symbol.name),
                    theme.fg_text,
                    theme,
                )
            })
        })
        .collect();

    popup::render_list(
        frame,
        results_area,
        items,
        app.symbol_search_selected,
        theme,
    );
}

/// Find-in-file is a compact panel pinned to the top right rather than a
/// centered one — it stays out of the way of the text being searched.
fn render_find_search(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let area = frame.area();

    let width = (area.width * 30 / 100).clamp(24, 40).min(area.width);
    let panel = Rect {
        x: area.width.saturating_sub(width),
        y: 0,
        width,
        // Title, the query line, and a row of padding either side of it.
        height: 4,
    };

    // The match count doubles as the panel title so the query line stays
    // free for typing.
    let title = if app.find_query.is_empty() {
        "Find".to_string()
    } else {
        let count = app.find_match_lines().len();
        let plural = if count == 1 { "" } else { "s" };
        format!("Find — {count} line{plural}")
    };

    let inner = popup::render_panel(frame, panel, &title, theme);
    popup::render_query(frame, inner, "▸", &app.find_query, theme);
}
