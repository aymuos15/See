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
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_read_file_content() {
        let temp_file = "/tmp/test_viewer_content.txt";
        let content = "Hello, world!";

        let mut file = File::create(temp_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        drop(file);

        let result = read_file_content(Path::new(temp_file)).unwrap();
        assert_eq!(result, content);

        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file_content(Path::new("/tmp/nonexistent_viewer_file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_binary_file_detection() {
        let temp_file = "/tmp/test_viewer_binary.bin";
        let mut file = File::create(temp_file).unwrap();
        file.write_all(&[0u8, 1, 2, 3]).unwrap();
        drop(file);

        let result = read_file_content(Path::new(temp_file)).unwrap();
        assert_eq!(result, "[Binary file - cannot preview]");

        std::fs::remove_file(temp_file).unwrap();
    }
}
