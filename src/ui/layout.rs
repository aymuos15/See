use ratatui::prelude::*;

pub struct AppLayout {
    pub file_list_area: Rect,
    pub preview_area: Rect,
    pub status_area: Rect,
}

impl AppLayout {
    pub fn new(area: Rect) -> Self {
        let vertical = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(1),
        ]);
        let [main_area, status_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]);
        let [file_list_area, preview_area] = horizontal.areas(main_area);

        Self {
            file_list_area,
            preview_area,
            status_area,
        }
    }
}
