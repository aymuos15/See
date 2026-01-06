use crate::app::selection::TextPosition;
use ratatui::prelude::Rect;
use std::fs::OpenOptions;
use std::io::Write;

pub const LINE_NUMBER_WIDTH: u16 = 5;

fn debug_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/viewer_debug.log")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

/// Maps screen coordinates to a text position in the document
pub fn screen_to_text_position(
    screen_col: u16,
    screen_row: u16,
    preview_area: Rect,
    scroll_offset: u16,
    total_lines: usize,
    raw_lines: &[String],
) -> Option<TextPosition> {
    debug_log(&format!(
        "  [COORD] screen=({}, {}), preview_area=({}, {}, w={}, h={}), scroll={}, total_lines={}",
        screen_col, screen_row, preview_area.x, preview_area.y, preview_area.width, preview_area.height, scroll_offset, total_lines
    ));

    // Check if click is within preview area
    if screen_col < preview_area.x || screen_col >= preview_area.x + preview_area.width {
        debug_log("  [COORD] FAIL: outside preview area horizontally");
        return None;
    }
    if screen_row < preview_area.y || screen_row >= preview_area.y + preview_area.height {
        debug_log("  [COORD] FAIL: outside preview area vertically");
        return None;
    }

    // Calculate content area (excluding block border)
    let content_start_x = preview_area.x + LINE_NUMBER_WIDTH;
    debug_log(&format!("  [COORD] content_start_x={}", content_start_x));

    // Check if click is in line number area (ignore those clicks)
    if screen_col < content_start_x {
        debug_log("  [COORD] FAIL: click in line number area");
        return None;
    }

    // Calculate line index
    let relative_row = screen_row.saturating_sub(preview_area.y);
    let line_index = scroll_offset as usize + relative_row as usize;
    debug_log(&format!("  [COORD] relative_row={}, line_index={}", relative_row, line_index));

    if line_index >= total_lines {
        debug_log(&format!("  [COORD] FAIL: line_index {} >= total_lines {}", line_index, total_lines));
        return None;
    }

    // Calculate column position within the line
    let relative_col = screen_col.saturating_sub(content_start_x);
    let line_content = raw_lines.get(line_index)?;

    // Clamp column to line length
    let column = (relative_col as usize).min(line_content.len());
    debug_log(&format!("  [COORD] SUCCESS: line={}, col={}", line_index, column));

    Some(TextPosition::new(line_index, column))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_in_content_area() {
        let preview_area = Rect::new(10, 5, 80, 20);
        let raw_lines = vec!["Hello, World!".to_string(), "Second line".to_string()];

        // Click in first line, after line numbers
        let pos = screen_to_text_position(
            15 + LINE_NUMBER_WIDTH, // After line numbers
            5,                      // First row
            preview_area,
            0,
            2,
            &raw_lines,
        );

        assert!(pos.is_some());
        let pos = pos.expect("position should exist");
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_click_in_line_numbers() {
        let preview_area = Rect::new(10, 5, 80, 20);
        let raw_lines = vec!["Hello".to_string()];

        // Click in line number area
        let pos = screen_to_text_position(
            12, // Before LINE_NUMBER_WIDTH offset
            5,
            preview_area,
            0,
            1,
            &raw_lines,
        );

        assert!(pos.is_none());
    }

    #[test]
    fn test_click_outside_preview() {
        let preview_area = Rect::new(10, 5, 80, 20);
        let raw_lines = vec!["Hello".to_string()];

        // Click before preview area
        let pos = screen_to_text_position(5, 5, preview_area, 0, 1, &raw_lines);
        assert!(pos.is_none());

        // Click after preview area
        let pos = screen_to_text_position(100, 5, preview_area, 0, 1, &raw_lines);
        assert!(pos.is_none());
    }

    #[test]
    fn test_click_with_scroll() {
        let preview_area = Rect::new(0, 0, 80, 20);
        let raw_lines = vec![
            "Line 0".to_string(),
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];

        // Click on first visible row with scroll offset of 2
        let pos = screen_to_text_position(LINE_NUMBER_WIDTH, 0, preview_area, 2, 4, &raw_lines);

        assert!(pos.is_some());
        let pos = pos.expect("position should exist");
        assert_eq!(pos.line, 2); // Should be line 2, not line 0
    }

    #[test]
    fn test_column_clamp_to_line_length() {
        let preview_area = Rect::new(0, 0, 80, 20);
        let raw_lines = vec!["Hi".to_string()]; // Short line

        // Click way past end of line
        let pos = screen_to_text_position(LINE_NUMBER_WIDTH + 50, 0, preview_area, 0, 1, &raw_lines);

        assert!(pos.is_some());
        let pos = pos.expect("position should exist");
        assert_eq!(pos.column, 2); // Clamped to line length
    }
}
