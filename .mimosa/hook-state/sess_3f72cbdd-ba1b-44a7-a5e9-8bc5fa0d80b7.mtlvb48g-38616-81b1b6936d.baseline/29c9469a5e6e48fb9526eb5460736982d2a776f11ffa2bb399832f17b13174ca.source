#!/usr/bin/env bash
# End-to-end smoke test for examples/new/WitnessGuardShowcase.sol using Neo-Express.
#
# Verifies:
# - strict-manifest compilation and deploy succeed
# - owner can set guardian (HALT)
# - guardian can lock account (HALT)
# - privileged action while locked faults in dry-run mode (FAULT)
# - owner unlocks and privileged action succeeds (HALT)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-witness-guard.XXXXXX")"

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

"$NEO_SOLC_BIN" "$ROOT_DIR/examples/new/WitnessGuardShowcase.sol" \
  -o "$WORK_DIR/WitnessGuardShowcase" \
  --deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods >/dev/null

DEPLOY_OUT="$(run_neoxp contract deploy -i "$CHAIN" "$WORK_DIR/WitnessGuardShowcase.nef" node1 -j)"
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

cat > invoke-set-guardian.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "setGuardian",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}, true]
}
JSON

SET_GUARDIAN_TX="$(invoke_tx_hash invoke-set-guardian.neo-invoke.json node1)"
if [ "$(tx_vmstate "$SET_GUARDIAN_TX")" != "HALT" ]; then
  echo "error: setGuardian should HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$SET_GUARDIAN_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

cat > invoke-lock-account.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "lockAccount",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}, {"type": "Hash160", "value": "$NODE1_HASH"}, 120]
}
JSON

LOCK_TX="$(invoke_tx_hash invoke-lock-account.neo-invoke.json node1)"
if [ "$(tx_vmstate "$LOCK_TX")" != "HALT" ]; then
  echo "error: lockAccount should HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$LOCK_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

cat > invoke-is-locked.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "isLocked",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}]
}
JSON

LOCKED_OUT="$(invoke_read_json invoke-is-locked.neo-invoke.json node1)"
if [ "$(echo "$LOCKED_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: isLocked() should HALT" >&2
  echo "$LOCKED_OUT" >&2
  exit 1
fi
if [ "$(echo "$LOCKED_OUT" | jq -r '.stack[0].type')" != "Boolean" ]; then
  echo "error: isLocked() returned non-Boolean" >&2
  echo "$LOCKED_OUT" >&2
  exit 1
fi
if [ "$(echo "$LOCKED_OUT" | jq -r '.stack[0].value' | tr '[:upper:]' '[:lower:]')" != "true" ]; then
  echo "error: expected account to be locked" >&2
  echo "$LOCKED_OUT" >&2
  exit 1
fi

cat > invoke-privileged-locked.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "privilegedAction",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}, 7, "locked"]
}
JSON

PRIV_LOCKED_OUT="$(invoke_read_json invoke-privileged-locked.neo-invoke.json node1)"
if [ "$(echo "$PRIV_LOCKED_OUT" | jq -r '.state')" != "FAULT" ]; then
  echo "error: privilegedAction while locked should FAULT" >&2
  echo "$PRIV_LOCKED_OUT" >&2
  exit 1
fi

cat > invoke-unlock-account.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "unlockAccount",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}]
}
JSON

UNLOCK_TX="$(invoke_tx_hash invoke-unlock-account.neo-invoke.json node1)"
if [ "$(tx_vmstate "$UNLOCK_TX")" != "HALT" ]; then
  echo "error: unlockAccount should HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$UNLOCK_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

UNLOCKED_OUT="$(invoke_read_json invoke-is-locked.neo-invoke.json node1)"
if [ "$(echo "$UNLOCKED_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: isLocked() after unlock should HALT" >&2
  echo "$UNLOCKED_OUT" >&2
  exit 1
fi
if [ "$(echo "$UNLOCKED_OUT" | jq -r '.stack[0].value' | tr '[:upper:]' '[:lower:]')" != "false" ]; then
  echo "error: expected account to be unlocked" >&2
  echo "$UNLOCKED_OUT" >&2
  exit 1
fi

cat > invoke-privileged-unlocked.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "privilegedAction",
  "args": [{"type": "Hash160", "value": "$NODE1_HASH"}, 9, "after-unlock"]
}
JSON

PRIV_TX="$(invoke_tx_hash invoke-privileged-unlocked.neo-invoke.json node1)"
if [ "$(tx_vmstate "$PRIV_TX")" != "HALT" ]; then
  echo "error: privilegedAction after unlock should HALT" >&2
  run_neoxp show transaction -i "$CHAIN" "$PRIV_TX" | jq '.["application-log"].executions[0]' >&2
  exit 1
fi

echo "✅ neoxp WitnessGuardShowcase smoke test passed"
