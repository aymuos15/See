//! Background worker for CPU-intensive tasks

use crate::config::Config;
use crate::constants::{PDF_RENDER_SCALE, THUMBNAIL_SIZE};
use crate::files::{extract_symbols, find_all_files_recursive, Symbol};
use image::imageops::FilterType;
use pdfium_render::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub enum WorkerRequest {
    IndexSymbols {
        root_dir: Box<Path>,
        config: Box<Config>,
    },
    LoadImage {
        path: Box<Path>,
    },
    LoadThumbnail {
        path: Box<Path>,
    },
    /// Count lines across the tree for the file tree popup
    CountTreeLines {
        root_dir: Box<Path>,
        config: Box<Config>,
    },
    /// Syntax-highlight a file's text off the UI thread
    HighlightFile {
        path: Box<Path>,
        text: String,
    },
    /// Load a PDF file and render a specific page
    LoadPdfPage {
        path: Box<Path>,
        page: usize,
    },
    Shutdown,
}

pub enum WorkerResponse {
    SymbolsIndexed(Vec<Symbol>),
    IndexingProgress {
        processed: usize,
        total: usize,
    },
    ImageLoaded {
        path: Box<Path>,
        result: anyhow::Result<image::DynamicImage>,
    },
    ThumbnailLoaded {
        path: Box<Path>,
        result: anyhow::Result<image::DynamicImage>,
    },
    /// Line counts for every file in the tree, and per-directory totals
    TreeLinesCounted(TreeLineCounts),
    /// A file's highlighted lines, ready to replace its plain-text preview
    FileHighlighted {
        path: Box<Path>,
        lines: Vec<ratatui::text::Line<'static>>,
    },
    /// PDF page rendered as an image
    PdfPageLoaded {
        path: Box<Path>,
        page: usize,
        total_pages: usize,
        result: anyhow::Result<image::DynamicImage>,
    },
}

pub struct BackgroundWorker {
    request_tx: Sender<WorkerRequest>,
    response_rx: Receiver<WorkerResponse>,
    _handle: JoinHandle<()>,
}

impl BackgroundWorker {
    pub fn spawn() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<WorkerRequest>();
        let (response_tx, response_rx) = mpsc::channel::<WorkerResponse>();

        let handle = thread::spawn(move || {
            worker_loop(&request_rx, &response_tx);
        });

        Self {
            request_tx,
            response_rx,
            _handle: handle,
        }
    }

    pub fn request_symbol_indexing(&self, root_dir: &Path, config: Config) {
        let _ = self.request_tx.send(WorkerRequest::IndexSymbols {
            root_dir: root_dir.into(),
            config: Box::new(config),
        });
    }

    pub fn request_image_load(&self, path: &Path) {
        let _ = self
            .request_tx
            .send(WorkerRequest::LoadImage { path: path.into() });
    }

    pub fn request_thumbnail_load(&self, path: &Path) {
        let _ = self
            .request_tx
            .send(WorkerRequest::LoadThumbnail { path: path.into() });
    }

    pub fn request_tree_line_counts(&self, root_dir: &Path, config: Config) {
        let _ = self.request_tx.send(WorkerRequest::CountTreeLines {
            root_dir: root_dir.into(),
            config: Box::new(config),
        });
    }

    pub fn request_highlight(&self, path: &Path, text: String) {
        let _ = self.request_tx.send(WorkerRequest::HighlightFile {
            path: path.into(),
            text,
        });
    }

    pub fn request_pdf_page(&self, path: &Path, page: usize) {
        let _ = self.request_tx.send(WorkerRequest::LoadPdfPage {
            path: path.into(),
            page,
        });
    }

    pub fn poll_response(&self) -> Option<WorkerResponse> {
        self.response_rx.try_recv().ok()
    }

    pub fn shutdown(&self) {
        let _ = self.request_tx.send(WorkerRequest::Shutdown);
    }
}

impl Drop for BackgroundWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn worker_loop(request_rx: &Receiver<WorkerRequest>, response_tx: &Sender<WorkerResponse>) {
    // Initialize Pdfium once for the worker thread
    let pdfium = init_pdfium();
    // The syntax and theme sets are expensive to build, so build them once here
    let highlighter = crate::highlight::SyntaxHighlighter::new();

