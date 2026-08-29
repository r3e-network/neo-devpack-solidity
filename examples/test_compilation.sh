#!/bin/bash
# Test script for Neo DevPack for Solidity
# Author: Jimmy <jimmy@r3e.network>
# Tests complete compilation to Neo N3 contract format

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/neo-devpack-solidity-test.XXXXXX")"

file_size_bytes() {
  local file="$1"
  if stat -f%z "$file" >/dev/null 2>&1; then
    stat -f%z "$file"
    return
  fi
  if stat -c%s "$file" >/dev/null 2>&1; then
    stat -c%s "$file"
    return
  fi
  wc -c < "$file" | tr -d ' '
}

cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

resolve_neo_solc() {
  is_stale_binary() {
    local bin="$1"
    if [ ! -x "$bin" ]; then
      return 0
    fi
    if [ "$bin" -ot "$ROOT_DIR/Cargo.lock" ] || [ "$bin" -ot "$ROOT_DIR/Cargo.toml" ]; then
      return 0
    fi
    if find "$ROOT_DIR/src" "$ROOT_DIR/crates" -type f -name '*.rs' -newer "$bin" -print -quit 2>/dev/null | grep -q .; then
      return 0
    fi
    return 1
  }

  # Force rebuild when requested.
  if [ -n "${NEO_SOLC_REBUILD:-}" ]; then
    echo "(info) NEO_SOLC_REBUILD set; rebuilding neo-solc..." >&2
    (cd "$ROOT_DIR" && cargo build --release --bin neo-solc >/dev/null)
    echo "$ROOT_DIR/target/release/neo-solc"
    return
  fi

  if [ -n "${NEO_SOLC:-}" ]; then
    echo "$NEO_SOLC"
    return
  fi

  if command -v neo-solc >/dev/null 2>&1; then
    echo "neo-solc"
    return
  fi

  if [ -x "$ROOT_DIR/target/release/neo-solc" ]; then
    # Rebuild if sources changed since the last release build.
    if is_stale_binary "$ROOT_DIR/target/release/neo-solc"; then
      echo "(info) neo-solc release binary is out of date; rebuilding..." >&2
      (cd "$ROOT_DIR" && cargo build --release --bin neo-solc >/dev/null)
    fi
    echo "$ROOT_DIR/target/release/neo-solc"
    return
  fi

  if [ -x "$ROOT_DIR/target/debug/neo-solc" ]; then
    if is_stale_binary "$ROOT_DIR/target/debug/neo-solc"; then
      echo "(info) neo-solc debug binary is out of date; rebuilding..." >&2
      (cd "$ROOT_DIR" && cargo build --bin neo-solc >/dev/null)
    fi
    echo "$ROOT_DIR/target/debug/neo-solc"
    return
  fi

  echo "(info) neo-solc not found; building it first..." >&2
  (cd "$ROOT_DIR" && cargo build --bin neo-solc >/dev/null)
  echo "$ROOT_DIR/target/debug/neo-solc"
}

NEO_SOLC_BIN="$(resolve_neo_solc)"

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for this test script"
  exit 1
fi

if ! command -v hexdump >/dev/null 2>&1; then
  echo "error: hexdump is required for this test script"
  exit 1
fi

cd "$WORK_DIR"

echo "🔨 Testing Neo DevPack for Solidity - Neo N3 Contract Generation"
echo "============================================================"
echo "(info) Using compiler: $NEO_SOLC_BIN"
echo "(info) Work dir: $WORK_DIR"

# Create a simple test contract
cat > TestContract.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract TestContract {
    uint256 private value;
    
    event ValueChanged(uint256 newValue);
    
    constructor(uint256 _initialValue) {
        value = _initialValue;
        emit ValueChanged(_initialValue);
    }
    
    function setValue(uint256 _value) public {
        value = _value;
        emit ValueChanged(_value);
    }
    
    function getValue() public view returns (uint256) {
        return value;
    }
}
EOF

echo "📄 Created test contract: TestContract.sol"

# Test 1: Default compilation (should generate .nef + .manifest.json)
echo
echo "Test 1: Default compilation (complete format)"
echo "--------------------------------------------"
"$NEO_SOLC_BIN" "$WORK_DIR/TestContract.sol" -I "$ROOT_DIR/devpack" -o TestContract

# Verify outputs
if [ -f "TestContract.nef" ]; then
    echo "✅ TestContract.nef generated successfully"
    echo "   Size: $(file_size_bytes TestContract.nef) bytes"
else
    echo "❌ TestContract.nef not found"
    exit 1
fi

if [ -f "TestContract.manifest.json" ]; then
    echo "✅ TestContract.manifest.json generated successfully"
    echo "   Size: $(file_size_bytes TestContract.manifest.json) bytes"
    
    # Validate manifest structure
    if jq -e '.abi.methods' TestContract.manifest.json > /dev/null; then
        METHOD_COUNT=$(jq '.abi.methods | length' TestContract.manifest.json)
        echo "   Methods: $METHOD_COUNT"
    fi
    
    if jq -e '.abi.events' TestContract.manifest.json > /dev/null; then
        EVENT_COUNT=$(jq '.abi.events | length' TestContract.manifest.json)
        echo "   Events: $EVENT_COUNT"
    fi
else
    echo "❌ TestContract.manifest.json not found"
    exit 1
fi

# Test 2: NEF-only format
echo
echo "Test 2: NEF-only format"
echo "----------------------"
"$NEO_SOLC_BIN" "$WORK_DIR/TestContract.sol" -I "$ROOT_DIR/devpack" -f nef -o TestOnly.nef

