pub mod coordinates;
pub mod file_list;
pub mod layout;
pub mod preview;
pub mod search;
pub mod theme_picker;

use crate::app::App;
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = layout::AppLayout::new(frame.area(), app.split_percent);

    file_list::render(frame, app, layout.file_list_area);
    preview::render(frame, app, layout.preview_area);

    // Render search popup overlay if active (file or symbol search)
    if app.search_mode || app.symbol_search_mode {
        search::render(frame, app);
    }

    // Render theme picker popup if active
    if app.theme_preview_mode {
        theme_picker::render(frame, app);
    }
}
