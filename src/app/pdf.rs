//! Continuous ("longitudinal") PDF viewing state and navigation.
//!
//! Pages are stacked vertically into one virtual document measured in terminal
//! rows; scrolling moves through that document rather than jumping page by
//! page, and the renderer slices whichever pages overlap the viewport.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use image::DynamicImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

use super::{App, PreviewContentType};

/// Blank rows drawn between two consecutive pages.
pub const PAGE_GAP_ROWS: u32 = 1;

/// Aspect ratio (height / width) assumed for pages that have not been rendered
/// yet, so the document has a plausible length before anything is loaded.
const DEFAULT_PAGE_ASPECT: f32 = 1.414;

/// Pages further than this many pages from the viewport have their bitmaps
/// dropped again, so a long document does not accumulate every page in memory.
const PAGE_CACHE_RADIUS: usize = 3;

/// One horizontal strip of a page, scaled for a particular cell geometry.
#[derive(Clone, PartialEq, Eq, Hash)]
struct SliceKey {
    page: usize,
    cols: u16,
    rows: u32,
    row_offset: u32,
    slice_rows: u16,
}

/// Continuous-scroll state for the PDF currently being previewed.
pub struct PdfView {
    pub path: PathBuf,
    pub total_pages: usize,
    /// Scroll position in terminal rows from the top of the whole document.
    pub scroll: u32,
    /// Rows a single page occupied at the last render. Zero until first drawn.
    pub page_rows: u32,
    /// Full-resolution page bitmaps as they arrive from the worker.
    pages: HashMap<usize, Rc<DynamicImage>>,
    /// Pages resized to exactly fill their on-screen box.
    scaled: HashMap<(usize, u16, u32), Rc<DynamicImage>>,
    /// Graphics protocols for the strips currently on screen.
    slices: HashMap<SliceKey, StatefulProtocol>,
    /// Strips touched during the frame being drawn, used to evict the rest.
    live_slices: HashSet<SliceKey>,
    /// Pages already asked of the worker, so requests are not duplicated.
    requested: HashSet<usize>,
    /// Pages the renderer needs but that have not been requested yet.
    wanted: Vec<usize>,
}

