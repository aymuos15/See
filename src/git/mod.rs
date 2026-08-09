//! Reading commits out of a git repository.
//!
//! Everything here shells out to `git` and parses its output: the viewer only
//! ever reads history, so a linked library would buy nothing for its weight.
//! Records are separated with NUL and fields with unit separators, so commit
//! subjects containing any printable character still parse.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Separates fields within one record.
const FIELD_SEP: char = '\x1f';
/// Separates whole records.
const RECORD_SEP: char = '\0';
/// The same two separators as git format placeholders. They cannot be written
/// literally: an argument containing a NUL byte cannot be passed to a process.
const FIELD_SEP_FMT: &str = "%x1f";
const RECORD_SEP_FMT: &str = "%x00";

/// A single commit as shown in the log list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    /// Author date, as a Unix timestamp.
    pub timestamp: i64,
    /// First line of the commit message.
    pub subject: String,
}

/// How one file changed in a commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeKind {
    pub const fn letter(self) -> char {
        match self {
            Self::Added => 'A',
            Self::Modified => 'M',
            Self::Deleted => 'D',
            Self::Renamed => 'R',
        }
    }
}

/// One file's entry in a commit's change list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStat {
    pub path: String,
    pub added: Option<u32>,
    pub removed: Option<u32>,
    pub kind: ChangeKind,
    /// Line in the diff body where this file's section starts.
    pub diff_line: usize,
}

impl FileStat {
    /// True for files git reported as binary, which have no line counts.
    pub const fn is_binary(&self) -> bool {
        self.added.is_none() || self.removed.is_none()
    }
}

/// A commit's full message, changed files, and patch text.
#[derive(Debug, Clone, Default)]
pub struct CommitDetail {
    pub message: String,
    pub files: Vec<FileStat>,
    /// The patch, already split into lines.
    pub diff: Vec<String>,
}

/// Locate the repository containing `path`, if any.
pub fn repo_root(path: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(PathBuf::from(root))
    }
}

/// Read up to `limit` commits, starting `skip` commits back from HEAD.
pub fn log(repo: &Path, skip: usize, limit: usize) -> anyhow::Result<Vec<Commit>> {
    let field = FIELD_SEP_FMT;
    let format =
        format!("--pretty=format:%H{field}%h{field}%an{field}%at{field}%s{RECORD_SEP_FMT}");
    let output = run_git(
        repo,
        &[
            "log",
            &format,
            &format!("--skip={skip}"),
            &format!("-n{limit}"),
        ],
    )?;
    Ok(parse_log(&output))
}

/// Read one commit's message, file stats, and patch.
pub fn commit_detail(repo: &Path, hash: &str) -> anyhow::Result<CommitDetail> {
    let message = run_git(repo, &["show", "--no-patch", "--pretty=format:%B", hash])?;

    // `--numstat` and the patch come from one call so the two always describe
    // the same set of files, in the same order.
    let body = run_git(
        repo,
        &[
            "show",
            "--numstat",
            "--patch",
            "--pretty=format:",
            "--no-color",
            hash,
        ],
    )?;

    Ok(parse_commit_detail(message.trim_end(), &body))
}

fn run_git(repo: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {}: {}", args[0], stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn parse_log(output: &str) -> Vec<Commit> {
    output
        .split(RECORD_SEP)
        .map(str::trim_start)
        .filter(|record| !record.is_empty())
        .filter_map(|record| {
            let mut fields = record.split(FIELD_SEP);
            Some(Commit {
                hash: fields.next()?.to_string(),
                short_hash: fields.next()?.to_string(),
                author: fields.next()?.to_string(),
                timestamp: fields.next()?.parse().unwrap_or(0),
                subject: fields.next()?.to_string(),
            })
        })
        .collect()
}

/// Split `git show --numstat --patch` output into its stat block and patch.
///
/// The numstat lines come first, then a blank line, then the patch. Each file's
/// stat is matched back to the patch by the `diff --git` header order, which is
/// the same in both halves.
pub fn parse_commit_detail(message: &str, body: &str) -> CommitDetail {
    let mut stats: Vec<(String, Option<u32>, Option<u32>)> = Vec::new();
    let mut diff: Vec<String> = Vec::new();
    let mut in_patch = false;

    for line in body.lines() {
        if in_patch {
            diff.push(line.to_string());
            continue;
        }
        if line.starts_with("diff --git ") {
            in_patch = true;
            diff.push(line.to_string());
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if let Some(stat) = parse_numstat_line(line) {
            stats.push(stat);
        }
    }

    let mut files = Vec::new();
    let mut stats = stats.into_iter();
    for (index, line) in diff.iter().enumerate() {
        if !line.starts_with("diff --git ") {
            continue;
        }
        let Some((path, added, removed)) = stats.next() else {
            break;
        };
        files.push(FileStat {
            kind: change_kind(&diff[index..]),
            path,
            added,
            removed,
            diff_line: index,
        });
    }

    CommitDetail {
        message: message.to_string(),
        files,
        diff,
    }
}

/// Parse one `added<TAB>removed<TAB>path` line. Binary files report `-`.
fn parse_numstat_line(line: &str) -> Option<(String, Option<u32>, Option<u32>)> {
    let mut fields = line.splitn(3, '\t');
    let added = fields.next()?;
    let removed = fields.next()?;
    let path = fields.next()?;
    if path.is_empty() {
        return None;
    }
    Some((path.to_string(), added.parse().ok(), removed.parse().ok()))
}

/// Read the change kind out of the header lines following a `diff --git` line.
fn change_kind(section: &[String]) -> ChangeKind {
    for line in section.iter().skip(1) {
        if line.starts_with("diff --git ") {
            break;
        }
        if line.starts_with("new file mode") {
            return ChangeKind::Added;
        }
        if line.starts_with("deleted file mode") {
            return ChangeKind::Deleted;
        }
        if line.starts_with("rename from") || line.starts_with("similarity index") {
            return ChangeKind::Renamed;
        }
        if line.starts_with("@@") {
            break;
        }
    }

    ChangeKind::Modified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_log_records() {
        let output = "abc123\x1fabc\x1fAda\x1f1700000000\x1fFix the thing\0def456\x1fdef\x1fGrace\x1f1700000100\x1fAdd another\0";
        let commits = parse_log(output);

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].short_hash, "abc");
        assert_eq!(commits[0].author, "Ada");
        assert_eq!(commits[0].timestamp, 1_700_000_000);
        assert_eq!(commits[1].subject, "Add another");
    }

    #[test]
    fn parses_subjects_containing_separators_and_newlines() {
        let output = "abc123\x1fabc\x1fAda\x1f1700000000\x1fUse a\ttab and | pipe\0";
        let commits = parse_log(output);

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].subject, "Use a\ttab and | pipe");
    }

    #[test]
    fn skips_malformed_log_records() {
        let output = "abc123\x1fabc\0def456\x1fdef\x1fGrace\x1f1700000100\x1fAdd another\0";
        let commits = parse_log(output);

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].short_hash, "def");
    }

    const BODY: &str = "\
12\t3\tsrc/main.rs
5\t0\tsrc/new.rs
-\t-\tassets/logo.png

