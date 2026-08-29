#!/usr/bin/env bash
# End-to-end try/catch smoke test using bundled Neo-Express (neoxp).
#
# Guards two real-node behaviors the in-tree simulator CANNOT (its lenient
# CAT/ISTYPE/EQUAL mask the divergence):
# - `catch Panic(uint code)` and `catch Error(string msg)` MATCH faults
#   propagated from an external call (the EVM revert envelope is delivered as a
#   Buffer; the selector guard must normalize Buffer↔ByteString or it silently
#   falls through to `catch (bytes)`), and decode their payloads.
# - the NeoVM TRY frame stays balanced across catch exits (matched-clause exit
#   and return-in-catch) so invocations HALT rather than faulting.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-trycatch.XXXXXX")"
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
  echo "error: neoxp not found" >&2; exit 1
}

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for this smoke test" >&2; exit 1
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

cat > TryCatchSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract TryCatchSmoke {
    function willPanic() public pure returns (uint256) { uint256 a = 1; uint256 b = 0; return a / b; }
    function willRevert() public pure returns (uint256) { require(false, "boom"); return 1; }
    function ok() public pure returns (uint256) { return 7; }

    // catch Panic must match the propagated div-by-zero (0x12) and decode code.
    function panicCode() public returns (uint256) {
        try this.willPanic() returns (uint256) { return 0; }
        catch Panic(uint256 code) { return code; }
        catch (bytes memory) { return 999; }
    }

    // catch Error must match the propagated require() and decode the message length.
    function errorLen() public returns (uint256) {
        try this.willRevert() returns (uint256) { return 0; }
        catch Error(string memory reason) { return bytes(reason).length; }
        catch (bytes memory) { return 999; }
    }

    // Matched non-fallback catch that falls through, then returns (frame balance).
    function fallThrough() public returns (uint256) {
        uint256 total = 0;
        try this.willPanic() returns (uint256 r) { total += r; }
        catch Panic(uint256 code) { total += code; }
        catch (bytes memory) { total += 100; }
        return total;
    }

    // Return from inside a matched catch (frame must be popped before RET).
    function returnInCatch() public returns (uint256) {
        try this.willPanic() returns (uint256) { return 0; }
        catch Panic(uint256 code) { return code + 1; }
        catch (bytes memory) { return 100; }
    }

    // Success handler returns (frame already popped on the success edge).
    function successReturns() public returns (uint256) {
        try this.ok() returns (uint256 r) { return r + 1; }
        catch { return 0; }
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/TryCatchSmoke.sol" -o TryCatchSmoke >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null
HASH="$(run_neoxp contract hash -i "$CHAIN" TryCatchSmoke.nef node1 | tr -d '\r')"
echo "(info) Contract hash: $HASH"
run_neoxp contract deploy -i "$CHAIN" TryCatchSmoke.nef node1 -j >/dev/null

assert_invoke() { # $1=method $2=expected-integer
  cat > "$WORK_DIR/inv.json" <<JSON
{ "contract": "$HASH", "operation": "$1", "args": [] }
JSON
  local out state value
  out="$(run_neoxp contract invoke -r -j -i "$CHAIN" "$WORK_DIR/inv.json" node1)"
  state="$(echo "$out" | jq -r '.state')"
  value="$(echo "$out" | jq -r '.stack[0].value // empty')"
  if [ "$state" != "HALT" ]; then
    echo "error: $1 expected HALT, got $state ($(echo "$out" | jq -r '.exception // empty'))" >&2
    echo "$out" >&2; exit 1
  fi
  if [ "$value" != "$2" ]; then
    echo "error: $1 returned $value, expected $2 (catch clause mismatch or decode error)" >&2
    echo "$out" >&2; exit 1
  fi
  echo "✅ $1 → $value"
}

assert_invoke panicCode 18      # Panic(0x12) matched + decoded
assert_invoke errorLen 4        # Error("boom") matched, length 4
assert_invoke fallThrough 18    # Panic matched, fell through, returned total
assert_invoke returnInCatch 19  # return from inside matched catch (code+1)
assert_invoke successReturns 8  # success handler return

echo "✅ neoxp try/catch smoke test passed"
