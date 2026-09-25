#!/usr/bin/env bash
# Neo-Express smoke for the unmodified Complete NEP-17 and NEP-11 examples.
# This is a Complete-example integration smoke, not a minimal standalone
# NEP conformance suite. It checks canonical manifest/read surfaces and stable
# state flows without patching the shipped Solidity fixtures.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-nep-smoke.XXXXXX")"
cleanup() { rm -rf "$WORK_DIR"; }
trap cleanup EXIT
resolve_neo_solc() {
  if [ -n "${NEO_SOLC:-}" ]; then echo "$NEO_SOLC"; return; fi
  if command -v neo-solc >/dev/null 2>&1; then echo neo-solc; return; fi
  echo "error: NEO_SOLC or neo-solc is required" >&2; return 1
}
resolve_neoxp() {
  if [ -n "${NEOXP:-}" ]; then echo "$NEOXP"; return; fi
  if [ -x "$ROOT_DIR/build/dotnet-tools/neoxp" ]; then echo "$ROOT_DIR/build/dotnet-tools/neoxp"; return; fi
  if command -v neoxp >/dev/null 2>&1; then echo neoxp; return; fi
  echo "error: NEOXP or neoxp is required" >&2; return 1
}
command -v jq >/dev/null 2>&1 || { echo "error: jq is required" >&2; exit 1; }
NEO_SOLC_BIN="$(resolve_neo_solc)"
NEOXP_BIN="$(resolve_neoxp)"
NEOXP_HOME="$WORK_DIR/neoxp-home"
mkdir -p "$NEOXP_HOME"
run_neoxp() { HOME="$NEOXP_HOME" "$NEOXP_BIN" "$@"; }
cd "$WORK_DIR"
# Preserve the shipped devpack/examples relative-import layout: Complete*.sol
# import ../standards, ../contracts, and ../libraries. Copying the complete
# devpack tree keeps those paths intact without modifying the source fixtures.
cp -R "$ROOT_DIR/devpack" "$WORK_DIR/devpack"
mkdir -p "$WORK_DIR/examples"
cp "$ROOT_DIR/examples/NEPStandardsE2ESmoke.sol" "$WORK_DIR/examples/NEPStandardsE2ESmoke.sol"
"$NEO_SOLC_BIN" "$WORK_DIR/devpack/examples/CompleteNEP17Token.sol" -I "$WORK_DIR/devpack" -o CompleteNEP17Token >/dev/null
"$NEO_SOLC_BIN" "$WORK_DIR/devpack/examples/CompleteNEP11NFT.sol" -I "$WORK_DIR/devpack" -o CompleteNEP11NFT >/dev/null
"$NEO_SOLC_BIN" "$WORK_DIR/examples/NEPStandardsE2ESmoke.sol" -I "$WORK_DIR/devpack" -o NEPStandardsE2ESmoke >/dev/null
find_artifact() {
  local prefix="$1"
  local contract="$2"
  local suffix="$3"
  local artifact_name="${prefix}-${contract}${suffix}"
  local matches=()
  mapfile -t matches < <(find "$WORK_DIR" -maxdepth 1 -type f -name "$artifact_name" -print | sort)
  if [ "${#matches[@]}" -ne 1 ]; then
    echo "error: expected exactly one $artifact_name artifact, found ${#matches[@]}" >&2
    printf '%s\n' "${matches[@]}" >&2
    exit 1
  fi
  printf '%s\n' "${matches[0]}"
}
NEP17_NEF="$(find_artifact CompleteNEP17Token CompleteNEP17Token .nef)"
NEP17_MANIFEST="$(find_artifact CompleteNEP17Token CompleteNEP17Token .manifest.json)"
NEP11_NEF="$(find_artifact CompleteNEP11NFT CompleteNEP11NFT .nef)"
NEP11_MANIFEST="$(find_artifact CompleteNEP11NFT CompleteNEP11NFT .manifest.json)"
WRAPPER17_NEF="$(find_artifact NEPStandardsE2ESmoke NEP17E2ESmoke .nef)"
WRAPPER17_MANIFEST="$(find_artifact NEPStandardsE2ESmoke NEP17E2ESmoke .manifest.json)"
WRAPPER11_NEF="$(find_artifact NEPStandardsE2ESmoke NEP11E2ESmoke .nef)"
WRAPPER11_MANIFEST="$(find_artifact NEPStandardsE2ESmoke NEP11E2ESmoke .manifest.json)"
for manifest in "$NEP17_MANIFEST" "$NEP11_MANIFEST"; do
  test -s "$manifest"
  jq -e '.abi.methods | length > 0' "$manifest" >/dev/null
  jq -e '.supportedstandards | length > 0' "$manifest" >/dev/null
