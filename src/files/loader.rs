use crate::app::PreviewContentType;
use crate::constants::{BINARY_DETECTION_BYTES, MAX_FILE_SIZE, MAX_IMAGE_SIZE, MAX_PDF_SIZE};
use crate::files::{is_image_file, is_pdf_file};
use std::fs;
use std::path::Path;

/// Load preview content (text or image)
pub fn load_preview_content(path: &Path) -> anyhow::Result<PreviewContentType> {
    let metadata = fs::metadata(path)?;

    // Check for PDF file first
    if is_pdf_file(path) {
        if metadata.len() > MAX_PDF_SIZE {
            return Ok(PreviewContentType::text(
                vec![ratatui::text::Line::from(format!(
                    "[PDF too large to preview: {} bytes (max: {} MB)]",
                    metadata.len(),
                    MAX_PDF_SIZE / 1024 / 1024
                ))],
                vec![],
            ));
        }

        // Return PDF content type - actual rendering happens via worker
        return Ok(PreviewContentType::Pdf {
            path: path.to_path_buf(),
        });
    }

    // Check for image file
    if is_image_file(path) {
        if metadata.len() > MAX_IMAGE_SIZE {
            return Ok(PreviewContentType::text(
                vec![ratatui::text::Line::from(format!(
                    "[Image too large to preview: {} bytes (max: {} MB)]",
                    metadata.len(),
                    MAX_IMAGE_SIZE / 1024 / 1024
                ))],
                vec![],
            ));
        }

        let dimensions = crate::files::get_image_dimensions(path)?;
        return Ok(PreviewContentType::Image {
            path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
            dimensions,
        });
    }

    // Existing text loading logic
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(PreviewContentType::text(
            vec![ratatui::text::Line::from(format!(
                "[File too large to preview: {} bytes]",
                metadata.len()
            ))],
            vec![],
        ));
    }

    let content = fs::read(path)?;

    // Check for binary content (null bytes in first chunk)
    if content.iter().take(BINARY_DETECTION_BYTES).any(|&b| b == 0) {
        return Ok(PreviewContentType::text(
            vec![ratatui::text::Line::from("[Binary file - cannot preview]")],
            vec![],
        ));
    }

    let text = String::from_utf8(content)?;
    Ok(PreviewContentType::text(
        vec![],
        text.lines().map(String::from).collect(),
    ))
}
