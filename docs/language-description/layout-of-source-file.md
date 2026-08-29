# Layout of a Solidity Source File

Source files can contain an arbitrary number of contract definitions, import directives, pragma directives and struct/enum/function/error/constant variable definitions.

## SPDX License Identifier

Trust in smart contract can be better established if their source code is available. Since making source code available always touches on legal problems with regards to copyright, the Solidity compiler encourages the use of machine-readable SPDX license identifiers. Every source file should start with a comment indicating its license:

```solidity
// SPDX-License-Identifier: MIT
```

## Pragmas

The `pragma` keyword is used to enable certain compiler features or checks. A pragma directive is always local to a source file, so you have to add the pragma to all your files if you want enable it in all of your project.

### Version Pragma

Source files can (and should) be annotated with a version pragma to reject compilation with future compiler versions that might introduce incompatible changes. Neo DevPack for Solidity parses these pragmas exactly like mainline Solidity:

```solidity
pragma solidity ^0.8.20;
```

## Importing other Source Files

Neo DevPack for Solidity supports import statements that are very similar to those available in JavaScript (from ES6 on). 

```solidity
import "filename";
```

### Path Resolution

Neo DevPack for Solidity resolves imports relative to the current directory or relative to defined include paths (passed via `-I`). Absolute URL imports are not currently supported by the local compiler.

```solidity
// Import from standard libraries installed in the devpack
import {Runtime} from "@neo/Runtime.sol";
import {NativeCalls} from "@neo/NativeCalls.sol";
```

## Comments

Single-line comments (`//`) and multi-line comments (`/* ... */`) are possible.

```solidity
// This is a single-line comment.

/*
This is a
multi-line comment.
*/
```

### NatSpec Comments

Additionally, there is another type of comment called a NatSpec comment, which the Neo DevPack for Solidity compiler uses for manifest permission overrides and interface generation:

```solidity
/// @title My Token Contract
/// @notice Implements NEP-17
/// @custom:neo.manifest.permissions {"contract": "*", "methods": "*"}
contract MyToken {
    // ...
}
```
