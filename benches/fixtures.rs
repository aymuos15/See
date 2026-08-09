//! Deterministic inputs for the benchmarks.
//!
//! Everything here is generated rather than committed: a repository of large
//! sample files would be dead weight, and generated input is guaranteed
//! identical on every machine and every run, which is what makes two
//! measurements comparable.
//!
//! Bump [`FIXTURE_VERSION`] whenever a generator changes. Numbers recorded
//! under different versions describe different work and must not be compared.

// Each bench target uses only the fixtures it needs.
#![allow(dead_code)]
// These generators run once during setup, outside anything being measured, so
// building strings the readable way is worth more than saving an allocation.
#![allow(clippy::format_push_string)]

/// Identifies the shape of the generated inputs. Recorded alongside results.
pub const FIXTURE_VERSION: u32 = 1;

/// A terminal-sized viewport: the amount of drawing is fixed by this, whatever
/// the content behind it.
pub const VIEWPORT: (u16, u16) = (200, 50);

/// An app rooted at a directory holding `files` generated files.
pub fn app_with_files(files: usize) -> (viewer::app::App, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    for index in 0..files {
        let path = dir.path().join(format!("file_{index:05}.rs"));
        std::fs::write(&path, "fn main() {}\n").expect("write fixture");
    }

    // A default config, not the developer's: exclusion patterns and indent
    // guides would otherwise vary the work from machine to machine.
    let app =
        viewer::app::App::with_config(dir.path().to_path_buf(), viewer::config::Config::default())
            .expect("app");
    (app, dir)
}

/// The preview content the app would hold after loading `source` as text.
pub fn text_preview(source: &str) -> viewer::app::PreviewContentType {
    let raw_lines = lines_of(source);
    let lines: Vec<ratatui::text::Line<'static>> = raw_lines
        .iter()
        .cloned()
        .map(ratatui::text::Line::from)
        .collect();
    viewer::app::PreviewContentType::text(lines, raw_lines)
}

/// A test terminal at [`VIEWPORT`] size.
pub fn terminal() -> ratatui::Terminal<ratatui::backend::TestBackend> {
    ratatui::Terminal::new(ratatui::backend::TestBackend::new(VIEWPORT.0, VIEWPORT.1))
        .expect("test terminal")
}

/// A Rust source file of roughly `lines` lines, with the mix of comments,
/// strings, control flow and nesting that syntax highlighting has to work
/// through.
pub fn rust_source(lines: usize) -> String {
    let mut out = String::with_capacity(lines * 40);
    out.push_str("//! Generated fixture.\n\nuse std::collections::HashMap;\n\n");

    let mut written = 4;
    let mut index = 0;
    while written < lines {
        out.push_str(&format!(
            "/// Item {index}, doing the {index}th thing.\n\
             pub fn item_{index}(input: &str, count: usize) -> Option<String> {{\n\
             \x20   let mut totals: HashMap<&str, usize> = HashMap::new();\n\
             \x20   for (i, part) in input.split(\"sep_{index}\").enumerate() {{\n\
             \x20       if i % 2 == 0 && !part.is_empty() {{\n\
             \x20           *totals.entry(part).or_insert(0) += count;\n\
             \x20       }} else {{\n\
             \x20           return None; // bail out on the odd ones\n\
             \x20       }}\n\
             \x20   }}\n\
             \x20   Some(format!(\"{{totals:?}} for {index}\"))\n\
             }}\n\n"
        ));
        written += 12;
        index += 1;
    }
    out
}

/// A markdown document with `tables` pipe tables of `rows` rows each,
/// separated by prose. Exercises the table aligner as well as highlighting.
pub fn markdown_tables(tables: usize, rows: usize) -> String {
    let mut out = String::from("# Generated fixture\n\nSome opening prose.\n\n");

    for table in 0..tables {
        out.push_str(&format!("## Table {table}\n\n"));
        out.push_str("| dataset | metric | value | notes |\n|---|---:|---|---|\n");
        for row in 0..rows {
            out.push_str(&format!(
                "| dataset_{row} | {value}.{row} | `code_{row}` | **bold** note {row} |\n",
                value = row * 7 % 100
            ));
        }
        out.push_str("\nProse between the tables, long enough to be a real paragraph.\n\n");
    }
    out
}

/// A markdown table whose cells are double-width, the case that the width
/// measurement has to get right.
pub fn markdown_wide_table(rows: usize) -> String {
    let mut out = String::from("| dataset | ok | notes |\n|---|---|---|\n");
    for row in 0..rows {
        let mark = if row % 3 == 0 { "✅" } else { "❌" };
        out.push_str(&format!("| データ_{row} | {mark} | 説明 {row} |\n"));
    }
    out
}

/// Plain lines, for the aligner and the indent inference.
pub fn lines_of(source: &str) -> Vec<String> {
    source.lines().map(str::to_string).collect()
}

/// File paths as the search index holds them, spread over a directory tree.
pub fn file_paths(count: usize) -> Vec<String> {
    (0..count)
        .map(|index| {
            let dir = index % 37;
            let sub = index % 11;
            format!("src/module_{dir}/section_{sub}/component_{index}.rs")
        })
        .collect()
}

/// `git log` output in the format [`viewer::git`] asks for: fields separated
/// by unit separators, records by NUL.
pub fn git_log_output(commits: usize) -> String {
    let mut out = String::new();
    for index in 0..commits {
        let hash = format!("{index:040x}");
        out.push_str(&format!(
            "{hash}\u{1f}{short}\u{1f}Author {author}\u{1f}{time}\u{1f}Subject line {index} describing a change\0",
            short = &hash[..7],
            author = index % 5,
            time = 1_700_000_000 + i64::try_from(index).unwrap_or(0) * 3600,
        ));
    }
    out
}

/// The body of `git show --numstat --patch`: a stat block, then the patch.
pub fn git_show_body(files: usize, hunk_lines: usize) -> String {
    let mut stats = String::new();
    let mut patch = String::new();

    for file in 0..files {
        let file_path = format!("src/module_{}/file_{file}.rs", file % 7);
        stats.push_str(&format!("{}\t{}\t{file_path}\n", file * 3 % 40, file % 9));

        patch.push_str(&format!("diff --git a/{file_path} b/{file_path}\n"));
        patch.push_str("index 1111111..2222222 100644\n");
        patch.push_str(&format!("--- a/{file_path}\n+++ b/{file_path}\n"));
        patch.push_str(&format!("@@ -1,{hunk_lines} +1,{hunk_lines} @@\n"));
        for line in 0..hunk_lines {
            match line % 4 {
                0 => patch.push_str(&format!("+    let added_{line} = {line};\n")),
                1 => patch.push_str(&format!("-    let removed_{line} = {line};\n")),
                _ => patch.push_str(&format!("     let context_{line} = {line};\n")),
            }
        }
    }

    format!("{stats}\n{patch}")
}
