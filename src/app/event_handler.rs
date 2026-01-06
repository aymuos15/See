use crate::event::{poll_event, AppEvent};
use std::time::Duration;

use super::App;

impl App {
    pub fn run(&mut self, terminal: &mut crate::tui::Tui) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::render(frame, self))?;
            self.handle_next_event()?;
        }

        Ok(())
    }

    fn handle_next_event(&mut self) -> anyhow::Result<()> {
        let event = poll_event(
            Duration::from_millis(16),
            self.search_mode,
            &mut self.file_watcher,
            &mut self.search_index_timer,
        )?;

        match event {
            AppEvent::Quit => self.handle_quit(),
            AppEvent::DirectoryChanged => {
                self.refresh_current_directory();
            }
            AppEvent::PreviewFileChanged => {
                self.refresh_preview();
            }
            AppEvent::SearchIndexRefreshTimer => self.refresh_search_index(),
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
            AppEvent::MouseDown { column, row } => {
                if !self.search_mode {
                    self.handle_mouse_down(column, row);
                }
            }
            AppEvent::MouseDrag { column, row } => {
                if !self.search_mode {
                    self.handle_mouse_drag(column, row);
                }
            }
            AppEvent::MouseUp { column, row } => {
                if !self.search_mode {
                    self.handle_mouse_up(column, row);
                }
            }
            AppEvent::CopySelection => {
                self.copy_selection();
            }
            AppEvent::None => {}
        }

        Ok(())
    }

    fn handle_quit(&mut self) {
        if self.search_mode {
            self.exit_search_mode();
        } else {
            self.should_quit = true;
        }
    }
}
