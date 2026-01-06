use std::cmp::Ordering;

/// Represents a position in the text content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPosition {
    pub line: usize,
    pub column: usize,
}

impl TextPosition {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl Ord for TextPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.line.cmp(&other.line) {
            Ordering::Equal => self.column.cmp(&other.column),
            ord => ord,
        }
    }
}

impl PartialOrd for TextPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a text selection with start and end anchors
#[derive(Debug, Clone, Default)]
pub struct TextSelection {
    pub anchor: TextPosition,
    pub cursor: TextPosition,
    pub active: bool,
}

impl TextSelection {
    pub const fn new(anchor: TextPosition) -> Self {
        Self {
            anchor,
            cursor: anchor,
            active: true,
        }
    }

    /// Returns (start, end) positions in document order
    pub fn ordered(&self) -> (TextPosition, TextPosition) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }

    /// Check if the selection spans any text
    pub fn is_empty(&self) -> bool {
        self.anchor == self.cursor
    }

    /// Extract selected text from raw lines
    pub fn extract_text(&self, raw_lines: &[String]) -> String {
        if self.is_empty() {
            return String::new();
        }

        let (start, end) = self.ordered();

        if start.line == end.line {
            // Single line selection
            if let Some(line) = raw_lines.get(start.line) {
                let end_col = end.column.min(line.len());
                let start_col = start.column.min(end_col);
                return line[start_col..end_col].to_string();
            }
            return String::new();
        }

        // Multi-line selection
        let mut result = String::new();

        for line_idx in start.line..=end.line {
            if let Some(line) = raw_lines.get(line_idx) {
                if line_idx == start.line {
                    // First line: from start column to end
                    let start_col = start.column.min(line.len());
                    result.push_str(&line[start_col..]);
                } else if line_idx == end.line {
                    // Last line: from beginning to end column
                    let end_col = end.column.min(line.len());
                    result.push_str(&line[..end_col]);
                } else {
                    // Middle lines: entire line
                    result.push_str(line);
                }

                // Add newline between lines (except after last)
                if line_idx < end.line {
                    result.push('\n');
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_position_ordering() {
        let pos1 = TextPosition::new(0, 5);
        let pos2 = TextPosition::new(0, 10);
        let pos3 = TextPosition::new(1, 0);

        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_selection_ordered() {
        // Forward selection
        let sel = TextSelection {
            anchor: TextPosition::new(0, 0),
            cursor: TextPosition::new(1, 5),
            active: false,
        };
        let (start, end) = sel.ordered();
        assert_eq!(start, TextPosition::new(0, 0));
        assert_eq!(end, TextPosition::new(1, 5));

        // Backward selection
        let sel = TextSelection {
            anchor: TextPosition::new(1, 5),
            cursor: TextPosition::new(0, 0),
            active: false,
        };
        let (start, end) = sel.ordered();
        assert_eq!(start, TextPosition::new(0, 0));
        assert_eq!(end, TextPosition::new(1, 5));
    }

    #[test]
    fn test_extract_single_line() {
        let lines = vec!["Hello, World!".to_string()];
        let sel = TextSelection {
            anchor: TextPosition::new(0, 0),
            cursor: TextPosition::new(0, 5),
            active: false,
        };
        assert_eq!(sel.extract_text(&lines), "Hello");
    }

    #[test]
    fn test_extract_multi_line() {
        let lines = vec![
            "First line".to_string(),
            "Second line".to_string(),
            "Third line".to_string(),
        ];
        let sel = TextSelection {
            anchor: TextPosition::new(0, 6),
            cursor: TextPosition::new(2, 5),
            active: false,
        };
        assert_eq!(sel.extract_text(&lines), "line\nSecond line\nThird");
    }

    #[test]
    fn test_empty_selection() {
        let sel = TextSelection {
            anchor: TextPosition::new(0, 5),
            cursor: TextPosition::new(0, 5),
            active: false,
        };
        assert!(sel.is_empty());
        assert_eq!(sel.extract_text(&["Hello".to_string()]), "");
    }
}
