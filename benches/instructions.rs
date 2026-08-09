//! Instruction counts for the benchmarks whose timings are hardest to trust.
//!
//! Wall-clock numbers on a laptop drift a few percent between runs, which puts
//! a floor under what counts as a detectable regression. Callgrind counts
//! instructions instead: the same code over the same input gives the same
//! number, so a 2% move is real. It does not measure time — a change that
//! trades instructions for cache locality shows up here as a regression and in
//! `render.rs` as an improvement, and the timings are the ones that matter.
//!
//! Off by default: needs Valgrind and the matching runner.
//!
//!     sudo apt install valgrind
//!     cargo install iai-callgrind-runner --version 0.16.1
//!     cargo bench --features iai --bench instructions

#![allow(clippy::unwrap_used)]

use std::hint::black_box;
use std::path::Path;
use std::rc::Rc;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use ratatui::backend::TestBackend;
use ratatui::text::Line;
use ratatui::Terminal;
use viewer::app::App;
use viewer::highlight::syntax::SyntaxHighlighter;
use viewer::util::fuzzy::fuzzy_filter_indices;

mod fixtures;

#[library_benchmark]
fn highlight_rust_1k() -> Vec<Line<'static>> {
    let highlighter = SyntaxHighlighter::new();
    let source = fixtures::rust_source(1_000);
    black_box(highlighter.highlight(Path::new("fixture.rs"), &source))
}

#[library_benchmark]
fn fuzzy_10k_paths() -> Vec<usize> {
    let paths = fixtures::file_paths(10_000);
    black_box(fuzzy_filter_indices("comp", &paths, String::as_str))
}

/// Everything `render_frame_20k_line_file` needs, built outside the measured
/// function so the instruction count covers only the frame draw.
fn render_frame_setup() -> (App, tempfile::TempDir, Terminal<TestBackend>) {
    let (mut app, dir) = fixtures::app_with_files(1);
    let source = fixtures::rust_source(20_000);
    app.shared_preview_content = Some(Rc::new(fixtures::text_preview(&source)));
    (app, dir, fixtures::terminal())
}

#[library_benchmark]
#[bench::frame(setup = render_frame_setup)]
fn render_frame_20k_line_file(state: (App, tempfile::TempDir, Terminal<TestBackend>)) {
    let (mut app, _dir, mut terminal) = state;
    terminal
        .draw(|frame| viewer::ui::render(frame, black_box(&mut app)))
        .unwrap();
}

library_benchmark_group!(
    name = instructions;
    benchmarks = highlight_rust_1k, fuzzy_10k_paths, render_frame_20k_line_file
);

main!(library_benchmark_groups = instructions);
