use crate::app::split::SplitLayout;
use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, layout: &SplitLayout, area: Rect, theme: &Theme) {
    let spans: Vec<Span> = layout
        .panes
        .iter()
        .enumerate()
        .map(|(idx, pane)| {
            let tab_num = idx + 1;
            let name = pane
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("[Empty]");

            let label = format!(" {tab_num}:{name} ");

            if pane.id == layout.active_pane_index {
                Span::styled(
                    label,
                    Style::default().fg(theme.fg_selected).bg(theme.bg_selected),
                )
            } else {
                Span::styled(label, Style::default().fg(theme.fg_dim).bg(theme.bg_darker))
            }
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg_darker));

    frame.render_widget(paragraph, area);
}
