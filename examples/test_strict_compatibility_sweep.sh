#!/usr/bin/env bash
# Strict compatibility sweep for Neo DevPack for Solidity contracts.
#
# Compiles strict-safe Solidity sets with strict manifest denial flags and fails on:
# - any compilation error
# - missing .nef/.manifest.json output
#
# Notes:
# - Intentionally negative fixtures (used by test suites to validate diagnostics)
#   are excluded from this sweep.
# - Warnings are reported but do not fail the sweep by default. Set
#   STRICT_SWEEP_FAIL_ON_WARNINGS=1 to make warnings fatal.
# - STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS=1 fails only on warnings from
#   contracts outside the known warning allowlist below.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

resolve_neo_solc_once() {
  if [ -n "${NEO_SOLC:-}" ]; then
    return
  fi

  if command -v neo-solc >/dev/null 2>&1; then
    export NEO_SOLC="neo-solc"
    return
  fi

  echo "(info) Building neo-solc once for strict compatibility sweep..."
  (cd "$ROOT_DIR" && cargo build --bin neo-solc >/dev/null)
  export NEO_SOLC="$ROOT_DIR/target/debug/neo-solc"
}

resolve_neo_solc_once

echo "(info) Using compiler: $NEO_SOLC"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-strict-sweep.XXXXXX")"
trap 'rm -rf "$WORK_DIR"' EXIT

STRICT_FLAGS=(
  --deny-wildcard-permissions
  --deny-wildcard-contracts
  --deny-wildcard-methods
)

NEGATIVE_FIXTURES=(
  "examples/new/CatchPanicShowcase.sol"
  "examples/new/EvmCompatEtherUnits.sol"
  "examples/new/FixedPointError.sol"
  "examples/new/LibraryConstructorError.sol"
  "examples/new/LibraryStateVarError.sol"
)

ALLOWED_WARNING_FIXTURES=(
  "devpack/contracts/Framework.sol"
  "devpack/contracts/FrameworkBase.sol"
  "devpack/contracts/NEP17Rescue.sol"
  "devpack/contracts/NativeCalls.sol"
  "devpack/contracts/OracleService.sol"
  "devpack/standards/NEP11.sol"
  "devpack/standards/NEP17.sol"
  "devpack/standards/NEP22.sol"
  "devpack/examples/CompleteNEP11NFT.sol"
  "devpack/examples/CompleteNEP17Token.sol"
  "devpack/examples/VaultPattern.sol"
  "examples/new/Bank.sol"
  "examples/new/EvmCompatAddressCode.sol"
  "examples/new/EventIndexedShowcase.sol"
  "examples/new/EvmCompatAddressCodehash.sol"
  "examples/new/EvmCompatBlockErrors.sol"
  "examples/new/EvmCompatBlockhashError.sol"
  "examples/new/EvmCompatEncodeCalldata.sol"
  "examples/new/EVMCompatibilityShowcase.sol"
  "examples/new/EvmCompatMsgData.sol"
  "examples/new/EvmCompatMsgSig.sol"
  "examples/new/EvmCompatSelfdestructError.sol"
  "examples/new/EvmCompatTxOrigin.sol"
  "examples/new/FunctionPolishShowcase.sol"
  "examples/new/InterfaceShowcase.sol"
  "examples/new/ModifierShowcase.sol"
  "examples/new/MultiSigWalletNEP17.sol"
  "examples/new/MultiStandardToken.sol"
  "examples/new/NFT.sol"
  "examples/new/TryCatchShowcase.sol"
)

is_negative_fixture() {
  local rel="$1"
  for fixture in "${NEGATIVE_FIXTURES[@]}"; do
    if [ "$rel" = "$fixture" ]; then
      return 0
    fi
  done
  return 1
}

is_allowed_warning_fixture() {
  local rel="$1"
  for fixture in "${ALLOWED_WARNING_FIXTURES[@]}"; do
    if [ "$rel" = "$fixture" ]; then
      return 0
    fi
  done
  return 1
}

declare -a FILES=()
while IFS= read -r -d '' f; do FILES+=("$f"); done < <(find "$ROOT_DIR/devpack/contracts" -maxdepth 1 -type f -name '*.sol' -print0 | sort -z)
while IFS= read -r -d '' f; do FILES+=("$f"); done < <(find "$ROOT_DIR/devpack/standards" -maxdepth 1 -type f -name '*.sol' -print0 | sort -z)
while IFS= read -r -d '' f; do FILES+=("$f"); done < <(find "$ROOT_DIR/devpack/examples" -maxdepth 1 -type f -name '*.sol' -print0 | sort -z)
while IFS= read -r -d '' f; do FILES+=("$f"); done < <(find "$ROOT_DIR/examples/new" -maxdepth 1 -type f -name '*.sol' -print0 | sort -z)

