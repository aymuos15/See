//! Indent guides: faint vertical rules marking each indentation level, the way
//! an editor draws them.

use crate::theme::{darken, Theme};
use ratatui::prelude::*;

/// The rule drawn at each indentation stop.
const GUIDE: char = '│';
/// How far the guide color sits below the line-number color.
const GUIDE_DARKEN: u16 = 70;
/// Indent widths we are willing to infer, narrowest first.
const CANDIDATE_WIDTHS: [usize; 4] = [2, 4, 8, 3];
/// How many lines to sample when inferring a file's indent width.
const SAMPLE_LINES: usize = 400;

/// Infers a file's indent width from the shallowest indentation it uses,
/// falling back to 4 for files with no indented lines.
pub fn infer_width(raw_lines: &[String]) -> usize {
    let shallowest = raw_lines
        .iter()
        .take(SAMPLE_LINES)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches(' ').len())
        .filter(|&indent| indent > 0)
        .min();

    shallowest
        .filter(|indent| CANDIDATE_WIDTHS.contains(indent))
        .unwrap_or(4)
}

/// Draws guides into the leading whitespace of each line, leaving the rest of
/// the styled spans untouched. Lines that are blank or unindented come back
/// unchanged.
pub fn apply(
    lines: &[Line<'static>],
    raw_lines: &[String],
    width: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let color = darken(theme.line_num, GUIDE_DARKEN);

    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let indent = raw_lines
                .get(i)
                .filter(|raw| !raw.trim().is_empty())
                .map_or(0, |raw| raw.len() - raw.trim_start_matches(' ').len());

            if indent == 0 {
                return line.clone();
            }

            with_guides(line, indent, width, color)
        })
        .collect()
}

/// Rebuilds one line with its leading spaces replaced by guides.
fn with_guides(line: &Line<'static>, indent: usize, width: usize, color: Color) -> Line<'static> {
    // Keep whatever background the line's own styling gave the indent, so
    // selection and search highlights still read correctly underneath.
    let background = line.spans.first().and_then(|span| span.style.bg);

    let guides: String = (0..indent)
        .map(|column| if column % width == 0 { GUIDE } else { ' ' })
        .collect();

    let mut style = Style::default().fg(color);
    if let Some(bg) = background {
        style = style.bg(bg);
    }

    let mut spans = vec![Span::styled(guides, style)];
    spans.extend(strip_leading_spaces(&line.spans, indent));

    Line::from(spans)
}

/// Drops `count` leading space characters across the front of the span list,
/// preserving every span's styling.
fn strip_leading_spaces(spans: &[Span<'static>], count: usize) -> Vec<Span<'static>> {
    let mut remaining = count;

    spans
        .iter()
        .filter_map(|span| {
            if remaining == 0 {
                return Some(span.clone());
            }

            let leading = span.content.len() - span.content.trim_start_matches(' ').len();
            let dropped = leading.min(remaining);
            remaining -= dropped;

            let rest = span.content[dropped..].to_string();
            if rest.is_empty() {
                None
            } else {
                Some(Span::styled(rest, span.style))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str) -> Line<'static> {
        Line::from(vec![Span::raw(text.to_string())])
    }

    fn rendered(line: &Line<'static>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn infers_width_from_shallowest_indent() {
        let lines = vec!["def f():".to_string(), "    return 1".to_string()];
        assert_eq!(infer_width(&lines), 4);

        let lines = vec!["a:".to_string(), "  b: 1".to_string()];
        assert_eq!(infer_width(&lines), 2);
    }

    #[test]
    fn falls_back_to_four_without_indentation() {
        let lines = vec!["flat".to_string(), "also flat".to_string()];
        assert_eq!(infer_width(&lines), 4);
    }

    #[test]
    fn draws_a_guide_at_each_stop() {
        let raw = vec!["        deep".to_string()];
        let guided = apply(&[line("        deep")], &raw, 4, &Theme::default());

        // Guides at columns 0 and 4, and the text keeps its position.
        assert_eq!(rendered(&guided[0]), "│   │   deep");
    }

    #[test]
    fn leaves_unindented_and_blank_lines_alone() {
        let raw = vec!["top".to_string(), "   ".to_string()];
        let guided = apply(&[line("top"), line("   ")], &raw, 4, &Theme::default());

        assert_eq!(rendered(&guided[0]), "top");
        assert_eq!(rendered(&guided[1]), "   ");
    }

    #[test]
    fn preserves_styling_of_the_code_after_the_indent() {
        let styled = Line::from(vec![
            Span::raw("    ".to_string()),
            Span::styled("fn".to_string(), Style::default().fg(Color::Red)),
        ]);
        let raw = vec!["    fn".to_string()];

        let guided = apply(&[styled], &raw, 4, &Theme::default());
        let code = guided[0].spans.last().expect("code span");

        assert_eq!(code.content.as_ref(), "fn");
        assert_eq!(code.style.fg, Some(Color::Red));
    }
}
