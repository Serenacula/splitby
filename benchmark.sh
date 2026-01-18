#!/usr/bin/env bash
# bench_splitby_vs_cut.sh  — portable cut vs splitby micro-benchmark
#
# Usage:
#   ./benchmark.sh [LINES] [FIELDS] [ITERATIONS] [SINGLE_CORE]
#   ./benchmark.sh 10000 20 3
#   ./benchmark.sh 10000 20 3 true         # single-core mode (fair comparison with cut)
#
# Defaults: LINE_COUNT=10000, FIELD_COUNT=20, ITERATIONS=3, SINGLE_CORE=false

set -euo pipefail

LINE_COUNT=${1:-10000}
FIELD_COUNT=${2:-20}
ITER=${3:-3}
SINGLE_CORE=${4:-false}

# Enable single-core mode via environment variable (Rust binary will respect it)
if [[ "$SINGLE_CORE" == "true" ]]; then
    echo "Single-core mode enabled (using SPLITBY_SINGLE_CORE=1)"
fi

# Build Rust version
echo "Building Rust release version..."
echo "----------------------------------------"
cargo build --release
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to build Rust binary"
    exit 1
fi
echo "Build complete."
echo

SPLITBY_CMD="./target/release/splitby"
if [[ ! -f "$SPLITBY_CMD" ]]; then
    echo "Error: Rust binary not found at $SPLITBY_CMD"
    echo "Please build it first with: cargo build --release"
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
printf "Generating %'d lines × %d fields … " "$LINE_COUNT" "$FIELD_COUNT"
TMP_DATA=$(mktemp)
trap 'rm -f "$TMP_DATA"' EXIT

awk -v line_count="$LINE_COUNT" -v field_count="$FIELD_COUNT" '
BEGIN {
  srand(42);
  for (i=1; i<=line_count; i++) {
    for (field_index=1; field_index<=field_count; field_index++) {
      printf "%d%s", rand()*1000, (field_index==field_count ? ORS : ",");
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

  # Warmup run
  local start=$(perl -MTime::HiRes=time -e 'print time')
  "${cmd[@]}" >/dev/null
  local stop=$(perl -MTime::HiRes=time -e 'print time')
  local t=$(perl -e "printf '%.3f', $stop - $start")
  printf "  warmup run: %s s\n" "$t"

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

if [[ "$SINGLE_CORE" == "true" ]]; then
    bench "splitby   (3 5 7) [single-core]" \
          env SPLITBY_SINGLE_CORE=1 "$SPLITBY_CMD" -i "$TMP_DATA" -d ',' 3 5 7
else
    bench "splitby   (3 5 7)" \
          "$SPLITBY_CMD" -i "$TMP_DATA" -d ',' 3 5 7
fi
