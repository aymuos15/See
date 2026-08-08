//! Renders markdown pipe tables as aligned columns.
//!
//! Source tables rarely line up — authors pad them by hand, if at all — so read
//! as ragged pipe soup. Each source row is rewritten into a fixed-width row, one
//! output line per input line, so line numbers and scrolling stay in step with
//! the file.

use ratatui::prelude::*;

/// Column borders and rules, in the muted gray of the surrounding chrome.
const RULE_COLOR: Color = Color::Rgb(0x6a, 0x6a, 0x6a);

/// Rewrites every pipe table in the file, leaving all other lines untouched.
pub fn format_tables(lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let mut out = lines;
    let mut start = 0;

    while start < out.len() {
        if !is_table_row(&out[start]) {
            start += 1;
            continue;
        }

        let mut end = start;
        while end < out.len() && is_table_row(&out[end]) {
            end += 1;
        }

        // A pipe table needs a header and its separator; anything shorter is
        // prose that merely happens to contain pipes.
        if end - start >= 2 && is_separator_row(&out[start + 1]) {
            let widths = format_block(&mut out[start..end]);

            // There is no source line above the header or below the last row to
            // hold a border, but a table is nearly always surrounded by blank
            // lines — those can carry the caps without shifting any line number.
            if start > 0 && is_blank_line(&out[start - 1]) {
                out[start - 1] = cap_line(&widths, Cap::Top);
            }
            if end < out.len() && is_blank_line(&out[end]) {
                out[end] = cap_line(&widths, Cap::Bottom);
            }
        }

        start = end;
    }

    out
}

/// Aligns one table block in place, returning its column widths.
fn format_block(block: &mut [Line<'static>]) -> Vec<usize> {
    let rows: Vec<Vec<Cell>> = block.iter().map(split_cells).collect();
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);

    let widths: Vec<usize> = (0..columns)
        .map(|column| {
            rows.iter()
                .enumerate()
                .filter(|(i, _)| *i != 1) // the separator sets no width
                .filter_map(|(_, row)| row.get(column))
                .map(Cell::width)
                .max()
                .unwrap_or(0)
        })
        .collect();

    for (i, (line, row)) in block.iter_mut().zip(rows).enumerate() {
        *line = if i == 1 {
            separator_line(&widths)
        } else {
            row_line(&row, &widths)
        };
    }

    widths
}

/// Which end of the table a border closes.
#[derive(Clone, Copy)]
enum Cap {
    Top,
    Bottom,
}

/// `┌──────┬──────┐` above the table, `└──────┴──────┘` below it.
fn cap_line(widths: &[usize], cap: Cap) -> Line<'static> {
    let (left, join, right) = match cap {
        Cap::Top => ('┌', '┬', '┐'),
        Cap::Bottom => ('└', '┴', '┘'),
    };

    Line::from(Span::styled(
        rule(widths, left, join, right),
        Style::default().fg(RULE_COLOR),
    ))
}

/// A table cell: its text carrying the style each character was highlighted
/// with, so bold or colored markup inside a cell survives the realignment.
struct Cell {
    chars: Vec<(char, Style)>,
}

impl Cell {
    const fn width(&self) -> usize {
        self.chars.len()
    }

    fn spans(&self) -> Vec<Span<'static>> {
        let mut spans: Vec<Span<'static>> = Vec::new();

        for &(c, style) in &self.chars {
            match spans.last_mut() {
                Some(last) if last.style == style => last.content.to_mut().push(c),
                _ => spans.push(Span::styled(c.to_string(), style)),
            }
        }

        spans
    }
}

/// Splits a row into cells, dropping the outer pipes and surrounding spaces.
fn split_cells(line: &Line<'static>) -> Vec<Cell> {
    let chars = styled_chars(line);
    let mut cells = Vec::new();
    let mut current: Vec<(char, Style)> = Vec::new();
    let mut escaped = false;

    for &(c, style) in &chars {
        if escaped {
            current.push((c, style));
            escaped = false;
            continue;
        }

        match c {
            '\\' => {
                escaped = true;
                current.push((c, style));
            }
            '|' => {
                cells.push(std::mem::take(&mut current));
            }
            _ => current.push((c, style)),
        }
    }
    cells.push(current);

    // The text before the first pipe and after the last are empty for a
    // well-formed row; drop them so columns line up either way.
    if cells.first().is_some_and(|c| is_blank(c)) {
        cells.remove(0);
    }
    if cells.last().is_some_and(|c| is_blank(c)) {
        cells.pop();
    }

    cells
        .into_iter()
        .map(|chars| Cell {
            chars: trim(&chars),
        })
        .collect()
}

/// Pairs every character of a line with the style it was highlighted in.
fn styled_chars(line: &Line<'static>) -> Vec<(char, Style)> {
    line.spans
        .iter()
        .flat_map(|span| span.content.chars().map(move |c| (c, span.style)))
        .collect()
}