    while let Ok(request) = request_rx.recv() {
        match request {
            WorkerRequest::IndexSymbols { root_dir, config } => {
                index_symbols(&root_dir, &config, response_tx);
            }
            WorkerRequest::LoadImage { path } => {
                load_image(&path, response_tx);
            }
            WorkerRequest::LoadThumbnail { path } => {
                load_thumbnail(&path, response_tx);
            }
            WorkerRequest::CountTreeLines { root_dir, config } => {
                let counts = count_tree_lines(&root_dir, &config);
                let _ = response_tx.send(WorkerResponse::TreeLinesCounted(counts));
            }
            WorkerRequest::HighlightFile { path, text } => {
                let lines = highlighter.highlight(&path, &text);
                let _ = response_tx.send(WorkerResponse::FileHighlighted { path, lines });
            }
            WorkerRequest::LoadPdfPage { path, page } => {
                load_pdf_page(&path, page, pdfium.as_ref(), response_tx);
            }
            WorkerRequest::Shutdown => break,
        }
    }
}

/// Initialize Pdfium library (tries multiple strategies)
fn init_pdfium() -> Option<Pdfium> {
    // Try to bind to system library first
    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Some(Pdfium::new(bindings));
    }

    // Try common library locations
    let library_paths = [
        "libpdfium.so",
        "libpdfium.dylib",
        "pdfium.dll",
        "/usr/lib/libpdfium.so",
        "/usr/local/lib/libpdfium.so",
        "./libpdfium.so",
    ];

    for path in library_paths {
        if let Ok(bindings) = Pdfium::bind_to_library(path) {
            return Some(Pdfium::new(bindings));
        }
    }

    // Try path relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let lib_name = if cfg!(target_os = "windows") {
                "pdfium.dll"
            } else if cfg!(target_os = "macos") {
                "libpdfium.dylib"
            } else {
                "libpdfium.so"
            };
            let lib_path = exe_dir.join(lib_name);
            if let Ok(bindings) = Pdfium::bind_to_library(&lib_path) {
                return Some(Pdfium::new(bindings));
            }
        }
    }

    // Try using pdfium-render's platform library name helper
    let lib_name = Pdfium::pdfium_platform_library_name_at_path("./");
    if let Ok(bindings) = Pdfium::bind_to_library(&lib_name) {
        return Some(Pdfium::new(bindings));
    }

    None
}

fn index_symbols(root_dir: &Path, config: &Config, response_tx: &Sender<WorkerResponse>) {
    let Ok(all_files) = find_all_files_recursive(root_dir, config) else {
        let _ = response_tx.send(WorkerResponse::SymbolsIndexed(Vec::new()));
        return;
    };

    let source_files: Vec<_> = all_files.into_iter().filter(|f| f.is_file).collect();
    let total = source_files.len();
    let mut symbols = Vec::new();

    for (idx, file_entry) in source_files.iter().enumerate() {
        if let Ok(content) = std::fs::read_to_string(&file_entry.path) {
            let file_symbols = extract_symbols(&file_entry.path, &content);
            symbols.extend(file_symbols);
        }

        if idx % 50 == 0 || idx == total - 1 {
            let _ = response_tx.send(WorkerResponse::IndexingProgress {
                processed: idx + 1,
                total,
            });
        }
    }

    let _ = response_tx.send(WorkerResponse::SymbolsIndexed(symbols));
}

fn load_image(path: &Path, response_tx: &Sender<WorkerResponse>) {
    let result: anyhow::Result<image::DynamicImage> = image::ImageReader::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open image: {e}"))
        .and_then(|reader| {
            reader
                .decode()
                .map_err(|e| anyhow::anyhow!("Failed to decode image: {e}"))
        });

    let _ = response_tx.send(WorkerResponse::ImageLoaded {
        path: path.into(),
        result,
    });
}

fn load_thumbnail(path: &Path, response_tx: &Sender<WorkerResponse>) {
    let result: anyhow::Result<image::DynamicImage> = image::ImageReader::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open image: {e}"))
        .and_then(|reader| {
            reader
                .decode()
                .map_err(|e| anyhow::anyhow!("Failed to decode image: {e}"))
        })
        .map(|img| {
            // Use nearest neighbor filter for speed (fastest option)
            img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Nearest)
        });

    let _ = response_tx.send(WorkerResponse::ThumbnailLoaded {
        path: path.into(),
        result,
    });
}

