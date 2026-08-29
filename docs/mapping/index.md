# Semantic Mapping

Neo DevPack for Solidity accepts Solidity syntax and lowers it to Neo N3 contracts. This
section splits the EVM-to-Neo behavior map by topic so readers do not have to
scan the language, internals, devpack, and standards references on one page.

## Reading Path

| Topic | Start Here | Use When |
| --- | --- | --- |
| Execution context | [Execution Context](/mapping/execution-context) | You need `msg`, `tx`, `block`, `this`, or `gasleft()` behavior. |
| Types | [Types and Values](/mapping/types-and-values) | You are checking how Solidity values become NeoVM stack items or ABI values. |
| Storage | [Storage and Mappings](/mapping/storage-and-mappings) | You are using state variables, mappings, arrays, structs, or upgradeable storage. |
| Calls and assets | [Calls and Assets](/mapping/calls-and-assets) | You are porting Ether transfers, `address.call`, callbacks, or token flows. |
| Syscalls | [Syscalls and Devpack](/mapping/syscalls-and-devpack) | You want to know which Neo syscall or devpack wrapper backs a Solidity construct. |
| Parity | [Parity and Limitations](/mapping/parity-and-limitations) | You need to distinguish exact, approximate, warning-only, and unsupported behavior. |
| Standards | [Standards Mapping](/mapping/standards) | You are porting ERC/EIP patterns to NEP standards or manifest capabilities. |
| Compiler lowering | [Indexed Storage Lowering](/mapping/indexed-storage-lowering) | You are working on mapping/indexed-storage compiler implementation details. |

## Mapping Rule of Thumb

Neo DevPack for Solidity preserves Solidity source compatibility where it can, but the
runtime target is Neo N3. Exact mappings compile silently. Approximate mappings
compile with diagnostics. Unsupported mappings are rejected or intentionally
blocked when Neo cannot provide the EVM security or state semantics.

## Canonical References

- [Units and Globally Available Variables](/language-description/units-and-global-variables)
- [Types](/language-description/types)
- [Layout of State Variables in Storage](/internals/layout-in-storage)
- [Syscalls](/internals/syscalls)
- [Parity and Limitations](/internals/parity-and-limitations)
- [Devpack Overview](/additional-material/neo-devpack)
- [Standards and Contracts](/additional-material/neo-standards)
