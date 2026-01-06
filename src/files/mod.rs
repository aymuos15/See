pub mod directory;
pub mod loader;

pub use directory::{find_all_files_recursive, read_directory};
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

        Self {
            path,
            name,
            is_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_creation() {
        let path = PathBuf::from("/tmp/test.txt");
        let entry = FileEntry::new(path.clone());

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.path, path);
    }

    #[test]
    fn test_file_entry_with_no_extension() {
        let path = PathBuf::from("/tmp/testfile");
        let entry = FileEntry::new(path);

        assert_eq!(entry.name, "testfile");
    }
}
