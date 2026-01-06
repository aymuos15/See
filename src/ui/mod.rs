pub mod file_list;
pub mod layout;
pub mod preview;

use crate::app::App;
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = layout::AppLayout::new(frame.area(), app.split_percent);

    file_list::render(frame, app, layout.file_list_area);
    preview::render(frame, app, layout.preview_area);
}