diff --git a/src/main.rs b/src/main.rs
index 111..222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
+added line
 context
diff --git a/src/new.rs b/src/new.rs
new file mode 100644
--- /dev/null
+++ b/src/new.rs
@@ -0,0 +1,5 @@
+brand new
diff --git a/assets/logo.png b/assets/logo.png
Binary files a/assets/logo.png and b/assets/logo.png differ
";

    #[test]
    fn splits_stats_from_patch() {
        let detail = parse_commit_detail("Subject\n\nBody", BODY);

        assert_eq!(detail.message, "Subject\n\nBody");
        assert_eq!(detail.files.len(), 3);
        assert_eq!(detail.files[0].path, "src/main.rs");
        assert_eq!(detail.files[0].added, Some(12));
        assert_eq!(detail.files[0].removed, Some(3));
        assert!(detail.diff[0].starts_with("diff --git a/src/main.rs"));
    }

    #[test]
    fn detects_change_kinds() {
        let detail = parse_commit_detail("", BODY);

        assert_eq!(detail.files[0].kind, ChangeKind::Modified);
        assert_eq!(detail.files[1].kind, ChangeKind::Added);
        assert!(detail.files[2].is_binary());
    }

    #[test]
    fn points_each_file_at_its_diff_section() {
        let detail = parse_commit_detail("", BODY);

        for file in &detail.files {
            assert!(detail.diff[file.diff_line].starts_with("diff --git"));
            assert!(detail.diff[file.diff_line].contains(&file.path));
        }
    }

    /// Build a throwaway repository with one commit touching two files.
    fn scratch_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let git = |args: &[&str]| {
            let status = Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(args)
                .status()
                .expect("run git");
            assert!(status.success(), "git {args:?} failed");
        };

        git(&["init", "-q"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        std::fs::write(dir.path().join("kept.txt"), "one\ntwo\n").expect("write");
        std::fs::write(dir.path().join("gone.txt"), "bye\n").expect("write");
        git(&["add", "."]);
        git(&["commit", "-qm", "First: add files"]);

        std::fs::write(dir.path().join("kept.txt"), "one\nchanged\n").expect("write");
        std::fs::remove_file(dir.path().join("gone.txt")).expect("remove");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "Second: edit and delete"]);

        dir
    }

    #[test]
    fn reads_a_real_repository() {
        let dir = scratch_repo();
        let root = repo_root(dir.path()).expect("repo root");
        assert_eq!(
            root.canonicalize().ok(),
            dir.path().canonicalize().ok(),
            "repo_root should find the scratch repo"
        );

        let commits = log(&root, 0, 10).expect("log");
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].subject, "Second: edit and delete");
        assert_eq!(commits[0].author, "Test");
        assert!(commits[0].timestamp > 0);

        let detail = commit_detail(&root, &commits[0].hash).expect("detail");
        assert_eq!(detail.message.trim(), "Second: edit and delete");

        let paths: Vec<&str> = detail.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["gone.txt", "kept.txt"]);

        let deleted = &detail.files[0];
        assert_eq!(deleted.kind, ChangeKind::Deleted);
        assert_eq!(deleted.removed, Some(1));

        let edited = &detail.files[1];
        assert_eq!(edited.kind, ChangeKind::Modified);
        assert_eq!((edited.added, edited.removed), (Some(1), Some(1)));
        assert!(detail.diff[edited.diff_line].contains("kept.txt"));
    }

    #[test]
    fn reports_a_missing_commit_as_an_error() {
        let dir = scratch_repo();
        let error = commit_detail(dir.path(), "0000000000000000000000000000000000000000")
            .expect_err("unknown commit should fail");
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn handles_a_commit_with_no_changes() {
        let detail = parse_commit_detail("Empty commit", "");

        assert!(detail.files.is_empty());
        assert!(detail.diff.is_empty());
        assert_eq!(detail.message, "Empty commit");
    }
}
