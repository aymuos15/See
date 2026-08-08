use crate::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Padding, Paragraph};

/// Creates a centered area of the given size, clamped to fit within `area`.
pub fn centered_sized(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);

    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// Renders an elevated panel: a rounded, titled surface floating above the
/// content behind it. Returns the padded inner area to draw into.
pub fn render_panel(frame: &mut Frame, area: Rect, title: &str, theme: &Theme) -> Rect {
    frame.render_widget(Clear, area);

    let title_line = Line::from(Span::styled(
        title,
        Style::default()
            .fg(theme.fg_selected)
            .add_modifier(Modifier::BOLD),
    ));

    // No border: the surface reads as elevated through its background and
    // generous padding alone.
    let block = Block::default()
        .title(title_line)
        .title_alignment(Alignment::Center)
        .padding(Padding::symmetric(3, 1))
        .style(Style::default().bg(theme.bg_search));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

/// Marker drawn against the selected row of a panel list.
pub const SELECTION_MARKER: &str = "▍ ";

/// Splits a panel's inner area into a query line and the list below it,
/// separated by a blank line.
#[allow(clippy::tuple_array_conversions)]
pub fn split_query(inner: Rect) -> (Rect, Rect) {
    let [query_area, _gap, list_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    (query_area, list_area)
}

/// Renders a panel's query line: a dim prompt glyph, the typed text, and a
/// block cursor parked at the end.
pub fn render_query(frame: &mut Frame, area: Rect, prompt: &str, query: &str, theme: &Theme) {
    let line = Line::from(vec![
        Span::styled(format!("{prompt} "), Style::default().fg(theme.fg_dim)),
        Span::styled(query.to_string(), Style::default().fg(theme.fg_text)),
        Span::styled("▏", Style::default().fg(theme.fg_selected)),
    ]);

    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(theme.bg_search)),
        area,
    );
}

/// Renders a centered, dim message where a panel's list would go.
pub fn render_empty_message(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::default().fg(theme.fg_dim).bg(theme.bg_search))
            .alignment(Alignment::Center),
        area,
    );
}

/// Renders a panel list with the selected row marked by an accent bar.
pub fn render_list(
    frame: &mut Frame,
    area: Rect,
    items: Vec<ListItem>,
    selected: usize,
    theme: &Theme,
) {
    let list = List::new(items)
        .style(Style::default().bg(theme.bg_search))
        .highlight_style(
            Style::default()
                .bg(theme.bg_selected)
                .fg(theme.fg_selected)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(SELECTION_MARKER);

    let mut state = ListState::default();
    state.select(Some(selected));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Creates a centered popup area.
pub const fn centered_popup(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let popup_width = (area.width * width_percent) / 100;
    let popup_height = (area.height * height_percent) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_popup_calculation() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let popup = centered_popup(area, 70, 80);
        assert_eq!(popup.width, 70);
        assert_eq!(popup.height, 40);
        assert_eq!(popup.x, 15);
        assert_eq!(popup.y, 5);
    }
}
