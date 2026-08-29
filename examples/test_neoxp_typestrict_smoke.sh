#!/usr/bin/env bash
# End-to-end NeoVM type-strictness smoke test using bundled Neo-Express (neoxp).
#
# NeoVM's EQUAL is type-strict (Buffer != ByteString, Integer != Boolean) but the
# in-tree simulator is lenient and MASKS these — only a real node catches them.
# Guards three classes found by the type-strictness audit:
#  - A ternary `bytesN` operand compared to a hex-number literal: the literal must
#    canonicalize to a ByteString (else `==` is always false / `!=` always true,
#    silently defeating sentinel/role guards).
#  - A `bytesN` literal mapping key must hash to the SAME storage slot as a runtime
#    bytesN key of the same value (else a literal-key write lands in a different
#    slot than a param-key read -> silent wrong entry).
#  - `abi.encode`/`abi.encodePacked` results (CAT Buffers) compared with `==`
#    against a ByteString must match.
#
# Known residual (NOT asserted): `return b[i]` as `bytes1` yields an Integer
# rather than a 1-byte ByteString (deferred; narrow pattern).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-typestrict.XXXXXX")"
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

cat > TypeStrict.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface ISelf {
    function getId() external pure returns (bytes32);
    function id01() external pure returns (bytes32);
    function g6(bytes memory a, bytes memory b) external pure returns (bytes memory);
    function echo(bytes32 b) external pure returns (bytes32);
    function pick(bytes32 b, uint256 i) external pure returns (bytes1);
}

library TSLib {
    function lpick(bytes32 b) internal pure returns (bytes1) { return b[0]; }
}

