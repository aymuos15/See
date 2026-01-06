use ratatui::prelude::*;

pub struct AppLayout {
    pub file_list_area: Rect,
    pub preview_area: Rect,
}

impl AppLayout {
    pub fn new(area: Rect, split_percent: u16) -> Self {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(split_percent),
            Constraint::Percentage(100 - split_percent),
        ]);
        let [file_list_area, preview_area] = horizontal.areas(area);

        Self {
            file_list_area,
            preview_area,
        }
    }
}
