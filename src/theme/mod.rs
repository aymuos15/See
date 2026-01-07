mod builtins;
mod parser;

use crate::config::ThemeConfig;
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
    pub bg_selection: Color,
    pub fg_text: Color,
    pub fg_selected: Color,
    pub fg_dim: Color,
    pub fg_folder: Color,
    pub fg_modified: Color,
    pub border: Color,
    pub line_num: Color,
}

impl Default for Theme {
    fn default() -> Self {
        builtins::jellybeans()
    }
}

impl Theme {
    /// Get theme by name (built-in themes)
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "jellybeans" => Some(builtins::jellybeans()),
            "dracula" => Some(builtins::dracula()),
            _ => None,
        }
    }

    /// List all available built-in theme names
    pub fn list_builtins() -> Vec<&'static str> {
        vec!["jellybeans", "dracula"]
    }
}

#[derive(Deserialize, Default)]
struct HelixConfig {
    theme: Option<String>,
}

impl Theme {
    pub fn from_config(config: Option<ThemeConfig>) -> Self {
        if let Some(cfg) = config {
            // If helix_theme is specified, try to load from helix
            if let Some(ref theme_name) = cfg.helix_theme {
                if let Some(theme) = Self::load_helix_theme_by_name(theme_name) {
                    return theme;
                }
            }

            // Try to build theme from config values
            if Self::has_any_color(&cfg) {
                return Self::from_config_values(&cfg);
            }
        }

        // Fall back to helix theme or default
        if let Some(theme) = Self::load_helix_theme() {
            return theme;
        }

        Self::default()
    }

    fn from_config_values(cfg: &ThemeConfig) -> Self {
        let mut theme = Self::default();

        if let Some(hex) = &cfg.bg_main {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.bg_main = color;
            }
        }
        if let Some(hex) = &cfg.bg_darker {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.bg_darker = color;
            }
        }
        if let Some(hex) = &cfg.bg_selected {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.bg_selected = color;
            }
        }
        if let Some(hex) = &cfg.bg_search {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.bg_search = color;
            }
        }
        if let Some(hex) = &cfg.bg_selection {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.bg_selection = color;
            }
        }
        if let Some(hex) = &cfg.fg_text {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.fg_text = color;
            }
        }
        if let Some(hex) = &cfg.fg_selected {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.fg_selected = color;
            }
        }
        if let Some(hex) = &cfg.fg_dim {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.fg_dim = color;
            }
        }
        if let Some(hex) = &cfg.fg_folder {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.fg_folder = color;
            }
        }
        if let Some(hex) = &cfg.fg_modified {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.fg_modified = color;
            }
        }
        if let Some(hex) = &cfg.border {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.border = color;
            }
        }
        if let Some(hex) = &cfg.line_num {
            if let Some(color) = parser::parse_hex_color(hex) {
                theme.line_num = color;
            }
        }

        theme
    }

    const fn has_any_color(cfg: &ThemeConfig) -> bool {
        cfg.bg_main.is_some()
            || cfg.bg_darker.is_some()
            || cfg.bg_selected.is_some()
            || cfg.bg_search.is_some()
            || cfg.bg_selection.is_some()
            || cfg.fg_text.is_some()
            || cfg.fg_selected.is_some()
            || cfg.fg_dim.is_some()
            || cfg.fg_folder.is_some()
            || cfg.fg_modified.is_some()
            || cfg.border.is_some()
            || cfg.line_num.is_some()
    }

    fn load_helix_theme() -> Option<Self> {
        // Get theme name from helix config
        let config_path = dirs::home_dir()?.join(".config/helix/config.toml");
        let config_content = fs::read_to_string(&config_path).ok()?;
        let config: HelixConfig = toml::from_str(&config_content).ok()?;
        let theme_name = config.theme.unwrap_or_else(|| "default".to_string());

        Self::load_helix_theme_by_name(&theme_name)
    }

    fn load_helix_theme_by_name(theme_name: &str) -> Option<Self> {
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

    #[test]
    fn test_theme_from_config_empty() {
        let theme = Theme::from_config(None);
        // Should fall back to default theme
        assert!(!matches!(theme.bg_main, Color::Reset));
    }

    #[test]
    fn test_theme_from_config_with_colors() {
        use crate::config::ThemeConfig;

        let cfg = ThemeConfig {
            helix_theme: None,
            bg_main: Some("#1a1a1a".to_string()),
            bg_darker: Some("#0f0f0f".to_string()),
            bg_selected: Some("#2a2a2a".to_string()),
            bg_search: Some("#ff6600".to_string()),
            bg_selection: Some("#ffff00".to_string()),
            fg_text: Some("#e0e0e0".to_string()),
            fg_selected: Some("#00ff00".to_string()),
            fg_dim: Some("#808080".to_string()),
            fg_folder: Some("#00ccff".to_string()),
            fg_modified: Some("#ff9900".to_string()),
            border: Some("#666666".to_string()),
            line_num: Some("#666666".to_string()),
        };

        let theme = Theme::from_config(Some(cfg));

        // Verify colors were set
        match theme.bg_main {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 0x1a);
                assert_eq!(g, 0x1a);
                assert_eq!(b, 0x1a);
            }
            _ => panic!("Expected RGB color for bg_main"),
        }
    }

    #[test]
    fn test_theme_from_config_partial_colors() {
        use crate::config::ThemeConfig;

        let cfg = ThemeConfig {
            helix_theme: None,
            bg_main: Some("#1a1a1a".to_string()),
            bg_darker: None,
            bg_selected: None,
            bg_search: None,
            bg_selection: None,
            fg_text: Some("#e0e0e0".to_string()),
            fg_selected: None,
            fg_dim: None,
            fg_folder: None,
            fg_modified: None,
            border: None,
            line_num: None,
        };

        let theme = Theme::from_config(Some(cfg));

        // Verify specified colors were set
        match theme.bg_main {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 0x1a);
                assert_eq!(g, 0x1a);
                assert_eq!(b, 0x1a);
            }
            _ => panic!("Expected RGB color for bg_main"),
        }

        // Verify unspecified colors use defaults from jellybeans theme
        match theme.bg_darker {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 0xe8);
                assert_eq!(g, 0xe0);
                assert_eq!(b, 0xd0);
            }
            _ => panic!("Expected default Jellybeans color for bg_darker"),
        }
    }
}
