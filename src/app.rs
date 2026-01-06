use crate::event::{poll_event, AppEvent};
use crate::files::{read_directory, read_file_content, FileEntry};
use crate::highlight::SyntaxHighlighter;
use crate::theme::Theme;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::text::Line;
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::time::Duration;

pub struct PreviewContent {
    #[allow(dead_code)]
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
    pub split_percent: u16,
    pub theme: Theme,
    // Search mode state
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_selected: usize,
}

impl App {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let current_dir = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
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
            split_percent: 30,
            theme: Theme::load(),
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
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

            match poll_event(Duration::from_millis(16), self.search_mode)? {
                AppEvent::Quit => {
                    if self.search_mode {
                        self.exit_search_mode();
                    } else {
                        self.should_quit = true;
                    }
                }
                AppEvent::OpenSearch => self.enter_search_mode(),
                AppEvent::CloseSearch => self.exit_search_mode(),
                AppEvent::SearchInput(c) => self.search_input(c),
                AppEvent::SearchBackspace => self.search_backspace(),
                AppEvent::SearchNavigateUp => self.search_navigate_up(),
                AppEvent::SearchNavigateDown => self.search_navigate_down(),
                AppEvent::SearchConfirm => self.search_confirm(),
                AppEvent::NavigateDown => {
                    if !self.search_mode {
                        self.navigate_down();
                    }
                }
                AppEvent::NavigateUp => {
                    if !self.search_mode {
                        self.navigate_up();
                    }
                }
                AppEvent::ScrollPreviewDown => {
                    if !self.search_mode {
                        self.scroll_preview_down();
                    }
                }
                AppEvent::ScrollPreviewUp => {
                    if !self.search_mode {
                        self.scroll_preview_up();
                    }
                }
                AppEvent::ScrollPreviewPageDown => {
                    if !self.search_mode {
                        self.scroll_preview_page_down();
                    }
                }
                AppEvent::ScrollPreviewPageUp => {
                    if !self.search_mode {
                        self.scroll_preview_page_up();
                    }
                }
                AppEvent::ShrinkFileList => {
                    if !self.search_mode {
                        self.shrink_file_list();
                    }
                }
                AppEvent::GrowFileList => {
                    if !self.search_mode {
                        self.grow_file_list();
                    }
                }
                AppEvent::Enter => {
                    if !self.search_mode {
                        self.enter_directory();
                    }
                }
                AppEvent::GoBack => {
                    if !self.search_mode {
                        self.go_back();
                    }
                }
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
            if !preview.lines.is_empty() {
                self.preview_scroll =
                    (self.preview_scroll + 1).min((preview.lines.len() - 1) as u16);
            }
        }
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    fn scroll_preview_page_down(&mut self) {
        if let Some(preview) = &self.preview_content {
            if !preview.lines.is_empty() {
                self.preview_scroll =
                    (self.preview_scroll + 10).min((preview.lines.len() - 1) as u16);
            }
        }
    }

    fn scroll_preview_page_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(10);
    }

    fn shrink_file_list(&mut self) {
        self.split_percent = self.split_percent.saturating_sub(5).max(10);
    }

    fn grow_file_list(&mut self) {
        self.split_percent = (self.split_percent + 5).min(80);
    }

    fn enter_directory(&mut self) {
        if let Some(idx) = self.file_list_state.selected() {
            if let Some(entry) = self.files.get(idx) {
                if !entry.is_file {
                    if let Ok(files) = read_directory(&entry.path) {
                        self.current_dir = entry.path.clone();
                        self.files = files;
                        self.file_list_state.select(Some(0));
                        self.preview_scroll = 0;
                        self.load_preview();
                    }
                }
            }
        }
    }

    fn go_back(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();
            if let Ok(files) = read_directory(&parent_path) {
                self.current_dir = parent_path;
                self.files = files;
                self.file_list_state.select(Some(0));
                self.preview_scroll = 0;
                self.load_preview();
            }
        }
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

    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_selected = 0;
        self.apply_fuzzy_filter();
    }

    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_results.clear();
        self.search_selected = 0;
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.search_selected = 0;
        self.apply_fuzzy_filter();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.search_selected = 0;
        self.apply_fuzzy_filter();
    }

    pub fn search_navigate_up(&mut self) {
        if !self.search_results.is_empty() {
            self.search_selected = if self.search_selected == 0 {
                self.search_results.len() - 1
            } else {
                self.search_selected - 1
            };
        }
    }

    pub fn search_navigate_down(&mut self) {
        if !self.search_results.is_empty() {
            self.search_selected = (self.search_selected + 1) % self.search_results.len();
        }
    }

    pub fn search_confirm(&mut self) {
        if !self.search_results.is_empty() {
            let file_idx = self.search_results[self.search_selected];
            self.file_list_state.select(Some(file_idx));
            self.preview_scroll = 0;
            self.load_preview();
        }
        self.exit_search_mode();
    }

    fn apply_fuzzy_filter(&mut self) {
        if self.search_query.is_empty() {
            self.search_results = (0..self.files.len()).collect();
            return;
        }

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(usize, i64)> = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(idx, file)| {
                matcher
                    .fuzzy_match(&file.name, &self.search_query)
                    .map(|score| (idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        self.search_results = scored.into_iter().map(|(idx, _)| idx).collect();
    }
}
