//! Turning a file's text into styled lines.
//!
//! This is the work AGENTS.md calls out as too slow to do between keystrokes,
//! and the reason the background worker exists at all. A regression here is
//! felt as a delay before a file's colours appear.

use std::hint::black_box;
use std::path::Path;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use ratatui::text::Line;
use viewer::highlight::markdown_table::format_tables;
use viewer::highlight::syntax::SyntaxHighlighter;
use viewer::ui::indent;

mod fixtures;

fn syntax(c: &mut Criterion) {
    // Loading syntect's syntax and theme sets is a one-off cost paid when the
    // worker thread starts, so it is measured separately from highlighting.
    c.bench_function("syntax/highlighter_new", |b| {
        b.iter(|| black_box(SyntaxHighlighter::new()));
    });

    let highlighter = SyntaxHighlighter::new();
    let mut group = c.benchmark_group("syntax/highlight");
    // Highlighting a large file takes the best part of a second, so the
    // default hundred samples would make a full run unbearable.
    group.sample_size(20);

    for lines in [1_000, 10_000] {
        let source = fixtures::rust_source(lines);
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(format!("rust_{lines}_lines"), |b| {
            b.iter(|| black_box(highlighter.highlight(Path::new("fixture.rs"), &source)));
        });
    }

    // Markdown runs the table aligner as part of highlighting.
    let markdown = fixtures::markdown_tables(8, 40);
    group.throughput(Throughput::Bytes(markdown.len() as u64));
    group.bench_function("markdown_with_tables", |b| {
        b.iter(|| black_box(highlighter.highlight(Path::new("fixture.md"), &markdown)));
    });

    group.finish();
}

fn tables(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown/format_tables");

    let cases = [
        ("ascii_8x40", fixtures::markdown_tables(8, 40)),
        ("wide_cells_200_rows", fixtures::markdown_wide_table(200)),
        ("no_tables", fixtures::rust_source(500)),
    ];

    for (name, source) in cases {
        let lines: Vec<Line<'static>> = source.lines().map(|l| Line::from(l.to_string())).collect();
        group.throughput(Throughput::Elements(lines.len() as u64));
        group.bench_function(name, |b| {
            // The aligner consumes its input, so each iteration gets a clone;
            // BatchSize::LargeInput keeps that setup out of the measurement.
            b.iter_batched(
                || lines.clone(),
                |input| black_box(format_tables(input)),
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn indent_guides(c: &mut Criterion) {
    // Called with the whole file's lines on every frame, so its cost lands on
    // the render path rather than the worker.
    let source = fixtures::rust_source(10_000);
    let raw = fixtures::lines_of(&source);

    c.bench_function("indent/infer_width_10k_lines", |b| {
        b.iter(|| black_box(indent::infer_width(&raw)));
    });
}

criterion_group!(benches, syntax, tables, indent_guides);
criterion_main!(benches);
