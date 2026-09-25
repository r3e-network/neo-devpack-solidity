#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PID_FILE="/tmp/fuzz-continuous-master.pid"
LOG_FILE="/tmp/fuzz-continuous-master.log"

"$ROOT_DIR/scripts/fuzz_stop.sh" >/dev/null 2>&1 || true

cd "$ROOT_DIR"
if command -v setsid >/dev/null 2>&1; then
  setsid bash -lc "cd '$ROOT_DIR' && exec env \
    PROPTEST_CASES='${PROPTEST_CASES:-50}' \
    CARGO_FUZZ_TIME='${CARGO_FUZZ_TIME:-300}' \
    GATE_EVERY_ROUNDS='${GATE_EVERY_ROUNDS:-3}' \
    SMOKE_EVERY_ROUNDS='${SMOKE_EVERY_ROUNDS:-0}' \
    ./scripts/run_continuous_fuzz.sh" >"$LOG_FILE" 2>&1 &
else
  nohup bash -lc "cd '$ROOT_DIR' && exec env \
    PROPTEST_CASES='${PROPTEST_CASES:-50}' \
    CARGO_FUZZ_TIME='${CARGO_FUZZ_TIME:-300}' \
    GATE_EVERY_ROUNDS='${GATE_EVERY_ROUNDS:-3}' \
    SMOKE_EVERY_ROUNDS='${SMOKE_EVERY_ROUNDS:-0}' \
    ./scripts/run_continuous_fuzz.sh" >"$LOG_FILE" 2>&1 &
fi
echo $! > "$PID_FILE"

echo "started"
echo "pid=$(cat "$PID_FILE")"
echo "log=$LOG_FILE"
