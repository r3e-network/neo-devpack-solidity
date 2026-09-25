#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

GENERATIONS="${1:-100}"
CLIPPY_EVERY="${CLIPPY_EVERY:-5}"
DEVPACK_EVERY="${DEVPACK_EVERY:-10}"
LOG_DIR="${LOG_DIR:-build/audit}"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
SUMMARY_FILE="${LOG_DIR}/extreme-iteration-summary-${TIMESTAMP}.log"

if ! [[ "$GENERATIONS" =~ ^[0-9]+$ ]] || [ "$GENERATIONS" -le 0 ]; then
  echo "GENERATIONS must be a positive integer, got: $GENERATIONS" >&2
  exit 1
fi

mkdir -p "$LOG_DIR"

run_step() {
  local generation="$1"
  local step="$2"
  local command="$3"
  local log_file="${LOG_DIR}/gen-${generation}-${step}.log"
  local start_ts end_ts duration
  start_ts="$(date +%s)"

  if sh -c "$command" >"$log_file" 2>&1; then
    end_ts="$(date +%s)"
    duration="$((end_ts - start_ts))"
    echo "generation=${generation} step=${step} status=pass duration=${duration}s log=${log_file}" | tee -a "$SUMMARY_FILE"
  else
    end_ts="$(date +%s)"
    duration="$((end_ts - start_ts))"
    echo "generation=${generation} step=${step} status=fail duration=${duration}s log=${log_file}" | tee -a "$SUMMARY_FILE"
    echo "---- last 120 lines (${log_file}) ----" >&2
    tail -n 120 "$log_file" >&2 || true
    exit 1
  fi
}

echo "started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee "$SUMMARY_FILE"
echo "generations=${GENERATIONS} clippy_every=${CLIPPY_EVERY} devpack_every=${DEVPACK_EVERY}" | tee -a "$SUMMARY_FILE"

# Preflight hard gates.
run_step 0 "fmt" "cargo fmt --all -- --check"
run_step 0 "clippy" "cargo clippy --all-targets -- -D warnings"
run_step 0 "tests" "cargo test --all-targets -- --test-threads=1"
run_step 0 "plugin-build" "cd tooling/packages/hardhat-solc-neo && npm run build"
run_step 0 "deployer-build" "cd tooling/packages/hardhat-neo-deployer && npm run build"
run_step 0 "devpack" "cd devpack && npm run test:integration"

for generation in $(seq 1 "$GENERATIONS"); do
  run_step "$generation" "tests" "cargo test --all-targets --quiet -- --test-threads=1"

  if [ $((generation % CLIPPY_EVERY)) -eq 0 ]; then
    run_step "$generation" "clippy" "cargo clippy --all-targets -- -D warnings"
  fi

  if [ $((generation % DEVPACK_EVERY)) -eq 0 ]; then
    run_step "$generation" "devpack" "cd devpack && npm run test:integration"
  fi
done

echo "completed_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$SUMMARY_FILE"
echo "summary_file=${SUMMARY_FILE}"