if [ -f "TestOnly.nef" ]; then
    echo "✅ NEF-only output generated successfully"
    echo "   Size: $(file_size_bytes TestOnly.nef) bytes"
else
    echo "❌ NEF-only output failed"
    exit 1
fi

# Test 3: Manifest-only format
echo
echo "Test 3: Manifest-only format"
echo "---------------------------"
"$NEO_SOLC_BIN" "$WORK_DIR/TestContract.sol" -I "$ROOT_DIR/devpack" -f manifest -o TestOnly.manifest.json

if [ -f "TestOnly.manifest.json" ]; then
    echo "✅ Manifest-only output generated successfully"
    echo "   Size: $(file_size_bytes TestOnly.manifest.json) bytes"
else
    echo "❌ Manifest-only output failed"
    exit 1
fi

# Test 4: JSON format with all information
echo
echo "Test 4: Complete JSON format"
echo "---------------------------"
"$NEO_SOLC_BIN" "$WORK_DIR/TestContract.sol" -I "$ROOT_DIR/devpack" -f json -o TestContract.json

if [ -f "TestContract.json" ]; then
    echo "✅ JSON output generated successfully"
    echo "   Size: $(file_size_bytes TestContract.json) bytes"
    
    # Validate JSON structure
    if jq -e '.nef' TestContract.json > /dev/null; then
        echo "   ✓ Contains NEF data"
    fi
    
    if jq -e '.manifest' TestContract.json > /dev/null; then
        echo "   ✓ Contains manifest data"
    fi
else
    echo "❌ JSON output failed"
    exit 1
fi

# Test 5: Optimized compilation
echo
echo "Test 5: Optimized compilation (-O3)"
echo "---------------------------------"
"$NEO_SOLC_BIN" "$WORK_DIR/TestContract.sol" -I "$ROOT_DIR/devpack" -O3 -o TestContractOptimized

if [ -f "TestContractOptimized.nef" ] && [ -f "TestContractOptimized.manifest.json" ]; then
    echo "✅ Optimized compilation successful"
    
    # Compare sizes
    ORIGINAL_SIZE=$(file_size_bytes TestContract.nef)
    OPTIMIZED_SIZE=$(file_size_bytes TestContractOptimized.nef)
    
    echo "   Original NEF size: $ORIGINAL_SIZE bytes"
    echo "   Optimized NEF size: $OPTIMIZED_SIZE bytes"
    
    if [ $OPTIMIZED_SIZE -le $ORIGINAL_SIZE ]; then
        echo "   ✓ Optimization reduced or maintained size"
    else
        echo "   ⚠️ Optimization increased size (may be expected for small contracts)"
    fi
else
    echo "❌ Optimized compilation failed"
    exit 1
fi

# Test 6: Validate Neo contract format
echo
echo "Test 6: Validate Neo contract format"
echo "-----------------------------------"

# Check NEF magic number
NEF_MAGIC=$(dd if=TestContract.nef bs=1 count=4 2>/dev/null | hexdump -v -e '/1 "%02x"')
if [ "$NEF_MAGIC" = "4e454633" ]; then  # "NEF3" ASCII
    echo "✅ NEF file has correct magic number (NEF3)"
else
    echo "❌ NEF file has incorrect magic number: $NEF_MAGIC"
    exit 1
fi

# Check manifest structure
if jq -e '.name' TestContract.manifest.json > /dev/null && \
   jq -e '.abi' TestContract.manifest.json > /dev/null && \
   jq -e '.permissions' TestContract.manifest.json > /dev/null; then
    echo "✅ Manifest has correct Neo N3 structure"
else
    echo "❌ Manifest structure is invalid"
    exit 1
fi

# Test 7: Real-world contract compilation
echo
echo "Test 7: Real-world ERC20 contract"
echo "--------------------------------"
if [ -f "$ROOT_DIR/examples/ERC20Token.sol" ]; then
    "$NEO_SOLC_BIN" "$ROOT_DIR/examples/ERC20Token.sol" -I "$ROOT_DIR/devpack" -O2 -o ERC20
    
    if [ -f "ERC20.nef" ] && [ -f "ERC20.manifest.json" ]; then
        echo "✅ ERC20 contract compiled successfully"
        echo "   NEF size: $(file_size_bytes ERC20.nef) bytes"
        echo "   Manifest size: $(file_size_bytes ERC20.manifest.json) bytes"
        
        # Check for ERC20 methods in manifest
        TRANSFER_METHOD=$(jq '.abi.methods[] | select(.name == "transfer")' ERC20.manifest.json)
        if [ -n "$TRANSFER_METHOD" ]; then
            echo "   ✓ Transfer method found in manifest"
        fi
        
        TRANSFER_EVENT=$(jq '.abi.events[] | select(.name == "Transfer")' ERC20.manifest.json)
        if [ -n "$TRANSFER_EVENT" ]; then
            echo "   ✓ Transfer event found in manifest"
        fi
    else
        echo "❌ ERC20 contract compilation failed"
        exit 1
    fi
fi

echo
echo "🎉 All tests passed! Neo DevPack for Solidity correctly generates:"
echo "   ✓ .nef files (Neo Executable Format)"
echo "   ✓ .manifest.json files (Contract metadata)"
echo "   ✓ Proper Neo N3 contract structure"
echo "   ✓ Ready for deployment to Neo blockchain"
echo
echo "Usage examples:"
echo "  $NEO_SOLC_BIN MyContract.sol -I $ROOT_DIR/devpack -o MyContract     # Generates MyContract.nef + MyContract.manifest.json"
echo "  neo-cli contract deploy MyContract.nef MyContract.manifest.json"
