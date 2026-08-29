#!/usr/bin/env bash
# End-to-end low-level address.staticcall(...) failure smoke test using bundled Neo-Express (neoxp).
#
# Verifies that:
# - low-level calls are wrapped in NeoVM TRY/ENDTRY
# - a callee FAULT does not propagate as FAULT to the caller
# - instead, `(bool success, bytes data)` reports `success=false` and the caller HALTs
#
# This smoke intentionally avoids `abi.encodeWithSignature(...)` because the
# on-chain `abiEncode` helper is still isolated in the dedicated ABI smoke lane.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-lowlevel-call-fail.XXXXXX")"

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
    function foo() public pure returns (uint256) {
        require(false, "boom");
        return 123;
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

contract Caller {
    address constant CALLEE = address($CALLEE_HASH);
    bytes4 constant FOO_SELECTOR = bytes4(keccak256("foo()"));

    function run() public view returns (bool ok) {
        (bool success, bytes memory data) = CALLEE.staticcall(abi.encodeWithSelector(FOO_SELECTOR));
        data;
        return success;
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
  "args": []
}
JSON

OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-run.neo-invoke.json node1)"
if [ "$(echo "$OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: expected Caller.run() to HALT (callee failure should be caught)" >&2
  echo "$OUT" >&2
  exit 1
fi
if [ "$(echo "$OUT" | jq -r '.stack[0].type')" != "Boolean" ]; then
  echo "error: expected Caller.run() to return Boolean" >&2
  echo "$OUT" >&2
  exit 1
fi
if [ "$(echo "$OUT" | jq -r '.stack[0].value')" != "false" ]; then
  echo "error: expected Caller.run() to return false" >&2
  echo "$OUT" >&2
  exit 1
fi

echo "✅ neoxp low-level call failure smoke test passed"
