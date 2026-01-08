use crate::app::split::SplitLayout;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Tabs;

pub fn render(frame: &mut Frame, layout: &SplitLayout, area: Rect, theme: &Theme) {
    let titles: Vec<Line> = layout
        .panes
        .iter()
        .map(|pane| {
            let name = pane
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("[Empty]");

            if pane.id == layout.active_pane_index {
                Line::from(format!(" {} ", name))
                    .style(Style::default().fg(theme.fg_selected).bg(theme.bg_selected))
            } else {
                Line::from(format!(" {} ", name)).style(Style::default().fg(theme.fg_dim))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .divider("|")
        .style(Style::default().bg(theme.bg_darker));

    frame.render_widget(tabs, area);
}
