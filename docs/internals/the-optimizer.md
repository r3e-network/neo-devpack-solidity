# The Optimizer

The Neo DevPack for Solidity compiler includes a multi-stage Intermediate Representation (IR) optimizer designed to reduce the size and execution cost of the compiled NeoVM script.

By default, optimizations are enabled when compiling through standard workflows, but can be controlled explicitly using the `-O` flag.

## Optimizer Parameter Runs

You can specify the optimization level from `0` to `3`.

```bash
# Disable optimization
neo-solc contract.sol -O0

# Maximum optimization
neo-solc contract.sol -O3
```

## Optimization Levels

### O0 (Disabled)
No optimization passes are executed. The IR is lowered directly to NeoVM opcodes. This is useful for debugging structural compiler issues.

### O1 (Basic)
Applies fundamental passes that improve the structure without significantly altering the code flow:
* **Dead Code Elimination (DCE):** Removes instructions that are never executed (e.g., code following a `return` or `revert`).
* **Unreachable Block Elimination:** Removes basic blocks that have no incoming jumps.
* **Basic Peephole:** Flattens redundant pushes and drops.

### O2 (Standard)
The recommended level for most deployments.
* **Constant Folding:** Evaluates deterministic arithmetic and logical expressions at compile-time.
* **Copy Propagation:** Replaces variables with their known constant values.
* **Inlining (Simple):** Inlines very small internal function calls to remove `CALL` and `RET` overhead.

### O3 (Aggressive)
Maximum optimization focused on minimizing the `.nef` size and execution GAS.
* **Aggressive Inlining:** Inlines larger internal functions if they are called infrequently.
* **Control Flow Flattening:** Merges adjacent basic blocks and removes redundant `JMP` instructions.
* **Advanced Stack Optimization:** Reorders stack operations (`DUP`, `SWAP`) to minimize local variable load/store operations.