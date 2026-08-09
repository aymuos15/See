//! Parsing and searching: work that runs while the user is typing or waiting
//! for a pane to fill.

use std::hint::black_box;
use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use viewer::files::symbol_extractor::extract_symbols;
use viewer::git;
use viewer::util::fuzzy::fuzzy_filter_indices;

mod fixtures;

fn git_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("git/parse");

    // One batch of the commit list, as the worker fetches it.
    let log = fixtures::git_log_output(500);
    group.throughput(Throughput::Elements(500));
    group.bench_function("log_500_commits", |b| {
        b.iter(|| black_box(git::parse_log(&log)));
    });

    // A commit with a wide diff: stats and patch are split in one pass.
    let body = fixtures::git_show_body(40, 60);
    group.throughput(Throughput::Bytes(body.len() as u64));
    group.bench_function("commit_detail_40_files", |b| {
        b.iter(|| black_box(git::parse_commit_detail("Subject\n\nBody", &body)));
    });

    group.finish();
}

fn symbols(c: &mut Criterion) {
    // Runs over every file in the tree when the symbol index is built.
    let source = fixtures::rust_source(2_000);
    let mut group = c.benchmark_group("symbols/extract");
    group.sample_size(30);
    group.throughput(Throughput::Bytes(source.len() as u64));
    group.bench_function("rust_2k_lines", |b| {
        b.iter(|| black_box(extract_symbols(Path::new("fixture.rs"), &source)));
    });
    group.finish();
}

fn fuzzy(c: &mut Criterion) {
    // Re-run on every keystroke in the file and symbol search popups.
    let paths = fixtures::file_paths(10_000);
    let mut group = c.benchmark_group("fuzzy/filter");
    group.throughput(Throughput::Elements(paths.len() as u64));

    for query in ["c", "comp", "module_7/comp"] {
        group.bench_function(
            format!("10k_paths_query_{}", query.replace('/', "_")),
            |b| {
                b.iter(|| black_box(fuzzy_filter_indices(query, &paths, String::as_str)));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, git_parsing, symbols, fuzzy);
criterion_main!(benches);
