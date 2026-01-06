use crate::event::{poll_event, AppEvent};
use crate::files::{read_directory, read_file_content, FileEntry};
use crate::highlight::SyntaxHighlighter;
use ratatui::text::Line;
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::time::Duration;

pub struct PreviewContent {
    pub path: PathBuf,
    pub lines: Vec<Line<'static>>,
}

pub struct App {
    pub current_dir: PathBuf,
    pub files: Vec<FileEntry>,
    pub file_list_state: ListState,
    pub preview_content: Option<PreviewContent>,
    pub preview_scroll: u16,
    pub highlighter: SyntaxHighlighter,
    pub should_quit: bool,
}

impl App {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let current_dir = if path.is_dir() {
            path
        } else {
            path.parent()
                .unwrap_or(&PathBuf::from("."))
                .to_path_buf()
        };

        let files = read_directory(&current_dir)?;
        let highlighter = SyntaxHighlighter::new();

        let mut app = Self {
            current_dir,
            files,
            file_list_state: ListState::default(),
            preview_content: None,
            preview_scroll: 0,
            highlighter,
            should_quit: false,
        };

        if !app.files.is_empty() {
            app.file_list_state.select(Some(0));
            app.load_preview();
        }

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut crate::tui::Tui) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::render(frame, self))?;

            match poll_event(Duration::from_millis(16))? {
                AppEvent::Quit => self.should_quit = true,
                AppEvent::NavigateDown => self.navigate_down(),
                AppEvent::NavigateUp => self.navigate_up(),
                AppEvent::ScrollPreviewDown => self.scroll_preview_down(),
                AppEvent::ScrollPreviewUp => self.scroll_preview_up(),
                AppEvent::None => {}
            }
        }

        Ok(())
    }

    fn navigate_down(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current = self.file_list_state.selected().unwrap_or(0);
        let next = if current >= self.files.len() - 1 {
            0
        } else {
            current + 1
        };

        self.file_list_state.select(Some(next));
        self.preview_scroll = 0;
        self.load_preview();
    }

    fn navigate_up(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current = self.file_list_state.selected().unwrap_or(0);
        let prev = if current == 0 {
            self.files.len() - 1
        } else {
            current - 1
        };

        self.file_list_state.select(Some(prev));
        self.preview_scroll = 0;
        self.load_preview();
    }

    fn scroll_preview_down(&mut self) {
        if let Some(preview) = &self.preview_content {
            let max_scroll = preview.lines.len().saturating_sub(10) as u16;
            self.preview_scroll = (self.preview_scroll + 5).min(max_scroll);
        }
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(5);
    }

    fn load_preview(&mut self) {
        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if entry.is_file {
                    if let Ok(content) = read_file_content(&entry.path) {
                        let lines = self.highlighter.highlight(&entry.path, &content);
                        self.preview_content = Some(PreviewContent {
                            path: entry.path.clone(),
                            lines,
                        });
                        return;
                    }
                }
            }
        }
        self.preview_content = None;
    }
}
