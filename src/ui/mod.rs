pub mod coordinates;
pub mod file_list;
pub mod file_tree_popup;
pub mod help;
pub mod layout;
pub mod pane;
pub mod popup;
pub mod preview;
pub mod search;
pub mod tab_bar;
pub mod theme_picker;

use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

#[allow(clippy::too_many_lines)]
pub fn render(frame: &mut Frame, app: &mut App) {
    // Fill entire frame with background color first
    let bg_block = Block::default().style(Style::default().bg(app.config.theme.bg_main));
    frame.render_widget(bg_block, frame.area());

    let layout = layout::AppLayout::new(
        frame.area(),
        &app.split_layout,
        app.split_percent,
        app.config.divider_width,
    );

    if let Some(file_list_area) = layout.file_list_area {
        file_list::render(frame, app, file_list_area);
        app.last_file_list_area = Some(file_list_area);
    } else {
        app.last_file_list_area = None;
    }

    app.last_pane_areas.clone_from(&layout.pane_areas);

    if let Some(ref split_layout) = app.split_layout {
        tab_bar::render(frame, split_layout, layout.tab_bar_area, &app.config.theme);
        for (pane_id, area) in &layout.pane_areas {
            if let Some(pane) = split_layout.panes.iter().find(|p| p.id == *pane_id) {
                pane::render(
                    frame,
                    pane,
                    *area,
                    &app.config.theme,
                    pane.id == split_layout.active_pane_index,
                    app.config.wrap,
                    app.highlighted_word.as_deref(),
                    &mut app.image_protocols,
                );
            }
        }

        // Render dividers between panes
        let divider_width = app.config.divider_width;

        if divider_width == 1 {
            // Thin line characters for width 1
            let horizontal_dividers: Vec<&Rect> =
                layout.dividers.iter().filter(|d| d.height == 1).collect();
            let vertical_dividers: Vec<&Rect> =
                layout.dividers.iter().filter(|d| d.height > 1).collect();

            for divider in &layout.dividers {
                if divider.height > 1 {
                    // Vertical divider
                    let lines: Vec<Line> = (0..divider.height)
                        .map(|row| {
                            let y = divider.y + row;
                            let mut ch = "│";
                            for h in &horizontal_dividers {
                                if h.y == y {
                                    let h_start = h.x;
                                    let h_end = h.x + h.width;
                                    if divider.x == h_start.saturating_sub(1) {
                                        ch = "├";
                                    } else if divider.x == h_end {
                                        ch = "┤";
                                    } else if divider.x >= h_start && divider.x < h_end {
                                        ch = "┼";
                                    }
                                }
                            }
                            Line::from(ch).style(Style::default().fg(app.config.theme.fg_dim))
                        })
                        .collect();
                    let divider_widget =
                        Paragraph::new(lines).style(Style::default().bg(app.config.theme.bg_main));
                    frame.render_widget(divider_widget, *divider);
                } else {
                    // Horizontal divider
                    let line_str: String = (0..divider.width)
                        .map(|col| {
                            let x = divider.x + col;
                            for v in &vertical_dividers {
                                if x >= v.x
                                    && x < v.x + v.width
                                    && divider.y >= v.y
                                    && divider.y < v.y + v.height
                                {
                                    return '┼';
                                }
                            }
                            '─'
                        })
                        .collect();
                    let line =
                        Line::from(line_str).style(Style::default().fg(app.config.theme.fg_dim));
                    let divider_widget =
                        Paragraph::new(line).style(Style::default().bg(app.config.theme.bg_main));
                    frame.render_widget(divider_widget, *divider);
                }
            }
        } else {
            // Solid block for wider dividers
            for divider in &layout.dividers {
                let divider_block =
                    Block::default().style(Style::default().bg(app.config.theme.fg_dim));
                frame.render_widget(divider_block, *divider);
            }
        }
    } else {
        preview::render(frame, app, layout.pane_areas[0].1);
    }

    // Render search popup overlay if active (file or symbol search)
    if app.search_mode || app.symbol_search_mode || app.find_mode {
        search::render(frame, app);
    }

    // Render theme picker popup if active
    if app.theme_preview_mode {
        theme_picker::render(frame, app);
    }

    // Render help overlay if active
    if app.help_mode {
        help::render(frame, app, frame.area());
    }

    // Render file tree popup if active
    if app.file_tree_popup_mode {
        file_tree_popup::render(frame, app);
    }
}
