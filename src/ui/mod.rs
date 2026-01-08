pub mod coordinates;
pub mod file_list;
pub mod help;
pub mod layout;
pub mod pane;
pub mod preview;
pub mod search;
pub mod tab_bar;
pub mod theme_picker;

use crate::app::App;
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = layout::AppLayout::new(frame.area(), &app.split_layout, app.split_percent);

    if let Some(file_list_area) = layout.file_list_area {
        file_list::render(frame, app, file_list_area);
    }

    if let Some(ref split_layout) = app.split_layout {
        tab_bar::render(frame, split_layout, layout.tab_bar_area, &app.config.theme);
        for (pane_id, area) in &layout.pane_areas {
            if let Some(pane) = split_layout.panes.iter().find(|p| p.id == *pane_id) {
                pane::render(
                    frame,
                    pane,
                    *area,
                    &app.config.theme,
                    pane.id == split_layout.active_pane_index,
                );
            }
        }
    } else {
        preview::render(frame, app, layout.pane_areas[0].1);
    }

    // Render search popup overlay if active (file or symbol search)
    if app.search_mode || app.symbol_search_mode {
        search::render(frame, app);
    }

    // Render theme picker popup if active
    if app.theme_preview_mode {
        theme_picker::render(frame, app);
    }

    // Render help overlay if active
    if app.help_mode {
        help::render(frame, app, frame.area());
    }
}
