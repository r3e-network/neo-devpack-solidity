---
title: "Manifest Specification: NatSpec Manifest Overrides"
description: "NatSpec Manifest Overrides from Manifest Specification."
---

# NatSpec Manifest Overrides

[Back to Manifest Specification](/internals/contract-metadata)

The compiler supports NatSpec custom tags to override manifest fields directly from Solidity source. Both `@custom:neo.manifest.*` and `@custom:manifest.*` prefixes are recognized.

## Supported Tags

| Tag                                       | Expected Value             | Description                           |
| ----------------------------------------- | -------------------------- | ------------------------------------- |
| `@custom:neo.manifest.name`               | String or JSON string      | Override contract name                |
| `@custom:neo.manifest.groups`             | JSON array                 | Override groups array                 |
| `@custom:neo.manifest.features`           | JSON object (must be `{}`) | Override features (must remain empty) |
| `@custom:neo.manifest.supportedstandards` | JSON array                 | Override detected standards           |
| `@custom:neo.manifest.trusts`             | JSON array or `"*"`        | Override trust list                   |
| `@custom:neo.manifest.extra.<Key>`        | JSON value or plain string | Add custom extra metadata field       |

## Full Example

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title GoldToken
 * @custom:neo.manifest.name GoldToken
 * @custom:neo.manifest.supportedstandards ["NEP-17"]
 * @custom:neo.manifest.trusts ["0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"]
 * @custom:neo.manifest.extra.Author "Acme Corp"
 * @custom:neo.manifest.extra.Repository "https://github.com/acme/gold-token"
 * @custom:neo.manifest.extra.Build {"commit":"abc123","branch":"main"}
 */
contract GoldToken {
    mapping(address => uint256) private _balances;
    uint256 private _totalSupply;

    event Transfer(address indexed from, address indexed to, uint256 amount);

    function symbol() public pure returns (string memory) { return "GOLD"; }
    function decimals() public pure returns (uint256) { return 8; }
    function totalSupply() public view returns (uint256) { return _totalSupply; }
    function balanceOf(address account) public view returns (uint256) { return _balances[account]; }

    function transfer(address from, address to, uint256 amount, bytes memory data)
        public returns (bool)
    {
        require(_balances[from] >= amount, "insufficient balance");
        _balances[from] -= amount;
        _balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}
```

::: info
Values for override tags are parsed as JSON first. If JSON parsing fails, the raw string is used as-is. This means `@custom:neo.manifest.name MyToken` and `@custom:neo.manifest.name "MyToken"` both work.
:::
