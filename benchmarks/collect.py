#!/usr/bin/env python3
"""Collect criterion's results into one small file, and compare runs.

Criterion writes a large tree under target/criterion that is machine-specific
and gitignored. What is worth keeping is one number per benchmark, plus enough
context to know whether two runs can honestly be compared at all: the machine,
the CPU governor, and the fixture version the inputs were generated from.

Usage:
    collect.py capture              read target/criterion, print a run summary
    collect.py capture --save       ... and write it to history/
    collect.py compare              compare the latest capture with baseline.json
    collect.py promote              make the latest capture the new baseline
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
CRITERION = REPO / "target" / "criterion"
HISTORY = HERE / "history"
BASELINE = HERE / "baseline.json"
LATEST = HERE / "latest.json"

# Wall-clock timings on a developer machine drift by a few percent between runs
# for reasons that have nothing to do with the code. Two runs of the same commit
# here differed by up to 7%, so only flag changes larger than this.
NOISE_THRESHOLD = 0.10

# Results older than this relative to the newest come from an earlier run and
# are reported rather than quietly folded in.
STALE_SECONDS = 900


def run(*args: str) -> str:
    """Run a command, returning stripped stdout, or "" if it fails."""
    try:
        out = subprocess.run(args, capture_output=True, text=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ""
    return out.stdout.strip()


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return platform.processor() or "unknown"


def governor() -> str:
    path = Path("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
    try:
        return path.read_text().strip()
    except OSError:
        return "unknown"


def fixture_version() -> int:
    """Read FIXTURE_VERSION out of the bench fixtures, so a change to how the
    inputs are generated invalidates comparisons instead of skewing them."""
    source = (REPO / "benches" / "fixtures.rs").read_text()
    match = re.search(r"FIXTURE_VERSION: u32 = (\d+)", source)
    return int(match.group(1)) if match else 0


def capture() -> dict:
    """Read every benchmark's median from criterion's latest run.

    Criterion keeps results from previous runs, so a capture after running one
    target mixes fresh numbers with old ones. Those are reported rather than
    passed off as part of this run.
    """
    if not CRITERION.is_dir():
        sys.exit("No results in target/criterion — run benchmarks/run.sh first.")

    measurements = {}
    written = {}
    for estimates in CRITERION.glob("**/new/estimates.json"):
        meta = estimates.parent / "benchmark.json"
        if not meta.is_file():
            continue
        name = json.loads(meta.read_text())["full_id"]
        median = json.loads(estimates.read_text())["median"]["point_estimate"]
        measurements[name] = round(median, 1)
        written[name] = estimates.stat().st_mtime

    if not measurements:
        sys.exit("target/criterion holds no benchmark results.")

    newest = max(written.values())
    stale = sorted(n for n, t in written.items() if newest - t > STALE_SECONDS)
    if stale:
        print(f"warning: {len(stale)} result(s) are left over from an earlier run "
              "and were not measured just now:")
        for name in stale:
            print(f"  {name}")

    return {
        "commit": run("git", "-C", str(REPO), "rev-parse", "--short", "HEAD") or "unknown",
        "dirty": bool(run("git", "-C", str(REPO), "status", "--porcelain")),
        "captured": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "machine": {
            "cpu": cpu_model(),
            "governor": governor(),
            "os": platform.platform(),
            "rustc": run("rustc", "--version"),
        },
        "fixture_version": fixture_version(),
        # "full" or "quick": a quick run takes far fewer samples, so its
        # numbers are not comparable with a full run's.
        "mode": os.environ.get("BENCH_MODE", "unknown"),
        "stale": stale,
        # Nanoseconds, median of criterion's samples.
        "measurements": dict(sorted(measurements.items())),
    }


def comparable(new: dict, old: dict) -> list[str]:
    """Reasons the two runs cannot be honestly compared."""
    warnings = []
    if new["fixture_version"] != old["fixture_version"]:
        warnings.append(
            f"fixtures changed (v{old['fixture_version']} → v{new['fixture_version']}): "
            "the benchmarks are measuring different work"
        )
    if new["machine"]["cpu"] != old["machine"]["cpu"]:
        warnings.append(
            f"different CPU ({old['machine']['cpu']} → {new['machine']['cpu']})"
        )
    if new["machine"]["governor"] != old["machine"]["governor"]:
        warnings.append(
            f"different CPU governor ({old['machine']['governor']} → "
            f"{new['machine']['governor']})"
        )
    if new.get("mode") != old.get("mode"):
        warnings.append(
            f"sample counts differ ({old.get('mode')} baseline vs "
            f"{new.get('mode')} run): expect false regressions, this is not a "
            "like-for-like comparison"
        )
    if new.get("stale"):
        warnings.append(
            f"{len(new['stale'])} result(s) in this run are left over from an "
            "earlier one"
        )
    return warnings


def human(nanoseconds: float) -> str:
    for unit, scale in (("s", 1e9), ("ms", 1e6), ("µs", 1e3)):
        if nanoseconds >= scale:
            return f"{nanoseconds / scale:.3g}{unit}"
    return f"{nanoseconds:.3g}ns"


def compare(new: dict, old: dict) -> int:
    """Print a comparison table. Returns 1 if anything regressed."""
    for warning in comparable(new, old):
        print(f"warning: {warning}")

    print(f"\nbaseline {old['commit']} ({old['captured']})")
    print(f"current  {new['commit']}{' +dirty' if new['dirty'] else ''} ({new['captured']})\n")

    names = sorted(set(new["measurements"]) | set(old["measurements"]))
    if not names:
        print("No measurements in either run.")
        return 0
    width = max(len(n) for n in names)
    regressed = []

    print(f"{'benchmark'.ljust(width)}  {'baseline':>10}  {'current':>10}  change")
    print("-" * (width + 34))

    for name in names:
        before = old["measurements"].get(name)
        after = new["measurements"].get(name)

        if before is None:
            print(f"{name.ljust(width)}  {'—':>10}  {human(after):>10}  new")
            continue
        if after is None:
            print(f"{name.ljust(width)}  {human(before):>10}  {'—':>10}  gone")
            continue

        ratio = after / before if before else 1.0
        change = ratio - 1
        if change > NOISE_THRESHOLD:
            note = f"SLOWER {change:+.0%}"
            regressed.append(name)
        elif change < -NOISE_THRESHOLD:
            note = f"faster {change:+.0%}"
        else:
            note = f"{change:+.0%}"
        print(f"{name.ljust(width)}  {human(before):>10}  {human(after):>10}  {note}")

    if regressed:
        print(f"\n{len(regressed)} benchmark(s) slower by more than "
              f"{NOISE_THRESHOLD:.0%}:")
        for name in regressed:
            print(f"  {name}")
        print("\nRe-run before believing it: a single noisy run is not a regression.")
        return 1

    print(f"\nNothing slower by more than {NOISE_THRESHOLD:.0%}.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["capture", "compare", "promote"])
    parser.add_argument("--save", action="store_true",
                        help="capture: also write the run to history/")
    args = parser.parse_args()

    if args.command == "capture":
        run_data = capture()
        LATEST.write_text(json.dumps(run_data, indent=2) + "\n")
        print(f"Captured {len(run_data['measurements'])} benchmarks "
              f"at {run_data['commit']} → {LATEST.relative_to(REPO)}")

        if args.save:
            HISTORY.mkdir(exist_ok=True)
            stamp = run_data["captured"][:10]
            path = HISTORY / f"{stamp}-{run_data['commit']}.json"
            path.write_text(json.dumps(run_data, indent=2) + "\n")
            print(f"Saved to {path.relative_to(REPO)}")
        return 0

    if args.command == "compare":
        if not LATEST.is_file():
            sys.exit("No latest.json — run 'collect.py capture' first.")
        if not BASELINE.is_file():
            print("No baseline yet. Run 'collect.py promote' to set one.")
            return 0
        return compare(json.loads(LATEST.read_text()), json.loads(BASELINE.read_text()))

    # promote
    if not LATEST.is_file():
        sys.exit("No latest.json — run 'collect.py capture' first.")
    latest = json.loads(LATEST.read_text())
    if latest["dirty"]:
        print("warning: captured from a dirty working tree; the baseline will "
              "not correspond to any commit")
    BASELINE.write_text(json.dumps(latest, indent=2) + "\n")
    print(f"Baseline is now {latest['commit']} ({len(latest['measurements'])} benchmarks)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
