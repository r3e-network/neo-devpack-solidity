---
title: "Architecture: Module Organization"
description: "Module Organization from Architecture."
---

# Module Organization

[Back to Architecture](/internals/architecture)

## Source Tree

```
src/
├── main.rs                     # CLI entry point
├── lib.rs                      # Library root, public API exports
├── cli/                        # Command-line interface
│   ├── mod.rs                  # CLI orchestration
│   ├── bytecode/               # NeoVM bytecode generation (Stage 7)
│   ├── standard_json/          # Standard JSON interface
│   └── cli_parts/              # CLI components
│       ├── cli_run/            # Compilation orchestration, CLI args
│       ├── cli_compile/        # Compile logic, permissions, error types
│       └── cli_manifest/       # Manifest generation and permission inference
├── frontend.rs                 # Solang parser integration (Stage 1)
├── solidity.rs                 # Solidity metadata extraction (Stage 2)
├── semantic.rs                 # Semantic analysis (Stage 3)
├── semantic_model.rs           # Typed semantic model (Stage 4)
├── type_system/                # Type system definitions
├── ir/                         # Intermediate representation (Stage 5)
│   ├── ir_types.rs             # IR type definitions
│   ├── context/                # IR lowering context
│   ├── expressions/            # Expression lowering
│   └── statements/             # Statement lowering
├── optimizer.rs                # Optimizer entry point (Stage 6)
├── optimizer/                  # Optimization passes
│   ├── types.rs                # Optimizer types and config
│   ├── dispatch.rs             # Pass dispatch logic
│   ├── constant_folding.rs     # Constant folding pass
│   ├── dead_code.rs            # Dead code elimination
│   ├── inlining.rs             # Function inlining
│   ├── cse.rs                  # Common subexpression elimination
│   ├── strength.rs             # Strength reduction
│   ├── loops.rs                # Loop optimizations
│   ├── gas.rs                  # Gas-aware optimizations
│   └── stats.rs                # Optimization statistics
├── codegen.rs                  # Code generation utilities
├── lexer.rs                    # Yul lexer
├── parser.rs                   # Yul parser
├── neo.rs                      # Neo-specific utilities (Stage 8)
├── storage_key.rs              # Storage key handling
├── error.rs                    # Error types and diagnostic builder
├── warning.rs                  # Warning system
└── runtime/                    # Embedded NeoVM runtime
    ├── execution/              # Execution engine
    ├── spec.rs                 # Opcode/syscall/native contract specs
    ├── spec/                   # Spec data files
    │   ├── opcodes.rs          # Full NeoVM opcode table
    │   ├── syscalls.rs         # Syscall registry
    │   ├── native_contracts.rs # Native contract hash table
    │   └── gas.rs              # Syscall gas cost table
    └── helpers/                # Runtime helper functions
```
