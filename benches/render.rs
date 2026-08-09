//! Drawing one frame.
//!
//! AGENTS.md's rule is that per-frame work stays proportional to what is on
//! screen, not to how big the file or directory is. These benchmarks are how
//! that rule gets checked: each pair measures the same viewport against a
//! small and a large subject, so a cost that scales with the subject shows up
//! as a widening gap rather than as a number nobody can interpret.

use std::hint::black_box;
use std::rc::Rc;

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use viewer::app::App;

mod fixtures;

/// One measured case: draw frames of `app` into `terminal` until criterion is
/// satisfied.
fn bench_draw(
    group: &mut BenchmarkGroup<WallTime>,
    name: impl Into<String>,
    app: &mut App,
    terminal: &mut Terminal<TestBackend>,
) {
    group.bench_function(name.into(), |b| {
        b.iter(|| {
            terminal
                .draw(|frame| viewer::ui::render(frame, black_box(&mut *app)))
                .expect("draw");
        });
    });
}

/// The file list beside the preview, drawn against directories of two sizes.
fn file_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/file_list");

    for count in [50, 5_000] {
        let (mut app, _dir) = fixtures::app_with_files(count);
        let mut terminal = fixtures::terminal();
        bench_draw(
            &mut group,
            format!("{count}_entries"),
            &mut app,
            &mut terminal,
        );
    }

    group.finish();
}

/// The text preview, drawn against files of two sizes at the same viewport.
fn text(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/text_preview");

    for lines in [200, 20_000] {
        let (mut app, _dir) = fixtures::app_with_files(4);
        let source = fixtures::rust_source(lines);
        app.shared_preview_content = Some(Rc::new(fixtures::text_preview(&source)));
        let mut terminal = fixtures::terminal();
        bench_draw(
            &mut group,
            format!("{lines}_line_file"),
            &mut app,
            &mut terminal,
        );
    }

    group.finish();
}

/// Scrolling into the middle of a long file must cost the same as sitting at
/// the top: only the visible slice should be touched.
fn scrolled(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/scroll_position");
    let source = fixtures::rust_source(20_000);

    for (name, scroll) in [("top", 0_u16), ("middle", 9_000)] {
        let (mut app, _dir) = fixtures::app_with_files(4);
        app.shared_preview_content = Some(Rc::new(fixtures::text_preview(&source)));
        app.preview_scroll = scroll;
        let mut terminal = fixtures::terminal();
        bench_draw(&mut group, name, &mut app, &mut terminal);
    }

    group.finish();
}

/// Indent guides add per-frame work on top of the plain preview.
fn indent_guides(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/indent_guides");
    let source = fixtures::rust_source(20_000);

    for enabled in [false, true] {
        let (mut app, _dir) = fixtures::app_with_files(4);
        app.shared_preview_content = Some(Rc::new(fixtures::text_preview(&source)));
        app.config.indent_guides = enabled;
        let mut terminal = fixtures::terminal();
        bench_draw(
            &mut group,
            if enabled { "on" } else { "off" },
            &mut app,
            &mut terminal,
        );
    }

    group.finish();
}

criterion_group!(benches, file_list, text, scrolled, indent_guides);
criterion_main!(benches);
