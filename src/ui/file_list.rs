use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.config.theme;

    // Store area for mouse click handling
    app.last_file_list_area = Some(area);

    // Calculate inner width: area width minus highlight symbol width (2)
    let inner_width = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|entry| {
            let is_modified = app.is_file_modified(&entry.path);
            let fg_color = if is_modified {
                theme.fg_modified
            } else if entry.is_file {
                theme.fg_text
            } else {
                theme.fg_folder
            };

            // Add a dot prefix if the file is modified, otherwise just the name
            let text = if is_modified {
                format!("●{}", entry.name)
            } else {
                entry.name.clone()
            };

            // Pad to full width so background color fills the entire line
            let padded = format!("{text:<inner_width$}");
            ListItem::new(padded).style(Style::default().fg(fg_color))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().style(Style::default().bg(theme.bg_darker)))
        .highlight_style(Style::default().bg(theme.bg_selected).fg(theme.fg_selected))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.file_list_state);
}
