# Keyword Index

This page serves as a reference for all keywords reserved in the Neo DevPack for Solidity compiler. Because Neo DevPack for Solidity uses the upstream `solang-parser`, it reserves the exact same keywords as mainline Ethereum Solidity.

## Reserved Keywords

The following keywords are reserved and cannot be used as identifiers (e.g., variable or function names):

*   `after`
*   `alias`
*   `apply`
*   `auto`
*   `byte`
*   `case`
*   `catch`
*   `copyof`
*   `default`
*   `define`
*   `final`
*   `implements`
*   `in`
*   `inline`
*   `let`
*   `macro`
*   `match`
*   `mutable`
*   `null`
*   `of`
*   `partial`
*   `promise`
*   `reference`
*   `relocatable`
*   `sealed`
*   `sizeof`
*   `static`
*   `supports`
*   `switch`
*   `try`
*   `typedef`
*   `typeof`
*   `unchecked`

::: tip 💡 NeoVM Difference: Parsed Keywords
Keywords like `unchecked` and `assembly` are fully parsed by the frontend, but they do not have the same implementation status. `unchecked` is honored for Solidity 0.8 arithmetic overflow semantics, while `assembly` lowers a limited Yul subset and warns when unsupported EVM-only operations cannot be emitted.
:::
