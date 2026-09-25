#!/usr/bin/env bash
# Neo-Express smoke test: compound assignment correctness.
#
# Verifies that neo-solc correctly lowers compound assignments (e.g., `-=`) and that
# the resulting contract can be deployed + invoked on a fresh neo-express chain.

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

cat > Compound.sol <<'SOL'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Compound {
    mapping(uint256 => uint256) private m;
    struct Stake {
        uint256 amount;
    }
    mapping(address => Stake) private stakes;

    function run() public {
        m[1] = 10;
        stakes[msg.sender].amount = 10;
        unchecked {
            // Keep this smoke focused on compound-assignment storage semantics.
            // The checked-uint256 overflow-guard path is covered separately and
            // is currently the shared Neo-Express arithmetic blocker.
            m[1] -= 3;
            stakes[msg.sender].amount += 2;
            stakes[msg.sender].amount -= 1;
        }
    }

    function get() public view returns (uint256) {
        return m[1];
    }

    function getStake() public view returns (uint256) {
        return stakes[msg.sender].amount;
    }
}
SOL

"$NEO_SOLC_BIN" "$WORK_DIR/Compound.sol" -o Compound >/dev/null

CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null

# Ensure deployer has enough GAS for deploy + invoke.
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null

CONTRACT_HASH="$(run_neoxp contract hash -i "$CHAIN" Compound.nef node1 | tr -d '\r')"
if [ -z "$CONTRACT_HASH" ]; then
  echo "error: failed to compute contract hash"
  exit 1
fi
echo "(info) Contract hash: $CONTRACT_HASH"

run_neoxp contract deploy -i "$CHAIN" Compound.nef node1 -j >/dev/null

# Invoke `run()` (stateful) and ensure it HALTs.
cat > invoke-run.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "run",
  "args": []
}
JSON

TX_HASH="$(run_neoxp contract invoke -i "$CHAIN" invoke-run.neo-invoke.json node1 | grep -oE '0x[0-9a-fA-F]{64}' | head -n 1)"
if [ -z "$TX_HASH" ]; then
  echo "error: failed to capture invocation tx hash"
  exit 1
fi

APP_LOG="$(run_neoxp show transaction -i "$CHAIN" "$TX_HASH")"
VMSTATE="$(echo "$APP_LOG" | jq -r '.["application-log"].executions[0].vmstate')"
if [ "$VMSTATE" != "HALT" ]; then
  echo "error: run() vmstate=$VMSTATE"
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

# Invoke `getStake()` and confirm it returns Integer 11.
cat > invoke-get-stake.neo-invoke.json <<JSON
{
  "contract": "$CONTRACT_HASH",
  "operation": "getStake",
  "args": []
}
JSON

GET_STAKE_OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke-get-stake.neo-invoke.json node1)"
if [ "$(echo "$GET_STAKE_OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: getStake() did not HALT"
  echo "$GET_STAKE_OUT"
  exit 1
fi
if [ "$(echo "$GET_STAKE_OUT" | jq -r '.stack[0].type')" != "Integer" ]; then
  echo "error: getStake() returned non-Integer"
  echo "$GET_STAKE_OUT"
  exit 1
fi
if [ "$(echo "$GET_STAKE_OUT" | jq -r '.stack[0].value')" != "11" ]; then
  echo "error: getStake() returned unexpected value"
  echo "$GET_STAKE_OUT"
  exit 1
fi

echo "✅ neoxp compound assignment smoke test passed"
