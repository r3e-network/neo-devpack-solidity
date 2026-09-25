#!/usr/bin/env bash
# End-to-end contract update smoke test using bundled Neo-Express (neoxp).
#
# Verifies that neo-solc can:
# - compile two versions of a contract (v1 + v2) to valid .nef + .manifest.json
# - deploy v1 with parameterised constructor args
# - update the deployed contract to v2 via Neo-Express
# - ensure `_deploy(data, update)` behaves correctly on update (constructors/initializers are skipped)
#
# Note: neo-devpack-solidity injects `_deploy(data, update)` and expects `data` to be a JSON-encoded array
# (e.g. `[7]`) for parameterised constructors when deploying via Neo-Express.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-update.XXXXXX")"

cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

resolve_neo_solc() {
  if [ -n "${NEO_SOLC:-}" ]; then
    echo "$NEO_SOLC"
    return
  fi

  if command -v neo-solc >/dev/null 2>&1; then
    echo "neo-solc"
    return
  fi

  echo "(info) Building neo-solc..." >&2
  (cd "$ROOT_DIR" && cargo build --bin neo-solc >/dev/null)
  echo "$ROOT_DIR/target/debug/neo-solc"
}

resolve_neoxp() {
  if [ -n "${NEOXP:-}" ]; then
    echo "$NEOXP"
    return
  fi

  if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then
    echo "$ROOT_DIR/build/dotnet-tools/neoxp"
    return
  fi

  if command -v neoxp >/dev/null 2>&1; then
    echo "neoxp"
    return
  fi

  echo "error: neoxp not found (expected $ROOT_DIR/build/dotnet-tools/neoxp or on PATH)" >&2
  exit 1
}

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for this smoke test" >&2
  exit 1
fi

NEO_SOLC_BIN="$(resolve_neo_solc)"
NEOXP_BIN="$(resolve_neoxp)"
NEOXP_HOME="$WORK_DIR/neoxp-home"
mkdir -p "$NEOXP_HOME"

run_neoxp() {
  HOME="$NEOXP_HOME" "$NEOXP_BIN" "$@"
}

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"
echo "(info) Neo-Express: $NEOXP_BIN"

cd "$WORK_DIR"

cat > UpdateSmokeV1.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract UpdateSmoke {
    uint256 private value;

    constructor(uint256 initialValue) {
        value = initialValue;
    }

    // Neo-Express `contract update` expects a public `update(nef, manifest)` entrypoint.
    // It is the contract's responsibility to restrict this method in production deployments.
    // Neo-Express passes update data as the 3rd argument (even if it is `null`).
    function update(bytes calldata nef, string calldata manifest) public {
        NativeCalls.updateContract(nef, manifest, "");
    }

    function update(bytes calldata nef, string calldata manifest, bytes calldata data) public {
        NativeCalls.updateContract(nef, manifest, data);
    }

    function set(uint256 v) public {
        value = v;
    }

    function get() public view returns (uint256) {
        return value;
    }
}
SOL