impl PdfView {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            total_pages: 0,
            scroll: 0,
            page_rows: 0,
            pages: HashMap::new(),
            scaled: HashMap::new(),
            slices: HashMap::new(),
            live_slices: HashSet::new(),
            requested: HashSet::new(),
            wanted: Vec::new(),
        }
    }

    /// Rows from the top of one page to the top of the next.
    pub const fn page_stride(&self) -> u32 {
        self.page_rows + PAGE_GAP_ROWS
    }

    /// Height of the whole stacked document in rows.
    pub fn document_rows(&self) -> u32 {
        if self.page_rows == 0 || self.total_pages == 0 {
            return 0;
        }
        u32::try_from(self.total_pages).unwrap_or(u32::MAX) * self.page_stride() - PAGE_GAP_ROWS
    }

    /// Largest scroll offset that still keeps content on screen.
    pub fn max_scroll(&self, viewport_rows: u16) -> u32 {
        self.document_rows()
            .saturating_sub(u32::from(viewport_rows))
    }

    /// Page occupying the top of the viewport, 0-indexed.
    pub fn current_page(&self) -> usize {
        if self.page_stride() == 0 {
            return 0;
        }
        let page = (self.scroll + self.page_stride() / 2) / self.page_stride();
        (page as usize).min(self.total_pages.saturating_sub(1))
    }

    /// Aspect ratio (height / width) taken from any page already rendered.
    pub fn aspect(&self) -> f32 {
        self.pages
            .values()
            .next()
            .map_or(DEFAULT_PAGE_ASPECT, |img| {
                if img.width() == 0 {
                    DEFAULT_PAGE_ASPECT
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    let aspect = img.height() as f32 / img.width() as f32;
                    aspect
                }
            })
    }

    pub fn scroll_by(&mut self, delta: i32, viewport_rows: u16) {
        let max = self.max_scroll(viewport_rows);
        let target = i64::from(self.scroll) + i64::from(delta);
        self.scroll = u32::try_from(target.clamp(0, i64::from(max))).unwrap_or(0);
    }

    pub fn scroll_to_page(&mut self, page: usize, viewport_rows: u16) {
        let page = page.min(self.total_pages.saturating_sub(1));
        let target = u32::try_from(page)
            .unwrap_or(u32::MAX)
            .saturating_mul(self.page_stride());
        self.scroll = target.min(self.max_scroll(viewport_rows));
    }

    /// Store a page bitmap that just arrived from the worker.
    pub fn insert_page(&mut self, page: usize, image: DynamicImage) {
        self.pages.insert(page, Rc::new(image));
        // Any strip cut from a stale copy of this page is now invalid.
        self.scaled.retain(|(p, _, _), _| *p != page);
        self.slices.retain(|key, _| key.page != page);
    }

    /// Record that the renderer needs a page it does not have yet.
    pub fn want_page(&mut self, page: usize) {
        if page < self.total_pages.max(1)
            && !self.pages.contains_key(&page)
            && !self.requested.contains(&page)
            && !self.wanted.contains(&page)
        {
            self.wanted.push(page);
        }
    }

    /// Hand over the pages to request, marking them as in flight.
    fn take_wanted(&mut self) -> Vec<usize> {
        let wanted = std::mem::take(&mut self.wanted);
        for page in &wanted {
            self.requested.insert(*page);
        }
        wanted
    }

    /// Called before drawing a frame, to start tracking which strips are used.
    pub fn begin_frame(&mut self) {
        self.live_slices.clear();
    }

    /// Called after drawing, to drop the bitmaps and strips no longer on screen.
    pub fn end_frame(&mut self) {
        let live = std::mem::take(&mut self.live_slices);
        self.slices.retain(|key, _| live.contains(key));
        self.live_slices = live;

        let current = self.current_page();
        let keep = |page: usize| page.abs_diff(current) <= PAGE_CACHE_RADIUS;
        self.pages.retain(|page, _| keep(*page));
        self.scaled.retain(|(page, _, _), _| keep(*page));
        self.requested.retain(|page| keep(*page));
    }

    /// Graphics protocol for one strip of a page, built on demand.
    ///
    /// The page is first resized so it exactly fills its `cols` x `rows` box,
    /// then cut at cell boundaries, so the strip lands in its rectangle at
    /// natural size instead of being squeezed to fit.
    pub fn slice(
        &mut self,
        page: usize,
        cols: u16,
        rows: u32,
        row_offset: u32,
        slice_rows: u16,
        picker: &Picker,
    ) -> Option<&mut StatefulProtocol> {
        let key = SliceKey {
            page,
            cols,
            rows,
            row_offset,
            slice_rows,
        };

        if !self.slices.contains_key(&key) {
            let (cell_w, cell_h) = picker.font_size();
            let scaled = self.scaled_page(page, cols, rows, cell_w, cell_h)?;

            let top = row_offset * u32::from(cell_h);
            let height = u32::from(slice_rows) * u32::from(cell_h);
            if top >= scaled.height() {
                return None;
            }
            let height = height.min(scaled.height() - top);
            if height == 0 {
                return None;
            }

            let strip = scaled.crop_imm(0, top, scaled.width(), height);
            self.slices
                .insert(key.clone(), picker.new_resize_protocol(strip));
        }

        self.live_slices.insert(key.clone());
        self.slices.get_mut(&key)
    }

    /// The page bitmap resized to exactly fill a `cols` x `rows` cell box.
    fn scaled_page(
        &mut self,
        page: usize,
        cols: u16,
        rows: u32,
        cell_w: u16,
        cell_h: u16,
    ) -> Option<Rc<DynamicImage>> {
        let cache_key = (page, cols, rows);
        if let Some(scaled) = self.scaled.get(&cache_key) {
            return Some(Rc::clone(scaled));
        }

        let source = self.pages.get(&page)?;
        let width = u32::from(cols) * u32::from(cell_w);
        let height = rows * u32::from(cell_h);
        if width == 0 || height == 0 {
            return None;
        }

        let scaled = Rc::new(source.resize_exact(
            width,
            height,
            image::imageops::FilterType::Triangle,
        ));
        self.scaled.insert(cache_key, Rc::clone(&scaled));
        Some(scaled)
    }
}

