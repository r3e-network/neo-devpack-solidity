#!/usr/bin/env bash
# End-to-end abi.decode smoke test using bundled Neo-Express (neoxp).
#
# Guards two Buffer/Boolean-vs-real-type divergences the in-tree simulator masks
# with a lenient EQUAL (it only surfaces on a real node):
# - a decoded `address` came back as a NeoVM *Buffer* (0x30) from the byte
#   reversal, so `decoded == addressLiteral` (lowered to `EQUAL`) was always
#   false even when the bytes matched — must be coerced to ByteString;
# - a decoded `bool` came back as an *Integer* (1/0), so `decoded == true`
#   (`EQUAL` against a Boolean literal) was false — must be a real Boolean.
# Also covers bytesN / bytes / string / signed+unsigned int decode round-trips.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-abidecode.XXXXXX")"
trap 'rm -rf "$WORK_DIR"' EXIT

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
    mkdir -p "$ROOT_DIR/build/dotnet-tools"
    dotnet tool install Neo.Express --tool-path "$ROOT_DIR/build/dotnet-tools" >/dev/null
    [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ] && { echo "$ROOT_DIR/build/dotnet-tools/neoxp"; return; }
  fi
  echo "error: neoxp not found" >&2; exit 1
}
command -v jq >/dev/null 2>&1 || { echo "error: jq is required" >&2; exit 1; }

NEO_SOLC_BIN="$(resolve_neo_solc)"; NEOXP_BIN="$(resolve_neoxp)"
NEOXP_HOME="$WORK_DIR/neoxp-home"; mkdir -p "$NEOXP_HOME"
run_neoxp() { HOME="$NEOXP_HOME" "$NEOXP_BIN" "$@"; }
echo "(info) Compiler: $NEO_SOLC_BIN"; echo "(info) Neo-Express: $NEOXP_BIN"
cd "$WORK_DIR"

cat > AbiDecodeSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract AbiDecodeSmoke {
    // decoded address must equal the original literal (Buffer->ByteString)
    function addrEq() public pure returns (bool) {
        bytes memory e = abi.encode(address(0x1234567890123456789012345678901234567890));
        address z = abi.decode(e, (address));
        return z == address(0x1234567890123456789012345678901234567890);
    }
    // decoded bool must equal the `true` literal (Integer->Boolean)
    function boolEq() public pure returns (bool) {
        bytes memory e = abi.encode(true);
        bool y = abi.decode(e, (bool));
        return y == true;
    }
    // a decoded bool must also be a real Boolean stack item (manifest conformance)
    function boolType() public pure returns (bool) {
        bytes memory e = abi.encode(true);
        return abi.decode(e, (bool));
    }
    // mixed tuple — the FeatureMatrix round-trip pattern
    function tupleEq() public pure returns (bool) {
        bytes memory e = abi.encode(uint256(123), true, address(0x00000000000000000000000000000000000000AA));
        (uint256 x, bool y, address z) = abi.decode(e, (uint256, bool, address));
        return x == 123 && y == true && z == address(0x00000000000000000000000000000000000000AA);
    }
    // byte-shaped decodes round-trip and compare
    function bytesNEq() public pure returns (bool) {
        bytes memory e = abi.encode(bytes4(0x0a0b0c0d));
        return abi.decode(e, (bytes4)) == bytes4(0x0a0b0c0d);
    }
    function dynEq() public pure returns (bool) {
        bytes memory e = abi.encode(string("hello"), bytes(hex"aabbcc"));
        (string memory s, bytes memory b) = abi.decode(e, (string, bytes));
        return keccak256(bytes(s)) == keccak256(bytes("hello")) && keccak256(b) == keccak256(hex"aabbcc");
    }
    // signed/unsigned ints round-trip
    function intEq() public pure returns (bool) {
        bytes memory e = abi.encode(uint8(7), int256(-5), uint256(99));
        (uint8 a, int256 b, uint256 c) = abi.decode(e, (uint8, int256, uint256));
        return a == 7 && b == -5 && c == 99;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/AbiDecodeSmoke.sol" -o AbiDecodeSmoke >/dev/null
CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null
NEF="$(ls *.nef | head -1)"
HASH="$(run_neoxp contract hash -i "$CHAIN" "$NEF" node1 | tr -d '\r')"
echo "(info) Contract hash: $HASH"
run_neoxp contract deploy -i "$CHAIN" "$NEF" node1 -j >/dev/null

assert_true() { # $1=method
  cat > "$WORK_DIR/inv.json" <<JSON
{ "contract": "$HASH", "operation": "$1", "args": [] }
JSON
  local out st v
  out="$(run_neoxp contract invoke -r -j -i "$CHAIN" "$WORK_DIR/inv.json" node1)"
  st="$(echo "$out" | jq -r '.state')"; v="$(echo "$out" | jq -r '.stack[0].value')"
  if [ "$st" != "HALT" ]; then echo "error: $1 FAULT $(echo "$out" | jq -r '.exception // empty')" >&2; echo "$out" >&2; exit 1; fi
  if [ "$v" != "true" ]; then echo "error: $1 → $v (expected true)" >&2; echo "$out" >&2; exit 1; fi
  echo "✅ $1 → $v"
}

assert_true addrEq
assert_true boolEq
assert_true boolType
assert_true tupleEq
assert_true bytesNEq
assert_true dynEq
assert_true intEq

echo "✅ neoxp abi.decode smoke test passed"
