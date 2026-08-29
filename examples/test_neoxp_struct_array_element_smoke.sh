#!/usr/bin/env bash
# Neo-Express smoke test: struct-array element storage.
#
# Verifies that neo-solc can:
# - construct a struct value (named args)
# - push it into an array field inside a storage struct (mapping -> struct -> Entry[])
# - pop it back out without FAULTing (requires correct storage encoding)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-neoxp.XXXXXX")"

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

cat > StructArrayElement.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract StructArrayElement {
    struct Entry {
        uint256 amount;
        address actor;
    }

    struct Container {
        Entry[] entries;
    }

    mapping(address => Container) private containers;

    function push(uint256 amount) public {
        Entry memory entry = Entry({ amount: amount, actor: msg.sender });
        containers[msg.sender].entries.push(entry);
    }

    function popAmount() public returns (uint256) {
        Entry memory entry = containers[msg.sender].entries.pop();
        return entry.amount;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/StructArrayElement.sol" -o StructArrayElement >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null

# Ensure deployer has enough GAS for deploy + invoke.
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" StructArrayElement.nef node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash"
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" StructArrayElement.nef node1 -j >/dev/null

# Invoke `push(42)` and ensure it HALTs.
cat > invoke-push.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "push",
  "args": [42]
}
JSON

PUSH_TX_HASH="$(run_neoxp contract invoke -i "$CHAIN" invoke-push.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$PUSH_TX_HASH" ]; then
  echo "error: failed to capture push() invocation tx hash"
  exit 1
fi

PUSH_LOG="$(run_neoxp show transaction -i "$CHAIN" "$PUSH_TX_HASH")"
PUSH_VMSTATE="$(echo "$PUSH_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$PUSH_VMSTATE" != "HALT" ]; then
  echo "error: push() vmstate=$PUSH_VMSTATE"
  echo "$PUSH_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

# Invoke `popAmount()` (stateful) and confirm it returns Integer 42 and HALTs.
cat > invoke-pop.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "popAmount",
  "args": []
}
JSON

POP_TX_HASH="$(run_neoxp contract invoke -i "$CHAIN" invoke-pop.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$POP_TX_HASH" ]; then
  echo "error: failed to capture popAmount() invocation tx hash"
  exit 1
fi

POP_LOG="$(run_neoxp show transaction -i "$CHAIN" "$POP_TX_HASH")"
POP_VMSTATE="$(echo "$POP_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$POP_VMSTATE" != "HALT" ]; then
  echo "error: popAmount() vmstate=$POP_VMSTATE"
  echo "$POP_LOG" | jq '.["application-log"].executions[0]'
  exit 1
fi

if [ "$(echo "$POP_LOG" | jq -r '.["application-log"].executions[0].stack[0].type')" != "Integer" ]; then
  echo "error: popAmount() returned non-Integer"
  echo "$POP_LOG" | jq '.["application-log"].executions[0].stack'
  exit 1
fi
if [ "$(echo "$POP_LOG" | jq -r '.["application-log"].executions[0].stack[0].value')" != "42" ]; then
  echo "error: popAmount() returned unexpected value"
  echo "$POP_LOG" | jq '.["application-log"].executions[0].stack'
  exit 1
fi

echo "✅ neoxp struct-array element smoke test passed"

