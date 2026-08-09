# Benchmarks

Performance tracking for the viewer. The benchmark *code* lives in `benches/`
(cargo owns that directory); this directory holds the *results* and the tooling
that records them.

```
benches/            fixtures.rs, render.rs, highlight.rs, parse.rs, instructions.rs
benchmarks/
  run.sh            run under comparable conditions, then summarise
  collect.py        pull criterion's numbers into one small file; compare runs
  baseline.json     the numbers everything is compared against
  latest.json       the most recent run (not committed)
  history/          one file per recorded run, kept in git
```

## Running them

```bash
./benchmarks/run.sh                 # everything, compared with the baseline
./benchmarks/run.sh render          # one target: render, highlight or parse
./benchmarks/run.sh --quick         # fewer samples; rough, for a fast loop
./benchmarks/run.sh --save          # also record the run in history/
./benchmarks/run.sh -- --baseline x # anything after -- goes to criterion
```

A full run takes a few minutes, mostly in `syntax/highlight`. `--quick` cuts
that to well under a minute at the cost of wider error bars — fine for "did
this change make things obviously worse", useless for a 5% question.

Two things the comparison will warn you about rather than let you misread:

- **Comparing a `--quick` run against a full baseline** produces differences of
  10% or more from the sampling alone. The mode is recorded with the results,
  and a mismatch is called out.
- **Running one target** leaves the other targets' results in place from
  whenever they last ran, so a capture mixes fresh and old numbers. Those are
  listed rather than folded in silently.

Criterion also writes an HTML report to `target/criterion/report/index.html`,
with distributions and a comparison against the previous run. Read that when a
number moves and you want to know whether it really moved.

## Recording a result

```bash
./benchmarks/run.sh --save          # writes history/<date>-<commit>.json
python3 benchmarks/collect.py promote   # makes the latest run the baseline
```

Promote a new baseline when a change is *meant* to move the numbers, or when
you switch machines. Otherwise leave it: a baseline that follows every run
tracks nothing.

## Reading the output

`run.sh` prints one row per benchmark with the change against the baseline.
Anything over ±10% is called out; anything under it is noise on a laptop and is
printed only for information. A "SLOWER" row is a prompt to re-run, not a
verdict — one run is never evidence.

The comparison refuses to be quiet about apples and oranges: it warns when the
CPU, the governor, or the fixture version differs from the baseline's.

## Making numbers comparable

Wall-clock timings on a developer machine drift for reasons unrelated to the
code. Two runs of the *same commit* on this machine differed by up to 7%, which
is where the 10% threshold comes from — it is measured, not guessed.

`run.sh` handles what it can and tells you about the rest:

- **CPU governor.** It warns if the governor is not `performance`. Set it with
  `sudo cpupower frequency-set -g performance` before a run that matters.
- **Core pinning.** Runs under `taskset -c 0,1` when available, so the
  scheduler cannot migrate the benchmark between cores mid-run.
- **Background load.** Nothing can be done about this automatically. A browser
  or an indexer running during a benchmark is worth 10% on its own.
- **Thermal throttling.** A laptop that has been benchmarking for five minutes
  is slower than one that just woke up. Compare within a session where you can.
- **The build is not pinned.** `run.sh` compiles first, unpinned, then runs the
  benchmarks under `taskset`, so compilation neither crawls on two cores nor
  lands inside a measurement.

Two conventions carry the rest of the weight: compare **ratios against a
baseline taken on the same machine**, never absolute numbers between machines,
and treat the fixture version as part of the measurement.

## Fixtures

Inputs are generated, not committed — `benches/fixtures.rs` builds every source
file, markdown table, path list and diff from scratch, deterministically. That
keeps large sample files out of git and guarantees two runs measure identical
work.

`FIXTURE_VERSION` in that file must be bumped whenever a generator changes.
Results recorded under different versions describe different work, and
`collect.py` warns rather than comparing them silently.

## What is measured, and why

| Group | What it guards |
|---|---|
| `render/*` | The rule in AGENTS.md: per-frame work stays proportional to the viewport, not to the file or directory. Each pair holds the viewport fixed and varies the subject; a widening gap is a bug. |
| `syntax/highlight` | The cost that forced highlighting onto a background thread. Felt as a delay before colours appear. |
| `markdown/format_tables` | Table alignment, run on every markdown highlight. |
| `indent/infer_width` | Called with the whole file's lines on every frame. |
| `symbols/extract` | Tree-sitter parsing, run over every file when the symbol index is built. |
| `git/parse` | Reading `git log` and `git show` output. The subprocess itself is not measured — that is git's time, not ours. |
| `fuzzy/filter` | Re-run on every keystroke in the search popups. |

Deliberately not measured: anything that needs a real terminal, spawns `git`,
or loads pdfium. Process spawn and library load dominate those and are not our
code to make faster.

## Caveats

- **Bench builds are not release builds.** `[profile.bench]` overrides the
  release profile's full LTO with thin LTO, because a 40-second rebuild between
  measurements makes the loop unusable. The shipped binary is built with more
  aggressive codegen and will be somewhat faster than these numbers.
- **The render benchmarks draw to a `TestBackend`**, an in-memory buffer. They
  measure the work of building a frame, not the terminal's work of displaying
  it. A PDF page's graphics-protocol encoding is not covered here.
- **The numbers are medians**, which is what `collect.py` records. Criterion's
  own report has the distribution if you need it.

## Adding a benchmark

1. Put it in the target it belongs to, or add a new file and a matching
   `[[bench]]` entry in `Cargo.toml` with `harness = false`.
2. Generate its input from `benches/fixtures.rs`; bump `FIXTURE_VERSION` if you
   change an existing generator.
3. Where the point is a scaling property, benchmark a *pair* — small and large
   subject at a fixed viewport — so the comparison carries the meaning rather
   than an absolute number nobody can calibrate.
4. Run with `--save` and `promote` to fold it into the baseline.

## Instruction counts (optional)

Wall-clock numbers on this machine drift a few percent between runs, which sets
the floor on what a regression can be detected. Counting instructions instead
removes almost all of that noise, at the cost of not measuring time.

This is set up but **off by default**, because it needs Valgrind installed:

```bash
sudo apt install valgrind
cargo install iai-callgrind-runner --version 0.16.1   # must match the crate
cargo bench --features iai --bench instructions
```

Instruction counts are stable to a fraction of a percent, so a 2% change there
is real where a 2% change in wall-clock time is not. They are not a substitute
for the timings: a change that trades instructions for better cache behaviour
looks like a regression here and an improvement in practice.

Run it directly with `cargo bench` as above — `run.sh` cannot launch it, since
it never passes `--features iai`. Its results land in `target/iai`, which
`collect.py` does not read: instruction counts are outside the
baseline/history/compare flow entirely, so compare them by re-running on the
two commits of interest.
