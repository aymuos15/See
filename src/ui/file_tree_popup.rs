use crate::app::App;
use crate::constants::{SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_WIDTH_PERCENT};
use crate::theme::{darken, Theme};
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState};

/// One level of tree indentation.
const INDENT: &str = "  ";

/// How far the count column sits below the dim foreground, so the numbers
/// stay secondary to the paths they annotate.
const COUNT_DARKEN: u16 = 60;

/// Renders the file tree popup showing all files recursively.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.config.theme;

    let panel = popup::centered_popup(
        frame.area(),
        SEARCH_POPUP_WIDTH_PERCENT,
        SEARCH_POPUP_HEIGHT_PERCENT,
    );
    let inner = popup::render_panel(frame, panel, "File Tree", theme);

    if app.file_tree_popup_entries.is_empty() {
        popup::render_empty_message(frame, inner, "No files found", theme);
        return;
    }

    let width = inner.width as usize;
    let items: Vec<ListItem> = app
        .file_tree_popup_entries
        .iter()
        .map(|row_data| {
            let entry = &row_data.entry;

            // Indentation carries the hierarchy; directories add a chevron and
            // the folder color, files sit flush beneath their parent.
            let indent = INDENT.repeat(row_data.depth);
            let (marker, color) = if entry.is_file {
                ("  ", theme.fg_text)
            } else {
                ("▸ ", theme.fg_folder)
            };

            row(
                &format!("{indent}{marker}{}", entry.name),
                &count_label(app, entry),
                width,
                color,
                theme,
            )
        })
        .collect();

    // No selection marker: the highlighted row alone shows the position.
    let list = List::new(items)
        .style(Style::default().bg(theme.bg_search))
        .highlight_style(Style::default().bg(theme.bg_selected).fg(theme.fg_selected));

    let mut state = ListState::default();
    state.select(Some(app.file_tree_popup_selected));

    frame.render_stateful_widget(list, inner, &mut state);

    // The list keeps the selection just inside the view, so its offset after
    // rendering is the first visible row — exactly the scrollbar position.
    crate::ui::preview::render_scrollbar(
        frame,
        inner,
        theme,
        app.file_tree_popup_entries.len(),
        state.offset(),
    );
}

/// Lays a path out on the left with its counts right-aligned, so the numbers
/// form their own column regardless of path length.
fn row(path: &str, count: &str, width: usize, color: Color, theme: &Theme) -> ListItem<'static> {
    let gap = width
        .saturating_sub(path.chars().count())
        .saturating_sub(count.chars().count());

    ListItem::new(Line::from(vec![
        Span::styled(path.to_string(), Style::default().fg(color)),
        Span::raw(" ".repeat(gap.max(1))),
        Span::styled(
            count.to_string(),
            Style::default().fg(darken(theme.fg_dim, COUNT_DARKEN)),
        ),
    ]))
    .style(Style::default().bg(theme.bg_search))
}

/// Lines for a file; file count and total lines for a directory. Empty while
/// the worker is still counting.
fn count_label(app: &App, entry: &crate::files::FileEntry) -> String {
    let counts = &app.tree_line_counts;

    if entry.is_file {
        return counts
            .files
            .get(&entry.path)
            .map_or_else(String::new, |lines| format!("{} LOC", thousands(*lines)));
    }

    counts
        .directories
        .get(&entry.path)
        .map_or_else(String::new, |&(files, lines)| {
            let unit = if files == 1 { "file" } else { "files" };
            format!("{} {unit} · {} LOC", thousands(files), thousands(lines))
        })
}

/// Groups digits so large counts stay readable.
fn thousands(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(42), "42");
        assert_eq!(thousands(1234), "1,234");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }
}
