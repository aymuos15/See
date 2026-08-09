use crate::app::App;
use crate::theme::{darken, Theme};
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// A row in the help panel: either a group label or a binding.
enum Row {
    Section(&'static str),
    Binding(&'static str, &'static str),
}

use Row::{Binding, Section};

const ROWS: &[Row] = &[
    Section("Navigate"),
    Binding("j k  ↑ ↓", "Move through the file list"),
    Binding("h l  ← →", "Go back / enter directory"),
    Binding("PgUp PgDn", "Page through the preview"),
    Binding("H  L", "Shrink / grow the file list"),
    Section("Find"),
    Binding("/", "Search files"),
    Binding("?  \\", "Find word in the open file"),
    Binding("n  p", "Next / previous match"),
    Binding("f", "Search symbols"),
    Binding("Ctrl+t", "Global file tree"),
    Binding("Ctrl+b", "Show / hide the file list"),
    Section("Git"),
    Binding("Shift+G", "Browse commits / leave git mode"),
    Binding("Enter", "Open the selected commit's diff"),
    Binding("j k  ↑ ↓", "Move through commits or changed files"),
    Binding("Esc  Backspace", "Back to the commit list"),
    Section("Panes"),
    Binding("Alt+↑↓←→", "Split up / down / left / right"),
    Binding("Alt+h  Alt+l", "Resize the active split"),
    Binding("Tab", "Cycle panes"),
    Binding("Alt+q", "Close the active pane"),
    Binding("Alt+s", "Swap split orientation"),
    Section("Editor"),
    Binding("t", "Cycle themes"),
    Binding("Click+Drag", "Select text in the preview"),
    Binding("Ctrl+c", "Copy selection"),
    Binding("q  Esc", "Quit"),
];

/// How far group labels are pushed toward the background — quiet enough to
/// recede behind the bindings they head.
const SECTION_DARKEN: u16 = 60;
/// Width of the key column; descriptions align just past it.
const KEY_COLUMN: usize = 14;
/// Panel width: key column, description, borders and padding.
const PANEL_WIDTH: u16 = 58;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.config.theme;

    let panel = popup::centered_sized(area, PANEL_WIDTH, panel_height());
    let inner = popup::render_panel(frame, panel, "Keyboard Shortcuts", theme);

    frame.render_widget(
        Paragraph::new(lines(theme)).style(Style::default().bg(theme.bg_search)),
        inner,
    );
}

/// Body rows, a blank line above each group but the first, the footer and its
/// spacer, plus the panel's title row and vertical padding.
fn panel_height() -> u16 {
    let sections = ROWS
        .iter()
        .filter(|row| matches!(row, Section(_)))
        .count()
        .saturating_sub(1);
    let rows = ROWS.len() + sections + 2;

    u16::try_from(rows).unwrap_or(u16::MAX).saturating_add(3)
}

fn lines(theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(ROWS.len() + 6);

    for (i, row) in ROWS.iter().enumerate() {
        match *row {
            Section(label) => {
                if i > 0 {
                    lines.push(Line::default());
                }
                lines.push(section_line(label, theme));
            }
            Binding(keys, description) => lines.push(binding_line(keys, description, theme)),
        }
    }

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "F1  toggle this panel",
        Style::default().fg(theme.fg_dim),
    )));

    lines
}

/// A dim, letter-spaced group label — quiet enough to scan past.
fn section_line(label: &str, theme: &Theme) -> Line<'static> {
    let spaced = label
        .to_uppercase()
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    Line::from(Span::styled(
        spaced,
        Style::default().fg(darken(theme.fg_dim, SECTION_DARKEN)),
    ))
}

/// Keys highlighted in the left column, description trailing them.
fn binding_line(keys: &str, description: &str, theme: &Theme) -> Line<'static> {
    let padding = " ".repeat(KEY_COLUMN.saturating_sub(keys.chars().count()));

    Line::from(vec![
        Span::styled(
            keys.to_string(),
            Style::default()
                .fg(theme.fg_selected)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(padding),
        Span::styled(description.to_string(), Style::default().fg(theme.fg_text)),
    ])
}
