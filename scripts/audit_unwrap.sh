#!/usr/bin/env bash
# Audit .unwrap() usage in src/. Prints a summary and exits with the number
# of .unwrap() calls found. When run in CI with `continue-on-error: true` this
# is informational; remove that flag to make it blocking. A separate file
# (build/audit/unwrap-baseline.txt) can be used to track reduction progress.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BASELINE_FILE="${UNWRAP_BASELINE_FILE:-scripts/unwrap-baseline.txt}"
REPORT_DIR="build/audit"
mkdir -p "$REPORT_DIR"

echo "== unwrap() audit =="

TOTAL="$(grep -r '\.unwrap()' src/ --include='*.rs' | wc -l | tr -d ' ')"
echo "Total .unwrap() calls in src/: $TOTAL"

echo "Top files by .unwrap() count:"
grep -r '\.unwrap()' src/ --include='*.rs' | cut -d: -f1 | sort | uniq -c | sort -rn | head -20

if [ -f "$BASELINE_FILE" ]; then
    BASELINE="$(tr -d ' ' < "$BASELINE_FILE")"
    if [ "$TOTAL" -lt "$BASELINE" ]; then
        DELTA=$((BASELINE - TOTAL))
        echo "Progress: $DELTA fewer .unwrap() calls than baseline ($BASELINE)."
    elif [ "$TOTAL" -gt "$BASELINE" ]; then
        DELTA=$((TOTAL - BASELINE))
        echo "Regression: $DELTA more .unwrap() calls than baseline ($BASELINE)."
    else
        echo "No change from baseline ($BASELINE)."
    fi
else
    echo "No baseline found at $BASELINE_FILE; creating it."
    echo "$TOTAL" > "$BASELINE_FILE"
fi

exit "$TOTAL"
