---
title: "Standards and Contracts: NEP-11: Non-Fungible Tokens"
description: "NEP-11: Non-Fungible Tokens from Standards and Contracts."
---

# NEP-11: Non-Fungible Tokens

[Back to Standards and Contracts](/additional-material/neo-standards)

NEP-11 is Neo's non-fungible token standard. Compared to ERC-721, it requires
`tokensOf()`, uses ByteString-compatible token IDs (`bytes32` in the devpack
examples), and supports both indivisible and divisible (fractional ownership)
NFTs in a single standard. `properties(tokenId)` is optional in the NEP-11 spec
but included in the devpack's full NFT interface for metadata-rich contracts.

**Spec:** [NEP-11 Proposal](https://github.com/neo-project/proposals/blob/master/nep-11.mediawiki)

## Required Interface

| Method        | Parameters                                | Return     | Safe | Description                             |
| ------------- | ----------------------------------------- | ---------- | ---- | --------------------------------------- |
| `symbol`      | —                                         | `String`   | Yes  | Token symbol                            |
| `decimals`    | —                                         | `Integer`  | Yes  | `0` for indivisible, `>0` for divisible |
| `totalSupply` | —                                         | `Integer`  | Yes  | Total minted tokens                     |
| `balanceOf`   | `Hash160 owner`                           | `Integer`  | Yes  | Token count for owner                   |
| `ownerOf`     | `ByteArray tokenId`                       | `Hash160`  | Yes  | Token owner                             |
| `transfer`    | `Hash160 to, ByteArray tokenId, Any data` | `Boolean`  | No   | Transfer NFT                            |
| `tokensOf`    | `Hash160 owner`                           | `Iterator` | Yes  | Enumerate owner's tokens                |
| `properties`  | `ByteArray tokenId`                       | `Map`      | Yes  | Optional token metadata extension       |

Required event:

```
Transfer(Hash160 from, Hash160 to, Integer amount, ByteArray tokenId)
```

::: tip NEP-11 Transfer has 4 parameters
Unlike NEP-17's 3-parameter Transfer event, NEP-11 includes the `tokenId` as a fourth parameter. The `amount` field is `1` for indivisible NFTs.
:::

## Divisible vs Indivisible

NEP-11 supports two modes in a single standard:

|                    | Indivisible                   | Divisible                                   |
| ------------------ | ----------------------------- | ------------------------------------------- |
| `decimals()`       | Returns `0`                   | Returns `> 0`                               |
| `ownerOf(tokenId)` | Single `Hash160`              | Iterator of `Hash160` (multiple owners)     |
| `transfer`         | `transfer(to, tokenId, data)` | `transfer(from, to, amount, tokenId, data)` |
| Use case           | Unique collectibles, art      | Fractional real estate, shared assets       |

The checked-in `NEP11` contract implements the indivisible owner model. The
devpack also provides the `INEP11Divisible` interface so fractional contracts
can expose the correct method shape while implementing their own
`(owner, tokenId) -> balance` storage:

```solidity
interface INEP11Divisible is INEP11 {
    function balanceOf(address owner, bytes32 tokenId) external view returns (uint256);
    function transfer(address from, address to, uint256 amount, bytes32 tokenId, bytes calldata data)
        external returns (bool);
    function ownersOf(bytes32 tokenId) external view returns (address[] memory);
}
```

## Token IDs

|                  | ERC-721 (Ethereum)         | NEP-11 (Neo)                                  |
| ---------------- | -------------------------- | --------------------------------------------- |
| **Type**         | `uint256`                  | `ByteString` / `ByteArray` (`bytes32` in devpack examples) |
| **Neo ABI type** | —                          | `Hash256` / `ByteArray`                       |
| **Generation**   | Sequential counter or hash | Sequential counter, hash, or arbitrary bytes  |

```solidity
// ERC-721: uint256 token IDs
function ownerOf(uint256 tokenId) public view returns (address) { ... }

// NEP-11: bytes32 token IDs
function ownerOf(bytes32 tokenId) public view returns (address) { ... }
```

## Implementation Example

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "standards/NEP11.sol";
import "libraries/Runtime.sol";

/// @title MyNFT — A production NEP-11 non-fungible token
contract MyNFT is NEP11 {
    using Runtime for *;

    constructor()
        NEP11(
            "My NFT Collection",  // name
            "MNFT",               // symbol
            0,                    // decimals (0 = indivisible)
            "https://api.example.com/nft/",  // base URI
            10000,                // max supply
            false                 // not divisible
        )
    {}

    /// @dev Mint a new NFT with metadata properties
    function mintNFT(
        address to,
        bytes32 tokenId,
        bytes memory props
    ) external onlyMinter {
        require(Runtime.checkWitness(msg.sender), "MyNFT: unauthorized");
        mint(to, tokenId, props);
    }

    /// @dev NEP-11 3-parameter transfer with witness authorization
    function transfer(
        address to,
        bytes32 tokenId,
        bytes calldata data
    ) public override returns (bool) {
        address tokenOwner = ownerOf(tokenId);
        require(Runtime.checkWitness(tokenOwner), "MyNFT: unauthorized");
        return super.transfer(to, tokenId, data);
    }

    /// @dev Required: enumerate tokens owned by address
    function tokensOf(address owner) public view override returns (bytes32[] memory) {
        return super.tokensOf(owner);
    }

    /// @dev Optional metadata extension: return token metadata
    function properties(bytes32 tokenId) public view override returns (bytes memory) {
        return super.properties(tokenId);
    }

    /// @dev Callback for receiving NEP-11 tokens
    function onNEP11Payment(
        address from,
        uint256 amount,
        bytes32 tokenId,
        bytes calldata data
    ) external {
        // Handle incoming NFT
    }
}
```

## ERC-721 Migration Checklist

1. Replace `uint256 tokenId` with a ByteString-compatible type (`bytes32` in devpack examples)
2. Replace `transferFrom(from, to, tokenId)` with `transfer(to, tokenId, data)` (3 params)
3. Remove `approve()`, `setApprovalForAll()`, `getApproved()` — use witness model
4. Add required `tokensOf(owner)` method returning token ID array
5. Add `properties(tokenId)` when the NFT needs on-chain metadata
6. Add `decimals()` returning `0` for indivisible NFTs
7. Add `onNEP11Payment(from, amount, tokenId, data)` callback for receiving NFTs
8. Verify manifest shows `supportedstandards: ["NEP-11"]`
