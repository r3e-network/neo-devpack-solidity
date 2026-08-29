#!/usr/bin/env bash
# End-to-end CALLT + method token smoke test using bundled Neo-Express (neoxp).
#
# Verifies that neo-solc can:
# - compile Solidity into a Neo N3-valid .nef + .manifest.json that contains a method token table
# - deploy the contract to a fresh neo-express chain
# - invoke methods that exercise native contract calls via CALLT (e.g., block.number -> Ledger.currentIndex)
#
# The script compiles to `-f json` so we can assert `nef.tokens` is non-empty, then
# materializes `Smoke.nef` + `Smoke.manifest.json` from the JSON artifact.
#
# Event emission is intentionally excluded here so the remaining on-chain
# `abiEncode` / `abiDecode` gap stays isolated in `test_neoxp_encoding_smoke.sh`.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-callt.XXXXXX")"

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

if ! command -v hexdump >/dev/null 2>&1; then
  echo "error: hexdump is required for this smoke test" >&2
  exit 1
fi

if ! command -v xxd >/dev/null 2>&1; then
  echo "error: xxd is required for this smoke test" >&2
  exit 1
fi

reverse_hex_bytes() {
  local hex="$1"
  hex="$(printf '%s' "$hex" | tr '[:upper:]' '[:lower:]')"
  local out=""
  while [ -n "$hex" ]; do
    out="${hex:0:2}$out"
    hex="${hex:2}"
  done
  echo "$out"
}

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

cat > Smoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Smoke {
    uint256 private value;

    function sender() public view returns (address) {
        return msg.sender;
    }

    function origin() public view returns (address) {
        return tx.origin;
    }

    function height() public view returns (uint256) {
        return block.number;
    }

    function set(uint256 v) public {
        value = v;
    }

    function get() public view returns (uint256) {
        return value;
    }
}
SOL

# Compile to JSON with CALLT enabled, then materialize Smoke.nef + Smoke.manifest.json.
"$NEO_SOLC_BIN" "$WORK_DIR/Smoke.sol" --callt -f json -o Smoke.json >/dev/null

TOKEN_COUNT="$(jq -r '.nef.tokens | length' Smoke.json | tr -d '\r')"
if [ -z "$TOKEN_COUNT" ] || [ "$TOKEN_COUNT" -lt 1 ]; then
  echo "error: expected CALLT compilation to emit >= 1 method token (got $TOKEN_COUNT)" >&2
  exit 1
fi
echo "(info) NEF method tokens: $TOKEN_COUNT"

jq -r '.nef.image' Smoke.json | tr -d '\r' | xxd -r -p > Smoke.nef
jq '.manifest' Smoke.json > Smoke.manifest.json

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null

# Ensure deployer has enough GAS for deploy + invoke.
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" Smoke.nef node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash"
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" Smoke.nef node1 -j >/dev/null

# Verify msg.sender / tx.origin are the deployer account on entry calls.
NODE1_HASH_BE="$(run_neoxp wallet list -j -i "$CHAIN" | jq -r '.node1[0]["script-hash"]' | tr -d '\r' | sed 's/^0x//' )"
NODE1_HASH_LE="$(reverse_hex_bytes "$NODE1_HASH_BE")"

cat > invoke-sender.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "sender",
  "args": []
}
JSON

SENDER_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-sender.neo-invoke.json node1)"
if [ "$(echo "$SENDER_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: sender() did not HALT"
  echo "$SENDER_OUT"
  exit 1
fi
if [ "$(echo "$SENDER_OUT" | jq -r '.stack[0].type')" != "ByteString" ]; then
  echo "error: sender() returned non-ByteString"
  echo "$SENDER_OUT"
  exit 1
fi
SENDER_HEX="$(echo "$SENDER_OUT" | jq -r '.stack[0].value' | base64 -d | hexdump -v -e '/1 "%02x"')"
SENDER_HEX_LC="$(printf '%s' "$SENDER_HEX" | tr '[:upper:]' '[:lower:]')"
NODE1_HASH_LE_LC="$(printf '%s' "$NODE1_HASH_LE" | tr '[:upper:]' '[:lower:]')"
if [ "$SENDER_HEX_LC" != "$NODE1_HASH_LE_LC" ]; then
  echo "error: msg.sender mismatch"
  echo "  got (LE):  ${SENDER_HEX}"
  echo "  want (LE): ${NODE1_HASH_LE}"
  echo "  want (BE): ${NODE1_HASH_BE}"
  exit 1
fi

cat > invoke-origin.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "origin",
  "args": []
}
JSON

ORIGIN_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-origin.neo-invoke.json node1)"
if [ "$(echo "$ORIGIN_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: origin() did not HALT"
  echo "$ORIGIN_OUT"
  exit 1
fi
if [ "$(echo "$ORIGIN_OUT" | jq -r '.stack[0].type')" != "ByteString" ]; then
  echo "error: origin() returned non-ByteString"
  echo "$ORIGIN_OUT"
  exit 1
fi

# Invoke `set(7)` and confirm storage is updated.
cat > invoke-set.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "set",
  "args": [7]
}
JSON

TX_HASH="$(run_neoxp contract invoke -i "$CHAIN" invoke-set.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$TX_HASH" ]; then
  echo "error: failed to capture invocation tx hash"
  exit 1
fi

APP_LOG="$(run_neoxp show transaction -i "$CHAIN" "$TX_HASH")"
VMSTATE="$(echo "$APP_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$VMSTATE" != "HALT" ]; then
  echo "error: set(7) vmstate=$VMSTATE"
  echo "$APP_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

# Invoke `get()` and confirm it returns Integer 7.
cat > invoke-get.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "get",
  "args": []
}
JSON

GET_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-get.neo-invoke.json node1)"
if [ "$(echo "$GET_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: get() did not HALT"
  echo "$GET_OUT"
  exit 1
fi
if [ "$(echo "$GET_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: get() returned non-Integer"
  echo "$GET_OUT"
  exit 1
fi
if [ "$(echo "$GET_OUT" | jq -r '.stack[0].value')" != "7" ]; then
  echo "error: get() returned unexpected value"
  echo "$GET_OUT"
  exit 1
fi

# Invoke `height()` and confirm it returns an Integer (native call via CALLT token).
cat > invoke-height.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "height",
  "args": []
}
JSON

HEIGHT_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-height.neo-invoke.json node1)"
if [ "$(echo "$HEIGHT_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: height() did not HALT"
  echo "$HEIGHT_OUT"
  exit 1
fi
if [ "$(echo "$HEIGHT_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: height() returned non-Integer"
  echo "$HEIGHT_OUT"
  exit 1
fi

echo "✅ neoxp CALLT smoke test passed"
