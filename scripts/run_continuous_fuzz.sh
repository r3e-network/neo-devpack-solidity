#!/usr/bin/env bash
# Continuous fuzz runner for neo-devpack-solidity.
#
# Runs the full proptest suite + every cargo-fuzz target in a loop. Each
# round logs to /tmp/fuzz-continuous/; on a failure (proptest fail or
# cargo-fuzz crash) the script exits non-zero so CI and monitoring tools
# notice immediately.
#
# Environment variables:
#   PROPTEST_CASES   (default 50)  — proptest cases per proptest `#[test]`.
#   CARGO_FUZZ_TIME  (default 300) — seconds per cargo-fuzz target per round.
#   FUZZ_TARGETS     (default: all registered)
#                                  — space-separated subset to cycle through.
#
# Usage:
#   ./scripts/run_continuous_fuzz.sh                  # all targets
#   FUZZ_TARGETS="fuzz_target_1 fuzz_target_disasm" ./scripts/run_continuous_fuzz.sh

set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p /tmp/fuzz-continuous

PROPTEST_CASES="${PROPTEST_CASES:-50}"
export PROPTEST_CASES

CARGO_FUZZ_TIME="${CARGO_FUZZ_TIME:-300}"

# Resolve fuzz targets: explicit override, else every target registered in
# fuzz/Cargo.toml.
if [ -n "${FUZZ_TARGETS:-}" ]; then
  TARGETS=($FUZZ_TARGETS)
else
  mapfile -t TARGETS < <(cargo +nightly fuzz list)
fi

echo "Starting continuous fuzz loop..."
echo "  Proptest cases per run: $PROPTEST_CASES"
echo "  Cargo-fuzz time per target: ${CARGO_FUZZ_TIME}s"
echo "  Targets (${#TARGETS[@]}): ${TARGETS[*]}"
echo "  Press Ctrl+C to stop"
echo

ROUND=0
while true; do
  ROUND=$((ROUND + 1))
  echo "========== ROUND $ROUND =========="

  # 1. Proptest suite
  echo "[$(date)] Running proptest suite..."
  if cargo test --test fuzz_tests > /tmp/fuzz-continuous/proptest-$ROUND.log 2>&1; then
    PASSED=$(grep -c "^test .* ok$" /tmp/fuzz-continuous/proptest-$ROUND.log || true)
    echo "[$(date)] ✅ Proptest passed ($PASSED tests)"
  else
    echo "[$(date)] ❌ Proptest FAILED (round $ROUND)"
    tail -80 /tmp/fuzz-continuous/proptest-$ROUND.log
    exit 1
  fi

  # 2. Cycle through every cargo-fuzz target for CARGO_FUZZ_TIME each.
  for TARGET in "${TARGETS[@]}"; do
    LOGFILE="/tmp/fuzz-continuous/${TARGET}-$ROUND.log"
    echo "[$(date)] Running cargo-fuzz ${TARGET} for ${CARGO_FUZZ_TIME}s..."

    # Per-target libfuzzer dictionary (fuzz/dict/<target>.dict). Dictionaries
    # seed libfuzzer's mutation engine with domain-specific tokens (opcode
    # mnemonics, syscall names, manifest JSON keys, Solidity keywords, ...)
    # so the engine can synthesize valid tokens on iteration 1 instead of
    # discovering them by random byte mutation.
    DICT_FILE="fuzz/dict/${TARGET}.dict"
    DICT_ARG=()
    if [ -f "$DICT_FILE" ]; then
      DICT_ARG=("-dict=${DICT_FILE}")
    fi

    # `cargo +nightly fuzz run ... -- -max_total_time=N` self-terminates;
    # wrap in `timeout` as a belt-and-suspenders guard in case the target
    # ignores -max_total_time.
    timeout "$((CARGO_FUZZ_TIME + 30))" \
      cargo +nightly fuzz run "$TARGET" -- \
        "${DICT_ARG[@]}" \
        -max_total_time="$CARGO_FUZZ_TIME" \
      > "$LOGFILE" 2>&1 || true

    # A real crash shows as `ERROR:` in libFuzzer output. An OOM / ASAN
    # error also shows as `ERROR:`. Ignore the `ERROR:` that libFuzzer
    # prints when it rediscovers a known-benign corpus input (none of our
    # targets currently hit this).
    if grep -aq "^ERROR:" "$LOGFILE"; then
      echo "[$(date)] ❌ cargo-fuzz ${TARGET} FOUND A CRASH"
      grep -a "^ERROR:" "$LOGFILE" | head -3
      echo "See: $LOGFILE"
      exit 1
    fi
    COV=$(grep -a "cov:" "$LOGFILE" | tail -1 | grep -oE "cov: [0-9]+" | awk '{print $2}')
    echo "[$(date)] ✅ ${TARGET} clean (cov:${COV:-?})"
  done

  echo "[$(date)] Round $ROUND complete. Sleeping 5s..."
  sleep 5
done
