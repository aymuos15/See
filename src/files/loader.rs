use std::fs;
use std::path::Path;

const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1 MB

pub fn read_file_content(path: &Path) -> anyhow::Result<String> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(format!(
            "[File too large to preview: {} bytes]",
            metadata.len()
        ));
    }

    let content = fs::read(path)?;

    // Check for binary content (null bytes in first 8KB)
    if content.iter().take(8000).any(|&b| b == 0) {
        return Ok("[Binary file - cannot preview]".to_string());
    }

    String::from_utf8(content).map_err(|_| anyhow::anyhow!("File is not valid UTF-8"))
}