if [ "${#FILES[@]}" -eq 0 ]; then
  echo "error: no Solidity files found for strict compatibility sweep"
  exit 1
fi

failures=0
warnings=0
skipped=0
unexpected_warnings=0
negative_checked=0
negative_expected_failures=0
negative_unexpected_passes=0
negative_missing=0

for file in "${FILES[@]}"; do
  rel="${file#"$ROOT_DIR"/}"
  if is_negative_fixture "$rel"; then
    skipped=$((skipped + 1))
    continue
  fi
  stem="${rel//\//__}"
  stem="${stem%.sol}"
  prefix="$WORK_DIR/$stem"
  out="$WORK_DIR/$stem.out"
  err="$WORK_DIR/$stem.err"

  if ! "$NEO_SOLC" "$file" -I "$ROOT_DIR/devpack" "${STRICT_FLAGS[@]}" -o "$prefix" >"$out" 2>"$err"; then
    echo "❌ strict compile failed: $rel"
    sed -n '1,80p' "$out"
    sed -n '1,80p' "$err"
    failures=$((failures + 1))
    continue
  fi

  shopt -s nullglob
  nef_files=("${prefix}"*.nef)
  manifest_files=("${prefix}"*.manifest.json)
  shopt -u nullglob

  if [ "${#nef_files[@]}" -eq 0 ] || [ "${#manifest_files[@]}" -eq 0 ]; then
    echo "❌ missing output artifacts: $rel"
    failures=$((failures + 1))
    continue
  fi

  if grep -Eiq '(^|[^a-z])(warning|compat)([^a-z]|$)' "$out" "$err"; then
    echo "⚠️  strict compile produced diagnostics: $rel"
    sed -n '1,80p' "$out" | rg -n "warning|compat" || true
    sed -n '1,80p' "$err" | rg -n "warning|compat" || true
    warnings=$((warnings + 1))
    if ! is_allowed_warning_fixture "$rel"; then
      echo "❌ unexpected warning contract: $rel"
      unexpected_warnings=$((unexpected_warnings + 1))
    fi
  fi
done

# Verify intentionally negative fixtures still fail under strict flags.
for rel in "${NEGATIVE_FIXTURES[@]}"; do
  file="$ROOT_DIR/$rel"
  if [ ! -f "$file" ]; then
    echo "❌ missing negative fixture: $rel"
    negative_missing=$((negative_missing + 1))
    continue
  fi

  negative_checked=$((negative_checked + 1))
  stem="${rel//\//__}"
  stem="${stem%.sol}"
  prefix="$WORK_DIR/${stem}.negative"
  out="$WORK_DIR/${stem}.negative.out"
  err="$WORK_DIR/${stem}.negative.err"

  if "$NEO_SOLC" "$file" -I "$ROOT_DIR/devpack" "${STRICT_FLAGS[@]}" -o "$prefix" >"$out" 2>"$err"; then
    echo "❌ negative fixture unexpectedly compiled: $rel"
    sed -n '1,80p' "$out"
    sed -n '1,80p' "$err"
    negative_unexpected_passes=$((negative_unexpected_passes + 1))
  else
    negative_expected_failures=$((negative_expected_failures + 1))
  fi
done

echo "strict_sweep_total=${#FILES[@]}"
echo "strict_sweep_skipped=$skipped"
echo "strict_sweep_failures=$failures"
echo "strict_sweep_warning_contracts=$warnings"
echo "strict_sweep_unexpected_warning_contracts=$unexpected_warnings"
echo "strict_sweep_negative_checked=$negative_checked"
echo "strict_sweep_negative_expected_failures=$negative_expected_failures"
echo "strict_sweep_negative_unexpected_passes=$negative_unexpected_passes"
echo "strict_sweep_negative_missing=$negative_missing"

if [ "$failures" -ne 0 ]; then
  exit 1
fi

if [ "$negative_unexpected_passes" -ne 0 ] || [ "$negative_missing" -ne 0 ]; then
  exit 1
fi

if [ "${STRICT_SWEEP_FAIL_ON_WARNINGS:-0}" = "1" ] && [ "$warnings" -ne 0 ]; then
  exit 1
fi

if [ "${STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS:-0}" = "1" ] && [ "$unexpected_warnings" -ne 0 ]; then
  exit 1
fi

echo "✅ strict compatibility sweep passed"
