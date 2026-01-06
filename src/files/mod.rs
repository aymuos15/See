pub mod directory;
pub mod loader;

pub use directory::read_directory;
pub use loader::read_file_content;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_file: bool,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let is_file = path.is_file();

        Self { path, name, is_file }
    }
}
