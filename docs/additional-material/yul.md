# Yul

Yul (previously also called JULIA or IULIA) is an intermediate language that can be compiled to bytecode for different backends. 

::: tip 💡 NeoVM Difference
In Ethereum, standard Solidity compiles down to Yul, which is then lowered into EVM bytecode. **Neo DevPack for Solidity does not use Yul.**

Because NeoVM relies heavily on high-level `StackItem` objects (`Array`, `Map`, `ByteArray`) rather than linear byte manipulation, standard Yul cannot be cleanly compiled to Neo. Instead, Neo DevPack for Solidity uses its own custom Intermediate Representation (NeoIR) specifically designed for the stack-based, non-linear architecture of the Neo Virtual Machine.
:::

## Motivation and High-level Description

Yul was designed to be highly readable, even if the control flow is complex, and to serve as a common denominator for compiling Solidity to EVM and eWASM. 

In Neo DevPack for Solidity, the intermediate representation (NeoIR) serves a similar purpose:
1. It decouples the frontend parsing (`solang-parser`) from the backend NeoVM code generator.
2. It allows for advanced, backend-independent optimization passes (like constant folding and dead code elimination) before committing to specific NeoVM opcodes.
3. It maps complex Solidity features (like `mapping` storage logic) into deterministic, structured operations (`SHA256` key derivation and `System.Storage.Get`).

## Simple Example

A simple example of how Neo DevPack for Solidity represents a storage read internally (instead of Yul):

```rust
// Neo DevPack for Solidity IR for reading a mapping
LoadMappingElement {
    state_index: 0, 
    key_types: [Address],
    element_type: Integer
}
```

This structured IR node directly instructs the NeoVM code generator to construct the appropriate `SHA256` hashing payload and invoke the `System.Storage.Get` syscall, bypassing the need for low-level memory offset management.

## Stand-Alone Usage

Unlike Yul, the Neo DevPack for Solidity IR is an internal compiler structure and is not currently intended for stand-alone usage or external text-based compilation. It exists purely within the memory of the `neo-solc` compilation pipeline.

If you wish to view the generated NeoVM opcodes before they are serialized into a `.nef` binary, you can use the assembly output format:

```bash
neo-solc MyContract.sol -f assembly
```

## Yul Optimizer

Because Neo DevPack for Solidity does not use Yul, it does not use the standard Solidity Yul Optimizer. Instead, the `neo-solc` compiler contains a custom optimizer (accessible via the `-O` flags) that operates directly on the NeoIR. See [The Optimizer](/internals/the-optimizer) for more details on these passes.