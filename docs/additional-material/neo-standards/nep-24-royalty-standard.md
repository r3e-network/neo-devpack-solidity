---
title: "Standards and Contracts: NEP-24: Royalty Standard"
description: "NEP-24: Royalty Standard from Standards and Contracts."
---

# NEP-24: Royalty Standard

[Back to Standards and Contracts](/additional-material/neo-standards)

NEP-24 specifies a `royaltyInfo` method for NEP-11 NFTs so marketplaces can fetch royalty recipients and amounts for secondary sales. Unlike ERC-2981 which returns a single recipient, NEP-24 supports multiple royalty recipients in a single response.

**Spec:** [NEP-24 Proposal](https://github.com/neo-project/proposals/blob/master/nep-24.mediawiki)

## Interface

| Method        | Parameters                                                   | Return                      | Description                    |
| ------------- | ------------------------------------------------------------ | --------------------------- | ------------------------------ |
| `royaltyInfo` | `ByteArray tokenId, Hash160 royaltyToken, Integer salePrice` | `Array<[Hash160, Integer]>` | Royalty recipients and amounts |

```solidity
interface INEP24Royalty {
    struct RoyaltyInfo {
        address royaltyRecipient;
        uint256 royaltyAmount;
    }

    function royaltyInfo(bytes32 tokenId, address royaltyToken, uint256 salePrice)
        external view returns (RoyaltyInfo[] memory);
}
```

## Key Differences from ERC-2981

|                         | ERC-2981 (Ethereum)             | NEP-24 (Neo)                              |
| ----------------------- | ------------------------------- | ----------------------------------------- |
| **Return type**         | Single `(address, uint256)`     | Array of `[recipient, amount]` pairs      |
| **Multiple recipients** | No — single recipient only      | Yes — split royalties natively            |
| **Royalty token**       | Implied (same as sale token)    | Explicit `royaltyToken` parameter         |
| **Returned amount**     | Absolute amount for `salePrice` | Absolute amount for `salePrice`           |
| **Interface detection** | `supportsInterface(0x2a55205a)` | Manifest `supportedstandards: ["NEP-24"]` |

::: info Royalty Token Parameter
The `royaltyToken` parameter specifies which token royalties should be paid in (e.g., GAS, a specific NEP-17 token). This is reserved for future use in the devpack's minimal implementation but enables token-specific royalty rules.
:::

::: tip Basis points are internal configuration
The NEP-24 interface returns royalty amounts, not percentages. The devpack mixin
stores royalty rules as basis points (`10000 = 100%`) and computes the final
`royaltyAmount` for each `salePrice` before returning from `royaltyInfo`.
:::

## Implementation Example

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "standards/NEP11.sol";
import "standards/NEP24.sol";
import "libraries/Runtime.sol";

/// @title RoyaltyNFT — NEP-11 NFT with NEP-24 royalties
contract RoyaltyNFT is NEP11, NEP24Royalty {

    constructor()
        NEP11("Royalty NFT", "RNFT", 0, "", 10000, false)
    {
        // Set default royalty: 5% (500 basis points) to deployer
        _setDefaultRoyalty(msg.sender, 500);
    }

    /// @dev Mint with per-token royalty override
    function mintWithRoyalty(
        address to,
        bytes32 tokenId,
        bytes memory props,
        address royaltyRecipient,
        uint96 royaltyBps
    ) external onlyMinter {
        require(Runtime.checkWitness(msg.sender), "unauthorized");
        mint(to, tokenId, props);

        // Override default royalty for this specific token
        if (royaltyRecipient != address(0) && royaltyBps > 0) {
            _setTokenRoyalty(tokenId, royaltyRecipient, royaltyBps);
        }
    }

    /// @dev Query royalty info — returns array of [recipient, amount] pairs
    /// For a sale of 100 GAS with 5% royalty: returns [(creator, 5 GAS)]
    function royaltyInfo(bytes32 tokenId, address royaltyToken, uint256 salePrice)
        public view override returns (RoyaltyInfo[] memory)
    {
        return super.royaltyInfo(tokenId, royaltyToken, salePrice);
    }
}
```

The `NEP24Royalty` abstract contract provides:

| Internal Method                             | Description                                       |
| ------------------------------------------- | ------------------------------------------------- |
| `_setDefaultRoyalty(recipient, bps)`        | Set fallback royalty for all tokens               |
| `_setTokenRoyalty(tokenId, recipient, bps)` | Override royalty for a specific token             |
| `_clearTokenRoyalty(tokenId)`               | Remove per-token override, fall back to default   |
| `_getRoyaltyRule(tokenId)`                  | Get effective royalty rule (per-token or default) |

::: warning Basis Points Validation
The devpack enforces `bps <= 10000` and `recipient != address(0)`. Setting invalid values reverts with `NEP24InvalidRoyaltyBps` or `NEP24InvalidRoyaltyRecipient`.
:::
