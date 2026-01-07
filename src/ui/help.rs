use crate::app::App;
use crate::constants::{SEARCH_INPUT_HEIGHT, SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.config.theme;

    // Calculate centered popup size
    let popup_width = (area.width * 70) / 100; // 70% width for help
    let popup_height = (area.height * SEARCH_POPUP_HEIGHT_PERCENT) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear area and render opaque popup block
    frame.render_widget(Clear, popup_area);
    frame.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(theme.bg_search)),
        popup_area,
    );

    let inner = popup_area.inner(Margin::new(SEARCH_POPUP_MARGIN, SEARCH_POPUP_MARGIN));
    let [header_area, shortcuts_area] =
        Layout::vertical([Constraint::Length(SEARCH_INPUT_HEIGHT), Constraint::Min(0)])
            .areas(inner);

    // Render header
    let header = Paragraph::new("Keyboard Shortcuts")
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search));
    frame.render_widget(header, header_area);

    let help_text = "UP/DOWN, j/k     Scroll file list / preview\n\
        PgUp/PgDn        Page up/down in preview\n\
        h/l, ←/→         Go back / Enter directory\n\
        Shift+H / Shift+L  Shrink / Grow file list pane\n\
        \n\
        /                Open file search\n\
        f                Open symbol search\n\
        g                Toggle git highlighting\n\
        d                Toggle git diff view\n\
        t                Cycle through themes\n\
        ?                Show this help\n\
        \n\
        Click+Drag       Select text in preview\n\
        Ctrl+c           Copy selected text\n\
        q, Esc           Quit";

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, shortcuts_area);
}
