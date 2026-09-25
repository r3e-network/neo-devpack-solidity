#!/usr/bin/env bash
# Audit Rust source files in src/ for length. Prints a summary and exits with
# the number of files exceeding the threshold. When run in CI with
# `continue-on-error: true` this is informational; remove that flag to make it
# blocking.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LIMIT="${FILE_LENGTH_LIMIT:-800}"
OFFENDERS=0

echo "== File length audit (limit: $LIMIT lines) =="

while IFS= read -r file; do
    lines="$(wc -l < "$file")"
    if [ "$lines" -gt "$LIMIT" ]; then
        echo "  $lines  $file"
        OFFENDERS=$((OFFENDERS + 1))
    fi
done < <(find src/ -type f -name '*.rs' | sort)

if [ "$OFFENDERS" -eq 0 ]; then
    echo "No source files exceed $LIMIT lines."
else
    echo "Files exceeding $LIMIT lines: $OFFENDERS"
fi

exit "$OFFENDERS"