fn load_pdf_page(
    path: &Path,
    page: usize,
    pdfium: Option<&Pdfium>,
    response_tx: &Sender<WorkerResponse>,
) {
    let result = render_pdf_page(path, page, pdfium);
    let total_pages = result.as_ref().map(|(_, total)| *total).unwrap_or(0);

    let _ = response_tx.send(WorkerResponse::PdfPageLoaded {
        path: path.into(),
        page,
        total_pages,
        result: result.map(|(img, _)| img),
    });
}

/// Render a specific page of a PDF to an image
fn render_pdf_page(
    path: &Path,
    page: usize,
    pdfium: Option<&Pdfium>,
) -> anyhow::Result<(image::DynamicImage, usize)> {
    let pdfium = pdfium.ok_or_else(|| anyhow::anyhow!("PDFium library not available"))?;

    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| anyhow::anyhow!("Failed to load PDF: {e}"))?;

    let total_pages = document.pages().len() as usize;

    if page >= total_pages {
        anyhow::bail!("Page {page} out of range (document has {total_pages} pages)");
    }

    let page_u16 =
        u16::try_from(page).map_err(|_| anyhow::anyhow!("Page number {page} exceeds u16 range"))?;

    let pdf_page = document
        .pages()
        .get(page_u16)
        .map_err(|e| anyhow::anyhow!("Failed to get page {page}: {e}"))?;

    // Calculate render size based on page dimensions and scale factor
    let width = pdf_page.width().value * PDF_RENDER_SCALE;
    let height = pdf_page.height().value * PDF_RENDER_SCALE;

    // Render the page to a bitmap
    #[allow(clippy::cast_possible_truncation)]
    let bitmap = pdf_page
        .render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(width as i32)
                .set_target_height(height as i32)
                .render_form_data(true)
                .render_annotations(true),
        )
        .map_err(|e| anyhow::anyhow!("Failed to render page: {e}"))?;

    // Convert to DynamicImage
    let image = bitmap
        .as_image()
        .as_rgba8()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert bitmap to RGBA image"))?
        .clone();

    Ok((image::DynamicImage::ImageRgba8(image), total_pages))
}

/// Line counts gathered for the file tree: a count per file, and per directory
/// the number of files beneath it together with their combined lines.
#[derive(Default)]
pub struct TreeLineCounts {
    pub files: HashMap<PathBuf, usize>,
    pub directories: HashMap<PathBuf, (usize, usize)>,
}

/// Files above this size are assumed to be data rather than source and are
/// counted as zero lines rather than read into memory.
const MAX_COUNTED_BYTES: u64 = 4 * 1024 * 1024;

/// Walks the tree counting lines per file, then rolls each file's count up
/// into every directory above it.
fn count_tree_lines(root: &Path, config: &Config) -> TreeLineCounts {
    let mut counts = TreeLineCounts::default();

    let Ok(entries) = crate::files::find_all_files_recursive(root, config) else {
        return counts;
    };

    for entry in entries {
        if !entry.is_file {
            counts.directories.entry(entry.path).or_insert((0, 0));
            continue;
        }

        let lines = count_lines(&entry.path);
        counts.files.insert(entry.path.clone(), lines);

        // Roll up into each ancestor directory inside the tree.
        let mut parent = entry.path.parent();
        while let Some(dir) = parent {
            if !dir.starts_with(root) {
                break;
            }
            let totals = counts
                .directories
                .entry(dir.to_path_buf())
                .or_insert((0, 0));
            totals.0 += 1;
            totals.1 += lines;

            if dir == root {
                break;
            }
            parent = dir.parent();
        }
    }

    counts
}

/// Counts newlines in a file, treating a trailing partial line as a line.
fn count_lines(path: &Path) -> usize {
    let Ok(metadata) = std::fs::metadata(path) else {
        return 0;
    };
    if metadata.len() > MAX_COUNTED_BYTES {
        return 0;
    }

    let Ok(bytes) = std::fs::read(path) else {
        return 0;
    };
    if bytes.is_empty() || is_binary(&bytes) {
        return 0;
    }

    let newlines = bytecount(&bytes);
    if bytes.last() == Some(&b'\n') {
        newlines
    } else {
        newlines + 1
    }
}

#[allow(clippy::naive_bytecount)]
fn bytecount(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

/// Treats a NUL byte near the start as the mark of a binary file, the way most
/// tools do. Counting newlines in a PNG is meaningless.
fn is_binary(bytes: &[u8]) -> bool {
    const SNIFF_BYTES: usize = 8192;
    bytes[..bytes.len().min(SNIFF_BYTES)].contains(&0)
}
