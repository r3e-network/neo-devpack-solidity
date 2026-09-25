#!/usr/bin/env bash
# Audit production Rust source files for the configured line limit.
# Test-only paths are reported but never gate the build. Existing production
# exceptions must be listed in scripts/file-length-allowlist.txt so the
# baseline remains explicit and versioned.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LIMIT="${FILE_LENGTH_LIMIT:-800}"
ALLOWLIST_FILE="${FILE_LENGTH_ALLOWLIST_FILE:-scripts/file-length-allowlist.txt}"
OFFENDERS=0
TEST_OFFENDERS=0
ALLOWLISTED=0

is_test_path() {
    case "$1" in
        */tests/*|*/test/*|*/benches/*|*/fuzz/*|*_test.rs|*/tests.rs|*/test.rs)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

is_allowlisted() {
    local file="$1"
    [ -f "$ALLOWLIST_FILE" ] || return 1
    while IFS= read -r pattern || [ -n "$pattern" ]; do
        case "$pattern" in
            ""|\#*) continue ;;
        esac
        case "$file" in
            $pattern) return 0 ;;
        esac
    done < "$ALLOWLIST_FILE"
    return 1
}

echo "== Production Rust file length audit (limit: $LIMIT lines) =="
echo "Allowlist: $ALLOWLIST_FILE"

while IFS= read -r file; do
    lines="$(wc -l < "$file")"
    [ "$lines" -gt "$LIMIT" ] || continue
    if is_test_path "$file"; then
        echo "  TEST-ONLY $lines  $file (reported, not a gate)"
        TEST_OFFENDERS=$((TEST_OFFENDERS + 1))
    elif is_allowlisted "$file"; then
        echo "  ALLOWLISTED $lines  $file"
        ALLOWLISTED=$((ALLOWLISTED + 1))
    else
        echo "  PRODUCTION $lines  $file"
        OFFENDERS=$((OFFENDERS + 1))
    fi
done < <(find src -type f -name '*.rs' | sort)

echo "Production files over limit: $OFFENDERS"
echo "Allowlisted production files over limit: $ALLOWLISTED"
echo "Test-only files over limit (non-blocking): $TEST_OFFENDERS"
if [ "$OFFENDERS" -eq 0 ]; then
    echo "Production file length audit passed."
else
    echo "Production file length audit failed. Split files or add a reviewed entry to $ALLOWLIST_FILE."
fi
exit "$OFFENDERS"
