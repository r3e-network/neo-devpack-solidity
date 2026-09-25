#!/usr/bin/env bash
# End-to-end constructor-args deploy smoke test using bundled Neo-Express (neoxp).
#
# Verifies that neo-solc can:
# - compile a contract with a parameterised Solidity constructor
# - deploy it via Neo-Express while passing constructor args through `_deploy(data, update)`
# - observe the expected initialized state on-chain
#
# Note: neo-devpack-solidity injects `_deploy(data, update)` and expects `data` to be a JSON-encoded
# array (e.g. `[7]`) when the Solidity constructor requires arguments.
#
# Constructor-side event emission is intentionally excluded here so this
# script validates `_deploy(data, update)` independently from the still-open
# on-chain `abiEncode` / `abiDecode` gap.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-ctor.XXXXXX")"

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

cat > ConstructorSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract ConstructorSmoke {
    uint256 private value;

    constructor(uint256 initialValue) {
        value = initialValue;
    }

    function get() public view returns (uint256) {
        return value;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/ConstructorSmoke.sol" -o ConstructorSmoke >/dev/null

# Ensure the manifest declares permissions for StdLib.jsonDeserialize + StdLib.deserialize,
# required by the injected deploy prologue for parameterised constructors.
if ! jq -e 'any(.permissions[]; ((.methods | type) == "array") and any(.methods[]; . == "jsonDeserialize") and any(.methods[]; . == "deserialize"))' ConstructorSmoke.manifest.json >/dev/null; then
  echo "error: expected StdLib.jsonDeserialize + StdLib.deserialize permission in manifest"
  jq '.permissions' ConstructorSmoke.manifest.json
  exit 1
fi

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null

# Ensure deployer has enough GAS for deploy + invoke.
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

DEPLOY_OUT="$(run_neoxp contract deploy -i "$CHAIN" -d '[7]' ConstructorSmoke.nef node1 -j)"
CONTRACT_HASH="$(echo "$DEPLOY_OUT" | jq -r '.["contract-hash"]' | tr -d '\r')"
TX_HASH="$(echo "$DEPLOY_OUT" | jq -r '.["tx-hash"]' | tr -d '\r')"

if [ -z "$CONTRACT_HASH" ] || [ "$CONTRACT_HASH" = "null" ]; then
  echo "error: failed to read deployed contract hash"
  echo "$DEPLOY_OUT"
  exit 1
fi
if [ -z "$TX_HASH" ] || [ "$TX_HASH" = "null" ]; then
  echo "error: failed to read deploy tx hash"
  echo "$DEPLOY_OUT"
  exit 1
fi

APP_LOG="$(run_neoxp show transaction -i "$CHAIN" "$TX_HASH")"
VMSTATE="$(echo "$APP_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$VMSTATE" != "HALT" ]; then
  echo "error: deploy vmstate=$VMSTATE"
  echo "$APP_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

cat > invoke-get.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "get",
  "args": []
}
JSON

GET_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-get.neo-invoke.json node1)"
if [ "$(echo "$GET_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: get() did not HALT"
  echo "$GET_OUT"
  exit 1
fi
if [ "$(echo "$GET_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: get() returned non-Integer"
  echo "$GET_OUT"
  exit 1
fi
if [ "$(echo "$GET_OUT" | jq -r '.stack[0].value')" != "7" ]; then
  echo "error: get() returned unexpected value"
  echo "$GET_OUT"
  exit 1
fi

echo "✅ neoxp constructor deploy smoke test passed"
