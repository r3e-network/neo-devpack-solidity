#!/usr/bin/env bash
# Combined Neo-Express smoke runner for strict-safe new showcase contracts.
#
# Runs:
# - UpgradeLifecycleShowcase
# - WitnessGuardShowcase
# - OracleRelayStrictShowcase

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

resolve_neo_solc_once() {
  if [ -n "${NEO_SOLC:-}" ]; then
    return
  fi

  if command -v neo-solc >/dev/null 2>&1; then
    export NEO_SOLC="neo-solc"
    return
  fi

  echo "(info) Building neo-solc once for showcase smoke suite..."
  (cd "$ROOT_DIR" && cargo build --bin neo-solc >/dev/null)
  export NEO_SOLC="$ROOT_DIR/target/debug/neo-solc"
}

resolve_neo_solc_once

echo "(info) Using compiler: $NEO_SOLC"
echo "(info) Running strict-safe new showcase smoke suite"

bash "$SCRIPT_DIR/test_neoxp_upgrade_lifecycle_smoke.sh"
bash "$SCRIPT_DIR/test_neoxp_witness_guard_smoke.sh"
bash "$SCRIPT_DIR/test_neoxp_oracle_relay_smoke.sh"

echo "✅ neoxp new showcases smoke suite passed"
