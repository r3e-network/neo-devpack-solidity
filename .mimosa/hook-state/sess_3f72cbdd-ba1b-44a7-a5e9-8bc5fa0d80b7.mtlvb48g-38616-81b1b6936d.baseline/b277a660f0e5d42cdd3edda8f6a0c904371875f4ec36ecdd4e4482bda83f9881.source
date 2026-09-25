#!/usr/bin/env bash
# End-to-end bytesN smoke test using bundled Neo-Express (neoxp).
#
# Guards the bytesN canonicalization that the in-tree simulator only partly
# checks (integer-backed vs ByteString backing diverges on real nodes):
# - abi.encode of a bytesN literal in struct/local positions is LEFT-aligned,
# - abi.encodePacked emits exactly N big-endian bytes,
# - a bare `return <bytesN literal/constant>` comes back as a big-endian
#   ByteString matching the manifest's ByteArray type (NOT an Integer),
# - bytesN comparison holds.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-bytesn.XXXXXX")"
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

cat > BytesNSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract BytesNSmoke {
    struct S { bytes4 b; uint256 x; }
    function structField() public pure returns (bytes memory) { return abi.encode(S(0x01020304, 1)); }
    function localVar() public pure returns (bytes memory) { bytes4 v = 0x01020304; return abi.encode(v); }
    bytes4 constant SEL = 0x01020304;
    function packed() public pure returns (bytes memory) { return abi.encodePacked(SEL); }
    function b4lit() public pure returns (bytes4) { return 0x01020304; }
    function b32lit() public pure returns (bytes32) { return 0x00000000000000000000000000000000000000000000000000000000deadbeef; }
    function selReturn() public pure returns (bytes4) { return SEL; }
    function cmpEq() public pure returns (bool) { bytes4 v = 0x01020304; return v == 0x01020304; }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/BytesNSmoke.sol" -o BytesNSmoke >/dev/null
CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null
NEF="$(ls *.nef | head -1)"
HASH="$(run_neoxp contract hash -i "$CHAIN" "$NEF" node1 | tr -d '\r')"
echo "(info) Contract hash: $HASH"
run_neoxp contract deploy -i "$CHAIN" "$NEF" node1 -j >/dev/null

# Decode stack[0] to hex (for bytes/bytesN) or pass through (bool); assert.
assert_hex() { # $1=method $2=expected-hex(lower, no 0x)
  cat > "$WORK_DIR/inv.json" <<JSON
{ "contract": "$HASH", "operation": "$1", "args": [] }
JSON
  local out st typ val got
  out="$(run_neoxp contract invoke -r -j -i "$CHAIN" "$WORK_DIR/inv.json" node1)"
  st="$(echo "$out" | jq -r '.state')"; typ="$(echo "$out" | jq -r '.stack[0].type')"; val="$(echo "$out" | jq -r '.stack[0].value')"
  if [ "$st" != "HALT" ]; then echo "error: $1 FAULT $(echo "$out"|jq -r '.exception//empty')" >&2; exit 1; fi
  case "$typ" in
    Buffer|ByteString) got="$(echo "$val" | base64 -d | xxd -p | tr -d '\n')" ;;
    Boolean) got="$val" ;;
    *) echo "error: $1 unexpected stack type $typ (value $val) — expected byte-like" >&2; exit 1 ;;
  esac
  if [ "$got" != "$2" ]; then echo "error: $1 → $got (expected $2)" >&2; echo "$out" >&2; exit 1; fi
  echo "✅ $1 → $got"
}

zeros() { printf '0%.0s' $(seq 1 "$1"); }  # $1 = number of hex '0' chars
B4SLOT="01020304$(zeros 56)"   # bytes4 0x01020304 left-aligned in a 32-byte slot
U1SLOT="$(zeros 62)01"         # uint256 == 1 (right-aligned 32-byte slot)

assert_hex structField "${B4SLOT}${U1SLOT}"
assert_hex localVar    "${B4SLOT}"
assert_hex packed      "01020304"
assert_hex b4lit       "01020304"
assert_hex b32lit      "$(zeros 56)deadbeef"   # 28 zero bytes + deadbeef = 32 bytes
assert_hex selReturn   "01020304"
assert_hex cmpEq       "true"

echo "✅ neoxp bytesN smoke test passed"
