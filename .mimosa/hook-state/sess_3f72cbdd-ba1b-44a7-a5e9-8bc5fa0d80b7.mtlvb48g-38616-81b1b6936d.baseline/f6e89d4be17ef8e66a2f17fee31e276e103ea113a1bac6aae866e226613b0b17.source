#!/usr/bin/env bash
# Continuous Fuzz Suite Runner for Neo DevPack for Solidity
# Runs the full fuzz test suite with configurable case counts.
#
# Usage:
#   ./scripts/run_fuzz_suite.sh          # quick run (default cases)
#   ./scripts/run_fuzz_suite.sh deep     # deep run (100 cases each)
#   ./scripts/run_fuzz_suite.sh ci       # CI run (all workspace tests)

set -euo pipefail

cd "$(dirname "$0")/.."

MODE="${1:-quick}"
export RUST_BACKTRACE=1

PROPTEST_CASES=""
case "$MODE" in
  quick)
    PROPTEST_CASES=""
    echo "🧪 Running fuzz suite (default case count)..."
    ;;
  deep)
    PROPTEST_CASES="100"
    echo "🧪 Running fuzz suite with PROPTEST_CASES=$PROPTEST_CASES ..."
    ;;
  ci)
    echo "🧪 Running CI test suite (workspace, all features)..."
    cargo test --workspace --all-features 2>&1 | tee /tmp/fuzz-ci.log
    exit "${PIPESTATUS[0]}"
    ;;
  gate)
    echo "🧪 Running fuzz/compiler verification gate..."
    cargo test --test fuzz_tests 2>&1 | tee /tmp/fuzz-gate.log
    exit "${PIPESTATUS[0]}"
    ;;
  *)
    echo "Unknown mode: $MODE"
    echo "Usage: $0 [quick|deep|ci|gate]"
    exit 1
    ;;
esac

if [ -n "$PROPTEST_CASES" ]; then
  export PROPTEST_CASES
fi

LOGFILE="/tmp/fuzz-run-$(date +%Y%m%d-%H%M%S).log"
echo "Logging to $LOGFILE"

cargo test --test fuzz_tests 2>&1 | tee "$LOGFILE"
EXIT_CODE="${PIPESTATUS[0]}"

if [ "$EXIT_CODE" -eq 0 ]; then
  echo "✅ All fuzz tests passed."
  PASSED=$(grep -c "^test .* ok$" "$LOGFILE" || true)
  echo "   Tests passed: $PASSED"
else
  echo "❌ Fuzz suite failed with exit code $EXIT_CODE"
  FAILED=$(grep -c "^test .* FAILED$" "$LOGFILE" || true)
  echo "   Tests failed: $FAILED"
fi

exit "$EXIT_CODE"
