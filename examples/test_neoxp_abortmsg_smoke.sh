#!/usr/bin/env bash
# End-to-end revert reason smoke test using bundled Neo-Express (neoxp).
#
# Verifies that:
# - `require(false, "...")` is lowered to NeoVM THROW (fault state contains message)
# - the generated .nef/.manifest can be deployed and invoked on a fresh neo-express chain

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-abortmsg.XXXXXX")"

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

cat > AbortMsgSmoke.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract AbortMsgSmoke {
    function ok() public pure returns (uint256) {
        return 1;
    }

    function fail() public pure returns (uint256) {
        require(false, "oops");
        return 2;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/AbortMsgSmoke.sol" -o AbortMsgSmoke >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" AbortMsgSmoke.nef node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash" >&2
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" AbortMsgSmoke.nef node1 -j >/dev/null

cat > invoke-fail.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "fail",
  "args": []
}
JSON

OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-fail.neo-invoke.json node1)"
STATE="$(echo "$OUT" | jq -r '.state')"
EXCEPTION="$(echo "$OUT" | jq -r '.exception // empty')"

if [ "$STATE" != "FAULT" ]; then
  echo "error: expected fail() to FAULT" >&2
  echo "$OUT" >&2
  exit 1
fi

if [[ "$EXCEPTION" != *"oops"* ]]; then
  echo "error: expected exception to include 'oops'" >&2
  echo "exception: $EXCEPTION" >&2
  echo "$OUT" >&2
  exit 1
fi

echo "✅ neoxp revert reason smoke test passed"
