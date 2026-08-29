# Solidity IR-based Codegen Changes

In mainline Ethereum Solidity, the compiler has transitioned from a direct stack-based lowering engine to an intermediate representation (Yul) based codegen pipeline.

## Neo DevPack for Solidity Pipeline

::: tip 💡 NeoVM Difference
Neo DevPack for Solidity uses its own custom Intermediate Representation (IR), which is specifically designed for the stack-based, non-linear architecture of the Neo Virtual Machine.
:::

Because NeoVM relies heavily on high-level `StackItem` objects (`Array`, `Map`, `ByteArray`) rather than linear byte manipulation, standard Yul cannot be cleanly compiled to Neo.

Instead, Neo DevPack for Solidity's compilation pipeline works as follows:
1. **Frontend:** Parses standard Solidity 0.8.x syntax into an AST using `solang-parser`.
2. **Semantic Analysis:** Builds a fully resolved type graph and validates EVM constraints.
3. **Neo IR Generation:** Lowers the AST into Neo DevPack for Solidity IR, mapping EVM concepts (like mappings or struct memory) into Neo operations (like `SHA256` key derivation or `NEWARRAY`).
4. **Optimization:** Performs dataflow analysis, constant folding, and dead code elimination on the Neo IR.
5. **NeoVM Bytecode Generation:** Emits final Neo N3 opcodes (`JMP`, `CALL`, `SYSCALL`, etc.).

This custom pipeline is why Neo DevPack for Solidity is able to bypass EVM's `256-bit` packing limitations and offer seamless large-integer and array-based struct manipulations at runtime.