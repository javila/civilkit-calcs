#!/bin/sh
# Runs the gated xunit classes (tools/xunit_gated.txt) in release mode and prints
# passed/total per class. Usage: tools/run_gated.sh [filter] [seconds-per-class]
cd "$(dirname "$0")/.."
FILTER=${1:-}
LIMIT=${2:-300}
cargo test -q --release --features xunit_vm --no-run >/dev/null 2>&1 || { echo "build failed"; exit 1; }
total_p=0; total_n=0
for cls in $(grep -v '^#' tools/xunit_gated.txt); do
  case "$cls" in *"$FILTER"*) ;; *) continue ;; esac
  out=$(perl -e 'alarm shift; exec @ARGV' "$LIMIT" cargo test -q --release --features xunit_vm --test "xunit_$cls" 2>&1)
  line=$(printf '%s\n' "$out" | grep "test result" | tail -1)
  if [ -z "$line" ]; then
    echo "$cls TIMEOUT-OR-CRASH"
    continue
  fi
  p=$(echo "$line" | awk '{print $4}'); f=$(echo "$line" | awk '{print $6}')
  n=$((p + f)); total_p=$((total_p + p)); total_n=$((total_n + n))
  echo "$cls $p/$n"
done
echo "TOTAL $total_p/$total_n"
