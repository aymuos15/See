use super::Theme;
use ratatui::style::Color;

/// Jellybeans theme - dark purple background with cream file list
pub const fn jellybeans() -> Theme {
    Theme {
        bg_main: Color::Rgb(0x15, 0x15, 0x15),      // Dark background
        bg_darker: Color::Rgb(0x10, 0x10, 0x10),    // Darker background
        bg_selected: Color::Rgb(0x30, 0x30, 0x30),  // Selection background
        bg_search: Color::Rgb(0x1c, 0x1c, 0x1c),    // Search background
        bg_selection: Color::Rgb(0x40, 0x40, 0x40), // Selection highlight
        fg_text: Color::Rgb(0xe8, 0xe8, 0xe8),      // Light text
        fg_selected: Color::Rgb(0xff, 0xff, 0xff),  // Selected text
        fg_dim: Color::Rgb(0x80, 0x80, 0x80),       // Dim text
        fg_folder: Color::Rgb(0x81, 0xa2, 0xbe),    // Blue folders
        border: Color::Rgb(0x50, 0x50, 0x50),       // Border color
        line_num: Color::Rgb(0x60, 0x60, 0x60),     // Line numbers
    }
}

/// Dracula theme - dark background with purple tones
pub const fn dracula() -> Theme {
    Theme {
        bg_main: Color::Rgb(0x28, 0x2a, 0x36), // Dracula dark background
        bg_darker: Color::Rgb(0xf8, 0xf8, 0xf2), // Dracula light text/list background
        bg_selected: Color::Rgb(0x44, 0x47, 0x5a), // Dracula darker background
        bg_search: Color::Rgb(0x62, 0x72, 0xa4), // Dracula comment color for search
        bg_selection: Color::Rgb(0x44, 0x47, 0x5a), // Dracula selection
        fg_text: Color::Rgb(0xf8, 0xf8, 0xf2), // Dracula main text
        fg_selected: Color::Rgb(0xf1, 0xfa, 0x8c), // Dracula yellow
        fg_dim: Color::Rgb(0x62, 0x72, 0xa4),  // Dracula comment gray
        fg_folder: Color::Rgb(0x8b, 0xe9, 0xfd), // Dracula cyan
        border: Color::Rgb(0x62, 0x72, 0xa4),  // Dracula comment color
        line_num: Color::Rgb(0x62, 0x72, 0xa4), // Dracula comment color
    }
}