fn is_blank(chars: &[(char, Style)]) -> bool {
    chars.iter().all(|(c, _)| c.is_whitespace())
}

fn trim(chars: &[(char, Style)]) -> Vec<(char, Style)> {
    let start = chars
        .iter()
        .position(|(c, _)| !c.is_whitespace())
        .unwrap_or(chars.len());
    let end = chars
        .iter()
        .rposition(|(c, _)| !c.is_whitespace())
        .map_or(start, |i| i + 1);

    chars[start..end].to_vec()
}

/// `│ cell │ cell │`, every column padded to its width.
fn row_line(cells: &[Cell], widths: &[usize]) -> Line<'static> {
    let border = Style::default().fg(RULE_COLOR);
    let mut spans = vec![Span::styled("│", border)];

    for (column, width) in widths.iter().enumerate() {
        let cell = cells.get(column);
        let used = cell.map_or(0, Cell::width);

        spans.push(Span::styled(" ", border));
        if let Some(cell) = cell {
            spans.extend(cell.spans());
        }
        spans.push(Span::styled(
            " ".repeat(width - used.min(*width) + 1),
            border,
        ));
        spans.push(Span::styled("│", border));
    }

    Line::from(spans)
}

/// `├──────┼──────┤`, replacing the source's `|---|---|`.
fn separator_line(widths: &[usize]) -> Line<'static> {
    Line::from(Span::styled(
        rule(widths, '├', '┼', '┤'),
        Style::default().fg(RULE_COLOR),
    ))
}

/// Builds a horizontal rule spanning every column with the given corners.
fn rule(widths: &[usize], left: char, join: char, right: char) -> String {
    let mut out = String::from(left);

    for (i, width) in widths.iter().enumerate() {
        out.push_str(&"─".repeat(width + 2));
        out.push(if i + 1 == widths.len() { right } else { join });
    }

    out
}

fn is_blank_line(line: &Line<'_>) -> bool {
    text_of(line).trim().is_empty()
}

fn text_of(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

fn is_table_row(line: &Line<'_>) -> bool {
    text_of(line).trim_start().starts_with('|')
}

/// A separator is the `|:---|---:|` line under the header.
fn is_separator_row(line: &Line<'_>) -> bool {
    let text = text_of(line);
    let body = text.trim();

    body.starts_with('|')
        && body.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
        && body.contains('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(source: &[&str]) -> Vec<Line<'static>> {
        source
            .iter()
            .map(|l| Line::from(Span::raw((*l).to_string())))
            .collect()
    }

    fn rendered(lines: &[Line<'static>]) -> Vec<String> {
        lines.iter().map(text_of).collect()
    }

    #[test]
    fn aligns_ragged_columns() {
        let out = format_tables(lines(&[
            "| implementation | backend |",
            "|---|---|",
            "| bke | GPU |",
        ]));

        assert_eq!(
            rendered(&out),
            vec![
                "│ implementation │ backend │",
                "├────────────────┼─────────┤",
                "│ bke            │ GPU     │",
            ]
        );
    }

    #[test]
    fn caps_the_table_using_surrounding_blank_lines() {
        let out = format_tables(lines(&["", "| a | b |", "|---|---|", "| 1 | 2 |", ""]));

        assert_eq!(
            rendered(&out),
            vec![
                "┌───┬───┐",
                "│ a │ b │",
                "├───┼───┤",
                "│ 1 │ 2 │",
                "└───┴───┘",
            ]
        );
    }

    #[test]
    fn leaves_neighbouring_text_alone_when_there_is_no_blank_line() {
        let out = format_tables(lines(&["intro", "| a | b |", "|---|---|", "outro"]));
        let text = rendered(&out);

        assert_eq!(text[0], "intro");
        assert_eq!(text[3], "outro");
    }

    #[test]
    fn keeps_one_line_per_source_line() {
        let source = lines(&["| a | b |", "|---|---|", "| 1 | 2 |", "text"]);
        let count = source.len();

        assert_eq!(format_tables(source).len(), count);
    }

    #[test]
    fn leaves_prose_and_headerless_pipes_alone() {
        let source = ["a | b", "| not a table, no separator |"];
        let out = format_tables(lines(&source));

        assert_eq!(rendered(&out), source);
    }

    #[test]
    fn preserves_styling_inside_cells() {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let source = vec![
            Line::from(vec![
                Span::raw("| ".to_string()),
                Span::styled("bke".to_string(), bold),
                Span::raw(" | GPU |".to_string()),
            ]),
            Line::from(Span::raw("|---|---|".to_string())),
        ];

        let out = format_tables(source);
        let styled = out[0]
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "bke")
            .expect("cell text kept as its own span");

        assert!(styled.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn handles_rows_with_missing_trailing_cells() {
        let out = format_tables(lines(&["| a | b |", "|---|---|", "| 1 |"]));

        assert_eq!(rendered(&out), vec!["│ a │ b │", "├───┼───┤", "│ 1 │   │"]);
    }
}
