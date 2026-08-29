#!/usr/bin/env bash
# Smoke test: SimpleDAO — compile, deploy, invoke on Neo Express.
#
# Verifies:
#   - Compilation to NEF + manifest
#   - Deployment on neo-express
#   - QUORUM_PERCENT() returns 20
#   - totalStaked() starts at 0
#   - proposalCount() starts at 0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-sol-dao.XXXXXX")"
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
  if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then
    echo "$ROOT_DIR/build/dotnet-tools/neoxp"; return
  fi
  if command -v neoxp >/dev/null 2>&1; then echo "neoxp"; return; fi
  if command -v dotnet >/dev/null 2>&1; then
    echo "(info) Installing Neo-Express (neoxp)..." >&2
    mkdir -p "$ROOT_DIR/build/dotnet-tools"
    dotnet tool install Neo.Express \
      --tool-path "$ROOT_DIR/build/dotnet-tools" >/dev/null
    if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then
      echo "$ROOT_DIR/build/dotnet-tools/neoxp"; return
    fi
  fi
  echo "error: neoxp not found" >&2; exit 1
}

command -v jq >/dev/null 2>&1 || { echo "error: jq required" >&2; exit 1; }

NEO_SOLC_BIN="$(resolve_neo_solc)"
NEOXP_BIN="$(resolve_neoxp)"
NEOXP_HOME="$WORK_DIR/neoxp-home"
mkdir -p "$NEOXP_HOME"
run_neoxp() { HOME="$NEOXP_HOME" "$NEOXP_BIN" "$@"; }

echo "(info) Work dir: $WORK_DIR"
echo "(info) Compiler: $NEO_SOLC_BIN"
cd "$WORK_DIR"

# Compile
"$NEO_SOLC_BIN" "$ROOT_DIR/examples/famous/SimpleDAO.sol" \
  -I "$ROOT_DIR/devpack" -O2 -o SimpleDAO >/dev/null

# Create chain, fund, deploy
CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 1000 GAS genesis node1 >/dev/null
CONTRACT_HASH="$(run_neoxp contract hash \
  -i "$CHAIN" SimpleDAO.nef node1 | tr -d '\r')"
echo "(info) Contract hash: $CONTRACT_HASH"
run_neoxp contract deploy -i "$CHAIN" SimpleDAO.nef node1 -j >/dev/null

# Test: QUORUM_PERCENT() == 20
cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "QUORUM_PERCENT", "args": [] }
JSON
OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke.json node1)"
if [ "$(echo "$OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: QUORUM_PERCENT() did not HALT"; echo "$OUT"; exit 1
fi
VAL="$(echo "$OUT" | jq -r '.stack[0].value')"
if [ "$VAL" != "20" ]; then
  echo "error: QUORUM_PERCENT() expected 20, got '$VAL'"; exit 1
fi
echo "(ok) QUORUM_PERCENT() = 20"

# Test: totalStaked() == 0
cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "totalStaked", "args": [] }
JSON
OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke.json node1)"
if [ "$(echo "$OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: totalStaked() did not HALT"; echo "$OUT"; exit 1
fi
VAL="$(echo "$OUT" | jq -r '.stack[0].value')"
if [ "$VAL" != "0" ]; then
  echo "error: totalStaked() expected 0, got '$VAL'"; exit 1
fi
echo "(ok) totalStaked() = 0"

# Test: proposalCount() == 0
cat > invoke.json <<JSON
{ "contract": "$CONTRACT_HASH", "operation": "proposalCount", "args": [] }
JSON
OUT="$(run_neoxp contract invoke -r -j -i "$CHAIN" invoke.json node1)"
if [ "$(echo "$OUT" | jq -r '.state')" != "HALT" ]; then
  echo "error: proposalCount() did not HALT"; echo "$OUT"; exit 1
fi
VAL="$(echo "$OUT" | jq -r '.stack[0].value')"
if [ "$VAL" != "0" ]; then
  echo "error: proposalCount() expected 0, got '$VAL'"; exit 1
fi
echo "(ok) proposalCount() = 0"

echo "✅ SimpleDAO smoke test passed"
