#!/usr/bin/env bash
# End-to-end wide-integer arithmetic smoke test using bundled Neo-Express.
#
# The in-tree simulator uses arbitrary-precision BigInt and MASKS NeoVM's
# 32-byte integer faults, so these are only meaningfully validated on a real
# node:
# - addmod with a sum >= 2^256 (carry-correct limb reduction),
# - mulmod with a product >= 2^256 (512-bit reduction),
# - `**` whose wasted final squaring would otherwise overshoot 32 bytes
#   (`2 ** 200` faulted pre-fix; must HALT),
# - logical uint256 `>>` of a value >= 2^255.
#
# Known residual (NOT asserted): `**` whose RESULT magnitude is >= 2^255 still
# faults on-chain (33-byte intermediate) rather than a catchable Panic.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-arith.XXXXXX")"
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

cat > ArithSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract ArithSmoke {
    function addmodCarry() public pure returns (uint256) { uint256 m = ~uint256(0); return addmod(m, m, 5); } // (2^257-2) % 5 = 0
    function addmod7()     public pure returns (uint256) { uint256 m = ~uint256(0); return addmod(m, 1, 7); } // 2^256 % 7 = 2
    function mulmodLarge()  public pure returns (uint256) { uint256 a = uint256(1) << 200; return mulmod(a, a, 7); } // 2^400 % 7 = 2
    function pow2_200()     public pure returns (uint256) { uint256 b = 2; return b ** 200; }                 // must HALT (no 33-byte fault)
    function shrLarge()     public pure returns (uint256) { uint256 v = uint256(1) << 255; return v >> 1; }   // logical >> -> 2^254
    // uint256 ** overflow: checked must be a CATCHABLE Panic (not a VM fault),
    // unchecked must wrap mod 2^256.
    function uOverflow()    public pure returns (uint256) { uint256 b = 2; return b ** 256; }
    function uChecked256()  public returns (bool) { try this.uOverflow() returns (uint256) { return false; } catch { return true; } }
    function uWrap()        public pure returns (bool) { unchecked { uint256 b = 2; return (b ** 256) == 0; } }
    // int256 ** via unsigned magnitude + sign: negative base, overflow Panic,
    // and the int256.min boundary ((-2)**255 is in range, must not Panic).
    function iNeg()         public pure returns (int256) { int256 b = -3; return b ** 5; }       // -243
    function iMinPow()      public pure returns (bool) { int256 b = -2; return b ** 255 == type(int256).min; }
    function iOverflow()    public pure returns (int256) { int256 b = 2; return b ** 255; }       // checked -> Panic
    function iChecked()     public returns (bool) { try this.iOverflow() returns (int256) { return false; } catch { return true; } }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/ArithSmoke.sol" -o ArithSmoke >/dev/null
CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null
NEF="$(ls *.nef | head -1)"
HASH="$(run_neoxp contract hash -i "$CHAIN" "$NEF" node1 | tr -d '\r')"
echo "(info) Contract hash: $HASH"
run_neoxp contract deploy -i "$CHAIN" "$NEF" node1 -j >/dev/null

assert_eq() { # $1=method $2=expected-decimal
  cat > "$WORK_DIR/inv.json" <<JSON
{ "contract": "$HASH", "operation": "$1", "args": [] }
JSON
  local out st v
  out="$(run_neoxp contract invoke -r -j -i "$CHAIN" "$WORK_DIR/inv.json" node1)"
  st="$(echo "$out" | jq -r '.state')"; v="$(echo "$out" | jq -r '.stack[0].value')"
  if [ "$st" != "HALT" ]; then echo "error: $1 FAULT $(echo "$out" | jq -r '.exception // empty')" >&2; echo "$out" >&2; exit 1; fi
  if [ "$v" != "$2" ]; then echo "error: $1 → $v (expected $2)" >&2; echo "$out" >&2; exit 1; fi
  echo "✅ $1 → $v"
}

assert_eq addmodCarry 0
assert_eq addmod7     2
assert_eq mulmodLarge 2
assert_eq pow2_200    1606938044258990275541962092341162602522202993782792835301376
assert_eq shrLarge    28948022309329048855892746252171976963317496166410141009864396001978282409984
assert_eq uChecked256 true   # checked 2**256 overflow -> CATCHABLE Panic (was uncatchable fault)
assert_eq uWrap       true   # unchecked 2**256 wraps to 0
assert_eq iNeg        -243   # int256 (-3)**5
assert_eq iMinPow     true   # int256 (-2)**255 == int256.min (in range)
assert_eq iChecked    true   # checked int256 2**255 overflow -> CATCHABLE Panic

echo "✅ neoxp wide-integer arithmetic smoke test passed"
