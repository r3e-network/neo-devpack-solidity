# Breaking Changes

The official Solidity language evolves through major and minor version bumps that occasionally introduce breaking changes.

::: tip 💡 NeoVM Difference: Tracking 0.8.x
The `neo-devpack-solidity` compiler natively targets the **Solidity 0.8.x** feature set. Therefore, all historical breaking changes introduced in Solidity v0.5.0, v0.6.0, v0.7.0, and v0.8.0 are inherently enforced by the Neo DevPack for Solidity compiler frontend.
:::

## Overview of Enforced 0.8.x Semantics

When compiling Ethereum contracts for Neo N3, you must ensure your contracts already comply with Solidity 0.8.x standards. The compiler enforces the following historical Ethereum breaking changes automatically:

### SafeMath by Default (v0.8.0)

Arithmetic operations revert on underflow and overflow outside `unchecked` blocks. NeoVM evaluates integer operations with `BigInteger` internally, then Neo DevPack for Solidity enforces Solidity 0.8 fixed-width boundaries. Inside `unchecked`, supported fixed-width arithmetic suppresses those overflow guards and wraps. Division by zero and modulo by zero still revert.

### Explicit Conversions (v0.8.0)

Explicit conversions from negative literals and literals larger than `type(uint160).max` to `address` are disallowed.

### Getter Visibilities (v0.7.0)

Functions cannot have the same name as public state variables. The Neo DevPack for Solidity compiler uses the exact same `solang-parser` rules to enforce this.

### `fallback` vs `receive` (v0.6.0)

The unnamed function was split into `receive()` and `fallback()`. On Neo N3, however, **both concepts are replaced by the `onNEP17Payment` method**, which is required to receive NEP-17 tokens (including GAS). The compiler will emit a diagnostic warning if you attempt to use EVM `receive()` or `fallback()` functions.

### Explicit Data Locations (v0.5.0)

Explicit data location for all variables of struct, array or mapping types is now mandatory. This includes function parameters and return variables. Remember that on NeoVM, `calldata` behaves exactly identically to `memory`.

## Neo DevPack for Solidity Versioning

While `neo-devpack-solidity` tracks the upstream `0.8.x` Solidity syntax, the compiler itself has its own versioning (e.g., `0.18.0`).

Breaking changes introduced directly to the EVM-to-NeoVM semantic mapping layer (such as changing how `msg.sender` behaves or modifying implicit `address.transfer` warnings) will be documented in the [Releases](https://github.com/r3e-network/neo-devpack-solidity/releases) page of the GitHub repository.
