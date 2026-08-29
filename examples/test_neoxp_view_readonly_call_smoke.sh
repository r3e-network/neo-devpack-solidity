#!/usr/bin/env bash
# End-to-end "view external call is read-only" smoke test using bundled Neo-Express (neoxp).
#
# Verifies that external member calls performed from Solidity `view`/`pure` functions are
# lowered with Neo N3 CallFlags.ReadOnly (0x05). This mirrors Solidity's EVM STATICCALL behavior:
# if the callee attempts to write storage, the call should FAULT.
#
# Scenario:
# - Callee.foo(uint256) increments a counter (writes storage) and returns a value.
# - Caller.run(uint256) is declared `view` and calls Callee.foo(...) through an interface that
#   declares the method as `view`.
# - The call should FAULT on Neo because the compiler enforces ReadOnly call flags in view context.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-readonly-call.XXXXXX")"

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

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"
echo "(info) Neo-Express: $NEOXP_BIN"

cd "$WORK_DIR"

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

cat > Callee.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Callee {
    uint256 public counter;

    function foo(uint256 x) public returns (uint256) {
        counter = counter + 1;
        return x + counter;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/Callee.sol" -o Callee >/dev/null

CALLEE_HASH="$(run_neoxp contract hash -i "$CHAIN" Callee.nef node1 | tr -d '\r')"
if [ -z "$CALLEE_HASH" ]; then
  echo "error: failed to compute callee contract hash" >&2
  exit 1
fi
echo "(info) Callee hash: $CALLEE_HASH"

cat > Caller.sol <<SOL
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface ICallee {
    function foo(uint256 x) external view returns (uint256);
}

contract Caller {
    address constant CALLEE = address($CALLEE_HASH);

    function run(uint256 x) public view returns (uint256) {
        // This should FAULT because the callee tries to write storage, while the compiler
        // emits CallFlags.ReadOnly for external calls inside view/pure contexts.
        return ICallee(CALLEE).foo(x);
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/Caller.sol" -o Caller >/dev/null

CALLER_HASH="$(run_neoxp contract hash -i "$CHAIN" Caller.nef node1 | tr -d '\r')"
if [ -z "$CALLER_HASH" ]; then
  echo "error: failed to compute caller contract hash" >&2
  exit 1
fi
echo "(info) Caller hash: $CALLER_HASH"

run_neoxp contract deploy -i "$CHAIN" Callee.nef node1 -j >/dev/null
run_neoxp contract deploy -i "$CHAIN" Caller.nef node1 -j >/dev/null

cat > invoke-run.neo-invoke.json <<JSON
{
  "contract": "$CALLER_HASH",
  "operation": "run",
  "args": [41]
}
JSON

OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-run.neo-invoke.json node1)"
if [ "$(echo "$OUT" | jq -r '.state')" != "FAULT" ]; then
  echo "error: expected Caller.run(41) to FAULT under ReadOnly call flags" >&2
  echo "$OUT" >&2
  exit 1
fi

echo "✅ neoxp view ReadOnly external-call smoke test passed"

