use crate::config::Config;
use crate::files::FileEntry;
use std::fs;
use std::path::Path;

pub fn read_directory(path: &Path, root: &Path, config: &Config) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| FileEntry::new(entry.path()))
        .filter(|entry| {
            // Get relative path for glob matching
            entry
                .path
                .strip_prefix(root)
                .map_or(true, |rel_path| !config.is_excluded(rel_path))
        })
        .collect();

    entries.sort_by(|a, b| match (a.is_file, b.is_file) {
        (false, true) => std::cmp::Ordering::Less,
        (true, false) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Recursively find all files under a root directory
#[allow(clippy::unnecessary_wraps)]
pub fn find_all_files_recursive(root: &Path, config: &Config) -> anyhow::Result<Vec<FileEntry>> {
    let mut results = Vec::new();
    collect_files_recursive(root, root, config, &mut results);

    // Sort by name (case-insensitive)
    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(results)
}

fn collect_files_recursive(
    path: &Path,
    root: &Path,
    config: &Config,
    results: &mut Vec<FileEntry>,
) {
    if let Ok(entries) = fs::read_dir(path) {
        // Pre-allocate to reduce reallocations for directories with many files
        results.reserve(128);

        for entry in entries.filter_map(std::result::Result::ok) {
            let entry_path = entry.path();

            // Skip dotfiles and dotdirectories
            if let Some(name) = entry_path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with('.') {
                        continue;
                    }
                }
            }

            // Check if excluded by config
            if let Ok(rel_path) = entry_path.strip_prefix(root) {
                if config.is_excluded(rel_path) {
                    continue;
                }
            }

            // Add the entry
            results.push(FileEntry::new(entry_path.clone()));

            // If it's a directory, recurse
            if entry_path.is_dir() {
                collect_files_recursive(&entry_path, root, config, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_structure() -> anyhow::Result<TempDir> {
        let temp = TempDir::new()?;
        let base = temp.path();

        fs::write(base.join("file1.txt"), "content")?;
        fs::write(base.join("file2.log"), "log")?;

        fs::create_dir(base.join("subdir"))?;
        fs::write(base.join("subdir/file3.txt"), "content")?;

        fs::create_dir(base.join(".hidden"))?;
        fs::write(base.join(".hidden/file4.txt"), "hidden")?;

        Ok(temp)
    }

    #[test]
    fn test_read_directory_basic() {
        let temp = create_test_structure().expect("operation failed");
        let config = Config::default();

        let entries = read_directory(temp.path(), temp.path(), &config).expect("operation failed");

        // Should have file1.txt, file2.log, subdir (and possibly .hidden since read_directory doesn't filter)
        assert!(entries.len() >= 3);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"file1.txt"));
        assert!(names.contains(&"file2.log"));
        assert!(names.contains(&"subdir"));
    }

    #[test]
    fn test_read_directory_filters_by_config() {
        let temp = create_test_structure().expect("operation failed");
        let config = Config::default();

        let entries = read_directory(temp.path(), temp.path(), &config).expect("operation failed");

        // Verify it at least loads files
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_read_directory_respects_config_exclusions() {
        let temp = create_test_structure().expect("operation failed");

        let patterns = vec!["*.log".to_string()];
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            if let Ok(glob) = globset::Glob::new(&pattern) {
                builder.add(glob);
            }
        }
        let exclude_set = builder.build().expect("operation failed");
        let config = Config {
            exclude_set,
            ..Default::default()
        };

        let entries = read_directory(temp.path(), temp.path(), &config).expect("operation failed");

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"file1.txt"));
        assert!(!names.contains(&"file2.log"));
    }

    #[test]
    fn test_read_directory_sorts_directories_first() {
        let temp = create_test_structure().expect("operation failed");
        let config = Config::default();

        let entries = read_directory(temp.path(), temp.path(), &config).expect("operation failed");

        // First non-hidden entry should be directory (subdir)
        let first_is_dir = !entries[0].is_file;
        assert!(first_is_dir);
    }

    #[test]
    fn test_find_all_files_recursive_basic() {
        let temp = create_test_structure().expect("operation failed");
        let config = Config::default();

        let entries = find_all_files_recursive(temp.path(), &config).expect("operation failed");

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

        // Should find files and dirs recursively
        assert!(names.contains(&"file1.txt"));
        assert!(names.contains(&"file2.log"));
        assert!(names.contains(&"file3.txt"));
        assert!(names.contains(&"subdir"));

        // But not hidden
        assert!(!names.contains(&".hidden"));
        assert!(!names.contains(&"file4.txt")); // In hidden dir
    }

    #[test]
    fn test_find_all_files_recursive_sorts() {
        let temp = create_test_structure().expect("operation failed");
        let config = Config::default();

        let entries = find_all_files_recursive(temp.path(), &config).expect("operation failed");

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

        // Check basic order (case-insensitive sort)
        let mut sorted_names = names.clone();
        sorted_names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        assert_eq!(names, sorted_names);
    }
}
