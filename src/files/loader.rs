use crate::app::PreviewContentType;
use crate::constants::{BINARY_DETECTION_BYTES, MAX_FILE_SIZE, MAX_IMAGE_SIZE, MAX_PDF_SIZE};
use crate::files::{is_image_file, is_pdf_file};
use std::fs;
use std::path::Path;

pub fn read_file_content(path: &Path) -> anyhow::Result<String> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(format!(
            "[File too large to preview: {} bytes]",
            metadata.len()
        ));
    }

    let content = fs::read(path)?;

    // Check for binary content (null bytes in first chunk)
    if content.iter().take(BINARY_DETECTION_BYTES).any(|&b| b == 0) {
        return Ok("[Binary file - cannot preview]".to_string());
    }

    String::from_utf8(content).map_err(|_| anyhow::anyhow!("File is not valid UTF-8"))
}

/// Load preview content (text or image)
pub fn load_preview_content(path: &Path) -> anyhow::Result<PreviewContentType> {
    let metadata = fs::metadata(path)?;

    // Check for PDF file first
    if is_pdf_file(path) {
        if metadata.len() > MAX_PDF_SIZE {
            return Ok(PreviewContentType::Text {
                lines: vec![ratatui::text::Line::from(format!(
                    "[PDF too large to preview: {} bytes (max: {} MB)]",
                    metadata.len(),
                    MAX_PDF_SIZE / 1024 / 1024
                ))],
                raw_lines: vec![],
            });
        }

        // Return PDF content type - actual rendering happens via worker
        return Ok(PreviewContentType::Pdf {
            path: path.to_path_buf(),
            current_page: 0,
            total_pages: 0, // Will be updated when worker responds
        });
    }

    // Check for image file
    if is_image_file(path) {
        if metadata.len() > MAX_IMAGE_SIZE {
            return Ok(PreviewContentType::Text {
                lines: vec![ratatui::text::Line::from(format!(
                    "[Image too large to preview: {} bytes (max: {} MB)]",
                    metadata.len(),
                    MAX_IMAGE_SIZE / 1024 / 1024
                ))],
                raw_lines: vec![],
            });
        }

        let dimensions = crate::files::get_image_dimensions(path)?;
        return Ok(PreviewContentType::Image {
            path: path.to_path_buf(),
            dimensions,
        });
    }

    // Existing text loading logic
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(PreviewContentType::Text {
            lines: vec![ratatui::text::Line::from(format!(
                "[File too large to preview: {} bytes]",
                metadata.len()
            ))],
            raw_lines: vec![],
        });
    }

    let content = fs::read(path)?;

    // Check for binary content (null bytes in first chunk)
    if content.iter().take(BINARY_DETECTION_BYTES).any(|&b| b == 0) {
        return Ok(PreviewContentType::Text {
            lines: vec![ratatui::text::Line::from("[Binary file - cannot preview]")],
            raw_lines: vec![],
        });
    }

    let text = String::from_utf8(content)?;
    Ok(PreviewContentType::Text {
        lines: vec![],
        raw_lines: text.lines().map(String::from).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_file_content() {
        let mut temp_file = NamedTempFile::new().expect("operation failed");
        let content = "Hello, world!";
        temp_file.write_all(content.as_bytes()).expect("operation failed");

        let result = read_file_content(temp_file.path()).expect("operation failed");
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file_content(Path::new("/nonexistent/path/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_binary_file_detection() {
        let mut temp_file = NamedTempFile::new().expect("operation failed");
        temp_file.write_all(&[0u8, 1, 2, 3]).expect("operation failed");

        let result = read_file_content(temp_file.path()).expect("operation failed");
        assert_eq!(result, "[Binary file - cannot preview]");
    }
}
