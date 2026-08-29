#!/usr/bin/env bash
# Smoke test: EVM compatibility adapters — compile, deploy, and execute on Neo Express.
#
# Verifies:
#   - The devpack compatibility contracts compile through neo-solc imports.
#   - NEP-17 transfer callbacks can be consumed as a payable/msg.value-style adapter.
#   - Explicit fallback dispatch logic can be exercised as an EVM fallback alternative.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-sol-evm-compat.XXXXXX")"
cleanup() { rm -rf "$WORK_DIR"; }
trap cleanup EXIT

resolve_neo_solc() {
  if [ -n "${NEO_SOLC:-}" ]; then echo "$NEO_SOLC"; return; fi
  if command -v neo-solc >/dev/null 2>&1; then echo "neo-solc"; return; fi
  echo "(info) Building neo-solc..." >&2
  (cd "$ROOT_DIR" && cargo build --bin neo-solc >/dev/null)
  echo "$ROOT_DIR/target/debug/neo-solc"
}

resolve_neoxp() {
  if [ -n "${NEOXP:-}" ]; then echo "$NEOXP"; return; fi
  if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then echo "$ROOT_DIR/build/dotnet-tools/neoxp"; return; fi
  if command -v neoxp >/dev/null 2>&1; then echo "neoxp"; return; fi
  if command -v dotnet >/dev/null 2>&1; then
    echo "(info) Installing Neo-Express (neoxp)..." >&2
    mkdir -p "$ROOT_DIR/build/dotnet-tools"
    dotnet tool install Neo.Express --tool-path "$ROOT_DIR/build/dotnet-tools" >/dev/null
    if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then echo "$ROOT_DIR/build/dotnet-tools/neoxp"; return; fi
  fi
  echo "error: neoxp not found" >&2
  exit 1
}

command -v jq >/dev/null 2>&1 || { echo "error: jq required" >&2; exit 1; }

NEO_SOLC_BIN="$(resolve_neo_solc)"
NEOXP_BIN="$(resolve_neoxp)"
NEOXP_HOME="$WORK_DIR/neoxp-home"
mkdir -p "$NEOXP_HOME"
run_neoxp() { HOME="$NEOXP_HOME" "$NEOXP_BIN" "$@"; }

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"

cd "$WORK_DIR"

cat > EVMCompatSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "contracts/compat/EVMNativeAssetAdapter.sol";
import "contracts/compat/EVMFallbackDispatcher.sol";
import "contracts/compat/EVMContractFactory.sol";

contract EVMCompatSmoke is EVMNativeAssetAdapter, EVMFallbackDispatcher, EVMContractFactory {
    uint256 private _fallbackHits;

    function _onEVMValue(
        address token,
        address from,
        uint256 amount,
        bytes memory data
    ) internal override {
        token;
        from;
        data;
        require(amount > 0, "zero value");
        _fallbackHits += 1;
    }

    function _dispatch(bytes4 selector, bytes memory data)
        internal
        override
        returns (bytes memory)
    {
        require(selector == bytes4(hex"11111111"), "selector");
        _fallbackHits += data.length;
        return "";
    }

    function fallbackHits() public view returns (uint256) {
        return _fallbackHits;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/EVMCompatSmoke.sol" -I "$ROOT_DIR/devpack" -O2 -o EVMCompatSmoke >/dev/null

NEF="EVMCompatSmoke.nef"
if [ -f "EVMCompatSmoke-EVMCompatSmoke.nef" ]; then
  NEF="EVMCompatSmoke-EVMCompatSmoke.nef"
fi

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" "$NEF" node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash"
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" "$NEF" node1 -j >/dev/null

NODE1_HASH="$(run_neoxp wallet list -i "$CHAIN" -j | jq -r '.node1[0]["script-hash"]')"
GAS_HASH="0xd2a4cff31913016155e38e474a2c06d08be276cf"

cat > gas_deposit.json <<JSON
{ "contract": "$GAS_HASH", "operation": "transfer", "args": [
    {"type":"Hash160","value":"$NODE1_HASH"},
    {"type":"Hash160","value":"$CONTRACT_HASH"},
    {"type":"Integer","value":"123456789"},
    {"type":"ByteArray","value":""}
] }
JSON

DEPOSIT_OUT="$(run_neoxp contract invoke -w Global -i "$CHAIN" gas_deposit.json node1 2>&1)"
DEPOSIT_TX="$(echo "$DEPOSIT_OUT" | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$DEPOSIT_TX" ]; then
  echo "error: GAS transfer did not produce a transaction"
  echo "$DEPOSIT_OUT"
  exit 1
fi
APP_LOG="$(run_neoxp show transaction -i "$CHAIN" "$DEPOSIT_TX")"
VMSTATE="$(echo "$APP_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$VMSTATE" != "HALT" ]; then
  echo "error: GAS transfer vmstate=$VMSTATE"
  echo "$APP_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi
echo "(ok) GAS transfer triggered onNEP17Payment"

cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "lastEVMValueAmount", "args": [] }
JSON
OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke.json node1)"
if [ "$(echo "$OUT" | jq -r '.state')" != "HALT" ]; then echo "error: lastEVMValueAmount() did not HALT"; echo "$OUT"; exit 1; fi
AMOUNT="$(echo "$OUT" | jq -r '.stack[0].value')"
if [ "$AMOUNT" != "123456789" ]; then echo "error: expected amount 123456789, got $AMOUNT"; exit 1; fi
echo "(ok) lastEVMValueAmount() = 123456789"

cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "fallbackHits", "args": [] }
JSON
OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke.json node1)"
HITS="$(echo "$OUT" | jq -r '.stack[0].value')"
if [ "$HITS" != "1" ]; then echo "error: expected fallbackHits 1 after payment, got $HITS"; exit 1; fi
echo "(ok) fallbackHits() = 1 after payment"

cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "dispatch", "args": [
    {"type":"ByteArray","value":"EREREQ=="},
    {"type":"ByteArray","value":"AQI="}
] }
JSON
set +e
DISPATCH_OUT="$(run_neoxp contract invoke -i "$CHAIN" invoke.json node1 2>&1)"
DISPATCH_STATUS=$?
set -e
TX_HASH="$(echo "$DISPATCH_OUT" | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1 || true)"
if [ "$DISPATCH_STATUS" -ne 0 ] || [ -z "$TX_HASH" ]; then
  echo "error: dispatch(bytes4,bytes) failed"
  echo "$DISPATCH_OUT"
  exit 1
fi
set +e
APP_LOG="$(run_neoxp show transaction -i "$CHAIN" "$TX_HASH" 2>&1)"
APP_LOG_STATUS=$?
set -e
if [ "$APP_LOG_STATUS" -ne 0 ]; then
  echo "error: failed to read dispatch(bytes4,bytes) transaction"
  echo "$APP_LOG"
  exit 1
fi
VMSTATE="$(echo "$APP_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$VMSTATE" != "HALT" ]; then
  echo "error: dispatch(bytes4,bytes) vmstate=$VMSTATE"
  echo "$APP_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "fallbackHits", "args": [] }
JSON
OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke.json node1)"
HITS="$(echo "$OUT" | jq -r '.stack[0].value')"
if [ "$HITS" != "3" ]; then echo "error: expected fallbackHits 3 after fallback dispatch, got $HITS"; exit 1; fi
echo "(ok) explicit fallback dispatch updated state"

echo "✅ EVM compatibility smoke test passed"
