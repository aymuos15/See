use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|entry| {
            let icon = if entry.is_file { " " } else { " " };
            let style = if entry.is_file {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Blue).bold()
            };

            ListItem::new(format!("{}{}", icon, entry.name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Files ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Yellow)
                .bold(),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.file_list_state);
}
