use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;
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

#[derive(Deserialize, Default)]
struct ThemeFile {
    #[serde(default)]
    palette: HashMap<String, String>,
    #[serde(flatten)]
    styles: HashMap<String, toml::Value>,
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
                if let Ok(theme) = Self::parse_theme(&content) {
                    return Some(theme);
                }
            }
        }

        // If theme not found, try loading default helix theme.toml from repo-style location
        None
    }

    fn parse_theme(content: &str) -> anyhow::Result<Self> {
        let theme_file: ThemeFile = toml::from_str(content)?;
        let palette = &theme_file.palette;

        let mut theme = Self::default();

        // Helper to resolve color from palette or hex
        let resolve_color = |name: &str, palette: &HashMap<String, String>| -> Option<Color> {
            let color_str = palette.get(name).map_or(name, |s| s.as_str());
            parse_hex_color(color_str)
        };

        // Extract UI colors from styles
        if let Some(bg) = Self::get_style_bg(&theme_file.styles, "ui.background", palette) {
            theme.bg_main = bg;
        }
        if let Some(bg) = Self::get_style_bg(&theme_file.styles, "ui.popup", palette) {
            theme.bg_darker = bg;
        }
        if let Some(bg) = Self::get_style_bg(&theme_file.styles, "ui.selection", palette) {
            theme.bg_selected = bg;
        }
        if let Some(fg) = Self::get_style_fg(&theme_file.styles, "ui.text", palette) {
            theme.fg_text = fg;
        }
        if let Some(fg) = Self::get_style_fg(&theme_file.styles, "ui.text.focus", palette) {
            theme.fg_selected = fg;
        }
        if let Some(fg) = Self::get_style_fg(&theme_file.styles, "ui.linenr", palette) {
            theme.line_num = fg;
        }
        if let Some(fg) = Self::get_style_fg(&theme_file.styles, "ui.window", palette) {
            theme.border = fg;
        }

        // Try to get colors from palette directly for common names
        if let Some(c) = resolve_color("background", palette) {
            theme.bg_main = c;
        }
        if let Some(c) = resolve_color("foreground", palette) {
            theme.fg_text = c;
        }

        Ok(theme)
    }

    fn get_style_bg(
        styles: &HashMap<String, toml::Value>,
        key: &str,
        palette: &HashMap<String, String>,
    ) -> Option<Color> {
        let style = styles.get(key)?;
        match style {
            toml::Value::Table(t) => {
                let bg = t.get("bg")?.as_str()?;
                let resolved = palette.get(bg).map_or(bg, |s| s.as_str());
                parse_hex_color(resolved)
            }
            _ => None,
        }
    }

    fn get_style_fg(
        styles: &HashMap<String, toml::Value>,
        key: &str,
        palette: &HashMap<String, String>,
    ) -> Option<Color> {
        let style = styles.get(key)?;
        match style {
            toml::Value::Table(t) => {
                let fg = t.get("fg")?.as_str()?;
                let resolved = palette.get(fg).map_or(fg, |s| s.as_str());
                parse_hex_color(resolved)
            }
            toml::Value::String(s) => {
                let resolved = palette.get(s.as_str()).map_or(s.as_str(), |s| s.as_str());
                parse_hex_color(resolved)
            }
            _ => None,
        }
    }
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        let color = parse_hex_color("#FF00AA");
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
        let color = parse_hex_color("#ff00aa");
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
        let color = parse_hex_color("FF00AA");
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
        let color = parse_hex_color("#000000");
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
        let color = parse_hex_color("#FFFFFF");
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
        assert!(parse_hex_color("#FFF").is_none());
        assert!(parse_hex_color("#FF00AA00").is_none());
        assert!(parse_hex_color("#").is_none());
    }

    #[test]
    fn test_parse_hex_color_invalid_chars() {
        assert!(parse_hex_color("#GGGGGG").is_none());
        assert!(parse_hex_color("#ZZ00AA").is_none());
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
        let result = Theme::parse_theme("");
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
        let result = Theme::parse_theme(toml_content);
        assert!(result.is_ok());
    }
}
