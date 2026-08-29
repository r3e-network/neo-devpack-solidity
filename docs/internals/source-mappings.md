# Source Mappings

As part of the AST output, the Solidity compiler provides the range of the source code that is represented by the respective node in the AST. This can be used for various purposes ranging from static analysis tools that report errors based on the AST and debugging tools that highlight local variables and their uses.

## Source Mappings on NeoVM

::: tip 💡 NeoVM Difference
In EVM, standard Solidity generates a compressed `srcmap` string mapping EVM bytecode offsets back to source positions. **NeoVM integrations use explicit debug information structures rather than EVM `srcmap` strings.**
:::

Neo DevPack for Solidity has source-map/debug internals for downstream tooling, but `neo-solc` does not currently emit source-map or debug-info artifacts as standalone CLI output formats. Tooling that needs debugger integration should consume compiler internals or package-level debug/source-map types and validate the resulting artifacts with Neo-Express or Neo Toolkit.

Debugger integrations should package those internal mappings into the format expected by their NeoVM toolchain and validate the resulting stepping behavior against Neo-Express or Neo Toolkit.
