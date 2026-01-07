use super::Theme;
use ratatui::style::Color;

/// Jellybeans theme - dark purple background with cream file list
pub fn jellybeans() -> Theme {
    Theme {
        bg_main: Color::Rgb(0x4c, 0x3d, 0x57),     // Jellybeans background (#4c3d57)
        bg_darker: Color::Rgb(0xe8, 0xe0, 0xd0),   // dark cream (#e8e0d0)
        bg_selected: Color::Rgb(0x29, 0x29, 0x29), // Jellybeans darker
        bg_search: Color::Rgb(0xcf, 0x6a, 0x4c),   // Jellybeans dark orange
        bg_selection: Color::Rgb(0x37, 0x23, 0x2d),// Jellybeans selection
        fg_text: Color::Rgb(0x5a, 0x5a, 0x5a),     // dark gray
        fg_selected: Color::Rgb(0xf2, 0xaa, 0xc7), // Jellybeans selection fg
        fg_dim: Color::Rgb(0x8a, 0x8a, 0x8a),      // medium gray
        fg_folder: Color::Rgb(0x4a, 0x6a, 0x8a),   // dark blue
        fg_modified: Color::Rgb(0xa8, 0x4a, 0x4a), // dark red
        border: Color::Rgb(0x6d, 0x6d, 0x6d),      // Jellybeans light gray
        line_num: Color::Rgb(0x53, 0x53, 0x53),    // Jellybeans dark gray
    }
}

/// Dracula theme - dark background with purple tones
pub fn dracula() -> Theme {
    Theme {
        bg_main: Color::Rgb(0x28, 0x2a, 0x36),     // Dracula dark background
        bg_darker: Color::Rgb(0xf8, 0xf8, 0xf2),   // Dracula light text/list background
        bg_selected: Color::Rgb(0x44, 0x47, 0x5a),  // Dracula darker background
        bg_search: Color::Rgb(0x62, 0x72, 0xa4),   // Dracula comment color for search
        bg_selection: Color::Rgb(0x44, 0x47, 0x5a), // Dracula selection
        fg_text: Color::Rgb(0xf8, 0xf8, 0xf2),     // Dracula main text
        fg_selected: Color::Rgb(0xf1, 0xfa, 0x8c), // Dracula yellow
        fg_dim: Color::Rgb(0x62, 0x72, 0xa4),      // Dracula comment gray
        fg_folder: Color::Rgb(0x8b, 0xe9, 0xfd),   // Dracula cyan
        fg_modified: Color::Rgb(0xff, 0x79, 0xc6), // Dracula pink
        border: Color::Rgb(0x62, 0x72, 0xa4),      // Dracula comment color
        line_num: Color::Rgb(0x62, 0x72, 0xa4),    // Dracula comment color
    }
}