cat > UpdateSmokeV2.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract UpdateSmoke {
    uint256 private value;

    // If constructors are incorrectly executed on update, this will overwrite stored state.
    constructor(uint256 /* initialValue */) {
        value = 111;
    }

    function set(uint256 v) public {
        value = v;
    }

    function get() public view returns (uint256) {
        return value;
    }

    function version() public pure returns (uint256) {
        return 2;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/UpdateSmokeV1.sol" -o UpdateSmokeV1 >/dev/null
"$NEO_SOLC_BIN" "$WORK_DIR/UpdateSmokeV2.sol" -o UpdateSmokeV2 >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null

# Ensure deployer has enough GAS for deploy + invoke + update.
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

DEPLOY_OUT="$(run_neoxp contract deploy -i "$CHAIN" -d '[7]' UpdateSmokeV1.nef node1 -j)"
CONTRACT_HASH="$(echo "$DEPLOY_OUT" | jq -r '.["contract-hash"]' | tr -d '\r')"
DEPLOY_TX="$(echo "$DEPLOY_OUT" | jq -r '.["tx-hash"]' | tr -d '\r')"

if [ -z "$CONTRACT_HASH" ] || [ "$CONTRACT_HASH" = "null" ]; then
  echo "error: failed to read deployed contract hash"
  echo "$DEPLOY_OUT"
  exit 1
fi
if [ -z "$DEPLOY_TX" ] || [ "$DEPLOY_TX" = "null" ]; then
  echo "error: failed to read deploy tx hash"
  echo "$DEPLOY_OUT"
  exit 1
fi

DEPLOY_LOG="$(run_neoxp show transaction -i "$CHAIN" "$DEPLOY_TX")"
DEPLOY_VMSTATE="$(echo "$DEPLOY_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$DEPLOY_VMSTATE" != "HALT" ]; then
  echo "error: deploy vmstate=$DEPLOY_VMSTATE"
  echo "$DEPLOY_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

# Mutate contract state to ensure update does not re-run constructors/initializers.
cat > invoke-set.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "set",
  "args": [99]
}
JSON

SET_TX="$(run_neoxp contract invoke -i "$CHAIN" invoke-set.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$SET_TX" ]; then
  echo "error: failed to capture set(99) tx hash"
  exit 1
fi

SET_LOG="$(run_neoxp show transaction -i "$CHAIN" "$SET_TX")"
SET_VMSTATE="$(echo "$SET_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$SET_VMSTATE" != "HALT" ]; then
  echo "error: set(99) vmstate=$SET_VMSTATE"
  echo "$SET_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

# Update contract to v2 and ensure the update transaction HALTs.
UPDATE_OUT="$(run_neoxp contract update -i "$CHAIN" -d '""' "$CONTRACT_HASH" UpdateSmokeV2.nef node1 -j)"
UPDATE_TX="$(echo "$UPDATE_OUT" | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1 | tr -d '\r')"
if [ -z "$UPDATE_TX" ]; then
  echo "error: failed to capture update tx hash"
  echo "$UPDATE_OUT"
  exit 1
fi

UPDATE_LOG="$(run_neoxp show transaction -i "$CHAIN" "$UPDATE_TX")"
UPDATE_VMSTATE="$(echo "$UPDATE_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$UPDATE_VMSTATE" != "HALT" ]; then
  echo "error: update vmstate=$UPDATE_VMSTATE"
  echo "$UPDATE_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

# Ensure state was not overwritten by v2 constructor.
cat > invoke-get.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "get",
  "args": []
}
JSON

GET_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-get.neo-invoke.json node1)"
if [ "$(echo "$GET_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: get() did not HALT after update"
  echo "$GET_OUT"
  exit 1
fi
if [ "$(echo "$GET_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: get() returned non-Integer"
  echo "$GET_OUT"
  exit 1
fi
if [ "$(echo "$GET_OUT" | jq -r '.stack[0].value')" != "99" ]; then
  echo "error: state mismatch after update (got $(echo "$GET_OUT" | jq -r '.stack[0].value'), want 99)"
  echo "$GET_OUT"
  exit 1
fi

# Ensure v2 code is active by calling a method that exists only in v2.
cat > invoke-version.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "version",
  "args": []
}
JSON

VER_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-version.neo-invoke.json node1)"
if [ "$(echo "$VER_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: version() did not HALT (update may not have applied)"
  echo "$VER_OUT"
  exit 1
fi
if [ "$(echo "$VER_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: version() returned non-Integer"
  echo "$VER_OUT"
  exit 1
fi
if [ "$(echo "$VER_OUT" | jq -r '.stack[0].value')" != "2" ]; then
  echo "error: version() returned unexpected value"
  echo "$VER_OUT"
  exit 1
fi

echo "✅ neoxp contract update smoke test passed"
