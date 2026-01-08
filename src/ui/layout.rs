use crate::app::split::SplitLayout;
use ratatui::prelude::*;

pub struct AppLayout {
    pub file_list_area: Option<Rect>,
    pub pane_areas: Vec<(usize, Rect)>,
    pub tab_bar_area: Rect,
}

impl AppLayout {
    #[allow(clippy::ref_option, clippy::option_if_let_else)]
    pub fn new(area: Rect, split_layout: &Option<SplitLayout>, split_percent: u16) -> Self {
        if let Some(layout) = split_layout {
            let (file_list_area, panes_area) = if layout.file_list_visible {
                let chunks: [Rect; 2] =
                    Layout::horizontal([Constraint::Percentage(split_percent), Constraint::Min(0)])
                        .areas(area);
                (Some(chunks[0]), chunks[1])
            } else {
                (None, area)
            };

            let panes_with_tabs: [Rect; 2] =
                Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(panes_area);

            Self {
                file_list_area,
                pane_areas: layout.get_pane_areas(panes_with_tabs[1]),
                tab_bar_area: panes_with_tabs[0],
            }
        } else {
            let horizontal: [Rect; 2] = Layout::horizontal([
                Constraint::Percentage(split_percent),
                Constraint::Percentage(100 - split_percent),
            ])
            .areas(area);
            let file_list_area = horizontal[0];
            let preview_area = horizontal[1];

            Self {
                file_list_area: Some(file_list_area),
                pane_areas: vec![(0, preview_area)],
                tab_bar_area: Rect::default(),
            }
        }
    }
}
