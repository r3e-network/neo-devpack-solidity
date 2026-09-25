#!/usr/bin/env bash
# Neo-Express smoke test: external function taking a STRUCT PARAMETER.
#
# Regression for the deep-review CRITICAL fix: an external/public function with
# a struct parameter previously emitted `INITSLOT args=<flattened field count>`
# while the manifest declared one `Array` parameter, so a conformant call (one
# Array StackItem) underflowed the frame and FAULTed on a real node. The
# in-tree simulator masked it. This deploys such a contract and invokes it with
# a single Array argument, asserting it HALTs and returns the correct sum.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp.XXXXXX")"
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
  echo "error: neoxp not found" >&2
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
run_neoxp() { HOME="$NEOXP_HOME" "$NEOXP_BIN" "$@"; }

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"
echo "(info) Neo-Express: $NEOXP_BIN"

cd "$WORK_DIR"

cat > StructParam.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract StructParam {
    struct P { uint256 a; uint256 b; }

    // External function taking a struct parameter — the manifest declares one
    // `Array` parameter, so a caller pushes a single Array StackItem.
    function sum(P memory p) external pure returns (uint256) {
        return p.a + p.b;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/StructParam.sol" -o StructParam >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" StructParam.nef node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash"
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" StructParam.nef node1 -j >/dev/null

# Invoke `sum((11, 31))` — the struct is passed as a single Array argument.
cat > invoke-sum.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "sum",
  "args": [[11, 31]]
}
JSON

SUM_TX_HASH="$(run_neoxp contract invoke -i "$CHAIN" invoke-sum.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$SUM_TX_HASH" ]; then
  echo "error: failed to capture sum() invocation tx hash"
  exit 1
fi

SUM_LOG="$(run_neoxp show transaction -i "$CHAIN" "$SUM_TX_HASH")"
SUM_VMSTATE="$(echo "$SUM_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$SUM_VMSTATE" != "HALT" ]; then
  echo "error: sum() vmstate=$SUM_VMSTATE (struct-param INITSLOT regression — frame underflow?)"
  echo "$SUM_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

SUM_VALUE="$(echo "$SUM_LOG" | jq -r '.["application-log"].executions[0].stack[0].value')"
if [ "$SUM_VALUE" != "42" ]; then
  echo "error: sum((11,31)) returned $SUM_VALUE, expected 42"
  echo "$SUM_LOG" | jq '.["application-log"].executions[0].stack'
  exit 1
fi

echo "✅ neoxp struct-param smoke test passed (external struct-param fn HALTs + returns 42)"
