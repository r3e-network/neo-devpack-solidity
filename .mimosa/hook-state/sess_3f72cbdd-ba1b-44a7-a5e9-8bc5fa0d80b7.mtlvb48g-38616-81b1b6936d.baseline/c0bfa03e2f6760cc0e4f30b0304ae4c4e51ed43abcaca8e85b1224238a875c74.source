#!/usr/bin/env bash
# End-to-end smoke test for examples/new/OracleRelayStrictShowcase.sol using Neo-Express.
#
# Verifies deterministic pre-response behavior:
# - strict-manifest compilation and deploy succeed
# - initial nextRequestId is 1
# - querying missing result faults with UnknownRequest path

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-oracle-relay.XXXXXX")"

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

  if command -v dotnet >/dev/null 2>&1; then
    echo "(info) Installing Neo-Express (neoxp)..." >&2
    mkdir -p "$ROOT_DIR/build/dotnet-tools"
    dotnet tool install Neo.Express --tool-path "$ROOT_DIR/build/dotnet-tools" >/dev/null
    if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then
      echo "$ROOT_DIR/build/dotnet-tools/neoxp"
      return
    fi
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

invoke_read_json() {
  local invoke_file="$1"
  local signer="$2"
  run_neoxp contract invoke -r -j -i "$CHAIN" "$invoke_file" "$signer" 2>&1 || true
}

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"
echo "(info) Neo-Express: $NEOXP_BIN"

cd "$WORK_DIR"

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

"$NEO_SOLC_BIN" "$ROOT_DIR/examples/new/OracleRelayStrictShowcase.sol" \
  -o "$WORK_DIR/OracleRelayStrictShowcase" \
  --deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods >/dev/null

DEPLOY_OUT="$(run_neoxp contract deploy -i "$CHAIN" "$WORK_DIR/OracleRelayStrictShowcase.nef" node1 -j)"
CONTRACT_HASH="$(echo "$DEPLOY_OUT" | jq -r '.["contract-hash"]' | tr -d '\r')"

if [ -z "$CONTRACT_HASH" ] || [ "$CONTRACT_HASH" = "null" ]; then
  echo "error: failed to read contract hash from deploy output" >&2
  echo "$DEPLOY_OUT" >&2
  exit 1
fi

cat > invoke-next-request-id.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "nextRequestId",
  "args": []
}
JSON

NEXT_ID_OUT="$(invoke_read_json invoke-next-request-id.neo-invoke.json node1)"
if [ "$(echo "$NEXT_ID_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: nextRequestId() did not HALT" >&2
  echo "$NEXT_ID_OUT" >&2
  exit 1
fi
if [ "$(echo "$NEXT_ID_OUT" | jq -r '.stack[0].type')" != "Integer" ] || \
   [ "$(echo "$NEXT_ID_OUT" | jq -r '.stack[0].value')" != "1" ]; then
  echo "error: nextRequestId() returned unexpected value" >&2
  echo "$NEXT_ID_OUT" >&2
  exit 1
fi

cat > invoke-get-missing-result.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "getResult",
  "args": [1]
}
JSON

MISSING_OUT="$(invoke_read_json invoke-get-missing-result.neo-invoke.json node1)"
if [ "$(echo "$MISSING_OUT" | jq -r '.state')" != "FAULT" ]; then
  echo "error: getResult(1) should FAULT before any response" >&2
  echo "$MISSING_OUT" >&2
  exit 1
fi

echo "✅ neoxp OracleRelayStrictShowcase smoke test passed"
