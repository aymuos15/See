use crate::app::App;
use crate::constants::{SEARCH_INPUT_HEIGHT, SEARCH_POPUP_HEIGHT_PERCENT, SEARCH_POPUP_MARGIN};
use crate::ui::popup;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.config.theme;

    // Calculate centered popup size
    let popup_area = popup::centered_popup(area, 70, SEARCH_POPUP_HEIGHT_PERCENT);
    popup::render_popup_background(frame, popup_area, theme.bg_search);

    let inner = popup::popup_inner(popup_area, SEARCH_POPUP_MARGIN);
    let (header_area, shortcuts_area) = popup::split_popup(inner, SEARCH_INPUT_HEIGHT);

    // Render header
    let header = Paragraph::new("Keyboard Shortcuts")
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search));
    frame.render_widget(header, header_area);

    let help_text = "UP/DOWN, j/k       Scroll file list / preview\n\
        PgUp/PgDn          Page up/down in preview\n\
        h/l, ←/→           Go back / Enter directory\n\
        Shift+H / Shift+L  Shrink / Grow file list pane\n\
        \n\
        /                  Open file search\n\
        f                  Open symbol search\n\
        g                  Toggle git highlighting\n\
        d                  Toggle git diff view\n\
        t                  Cycle through themes\n\
        Ctrl+t             Toggle global file tree\n\
        ?                  Show this help\n\
        \n\
        Alt+↑/↓/←/→        Split pane up/down/left/right\n\
        Alt+h / Alt+l      Resize active split left/right\n\
        \n\
        Click+Drag         Select text in preview\n\
        Ctrl+c             Copy selected text\n\
        q, Esc             Quit";

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(theme.fg_text).bg(theme.bg_search))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, shortcuts_area);
}
