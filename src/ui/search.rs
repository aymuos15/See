use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calculate centered popup size (60% width, 70% height)
    let popup_width = (area.width * 60) / 100;
    let popup_height = (area.height * 70) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Render popup block with border
    let block = Block::default()
        .title("File Search")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border))
        .style(Style::default().bg(app.theme.bg_main));

    frame.render_widget(block, popup_area);

    // Calculate inner areas (input + results list)
    let inner = popup_area.inner(Margin::new(1, 1));
    let [input_area, results_area] = Layout::vertical([
        Constraint::Length(3), // Input field
        Constraint::Min(0),    // Results list
    ])
    .areas(inner);

    // Render search input with border
    let input_text = format!("/ {}", app.search_query);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(app.theme.fg_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border)),
        );
    frame.render_widget(input, input_area);

    // Render filtered results
    if app.search_results.is_empty() {
        let no_results = Paragraph::new("No matches")
            .style(Style::default().fg(app.theme.fg_dim))
            .alignment(Alignment::Center);
        frame.render_widget(no_results, results_area);
    } else {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .map(|&idx| {
                let file = &app.files[idx];
                ListItem::new(file.name.clone()).style(Style::default().fg(app.theme.fg_text))
            })
            .collect();

        let results_list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(app.theme.bg_selected)
                    .fg(app.theme.fg_selected),
            )
            .highlight_symbol("> ");

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.search_selected));

        frame.render_stateful_widget(results_list, results_area, &mut list_state);
    }
}
