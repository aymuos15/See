use crate::event::{poll_event, AppEvent};
use crate::worker::WorkerResponse;
use std::time::Duration;

use super::App;

/// Upper bound on how many queued input events are applied between frames,
/// so a stuck key cannot starve rendering entirely.
const MAX_COALESCED_EVENTS: usize = 32;

impl App {
    pub fn run(&mut self, terminal: &mut crate::tui::Tui) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::render(frame, self))?;
            self.poll_worker_responses();
            self.check_pending_full_quality();
            self.handle_next_event()?;
            self.drain_pending_input()?;
        }

        Ok(())
    }

    /// Handle input that is already queued before drawing again.
    ///
    /// A held scroll key or a spun mouse wheel produces events faster than a
    /// frame takes to draw, and a PDF frame re-encodes an image. Without this
    /// the queue backs up and scrolling lags behind the input by seconds;
    /// applying the backlog first collapses it into a single frame.
    fn drain_pending_input(&mut self) -> anyhow::Result<()> {
        let mut handled = 0;
        while handled < MAX_COALESCED_EVENTS
            && !self.should_quit
            && crossterm::event::poll(Duration::ZERO)?
        {
            self.handle_next_event()?;
            handled += 1;
        }
        Ok(())
    }

    fn poll_worker_responses(&mut self) {
        while let Some(response) = self.worker.poll_response() {
            match response {
                WorkerResponse::SymbolsIndexed(symbols) => {
                    self.symbol_index = symbols;
                    self.symbol_indexing_progress = None;
                    if self.symbol_search_mode {
                        self.apply_symbol_filter();
                    }
                }
                WorkerResponse::IndexingProgress { processed, total } => {
                    self.symbol_indexing_progress = Some((processed, total));
                }
                WorkerResponse::ImageLoaded { path, result } => {
                    self.handle_image_loaded(&path, &result);
                }
                WorkerResponse::ThumbnailLoaded { path, result } => {
                    self.handle_thumbnail_loaded(&path, &result);
                }
                WorkerResponse::TreeLinesCounted(counts) => {
                    self.tree_line_counts = counts;
                }
                WorkerResponse::FileHighlighted { path, lines } => {
                    self.apply_highlighted(&path, lines);
                }
                WorkerResponse::PdfPageLoaded {
                    path,
                    page,
                    total_pages,
                    result,
                } => {
                    self.handle_pdf_page_loaded(&path, page, total_pages, result);
                }
            }
        }
    }

    /// Closes whichever overlay is on top, innermost first.
    fn close_overlay(&mut self) {
        if self.help_mode {
            self.close_help();
        } else if self.theme_preview_mode {
            self.theme_preview_mode = false;
        } else if self.symbol_search_mode {
            self.exit_symbol_search_mode();
        } else if self.find_mode {
            self.exit_find_mode();
        } else if self.file_tree_popup_mode {
            self.file_tree_popup_mode = false;
        } else {
            self.exit_search_mode();
        }
    }

    /// True for overlays that show a list rather than a query, where a letter
    /// key is free to mean something other than typing.
    const fn takes_no_text_input(&self) -> bool {
        self.help_mode || self.theme_preview_mode || self.file_tree_popup_mode
    }

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn handle_next_event(&mut self) -> anyhow::Result<()> {
        let event = poll_event(
            Duration::from_millis(16),
            self.search_mode
                || self.find_mode
                || self.symbol_search_mode
                || self.theme_preview_mode
                || self.help_mode
                || self.file_tree_popup_mode,
            &mut self.file_watcher,
            &mut self.search_index_timer,
            &self.config.keys,
        )?;

        match event {
            AppEvent::Quit => self.handle_quit(),
            AppEvent::DirectoryChanged => {
                self.refresh_current_directory();
            }
            AppEvent::PreviewFileChanged => self.refresh_preview(),
            AppEvent::SearchIndexRefreshTimer => self.refresh_search_index(),
            AppEvent::OpenSearch => self.enter_search_mode(),
            AppEvent::OpenFind => self.enter_find_mode(),
            AppEvent::CloseSearch => self.close_overlay(),
            // Panels that take no text input also close on `q`, matching the
            // way `q` quits the viewer itself.
            AppEvent::SearchInput('q') if self.takes_no_text_input() => self.close_overlay(),
            AppEvent::SearchInput(c) => {
                if self.symbol_search_mode {
                    self.symbol_search_input(c);
                } else if self.find_mode {
                    // In find mode, some characters might be used for navigation
                    // if they are bound to Enter/GoBack actions in normal mode.
                    // However, handle_key in search mode prioritizes SearchInput
                    // for chars unless explicitly bound in search_mode_map.
                    self.find_input(c);
                } else {
                    self.search_input(c);
                }
            }
            AppEvent::SearchBackspace => {
                if self.symbol_search_mode {
                    self.symbol_search_backspace();
                } else if self.find_mode {
                    self.find_backspace();
                } else {
                    self.search_backspace();
                }
            }
            AppEvent::SearchNavigateUp => {
                if self.theme_preview_mode {
                    self.theme_search_navigate_up();
                } else if self.symbol_search_mode {
                    self.symbol_search_navigate_up();
                } else if self.find_mode {
                    self.navigate_up();
                } else {
                    self.search_navigate_up();
                }
            }
            AppEvent::SearchNavigateDown => {
                if self.theme_preview_mode {
                    self.theme_search_navigate_down();
                } else if self.symbol_search_mode {
                    self.symbol_search_navigate_down();
                } else if self.find_mode {
                    self.navigate_down();
                } else {
                    self.search_navigate_down();
                }
            }
            AppEvent::SearchConfirm => {
                if self.theme_preview_mode {
                    self.theme_search_confirm();
                } else if self.symbol_search_mode {
                    self.symbol_search_confirm();
                } else if self.find_mode {
                    self.find_confirm();
                } else {
                    self.search_confirm();
                }
            }
            AppEvent::OpenSymbolSearch => self.enter_symbol_search_mode(),
            AppEvent::CloseSymbolSearch => self.exit_symbol_search_mode(),
            AppEvent::ToggleThemePreview => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.toggle_theme_preview();
                }
            }
            AppEvent::ToggleHelp => {
                self.toggle_help();
            }
            AppEvent::ToggleFileTreePopup => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.toggle_file_tree_popup();
                }
            }
            AppEvent::SymbolSearchInput(c) => self.symbol_search_input(c),
            AppEvent::SymbolSearchBackspace => self.symbol_search_backspace(),
            AppEvent::SymbolSearchNavigateUp => self.symbol_search_navigate_up(),
            AppEvent::SymbolSearchNavigateDown => self.symbol_search_navigate_down(),
            AppEvent::SymbolSearchConfirm => self.symbol_search_confirm(),
            AppEvent::NavigateDown => {
                if self.file_tree_popup_mode {
                    self.file_tree_popup_navigate_down();
                } else if !self.search_mode && !self.symbol_search_mode {
                    self.navigate_down();
                }
            }
            AppEvent::NavigateUp => {
                if self.file_tree_popup_mode {
                    self.file_tree_popup_navigate_up();
                } else if !self.search_mode && !self.symbol_search_mode {
                    self.navigate_up();
                }
            }
            AppEvent::ScrollPreviewDown => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.scroll_preview_down();
                }
            }
            AppEvent::ScrollPreviewUp => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.scroll_preview_up();
                }
            }
            AppEvent::ScrollPreviewPageDown => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.scroll_preview_page_down();
                }
            }
            AppEvent::ScrollPreviewPageUp => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.scroll_preview_page_up();
                }
            }
            AppEvent::MouseScrollDown => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.mouse_scroll_down();
                }
            }
            AppEvent::MouseScrollUp => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.mouse_scroll_up();
                }
            }
            AppEvent::ShrinkFileList => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.shrink_file_list();
                }
            }
            AppEvent::GrowFileList => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.grow_file_list();
                }
            }
            AppEvent::Enter => {
                if self.file_tree_popup_mode {
                    self.file_tree_popup_confirm();
                } else if !self.search_mode && !self.symbol_search_mode {
                    self.enter_directory();
                }
            }
            AppEvent::GoBack => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.go_back();
                }
            }
            AppEvent::SplitHorizontal | AppEvent::SplitVertical => {
                let direction = match event {
                    AppEvent::SplitHorizontal => crate::app::split::SplitDirection::Right,
                    AppEvent::SplitVertical => crate::app::split::SplitDirection::Down,
                    _ => unreachable!(),
                };
                self.split_active_pane(direction);
            }
            AppEvent::SplitUp
            | AppEvent::SplitDown
            | AppEvent::SplitLeft
            | AppEvent::SplitRight => {
                let direction = match event {
                    AppEvent::SplitUp => crate::app::split::SplitDirection::Up,
                    AppEvent::SplitDown => crate::app::split::SplitDirection::Down,
                    AppEvent::SplitLeft => crate::app::split::SplitDirection::Left,
                    AppEvent::SplitRight => crate::app::split::SplitDirection::Right,
                    _ => unreachable!(),
                };
                self.split_active_pane(direction);
            }
            AppEvent::CloseActivePane => {
                if let Some(ref mut layout) = self.split_layout {
                    layout.close_active_pane();
                    if layout.panes.len() <= 1 {
                        self.split_layout = None;
                    }
                }
            }
            AppEvent::CyclePane => {
                if let Some(ref mut layout) = self.split_layout {
                    layout.cycle_active_pane();
                }
            }
            AppEvent::SwapSplitOrientation => {
                if let Some(ref mut layout) = self.split_layout {
                    layout.swap_active_split_orientation();
                }
            }
            AppEvent::ToggleFileList => {
                self.file_list_visible = !self.file_list_visible;
            }
            AppEvent::ResizeSplitLeft => {
                if let Some(ref mut layout) = self.split_layout {
                    layout.resize_active_split(-5);
                }
            }
            AppEvent::ResizeSplitRight => {
                if let Some(ref mut layout) = self.split_layout {
                    layout.resize_active_split(5);
                }
            }
            AppEvent::MouseDown { column, row } => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.handle_mouse_down(column, row);
                }
            }
            AppEvent::MouseDrag { column, row } => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.handle_mouse_drag(column, row);
                }
            }
            AppEvent::MouseUp { column, row } => {
                if !self.search_mode && !self.symbol_search_mode {
                    self.handle_mouse_up(column, row);
                }
            }
            AppEvent::CopySelection => {
                self.copy_selection();
            }
            AppEvent::SelectAll => {
                self.select_all();
            }
            AppEvent::ToggleWrap => {
                self.config.wrap = !self.config.wrap;
            }
            AppEvent::PdfNextPage => {
                if !self.search_mode && !self.symbol_search_mode && self.is_viewing_pdf() {
                    self.pdf_next_page();
                }
            }
            AppEvent::PdfPrevPage => {
                if !self.search_mode && !self.symbol_search_mode && self.is_viewing_pdf() {
                    self.pdf_prev_page();
                }
            }
            AppEvent::PdfFirstPage => {
                if !self.search_mode && !self.symbol_search_mode && self.is_viewing_pdf() {
                    self.pdf_first_page();
                }
            }
            AppEvent::PdfLastPage => {
                if !self.search_mode && !self.symbol_search_mode && self.is_viewing_pdf() {
                    self.pdf_last_page();
                }
            }
            AppEvent::None => {}
        }

        Ok(())
    }

    fn handle_quit(&mut self) {
        if self.search_mode {
            self.exit_search_mode();
        } else if self.symbol_search_mode {
            self.exit_symbol_search_mode();
        } else {
            self.should_quit = true;
        }
    }

    fn split_active_pane(&mut self, direction: crate::app::split::SplitDirection) {
        use std::rc::Rc;

        if self.split_layout.is_none() {
            let mut layout = crate::app::split::SplitLayout::new();
            // Use shared preview content (Rc::clone is O(1))
            if let Some(ref preview) = self.shared_preview_content {
                if let Some(pane) = layout.panes.get_mut(0) {
                    pane.preview_content = Some(Rc::clone(preview));
                    // Find current file path
                    if let Some(idx) = self.file_list_state.selected() {
                        if let Some(entry) = self.files.get(idx) {
                            pane.file_path = Some(entry.path.clone());
                        }
                    }
                }
            }
            self.split_layout = Some(layout);
        }

        if let Some(ref mut layout) = self.split_layout {
            let _ = layout.split_active_pane(direction, 50);
        }
    }
}