done
jq -e '.name == "CompleteNEP17Token"' "$NEP17_MANIFEST" >/dev/null
jq -e '.name == "CompleteNEP11NFT"' "$NEP11_MANIFEST" >/dev/null
NEP17_COMPLETE_SIZE="$(wc -c < "$NEP17_NEF")"
NEP11_COMPLETE_SIZE="$(wc -c < "$NEP11_NEF")"
echo "Complete fixture NEF sizes: NEP17=${NEP17_COMPLETE_SIZE}, NEP11=${NEP11_COMPLETE_SIZE}"
jq -e '.supportedstandards | any(. == "NEP-17")' "$NEP17_MANIFEST" >/dev/null
jq -e '.supportedstandards | any(. == "NEP-11")' "$NEP11_MANIFEST" >/dev/null
jq -e 'any(.abi.methods[]; .name == "symbol" and .parameters == [] and .returntype == "String") and any(.abi.methods[]; .name == "decimals" and .parameters == [] and .returntype == "Integer") and any(.abi.methods[]; .name == "totalSupply" and .parameters == [] and .returntype == "Integer") and any(.abi.methods[]; .name == "balanceOf" and (.parameters | length) == 1 and .returntype == "Integer")' "$NEP17_MANIFEST" >/dev/null
jq -e 'any(.abi.methods[]; .name == "decimals" and .parameters == [] and .returntype == "Integer") and any(.abi.methods[]; .name == "totalSupply" and .parameters == [] and .returntype == "Integer") and any(.abi.methods[]; .name == "ownerOf" and (.parameters | length) == 1 and .returntype == "Hash160") and any(.abi.methods[]; .name == "transfer" and (.parameters | length) == 3)' "$NEP11_MANIFEST" >/dev/null
# The wrapper is the deployable E2E artifact; Complete examples above remain
# compile/manifest fixtures because their generated scripts exceed NEF limits.
jq -e '.name == "NEP17E2ESmoke"' "$WRAPPER17_MANIFEST" >/dev/null
jq -e '.name == "NEP11E2ESmoke"' "$WRAPPER11_MANIFEST" >/dev/null
jq -e '.supportedstandards | any(. == "NEP-17")' "$WRAPPER17_MANIFEST" >/dev/null
jq -e '.supportedstandards | any(. == "NEP-11")' "$WRAPPER11_MANIFEST" >/dev/null
CHAIN="$WORK_DIR/chain.neo-express"
run_neoxp create -f -o "$CHAIN" >/dev/null
run_neoxp transfer -i "$CHAIN" 100 GAS genesis node1 >/dev/null
NEP17_DEPLOY="$(run_neoxp contract deploy -i "$CHAIN" "$WRAPPER17_NEF" node1 -j)"
NEP17_HASH="$(echo "$NEP17_DEPLOY" | jq -r '."contract-hash"')"
NEP17_TX="$(echo "$NEP17_DEPLOY" | jq -r '."tx-hash"')"
test -n "$NEP17_HASH"; test "$NEP17_HASH" != null; test -n "$NEP17_TX"; test "$NEP17_TX" != null
NEP17_LOG="$(run_neoxp show transaction -i "$CHAIN" "$NEP17_TX")"
test "$(echo "$NEP17_LOG" | jq -r '."application-log".executions[0].vmstate')" = HALT
cat > nep17-symbol.neo-invoke.json <<JSON
{"contract":"$NEP17_HASH","operation":"symbol","args":[]}
JSON
NEP17_READ="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep17-symbol.neo-invoke.json node1)"
test "$(echo "$NEP17_READ" | jq -r '.state')" = HALT
echo "$NEP17_READ" | jq -e '.stack | length > 0' >/dev/null
cat > nep17-decimals.neo-invoke.json <<JSON
{"contract":"$NEP17_HASH","operation":"decimals","args":[]}
JSON
NEP17_DECIMALS="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep17-decimals.neo-invoke.json node1)"
test "$(echo "$NEP17_DECIMALS" | jq -r '.state')" = HALT
echo "$NEP17_DECIMALS" | jq -e '.stack[0].value == "8"' >/dev/null
cat > nep17-supply.neo-invoke.json <<JSON
{"contract":"$NEP17_HASH","operation":"totalSupply","args":[]}
JSON
NEP17_SUPPLY="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep17-supply.neo-invoke.json node1)"
test "$(echo "$NEP17_SUPPLY" | jq -r '.state')" = HALT
echo "$NEP17_SUPPLY" | jq -e '.stack[0].value == "1000000"' >/dev/null
# The deployer account is the deterministic node1 wallet; query its script hash
# from Neo-Express rather than hard-coding an address.
NODE1_ADDRESS="$(run_neoxp wallet list -j -i "$CHAIN" | jq -r '.node1[0]["script-hash"]')"
test -n "$NODE1_ADDRESS"; test "$NODE1_ADDRESS" != null
cat > nep17-balance.neo-invoke.json <<JSON
{"contract":"$NEP17_HASH","operation":"balanceOf","args":["$NODE1_ADDRESS"]}
JSON
NEP17_BALANCE="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep17-balance.neo-invoke.json node1)"
test "$(echo "$NEP17_BALANCE" | jq -r '.state')" = HALT
echo "$NEP17_BALANCE" | jq -e '.stack[0].value == "1000000"' >/dev/null
NEP11_DEPLOY="$(run_neoxp contract deploy -i "$CHAIN" "$WRAPPER11_NEF" node1 -j)"
NEP11_HASH="$(echo "$NEP11_DEPLOY" | jq -r '."contract-hash"')"
NEP11_TX="$(echo "$NEP11_DEPLOY" | jq -r '."tx-hash"')"
test -n "$NEP11_HASH"; test "$NEP11_HASH" != null; test -n "$NEP11_TX"; test "$NEP11_TX" != null
NEP11_LOG="$(run_neoxp show transaction -i "$CHAIN" "$NEP11_TX")"
test "$(echo "$NEP11_LOG" | jq -r '."application-log".executions[0].vmstate')" = HALT
cat > nep11-decimals.neo-invoke.json <<JSON
{"contract":"$NEP11_HASH","operation":"decimals","args":[]}
JSON
NEP11_DECIMALS="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep11-decimals.neo-invoke.json node1)"
test "$(echo "$NEP11_DECIMALS" | jq -r '.state')" = HALT
echo "$NEP11_DECIMALS" | jq -e '.stack[0].value == "0"' >/dev/null
cat > nep11-supply.neo-invoke.json <<JSON
{"contract":"$NEP11_HASH","operation":"totalSupply","args":[]}
JSON
NEP11_SUPPLY="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep11-supply.neo-invoke.json node1)"
test "$(echo "$NEP11_SUPPLY" | jq -r '.state')" = HALT
echo "$NEP11_SUPPLY" | jq -e '.stack[0].value == "0"' >/dev/null
cat > nep11-stats.neo-invoke.json <<JSON
{"contract":"$NEP11_HASH","operation":"getCollectionStats","args":[]}
JSON
NEP11_READ="$(run_neoxp contract invoke -r -j -i "$CHAIN" nep11-stats.neo-invoke.json node1)"
test "$(echo "$NEP11_READ" | jq -r '.state')" = HALT
echo "$NEP11_READ" | jq -e '.stack | length > 0' >/dev/null
echo "NEP standards Neo-Express smoke passed"
