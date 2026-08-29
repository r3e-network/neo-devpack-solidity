# Style Guide

Neo DevPack for Solidity adheres perfectly to the official Ethereum Solidity Style Guide. Because the language syntax is identical, all naming conventions, code layouts, and NatSpec guidelines apply equally to Neo DevPack for Solidity development.

For the definitive guide on styling your smart contracts, please see the [official Solidity Style Guide](https://docs.soliditylang.org/en/latest/style-guide.html).

## Notable Neo Considerations

While the syntax style remains identical, consider these architectural layout practices when building specifically for Neo N3:

### 1. Interface Placements
Because Neo utilizes manifest-based ABI specifications, ensure that your interfaces (especially ones using `@custom:neo.manifest.permissions`) are declared prominently at the top of your files or in dedicated `Interfaces.sol` files to make security auditing of dynamic calls easier.

### 2. Standard Imports
Always group your Neo-specific intrinsic imports (`Runtime`, `NativeCalls`, `Storage`) at the top of the import block to clearly delineate Neo-native capabilities from standard EVM logic.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

// Neo Intrinsics
import {Runtime} from "@neo/Runtime.sol";
import {NativeCalls} from "@neo/NativeCalls.sol";

// Local logic
import "./MyLib.sol";
```