contract TypeStrict {
    using TSLib for bytes32;
    mapping(bytes32 => uint256) private mb;

    // A: ternary bytesN == hex-number literal (must be true)
    function ternEq() public pure returns (bool) {
        bytes32 a = 0x00000000000000000000000000000000000000000000000000000000000000ff;
        bytes32 b = a;
        bool c = true;
        return (c ? a : b) == 0x00000000000000000000000000000000000000000000000000000000000000ff;
    }
    // A guard form: value == sentinel so `!=` must be FALSE (guard NOT defeated)
    function ternGuard() public pure returns (bool) {
        bytes32 a = 0x00000000000000000000000000000000000000000000000000000000000000ff;
        bytes32 b = a;
        bool c = true;
        return (c ? a : b) != 0x00000000000000000000000000000000000000000000000000000000000000ff;
    }
    // A external-return form (control: was already OK)
    function getId() public pure returns (bytes32) {
        return 0x00000000000000000000000000000000000000000000000000000000000000ff;
    }
    function extEq() public view returns (bool) {
        return ISelf(address(this)).getId() == 0x00000000000000000000000000000000000000000000000000000000000000ff;
    }

    // C: bytesN literal mapping key vs runtime key — same slot (must be true)
    function id01() public pure returns (bytes32) {
        return 0x0000000000000000000000000000000000000000000000000000000000000001;
    }
    function keySlot() public returns (bool) {
        mb[0x0000000000000000000000000000000000000000000000000000000000000001] = 42;
        bytes32 key = ISelf(address(this)).id01();
        return mb[key] == 42;
    }

    // B: abi.encodePacked/abi.encode results compared with == (must be true)
    function packedEq() public pure returns (bool) {
        return abi.encodePacked(uint8(1), uint8(2)) == hex"0102";
    }
    function packedBytes1Eq() public pure returns (bool) {
        return abi.encodePacked(bytes1(0x01), bytes1(0x02)) == hex"0102";
    }
    function g6(bytes memory a, bytes memory b) public pure returns (bytes memory) {
        return abi.encodePacked(a, b);
    }
    function extEncodedEq() public view returns (bool) {
        return ISelf(address(this)).g6(hex"01", hex"02") == hex"0102";
    }

    // E: a byte index `b[i]` is a bytes1 (1-byte ByteString) — comparisons,
    // bare-literal comparisons, and assign-then-compare must all hold on a real
    // node (b[i] used to lower to an Integer → type-strict EQUAL false).
    function byteIdxEq() public pure returns (bool) {
        bytes32 b = 0x0102030000000000000000000000000000000000000000000000000000000000;
        return b[0] == bytes1(0x01);
    }
    function byteIdxLit() public pure returns (bool) {
        bytes32 b = 0x0102030000000000000000000000000000000000000000000000000000000000;
        return b[1] == 0x02;
    }
    function byteIdxNe() public pure returns (bool) {
        bytes32 b = 0x0102030000000000000000000000000000000000000000000000000000000000;
        return b[0] != bytes1(0x02);
    }
    function byteIdxAssign() public pure returns (bool) {
        bytes32 b = 0x0102030000000000000000000000000000000000000000000000000000000000;
        bytes1 x = b[2];
        return x == bytes1(0x03);
    }
    function byteIdxUint() public pure returns (bool) {
        bytes32 b = 0x0102030000000000000000000000000000000000000000000000000000000000;
        return uint8(b[1]) == 2; // control: numeric use still correct
    }

    // F: a bytesN literal passed as an external/internal call ARGUMENT must
    // reach the callee as a big-endian ByteString, not a little-endian Integer.
    function echo(bytes32 b) public pure returns (bytes32) { return b; }
    function pick(bytes32 b, uint256 i) public pure returns (bytes1) { return b[i]; }
    function gEcho(bytes32 b) internal pure returns (bytes32) { return b; }
    function extArgEcho() public view returns (bool) {
        return ISelf(address(this)).echo(0x01020304000000000000000000000000000000000000000000000000000000ff)
            == 0x01020304000000000000000000000000000000000000000000000000000000ff;
    }
    function extArgByte() public view returns (bool) {
        return ISelf(address(this)).pick(0x0102030000000000000000000000000000000000000000000000000000000000, 0) == bytes1(0x01);
    }
    function intArgEcho() public pure returns (bool) {
        return gEcho(0x01020304000000000000000000000000000000000000000000000000000000ff)
            == 0x01020304000000000000000000000000000000000000000000000000000000ff;
    }
    // F (library): a bytesN literal as a namespaced library ARGUMENT and as a
    // using-for RECEIVER must both reach the library callee as a BE ByteString.
    function libArgCall() public pure returns (bool) {
        return TSLib.lpick(0x0102030000000000000000000000000000000000000000000000000000000000) == bytes1(0x01);
    }
    function libRecvCall() public pure returns (bool) {
        return bytes32(0x0102030000000000000000000000000000000000000000000000000000000000).lpick() == bytes1(0x01);
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/TypeStrict.sol" -o TypeStrict >/dev/null
CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null
NEF="$(ls *.nef | head -1)"
HASH="$(run_neoxp contract hash -i "$CHAIN" "$NEF" node1 | tr -d '\r')"
echo "(info) Contract hash: $HASH"
run_neoxp contract deploy -i "$CHAIN" "$NEF" node1 -j >/dev/null

assert() { # $1=method $2=expected(true/false)
  cat > "$WORK_DIR/inv.json" <<JSON
{ "contract": "$HASH", "operation": "$1", "args": [] }
JSON
  local out st v
  out="$(run_neoxp contract invoke -r -j -i "$CHAIN" "$WORK_DIR/inv.json" node1)"
  st="$(echo "$out" | jq -r '.state')"; v="$(echo "$out" | jq -rc '.stack[0].value')"
  if [ "$st" != "HALT" ]; then echo "error: $1 FAULT $(echo "$out" | jq -r '.exception // empty')" >&2; echo "$out" >&2; exit 1; fi
  if [ "$v" != "$2" ]; then echo "error: $1 → $v (expected $2)" >&2; echo "$out" >&2; exit 1; fi
  echo "✅ $1 → $v"
}

assert ternEq         true
assert ternGuard      false
assert extEq          true
assert keySlot        true
assert packedEq       true
assert packedBytes1Eq true
assert extEncodedEq   true
assert byteIdxEq      true
assert byteIdxLit     true
assert byteIdxNe      true
assert byteIdxAssign  true
assert byteIdxUint    true
assert extArgEcho     true
assert extArgByte     true
assert intArgEcho     true
assert libArgCall     true
assert libRecvCall    true

echo "✅ neoxp type-strictness smoke test passed"
