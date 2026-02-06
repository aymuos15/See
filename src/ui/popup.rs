use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear};

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

/// Clears and renders a popup background.
pub fn render_popup_background(frame: &mut Frame, area: Rect, bg_color: Color) {
    frame.render_widget(Clear, area);
    let block = Block::default().style(Style::default().bg(bg_color));
    frame.render_widget(block, area);
}

/// Gets the inner area of a popup with margins.
pub const fn popup_inner(popup_area: Rect, margin: u16) -> Rect {
    popup_area.inner(Margin::new(margin, margin))
}

/// Splits popup inner area into header and body.
#[allow(clippy::tuple_array_conversions)]
pub fn split_popup(inner: Rect, header_height: u16) -> (Rect, Rect) {
    let [header_area, body_area] =
        Layout::vertical([Constraint::Length(header_height), Constraint::Min(0)]).areas(inner);
    (header_area, body_area)
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

    #[test]
    fn test_popup_inner_with_margin() {
        let popup_area = Rect {
            x: 10,
            y: 10,
            width: 80,
            height: 40,
        };
        let inner = popup_inner(popup_area, 2);
        assert_eq!(inner.x, 12);
        assert_eq!(inner.y, 12);
        assert_eq!(inner.width, 76);
        assert_eq!(inner.height, 36);
    }
}
