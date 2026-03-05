use super::{App, PreviewContentType};
use crate::files::read_file_content;
use crate::git::generate_diff_lines;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::path::Path;
use std::rc::Rc;

impl App {
    /// Toggle between normal view and diff view
    #[allow(dead_code)]
    pub(super) fn toggle_diff(&mut self) {
        // Only works on files, not directories
        if let Some(idx) = self.file_list_state.selected() {
            // Clone path to avoid borrow issues
            let file_path = self.files.get(idx).and_then(|entry| {
                if entry.is_file {
                    Some(entry.path.clone())
                } else {
                    None
                }
            });

            if let Some(path) = file_path {
                // Initialize git status if not already done
                if self.git_status.is_none() {
                    self.git_status = Some(crate::git::GitStatus::new(&self.current_dir));
                }

                // Check if file is modified
                let is_modified = self.git_status.as_ref().is_some_and(|status| {
                    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                    status.is_modified(&canonical_path)
                });

                if !is_modified {
                    return; // Silent no-op for unmodified files
                }

                if self.diff_mode {
                    // Restore original content
                    self.restore_original_content();
                } else {
                    // Generate and display diff
                    self.show_diff(&path);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn show_diff(&mut self, file_path: &Path) {
        // Read current file content
        if let Ok(current_content) = read_file_content(file_path) {
            // Generate diff lines
            if let Some(diff_lines) = generate_diff_lines(file_path, &current_content) {
                // Only proceed if we have diff lines (not empty)
                if !diff_lines.is_empty() {
                    // Cache original content for toggling back (Rc::clone is O(1))
                    self.original_preview_content =
                        self.shared_preview_content.as_ref().map(Rc::clone);

                    // Convert diff lines to styled Lines
                    let styled_lines = Self::style_diff_lines(&diff_lines);

                    // Update preview content with diff
                    self.shared_preview_content = Some(Rc::new(PreviewContentType::Text {
                        lines: styled_lines,
                        raw_lines: diff_lines,
                    }));

                    // Reset scroll and enter diff mode
                    self.preview_scroll = 0;
                    self.diff_mode = true;

                    // Clear selection in diff mode
                    self.selection = None;
                }
            }
        }
    }

    #[allow(dead_code)]
    fn restore_original_content(&mut self) {
        if let Some(original) = self.original_preview_content.take() {
            self.shared_preview_content = Some(original);
            self.preview_scroll = 0;
            self.diff_mode = false;
        }
    }

    #[allow(dead_code)]
    fn style_diff_lines(diff_lines: &[String]) -> Vec<Line<'static>> {
        diff_lines
            .iter()
            .map(|line| {
                if line.starts_with('+') && !line.starts_with("+++") {
                    // Added line - black text on green background
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::Black).bg(Color::LightGreen),
                    ))
                } else if line.starts_with('-') && !line.starts_with("---") {
                    // Deleted line - black text on red background
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::Black).bg(Color::LightRed),
                    ))
                } else if line.starts_with("@@") {
                    // Hunk header - bright yellow text with bold
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("diff ")
                    || line.starts_with("---")
                    || line.starts_with("+++")
                {
                    // File header lines - white
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::White),
                    ))
                } else {
                    // Context lines - white
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::White),
                    ))
                }
            })
            .collect()
    }
}
