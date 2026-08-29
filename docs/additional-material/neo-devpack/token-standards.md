---
title: "Devpack Token Standards"
description: "Token Standards from Devpack Overview."
---

# Devpack Token Standards

[Back to Devpack Overview](/additional-material/neo-devpack)

## NEP17.sol — Fungible Tokens

Complete NEP-17 implementation with ERC-20 compatibility:

```solidity
interface INEP17 {
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address from, address to, uint256 amount, Any calldata data) external returns (bool);
}
```

Key features:

- `onNEP17Payment` callback support via `INEP17Receiver`
- `Any` type for the `data` parameter (maps to NeoVM's unconstrained StackItem)
- Mint, burn, pause, and allowance management
- Inherits `FrameworkBase` for ownership and witness control

## NEP11.sol — Non-Fungible Tokens

Complete NEP-11 implementation for indivisible NFTs, plus an interface that
documents the divisible NEP-11 shape for contracts that implement fractional
ownership manually:

```solidity
interface INEP11 {
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address owner) external view returns (uint256);
    function tokensOf(address owner) external view returns (bytes32[] memory);
    function ownerOf(bytes32 tokenId) external view returns (address);
    function transfer(address to, bytes32 tokenId, bytes calldata data) external returns (bool);
    function properties(bytes32 tokenId) external view returns (bytes memory);
}

interface INEP11Divisible is INEP11 {
    function balanceOf(address owner, bytes32 tokenId) external view returns (uint256);
    function transfer(address from, address to, uint256 amount, bytes32 tokenId, bytes calldata data) external returns (bool);
    function ownersOf(bytes32 tokenId) external view returns (address[] memory);
}
```

The checked-in `NEP11` contract implements the indivisible owner model. For
divisible NFTs, implement the extra `(owner, tokenId) -> balance` storage and
5-parameter fractional `transfer` surface explicitly.

## NEP24.sol — Royalty Standard

Minimal NEP-24 royalty mixin for NEP-11 NFTs:

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

- Per-token and default royalty rules stored as basis points internally
- `_setDefaultRoyalty()`, `_setTokenRoyalty()`, `_clearTokenRoyalty()`
- Returns empty array when no royalty is configured
- `royaltyInfo` returns final royalty amounts for the provided sale price, not percentages
