pub mod directory;
pub mod loader;
pub mod symbol;
pub mod symbol_extractor;

pub use directory::{find_all_files_recursive, read_directory};
pub use loader::read_file_content;
pub use symbol::{Symbol, SymbolKind};
pub use symbol_extractor::extract_symbols;

use std::path::PathBuf;

/// Represents a file or directory entry in the file browser.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Full path to the file or directory.
    pub path: PathBuf,
    /// Display name (filename without parent path).
    pub name: String,
    /// True if this is a file, false if it's a directory.
    pub is_file: bool,
}

impl FileEntry {
    /// Creates a new `FileEntry` from a path.
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
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
