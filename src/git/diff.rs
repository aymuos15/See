use git2::Repository;
use similar::{ChangeTag, TextDiff};
use std::path::Path;

/// Generate unified diff content comparing current file with HEAD
#[allow(dead_code)]
pub fn generate_diff_lines(file_path: &Path, current_content: &str) -> Option<Vec<String>> {
    let repo = Repository::discover(file_path).ok()?;

    // Get HEAD version of file
    let head_content = get_head_content(&repo, file_path)?;

    // Convert current content to owned String for TextDiff
    let current_str = current_content.to_string();

    // Generate diff using similar crate
    let diff = TextDiff::from_lines(&head_content, &current_str);

    let mut diff_lines = Vec::new();

    // Add header
    let workdir = repo.workdir()?;
    let relative_path = file_path.strip_prefix(workdir).ok()?;
    diff_lines.push(format!(
        "diff --git a/{} b/{}",
        relative_path.display(),
        relative_path.display()
    ));
    diff_lines.push(format!("--- a/{}", relative_path.display()));
    diff_lines.push(format!("+++ b/{}", relative_path.display()));

    // Generate unified diff sections
    for group in diff.grouped_ops(3) {
        // Calculate line numbers for this hunk
        let old_start = group[0].old_range().start;
        let old_end = group
            .iter()
            .map(|op| op.old_range().end)
            .max()
            .unwrap_or(old_start);
        let new_start = group[0].new_range().start;
        let new_end = group
            .iter()
            .map(|op| op.new_range().end)
            .max()
            .unwrap_or(new_start);

        let old_len = old_end.saturating_sub(old_start);
        let new_len = new_end.saturating_sub(new_start);

        // Hunk header
        diff_lines.push(format!(
            "@@ -{},{} +{},{} @@",
            old_start + 1,
            old_len.max(1),
            new_start + 1,
            new_len.max(1)
        ));

        // Changes in this hunk
        for op in &group {
            for change in diff.iter_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                let line = format!("{sign}{change}");
                diff_lines.push(line.trim_end().to_string());
            }
        }
    }

    Some(diff_lines)
}

/// Get file content from HEAD commit
#[allow(dead_code)]
fn get_head_content(repo: &Repository, file_path: &Path) -> Option<String> {
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    let tree = commit.tree().ok()?;

    // Get relative path from repo root
    let workdir = repo.workdir()?;
    let relative_path = file_path.strip_prefix(workdir).ok()?;

    // Get tree entry for this file
    let entry = tree.get_path(relative_path).ok()?;
    let object = entry.to_object(repo).ok()?;
    let blob = object.as_blob()?;

    // Convert bytes to string
    std::str::from_utf8(blob.content()).ok().map(String::from)
}