impl App {
    /// Handle a PDF page rendered by the background worker.
    pub fn handle_pdf_page_loaded(
        &mut self,
        path: &Path,
        page: usize,
        total_pages: usize,
        result: anyhow::Result<image::DynamicImage>,
    ) {
        match result {
            Ok(dyn_img) => {
                self.pdf_error = None;

                let Some(view) = self.pdf_view.as_mut() else {
                    return;
                };
                if !same_file(&view.path, path) {
                    return;
                }

                view.total_pages = total_pages;
                view.insert_page(page, dyn_img);
            }
            Err(e) => {
                self.pdf_error = Some(e.to_string());
            }
        }
    }

    /// Start viewing a PDF from the top, requesting its first page.
    pub(super) fn begin_pdf_view(&mut self, path: &Path) {
        let already_open = self
            .pdf_view
            .as_ref()
            .is_some_and(|view| same_file(&view.path, path));
        if already_open {
            return;
        }

        self.pdf_error = None;
        let mut view = PdfView::new(path.to_path_buf());
        view.want_page(0);
        self.pdf_view = Some(view);
        self.flush_pdf_requests();
    }

    /// Ask the worker for whatever pages the renderer asked for.
    pub fn flush_pdf_requests(&mut self) {
        let Some(view) = self.pdf_view.as_mut() else {
            return;
        };
        let path = view.path.clone();
        for page in view.take_wanted() {
            self.worker.request_pdf_page(&path, page);
        }
    }

    /// Rows available to the PDF view, excluding its page indicator line.
    fn pdf_viewport_rows(&self) -> u16 {
        self.last_preview_area
            .map_or(20, |area| area.height.saturating_sub(1).max(1))
    }

    /// Scroll the continuous view by `delta` rows. Returns false if no PDF is
    /// open, so callers can fall through to their normal text scrolling.
    pub(super) fn pdf_scroll_by(&mut self, delta: i32) -> bool {
        let rows = self.pdf_viewport_rows();
        let Some(view) = self.pdf_view.as_mut() else {
            return false;
        };
        view.scroll_by(delta, rows);
        true
    }

    fn pdf_goto_page(&mut self, page: usize) {
        let rows = self.pdf_viewport_rows();
        if let Some(view) = self.pdf_view.as_mut() {
            view.scroll_to_page(page, rows);
        }
    }

    /// Jump to the top of the next page.
    pub fn pdf_next_page(&mut self) {
        let next = self
            .pdf_view
            .as_ref()
            .map(|view| view.current_page().saturating_add(1));
        if let Some(page) = next {
            self.pdf_goto_page(page);
        }
    }

    /// Jump to the top of the previous page.
    pub fn pdf_prev_page(&mut self) {
        let prev = self
            .pdf_view
            .as_ref()
            .map(|view| view.current_page().saturating_sub(1));
        if let Some(page) = prev {
            self.pdf_goto_page(page);
        }
    }

    pub fn pdf_first_page(&mut self) {
        self.pdf_goto_page(0);
    }

    pub fn pdf_last_page(&mut self) {
        let last = self
            .pdf_view
            .as_ref()
            .map(|view| view.total_pages.saturating_sub(1));
        if let Some(page) = last {
            self.pdf_goto_page(page);
        }
    }

    /// Check if currently viewing a PDF
    pub fn is_viewing_pdf(&self) -> bool {
        self.pdf_view.is_some()
            && self
                .shared_preview_content
                .as_ref()
                .is_some_and(|c| matches!(c.as_ref(), PreviewContentType::Pdf { .. }))
    }
}

/// Compare two paths, falling back to a plain comparison when either cannot be
/// canonicalized (the file may have been moved out from under us).
fn same_file(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    canon(a) == canon(b)
}
