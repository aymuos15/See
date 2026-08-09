#!/usr/bin/env bash
# Run the benchmarks under conditions that make two runs comparable, then
# summarise the results against the recorded baseline.
#
#   benchmarks/run.sh                 all benchmarks, compare with baseline
#   benchmarks/run.sh render          one bench target
#   benchmarks/run.sh --quick         fewer samples: rough, for a fast loop
#   benchmarks/run.sh --save          also record the run in history/
#
# Anything after `--` is passed through to criterion.

set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

save=0
quick=0
targets=()
passthrough=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --save) save=1 ;;
        --quick) quick=1 ;;
        --) shift; passthrough+=("$@"); break ;;
        -*) passthrough+=("$1") ;;
        *) targets+=("$1") ;;
    esac
    shift
done

# Frequency scaling is the largest source of run-to-run drift on a laptop.
governor_path=/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
if [[ -r $governor_path ]]; then
    governor=$(cat "$governor_path")
    if [[ $governor != performance ]]; then
        echo "note: CPU governor is '$governor', not 'performance'."
        echo "      Timings will drift as the clock scales. To pin it:"
        echo "        sudo cpupower frequency-set -g performance"
        echo
    fi
fi

# Pinning to fixed cores stops the scheduler migrating the benchmark between
# cores with different cache and turbo state mid-run.
runner=()
if command -v taskset >/dev/null 2>&1; then
    runner=(taskset -c 0,1)
else
    echo "note: taskset not found; the benchmark will move between cores."
    echo
fi

# One list of --bench flags, shared by the build and the measured run so the
# two can never select different targets.
target_flags=()
for target in ${targets[@]+"${targets[@]}"}; do
    target_flags+=(--bench "$target")
done

args=(bench ${target_flags[@]+"${target_flags[@]}"} --)
[[ $quick -eq 1 ]] && args+=(--quick)
[[ ${#passthrough[@]} -gt 0 ]] && args+=("${passthrough[@]}")

# Build first, unpinned and unmeasured: compiling on two cores would be slow
# for no benefit, and the build is not part of what is being measured.
echo "Building benchmarks..."
cargo bench --no-run ${target_flags[@]+"${target_flags[@]}"}

echo
echo "Running: ${runner[*]-} cargo ${args[*]}"
echo
${runner[@]+"${runner[@]}"} cargo "${args[@]}"

# Recorded with the results: a quick run's numbers cannot be compared with a
# full run's, and the comparison needs to know which it is looking at.
if [[ $quick -eq 1 ]]; then
    export BENCH_MODE=quick
else
    export BENCH_MODE=full
fi

echo
capture_flags=()
[[ $save -eq 1 ]] && capture_flags+=(--save)
python3 benchmarks/collect.py capture ${capture_flags[@]+"${capture_flags[@]}"}

echo
python3 benchmarks/collect.py compare
