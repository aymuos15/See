use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;

    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|entry| {
            ListItem::new(format!(" {}", entry.name)).style(Style::default().fg(theme.fg_text))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_darker)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.bg_selected)
                .fg(theme.fg_selected),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.file_list_state);
}
