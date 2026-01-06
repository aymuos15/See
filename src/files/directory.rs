use crate::files::FileEntry;
use std::fs;
use std::path::Path;

pub fn read_directory(path: &Path) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| FileEntry::new(entry.path()))
        .collect();

    entries.sort_by(|a, b| match (a.is_file, b.is_file) {
        (false, true) => std::cmp::Ordering::Less,
        (true, false) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Recursively find all files under a root directory
pub fn find_all_files_recursive(root: &Path) -> anyhow::Result<Vec<FileEntry>> {
    let mut results = Vec::new();
    collect_files_recursive(root, &mut results)?;
    
    // Sort by name (case-insensitive)
    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(results)
}

fn collect_files_recursive(path: &Path, results: &mut Vec<FileEntry>) -> anyhow::Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
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
            
            // Add the entry
            results.push(FileEntry::new(entry_path.clone()));
            
            // If it's a directory, recurse
            if entry_path.is_dir() {
                let _ = collect_files_recursive(&entry_path, results);
            }
        }
    }
    
    Ok(())
}
