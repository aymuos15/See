//! PDF viewing and navigation handlers

use std::path::{Path, PathBuf};

use super::{App, PreviewContentType};

impl App {
    /// Handle PDF page loaded from background worker
    pub fn handle_pdf_page_loaded(
        &mut self,
        path: &Path,
        page: usize,
        total_pages: usize,
        result: &anyhow::Result<image::DynamicImage>,
    ) {
        match result {
            Ok(dyn_img) => {
                // Clear any previous error
                self.pdf_error = None;

                if let Some(ref picker) = self.image_picker {
                    let protocol = picker.new_resize_protocol(dyn_img.clone());
                    // Store protocol by page-specific key
                    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                    let page_key = Self::pdf_page_key(&canonical_path, page);
                    self.image_protocols.insert(page_key, protocol);

                    // Update the preview content with total_pages if this is the current file
                    if let Some(ref content) = self.shared_preview_content {
                        if let PreviewContentType::Pdf {
                            path: content_path, ..
                        } = content.as_ref()
                        {
                            let content_canonical =
                                content_path.canonicalize().unwrap_or_else(|_| content_path.clone());
                            if content_canonical == canonical_path {
                                self.shared_preview_content =
                                    Some(std::rc::Rc::new(PreviewContentType::Pdf {
                                        path: content_path.clone(),
                                        current_page: page,
                                        total_pages,
                                    }));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Store error message for display
                self.pdf_error = Some(e.to_string());
            }
        }
    }

    /// Generate a unique key for PDF page caching
    fn pdf_page_key(path: &Path, page: usize) -> PathBuf {
        let mut key = path.to_path_buf();
        key.set_extension(format!("pdf.page{page}"));
        key
    }

    /// Navigate to next PDF page
    pub fn pdf_next_page(&mut self) {
        if let Some(ref content) = self.shared_preview_content.clone() {
            if let PreviewContentType::Pdf {
                path,
                current_page,
                total_pages,
            } = content.as_ref()
            {
                if *total_pages == 0 {
                    return;
                }
                let next_page = (*current_page + 1).min(*total_pages - 1);
                if next_page != *current_page {
                    self.load_pdf_page(path, next_page, *total_pages);
                }
            }
        }
    }

    /// Navigate to previous PDF page
    pub fn pdf_prev_page(&mut self) {
        if let Some(ref content) = self.shared_preview_content.clone() {
            if let PreviewContentType::Pdf {
                path,
                current_page,
                total_pages,
            } = content.as_ref()
            {
                if *current_page > 0 {
                    let prev_page = current_page - 1;
                    self.load_pdf_page(path, prev_page, *total_pages);
                }
            }
        }
    }

    /// Navigate to first PDF page
    pub fn pdf_first_page(&mut self) {
        if let Some(ref content) = self.shared_preview_content.clone() {
            if let PreviewContentType::Pdf {
                path,
                current_page,
                total_pages,
            } = content.as_ref()
            {
                if *current_page != 0 {
                    self.load_pdf_page(path, 0, *total_pages);
                }
            }
        }
    }

    /// Navigate to last PDF page
    pub fn pdf_last_page(&mut self) {
        if let Some(ref content) = self.shared_preview_content.clone() {
            if let PreviewContentType::Pdf {
                path,
                current_page,
                total_pages,
            } = content.as_ref()
            {
                if *total_pages > 0 {
                    let last_page = *total_pages - 1;
                    if *current_page != last_page {
                        self.load_pdf_page(path, last_page, *total_pages);
                    }
                }
            }
        }
    }

    /// Load a specific PDF page
    fn load_pdf_page(&mut self, path: &Path, page: usize, total_pages: usize) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let page_key = Self::pdf_page_key(&canonical, page);

        // Check if page is already cached
        if !self.image_protocols.contains_key(&page_key) {
            self.worker.request_pdf_page(path, page);
        }

        // Update current page in preview content
        self.shared_preview_content = Some(std::rc::Rc::new(PreviewContentType::Pdf {
            path: path.to_path_buf(),
            current_page: page,
            total_pages,
        }));
    }

    /// Check if currently viewing a PDF
    pub fn is_viewing_pdf(&self) -> bool {
        self.shared_preview_content
            .as_ref()
            .is_some_and(|c| matches!(c.as_ref(), PreviewContentType::Pdf { .. }))
    }
}
