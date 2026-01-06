use crate::files::FileEntry;
use std::fs;
use std::path::Path;

pub fn read_directory(path: &Path) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| FileEntry::new(entry.path()))
        .collect();

    entries.sort_by(|a, b| {
        match (a.is_file, b.is_file) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}
