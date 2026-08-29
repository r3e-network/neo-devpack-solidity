#!/usr/bin/env bash
# End-to-end smoke test for examples/new/UpgradeLifecycleShowcase.sol using Neo-Express.
#
# Verifies:
# - strict-manifest compilation succeeds
# - deploy with constructor args succeeds
# - valid owner transfer succeeds (HALT)
# - former owner can no longer transfer ownership (FAULT)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-upgrade-lifecycle.XXXXXX")"

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

invoke_tx_hash() {
  local invoke_file="$1"
  local signer="$2"
  local out tx_hash

  out="$(run_neoxp contract invoke -i "$CHAIN" "$invoke_file" "$signer")"
  tx_hash="$(echo "$out" | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"

  if [ -z "$tx_hash" ]; then
    echo "error: failed to capture tx hash for $invoke_file" >&2
    echo "$out" >&2
    exit 1
  fi

  echo "$tx_hash"
}

tx_vmstate() {
  local tx_hash="$1"
  run_neoxp show transaction -i "$CHAIN" "$tx_hash" | jq -r '.["application-log"].executions[0].vmstate'
}

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"
echo "(info) Neo-Express: $NEOXP_BIN"

cd "$WORK_DIR"

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

NODE1_HASH="$(run_neoxp wallet list -j -i "$CHAIN" | jq -r '.node1[0]["script-hash"]' | tr -d '\r' | sed 's/^0x//')"
if [ -z "$NODE1_HASH" ] || [ "$NODE1_HASH" = "null" ]; then
  echo "error: failed to resolve node1 script hash" >&2
  exit 1
fi

"$NEO_SOLC_BIN" "$ROOT_DIR/examples/new/UpgradeLifecycleShowcase.sol" \
  -o "$WORK_DIR/UpgradeLifecycleShowcase" \
  --deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods >/dev/null

DEPLOY_OUT="$(run_neoxp contract deploy -i "$CHAIN" -d "[5]" "$WORK_DIR/UpgradeLifecycleShowcase.nef" node1 -j)"
CONTRACT_HASH="$(echo "$DEPLOY_OUT" | jq -r '.["contract-hash"]' | tr -d '\r')"
DEPLOY_TX="$(echo "$DEPLOY_OUT" | jq -r '.["tx-hash"]' | tr -d '\r')"

if [ -z "$CONTRACT_HASH" ] || [ "$CONTRACT_HASH" = "null" ]; then
  echo "error: failed to read contract hash from deploy output" >&2
  echo "$DEPLOY_OUT" >&2
  exit 1
fi
if [ -z "$DEPLOY_TX" ] || [ "$DEPLOY_TX" = "null" ]; then
  echo "error: failed to read deploy tx hash" >&2
  echo "$DEPLOY_OUT" >&2
  exit 1
fi

if [ "$(tx_vmstate "$DEPLOY_TX")" != "HALT" ]; then
  echo "error: deploy did not HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$DEPLOY_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

cat > invoke-bootstrap-owner.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "bootstrapOwner",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}]
}
JSON

BOOTSTRAP_TX="$(invoke_tx_hash invoke-bootstrap-owner.neo-invoke.json node1)"
if [ "$(tx_vmstate "$BOOTSTRAP_TX")" != "HALT" ]; then
  echo "error: bootstrapOwner should HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$BOOTSTRAP_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

cat > invoke-version.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "version",
  "args": []
}
JSON

VERSION_OUT="$(invoke_read_json invoke-version.neo-invoke.json node1)"
if [ "$(echo "$VERSION_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: version() did not HALT" >&2
  echo "$VERSION_OUT" >&2
  exit 1
fi
if [ "$(echo "$VERSION_OUT" | jq -r '.stack[0].type')" != "Integer" ] || \
   [ "$(echo "$VERSION_OUT" | jq -r '.stack[0].value')" != "5" ]; then
  echo "error: version() returned unexpected value" >&2
  echo "$VERSION_OUT" >&2
  exit 1
fi

NEW_OWNER="1111111111111111111111111111111111111111"
cat > invoke-transfer-valid.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "transferOwnership",
  "args": [{"type": "Hash160", "value": "$NEW_OWNER"}]
}
JSON

TRANSFER_TX="$(invoke_tx_hash invoke-transfer-valid.neo-invoke.json node1)"
if [ "$(tx_vmstate "$TRANSFER_TX")" != "HALT" ]; then
  echo "error: valid transferOwnership should HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$TRANSFER_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

# Former owner should now fail owner-gated transfer.
cat > invoke-transfer-old-owner.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "transferOwnership",
  "args": [{"type": "Hash160", "value": "2222222222222222222222222222222222222222"}]
}
JSON

TRANSFER_OLD_OWNER_OUT="$(invoke_read_json invoke-transfer-old-owner.neo-invoke.json node1)"
if [ "$(echo "$TRANSFER_OLD_OWNER_OUT" | jq -r '.state')" != "FAULT" ]; then
  echo "error: old owner transfer should FAULT" >&2
  echo "$TRANSFER_OLD_OWNER_OUT" >&2
  exit 1
fi

cat > invoke-gas-balance.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "gasBalance",
  "args": []
}
JSON

BAL_OUT="$(invoke_read_json invoke-gas-balance.neo-invoke.json node1)"
if [ "$(echo "$BAL_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: gasBalance() did not HALT" >&2
  echo "$BAL_OUT" >&2
  exit 1
fi
if [ "$(echo "$BAL_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: gasBalance() returned non-Integer" >&2
  echo "$BAL_OUT" >&2
  exit 1
fi

echo "✅ neoxp UpgradeLifecycleShowcase smoke test passed"
