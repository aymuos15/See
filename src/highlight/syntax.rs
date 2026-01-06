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

        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
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
