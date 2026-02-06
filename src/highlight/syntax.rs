use crate::constants::DEFAULT_SYNTAX_THEME;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight(&self, path: &Path, content: &str) -> Vec<Line<'static>> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");

        // TOML files use INI syntax (similar structure: [sections] and key = value)
        let ext_for_lookup = if extension == "toml" {
            "ini"
        } else {
            extension
        };

        let syntax = self
            .syntax_set
            .find_syntax_by_extension(ext_for_lookup)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes[DEFAULT_SYNTAX_THEME];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();

        for line in LinesWithEndings::from(content) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(segments) => {
                    let spans: Vec<Span<'static>> = segments
                        .into_iter()
                        .map(|(style, text)| {
                            let fg = Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            );

                            let mut modifier = Modifier::empty();
                            if style.font_style.contains(FontStyle::BOLD) {
                                modifier |= Modifier::BOLD;
                            }
                            if style.font_style.contains(FontStyle::ITALIC) {
                                modifier |= Modifier::ITALIC;
                            }
                            if style.font_style.contains(FontStyle::UNDERLINE) {
                                modifier |= Modifier::UNDERLINED;
                            }

                            Span::styled(
                                text.to_owned(),
                                Style::default().fg(fg).add_modifier(modifier),
                            )
                        })
                        .collect();
                    lines.push(Line::from(spans));
                }
                Err(_) => {
                    lines.push(Line::from(line.trim_end().to_owned()));
                }
            }
        }

        lines
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_syntax_highlighter_new() {
        let highlighter = SyntaxHighlighter::new();
        // Just verify it doesn't panic
        assert!(!highlighter.syntax_set.syntaxes().is_empty());
    }

    #[test]
    fn test_syntax_highlighter_default() {
        let highlighter = SyntaxHighlighter::default();
        assert!(!highlighter.syntax_set.syntaxes().is_empty());
    }

    #[test]
    fn test_highlight_rust_code() {
        let highlighter = SyntaxHighlighter::new();
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

        let lines = highlighter.highlight(Path::new("main.rs"), code);
        assert!(!lines.is_empty());
        // Should produce highlighted output
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_highlight_plain_text() {
        let highlighter = SyntaxHighlighter::new();
        let code = "Plain text content";

        let lines = highlighter.highlight(Path::new("file.txt"), code);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_highlight_unknown_extension() {
        let highlighter = SyntaxHighlighter::new();
        let code = "Some content";

        // Unknown extension should fall back to plain text
        let lines = highlighter.highlight(Path::new("file.unknown"), code);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_highlight_multiline() {
        let highlighter = SyntaxHighlighter::new();
        let code = "line 1\nline 2\nline 3";

        let lines = highlighter.highlight(Path::new("file.txt"), code);
        // Should preserve line structure
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_highlight_empty_content() {
        let highlighter = SyntaxHighlighter::new();
        let code = "";

        let lines = highlighter.highlight(Path::new("file.txt"), code);
        assert!(lines.is_empty() || lines.len() == 1);
    }

    #[test]
    fn test_highlight_with_special_chars() {
        let highlighter = SyntaxHighlighter::new();
        let code = "Special chars: !@#$%^&*()";

        let lines = highlighter.highlight(Path::new("file.txt"), code);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_highlight_toml() {
        let highlighter = SyntaxHighlighter::new();
        let code = r#"[package]
name = "viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30"
"#;

        let lines = highlighter.highlight(Path::new("Cargo.toml"), code);
        assert!(!lines.is_empty());
        // Should produce highlighted output with multiple lines
        assert!(lines.len() > 1);
    }
}
