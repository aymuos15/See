pub mod file_list;
pub mod layout;
pub mod preview;

use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = layout::AppLayout::new(frame.area());

    file_list::render(frame, app, layout.file_list_area);
    preview::render(frame, app, layout.preview_area);
    render_status_bar(frame, app, layout.status_area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = format!(
        " {} | {} files | j/k: navigate | J/K: scroll | q: quit",
        app.current_dir.display(),
        app.files.len()
    );

    let paragraph = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(paragraph, area);
}
