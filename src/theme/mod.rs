mod parser;

use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg_main: Color,
    pub bg_darker: Color,
    pub bg_selected: Color,
    pub bg_search: Color,
    pub fg_text: Color,
    pub fg_selected: Color,
    pub fg_dim: Color,
    pub fg_folder: Color,
    pub border: Color,
    pub line_num: Color,
}

impl Default for Theme {
    fn default() -> Self {
        // Default Helix purple theme
        Self {
            bg_main: Color::Rgb(0x3b, 0x22, 0x4c),     // midnight
            bg_darker: Color::Rgb(0x28, 0x17, 0x33),   // revolver
            bg_selected: Color::Rgb(0x45, 0x28, 0x59), // bossanova
            bg_search: Color::Rgb(0xd9, 0x73, 0x0d),   // orange
            fg_text: Color::Rgb(0xa4, 0xa0, 0xe8),     // lavender
            fg_selected: Color::Rgb(0x9f, 0xf2, 0x8f), // mint
            fg_dim: Color::Rgb(0x69, 0x7c, 0x81),      // sirocco
            fg_folder: Color::Rgb(0x5c, 0xc9, 0xf5),   // bright cyan
            border: Color::Rgb(0x5a, 0x59, 0x77),      // comet
            line_num: Color::Rgb(0x5a, 0x59, 0x77),    // comet
        }
    }
}

#[derive(Deserialize, Default)]
struct HelixConfig {
    theme: Option<String>,
}

impl Theme {
    pub fn load() -> Self {
        // Try to load user's helix theme
        if let Some(theme) = Self::load_helix_theme() {
            return theme;
        }
        Self::default()
    }

    fn load_helix_theme() -> Option<Self> {
        // Get theme name from helix config
        let config_path = dirs::home_dir()?.join(".config/helix/config.toml");
        let config_content = fs::read_to_string(&config_path).ok()?;
        let config: HelixConfig = toml::from_str(&config_content).ok()?;
        let theme_name = config.theme.unwrap_or_else(|| "default".to_string());

        // Try loading theme from various locations
        let theme_paths = [
            dirs::home_dir()?.join(format!(".config/helix/themes/{theme_name}.toml")),
            PathBuf::from(format!("/usr/lib/helix/runtime/themes/{theme_name}.toml")),
            PathBuf::from(format!("/usr/share/helix/runtime/themes/{theme_name}.toml")),
        ];

        for path in &theme_paths {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(theme) = parser::parse_theme(&content) {
                    return Some(theme);
                }
            }
        }

        // If theme not found, try loading default helix theme.toml from repo-style location
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        let color = parser::parse_hex_color("#FF00AA");
        assert!(color.is_some());
        match color.unwrap() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 170);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_parse_hex_color_lowercase() {
        let color = parser::parse_hex_color("#ff00aa");
        assert!(color.is_some());
        match color.unwrap() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 170);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_parse_hex_color_no_hash() {
        let color = parser::parse_hex_color("FF00AA");
        assert!(color.is_some());
        match color.unwrap() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 170);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_parse_hex_color_black() {
        let color = parser::parse_hex_color("#000000");
        assert!(color.is_some());
        match color.unwrap() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 0);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_parse_hex_color_white() {
        let color = parser::parse_hex_color("#FFFFFF");
        assert!(color.is_some());
        match color.unwrap() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 255);
                assert_eq!(b, 255);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_parse_hex_color_invalid_length() {
        assert!(parser::parse_hex_color("#FFF").is_none());
        assert!(parser::parse_hex_color("#FF00AA00").is_none());
        assert!(parser::parse_hex_color("#").is_none());
    }

    #[test]
    fn test_parse_hex_color_invalid_chars() {
        assert!(parser::parse_hex_color("#GGGGGG").is_none());
        assert!(parser::parse_hex_color("#ZZ00AA").is_none());
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        // Just verify it has non-default color values
        assert!(!matches!(theme.bg_main, Color::Reset));
        assert!(!matches!(theme.fg_text, Color::Reset));
    }

    #[test]
    fn test_parse_theme_empty() {
        let result = parser::parse_theme("");
        assert!(result.is_ok());
        // Should return default theme on empty input
        let theme = result.unwrap();
        assert!(!matches!(theme.bg_main, Color::Reset));
    }

    #[test]
    fn test_parse_theme_minimal() {
        let toml_content = "\
[palette]\n\
background = \"#1e1e1e\"\n\
foreground = \"#ffffff\"\n\
";
        let result = parser::parse_theme(toml_content);
        assert!(result.is_ok());
    }
}
