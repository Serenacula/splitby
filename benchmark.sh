#!/usr/bin/env bash
# bench_splitby_vs_cut.sh  — portable cut vs splitby micro-benchmark
#
# Usage:
#   ./benchmark.sh [VERSION] [LINES] [FIELDS] [ITERATIONS]
#   ./benchmark.sh 10000 20 3              # Old API: tests rust version (default)
#   ./benchmark.sh rust 10000 20 3         # New API: test rust version
#   ./benchmark.sh bash 10000 20 3         # New API: test bash version
#   ./benchmark.sh both 10000 20 3         # New API: test both versions
#
# Defaults: VERSION=rust, LINES=10000, FIELDS=20, ITERATIONS=3

set -euo pipefail

# Determine if first argument is a version string or a number (for backward compatibility)
if [[ "${1:-}" =~ ^[0-9]+$ ]] || [[ -z "${1:-}" ]]; then
    # Old API: first arg is LINES (or empty), default to rust
    VERSION="rust"
    LINES=${1:-10000}
    FIELDS=${2:-20}
    ITER=${3:-3}
else
    # New API: first arg is version
    VERSION="$1"
    LINES=${2:-10000}
    FIELDS=${3:-20}
    ITER=${4:-3}
fi

# Determine which splitby command to use
if [[ "$VERSION" == "rust" ]]; then
    SPLITBY_CMD="./target/release/splitby"
    if [[ ! -f "$SPLITBY_CMD" ]]; then
        echo "Error: Rust binary not found at $SPLITBY_CMD"
        echo "Please build it first with: cargo build --release"
        exit 1
    fi
elif [[ "$VERSION" == "bash" ]]; then
    SPLITBY_CMD="./splitby.sh"
    if [[ ! -f "$SPLITBY_CMD" ]]; then
        echo "Error: Bash script not found at $SPLITBY_CMD"
        exit 1
    fi
elif [[ "$VERSION" == "both" ]]; then
    # Will test both versions
    RUST_CMD="./target/release/splitby"
    BASH_CMD="./splitby.sh"
    if [[ ! -f "$RUST_CMD" ]]; then
        echo "Warning: Rust binary not found at $RUST_CMD, skipping Rust benchmarks"
        RUST_CMD=""
    fi
    if [[ ! -f "$BASH_CMD" ]]; then
        echo "Warning: Bash script not found at $BASH_CMD, skipping bash benchmarks"
        BASH_CMD=""
    fi
else
    echo "Error: Invalid version '$VERSION'. Use 'rust', 'bash', or 'both'"
    exit 1
fi

# ---------------------------------------------------------------------
# Pick a timer: GNU/BSD 'time', 'gtime' (Homebrew), or bash built-in.
# ---------------------------------------------------------------------
TIME_CMD=
TIME_OPT=
if command -v /usr/bin/time >/dev/null 2>&1; then
  TIME_CMD=/usr/bin/time
  TIME_OPT='-f %e'
elif command -v gtime >/dev/null 2>&1; then          # macOS coreutils
  TIME_CMD=$(command -v gtime)
  TIME_OPT='-f %e'
else
  # Use bash built-in 'time -p' (prints 3 lines: real user sys)
  TIME_CMD=time
  TIME_OPT='-p'
  TIMEFORMAT='%3R'      # Bash variable: print just "real" in seconds
fi

printf "Using timer: %s\n\n" "$TIME_CMD"

# ---------------------------------------------------------------------
printf "Generating %'d lines × %d fields … " "$LINES" "$FIELDS"
TMP_DATA=$(mktemp)
trap 'rm -f "$TMP_DATA"' EXIT

awk -v lines="$LINES" -v fields="$FIELDS" '
BEGIN {
  srand(42);
  for (i=1; i<=lines; i++) {
    for (f=1; f<=fields; f++) {
      printf "%d%s", rand()*1000, (f==fields ? ORS : ",");
    }
  }
}' > "$TMP_DATA"
echo "done."
echo

bench() {
  local label=$1; shift
  local -a cmd=("$@")
  local total=0

  echo "$label:"
  for ((run=1; run<=ITER; run++)); do
    # Hi-res timestamp before
    local start=$(perl -MTime::HiRes=time -e 'print time')
    "${cmd[@]}" >/dev/null
    # Hi-res timestamp after
    local stop=$(perl -MTime::HiRes=time -e 'print time')
    # elapsed seconds with 3-decimals
    local t=$(perl -e "printf '%.3f', $stop - $start")
    printf "  run %d: %s s\n" "$run" "$t"
    total=$(perl -e "print $total + $t")
  done
  printf "  avg : %.3f s\n\n" "$(perl -e "printf '%.3f', $total / $ITER")"
}




# Columns 3,5,7         (cut syntax needs commas)
bench "cut       (3,5,7)" \
      cut -d',' -f3,5,7 "$TMP_DATA"

# Benchmark splitby based on selected version
if [[ "$VERSION" == "both" ]]; then
    if [[ -n "$RUST_CMD" ]]; then
        bench "splitby-rust (3 5 7)" \
              "$RUST_CMD" -i "$TMP_DATA" -d ',' 3 5 7
    fi
    if [[ -n "$BASH_CMD" ]]; then
        bench "splitby-bash (3 5 7)" \
              "$BASH_CMD" -i "$TMP_DATA" -d ',' 3 5 7
    fi
else
    bench "splitby   (3 5 7)" \
          "$SPLITBY_CMD" -i "$TMP_DATA" -d ',' 3 5 7
fi
