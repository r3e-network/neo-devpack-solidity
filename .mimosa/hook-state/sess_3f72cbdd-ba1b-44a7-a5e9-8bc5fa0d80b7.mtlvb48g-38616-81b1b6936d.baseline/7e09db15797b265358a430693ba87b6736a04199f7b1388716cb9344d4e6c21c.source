#!/usr/bin/env bash
# Neo-Express smoke test: nested struct storage slot derivation.
#
# Verifies that neo-solc can correctly access nested struct fields in storage
# (mapping -> struct -> struct -> field) and load nested structs into memory.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp-nested.XXXXXX")"

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

cat > NestedStructStorage.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract NestedStructStorage {
    struct Leaf {
        uint256 a;
    }

    struct Inner {
        Leaf leaf;
        uint256 x;
    }

    struct Outer {
        Inner inner;
        uint256 z;
    }

    mapping(address => Outer) private m;
    Outer private s;

    function run() public {
        Outer storage o = m[msg.sender];
        o.inner = Inner({ leaf: Leaf({ a: 7 }), x: 9 });
        o.z = 3;

        s = Outer({ inner: Inner({ leaf: Leaf({ a: 1 }), x: 2 }), z: 4 });
    }

    function getMapLeaf() public view returns (uint256) { return m[msg.sender].inner.leaf.a; }
    function getMapX() public view returns (uint256) { return m[msg.sender].inner.x; }
    function getMapZ() public view returns (uint256) { return m[msg.sender].z; }
    function getStateLeaf() public view returns (uint256) { return s.inner.leaf.a; }
    function getStateX() public view returns (uint256) { return s.inner.x; }
    function getStateZ() public view returns (uint256) { return s.z; }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/NestedStructStorage.sol" -o NestedStructStorage >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null

# Ensure deployer has enough GAS for deploy + invoke.
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" NestedStructStorage.nef node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash" >&2
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" NestedStructStorage.nef node1 -j >/dev/null

cat > invoke-run.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "run",
  "args": []
}
JSON

TX_HASH="$(run_neoxp contract invoke -i "$CHAIN" invoke-run.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$TX_HASH" ]; then
  echo "error: failed to capture run() invocation tx hash" >&2
  exit 1
fi

LOG="$(run_neoxp show transaction -i "$CHAIN" "$TX_HASH")"
VMSTATE="$(echo "$LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$VMSTATE" != "HALT" ]; then
  echo "error: run() vmstate=$VMSTATE" >&2
  echo "$LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

check_getter() {
  local method="$1"
  local expected="$2"
  cat > invoke-getter.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "$method",
  "args": []
}
JSON

  local out
  out="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-getter.neo-invoke.json node1)"
  if [ "$(echo "$out" | jq -r '.state')" != "HALT" ]; then
    echo "error: $method() did not HALT" >&2
    echo "$out"
    exit 1
  fi
  if [ "$(echo "$out" | jq -r '.stack[0].type')" != "Integer" ]; then
    echo "error: $method() returned non-Integer" >&2
    echo "$out"
    exit 1
  fi
  if [ "$(echo "$out" | jq -r '.stack[0].value')" != "$expected" ]; then
    echo "error: $method() returned unexpected value" >&2
    echo "$out"
    exit 1
  fi
}

check_getter getMapLeaf 7
check_getter getMapX 9
check_getter getMapZ 3
check_getter getStateLeaf 1
check_getter getStateX 2
check_getter getStateZ 4

echo "✅ neoxp nested struct storage smoke test passed"
