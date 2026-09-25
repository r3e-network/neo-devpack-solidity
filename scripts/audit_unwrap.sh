#!/usr/bin/env bash
# Audit production Rust panic-prone calls. Test paths and #[cfg(test)] modules
# are excluded by the Python scanner. Production expect() calls must be either
# documented with an adjacent invariant comment or explicitly allowlisted.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

exec python3 scripts/audit_rust_quality.py --strict-expect "$@"
