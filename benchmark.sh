#!/usr/bin/env bash
# bench_splitby_vs_cut.sh  — portable cut vs splitby micro-benchmark

set -euo pipefail

LINES=${1:-10000}     # records to generate
FIELDS=${2:-20}       # columns per record
ITER=${3:-3}          # repetition count

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

bench "splitby   (3 5 7)" \
      splitby -i "$TMP_DATA" -d ',' 3 5 7 
