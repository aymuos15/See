use crate::constants::{BINARY_DETECTION_BYTES, MAX_FILE_SIZE};
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_file_content() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Hello, world!";
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = read_file_content(temp_file.path()).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file_content(Path::new("/nonexistent/path/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_binary_file_detection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[0u8, 1, 2, 3]).unwrap();

        let result = read_file_content(temp_file.path()).unwrap();
        assert_eq!(result, "[Binary file - cannot preview]");
    }
}
