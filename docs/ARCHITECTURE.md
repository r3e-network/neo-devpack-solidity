# Neo DevPack for Solidity Architecture

This document describes the architecture of the Neo DevPack for Solidity compiler, a Rust-based production-focused toolchain for compiling Solidity smart contracts to Neo N3 blockchain.

## Overview

The compiler translates Solidity 0.8.x smart contracts to NeoVM bytecode, producing `.nef` (Neo Executable Format) files and `.manifest.json` deployment manifests.

## High-Level Pipeline

```
Solidity source (*.sol)
    │
    ▼
Frontend (solang-parser crate)
    │  - Parse Solidity → AST/IR
    │  - Built-in Solidity diagnostics
    ▼
Solidity Metadata Extraction
    │  - Contract definitions
    │  - Function signatures and metadata
    │  - State variables and events
    │  - Natspec documentation
    ▼
Semantic Model
    │  - Symbol table and scope resolution
    │  - Type checking and validation
    │  - Storage layout validation
    ▼
IR (Intermediate Representation)
    │  - Yul-like IR with Neo-specific extensions
    │  - Stack effects, storage annotations, syscall nodes
    ▼
Optimizer Pipeline
    │  - Constant folding/propagation
    │  - Dead code elimination
    │  - Function inlining (bounded)
    │  - Stack height reduction
    │  - NeoVM-specific cleanups
    ▼
Code Generator (NeoVM)
    │  - Map IR to NeoVM opcodes
    │  - Manage call frames (INITSLOT/RET)
    │  - Emit syscalls via interop IDs
    │  - Handle control flow (labels, jumps)
    ▼
Artifact Builder
    │  - NEF writer (tokens, method table, checksum)
    │  - Manifest writer (ABI, permissions, standards)
    ▼
Output (.nef, .manifest.json)
```

## Project Structure

```
neo-devpack-solidity/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library root, exports public API
│   ├── cli/                    # Command-line interface
│   │   ├── mod.rs
│   │   ├── bytecode/           # NeoVM bytecode generation
│   │   ├── standard_json/      # Standard JSON interface
│   │   └── cli_parts/          # CLI components (compile, run, manifest, etc.)
│   ├── solidity.rs             # Solidity metadata extraction
│   ├── frontend.rs             # Solang parser integration
│   ├── ir/                     # Intermediate representation
│   │   ├── ir_types.rs         # IR type definitions
│   │   ├── context/            # IR lowering context
│   │   ├── expressions/        # Expression lowering
│   │   └── statements/         # Statement lowering
│   ├── interop.rs              # Syscall interop-ID hashing
│   ├── runtime/                # NeoVM runtime emulation
│   │   ├── execution/          # NeoVM execution engine and syscall/native dispatch
│   │   ├── bridge/             # EVM-to-NeoVM compatibility bridge helpers
│   │   ├── state/              # Accounts, snapshots, storage overlays, and query state
│   │   ├── spec/               # Opcode, gas, syscall, and native contract specifications
│   │   └── types/              # Runtime value, stack-item, and wrapper types
│   ├── neo.rs                  # Neo-specific utilities (NEF, manifest)
│   ├── storage_key.rs          # Storage key handling
│   ├── type_system/            # Type system
│   └── semantic_model.rs       # Semantic model
├── devpack/                    # Solidity libraries for Neo N3
│   ├── contracts/              # Contract interfaces
│   ├── libraries/              # Utility libraries
│   ├── standards/              # NEP standards (NEP-17, NEP-11, etc.)
│   └── examples/               # Devpack usage examples
├── tests/                      # Test suite
│   ├── runtime_*.rs            # Runtime unit tests
│   ├── e2e_compilation_tests.rs # End-to-end compilation tests
│   └── conformance_tests.rs    # Conformance tests
├── examples/                   # Example Solidity contracts
│   ├── SimpleStorage.sol
│   ├── ERC20Token.sol
│   ├── Staking.sol
│   └── ... (other examples)
├── archive/
│   ├── go_implementation/      # Archived Go reference implementation
│   └── go_tests/               # Archived Go tests
├── docs/                       # Documentation
│   ├── ARCHITECTURE.md         # This file
│   ├── RUNTIME_SPEC.md         # Runtime specification
│   └── NEO_VM_PARITY_TODO.md   # Known gaps
└── Cargo.toml                  # Rust project manifest
```

## Key Components

### 1. Frontend Integration (`src/frontend.rs`)

Uses the `solang-parser` crate for Solidity parsing:

```rust
use solang_parser::{
    parse,
    pt::{ContractDefinition, FunctionDefinition, ...},
};
```

Provides:

- Solidity AST extraction
- Type information
- Source location mapping
- Diagnostic reporting

### 2. Solidity Metadata (`src/solidity.rs`)

Extracts contract metadata from Solidity source:

- `ContractMetadata` - Complete contract metadata
- `FunctionMetadata` - Function signatures and attributes
- `NatspecDoc` - Documentation extraction
- `EventDefinition` - Event signatures

### 3. IR Layer (`src/ir/`)

Custom IR inspired by Yul with Neo-specific extensions:

```rust
pub enum Instruction {
    PushLiteral(LiteralValue),
    Call(FunctionCall),
    Syscall(SyscallName),
    StorageOp(StorageOperation),
    // ... more variants
}
```

### 4. Optimizer (`src/optimizer.rs`)

Multi-level optimization (0-3):

| Level | Description                   |
| ----- | ----------------------------- |
| -O0   | No optimization               |
| -O1   | Basic (DCE, constant folding) |
| -O2   | Standard (inlining, peephole) |
| -O3   | Aggressive (max optimization) |

### 5. Code Generator (`src/cli/bytecode/`)

Translates IR to NeoVM bytecode:

- Opcode emission
- Stack management
- Jump resolution
- Local/argument allocation

### 6. Runtime (`src/runtime/`)

Embedded Neo N3 runtime for testing:

- Broad documented opcode support with explicit rejection for unsupported opcodes
- Syscall implementations
- Storage emulation
- Event logging

### 7. Artifact Builder (`src/neo.rs`)

Generates deployment artifacts:

- NEF file format
- Manifest JSON generation
- Method token tables
- Checksum computation

## Dependencies

### Core Dependencies

- `solang-parser` - Solidity parsing
- `clap` - CLI argument parsing
- `serde`/`serde_json` - Serialization
- `sha2`, `sha3`, `ripemd` - Cryptography
- `thiserror`/`anyhow` - Error handling

### Dev Dependencies

- `criterion` - Benchmarking
- `proptest` - Property testing
- `tempfile` - Test utilities

## Build System

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test --workspace

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Known Limitations

See `docs/NEO_VM_PARITY_TODO.md` for a comprehensive list of runtime parity gaps and planned improvements.

## Archived Go Implementation

The `archive/go_implementation/` directory contains an earlier Go-based reference implementation of the compiler. This code is no longer maintained and is kept for historical reference only.

## Contributing

See `CONTRIBUTING.md` for development guidelines.

## License

MIT License - see `LICENSE` file for details.
