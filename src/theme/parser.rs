use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;

use super::Theme;

#[derive(Deserialize, Default)]
pub(super) struct ThemeFile {
    #[serde(default)]
    pub(super) palette: HashMap<String, String>,
    #[serde(flatten)]
    pub(super) styles: HashMap<String, toml::Value>,
}

pub(super) fn parse_theme(content: &str) -> anyhow::Result<Theme> {
    let theme_file: ThemeFile = toml::from_str(content)?;
    let palette = &theme_file.palette;

    let mut theme = Theme::default();

    // Helper to resolve color from palette or hex
    let resolve_color = |name: &str, palette: &HashMap<String, String>| -> Option<Color> {
        let color_str = palette.get(name).map_or(name, |s| s.as_str());
        parse_hex_color(color_str)
    };

    // Extract UI colors from styles
    if let Some(bg) = get_style_bg(&theme_file.styles, "ui.background", palette) {
        theme.bg_main = bg;
    }
    if let Some(bg) = get_style_bg(&theme_file.styles, "ui.popup", palette) {
        theme.bg_darker = bg;
    }
    if let Some(bg) = get_style_bg(&theme_file.styles, "ui.selection", palette) {
        theme.bg_selected = bg;
    }
    if let Some(fg) = get_style_fg(&theme_file.styles, "ui.text", palette) {
        theme.fg_text = fg;
    }
    if let Some(fg) = get_style_fg(&theme_file.styles, "ui.text.focus", palette) {
        theme.fg_selected = fg;
    }
    if let Some(fg) = get_style_fg(&theme_file.styles, "ui.linenr", palette) {
        theme.line_num = fg;
    }
    if let Some(fg) = get_style_fg(&theme_file.styles, "ui.window", palette) {
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

pub(super) fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

pub(super) fn get_style_bg(
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

pub(super) fn get_style_fg(
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
