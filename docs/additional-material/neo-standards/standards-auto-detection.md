---
title: "Neo Standards Auto-Detection"
description: "Standards Auto-Detection from Standards and Contracts."
---

# Neo Standards Auto-Detection

[Back to Standards and Contracts](/additional-material/neo-standards)

The `neo-solc` compiler analyzes your contract's public methods and events to automatically detect which NEP standards it implements. Detection results populate the manifest's `supportedstandards` array.

::: warning Detection is not full compliance
Auto-detection checks method names plus the `transfer` signature and
`Transfer` event shape, and emits diagnostics for near misses. A production
contract should still be checked against the canonical NEP ABI, parameter
types, authorization rules, and receiver-callback behavior.
:::

## NEP-17 Detection

All 5 methods must be present as public/external functions:

```
symbol, decimals, totalSupply, balanceOf, transfer
```

Additional rules:

- `ownerOf` must **not** be present (its presence signals NEP-11 instead)
- A `Transfer` event with 3 parameters must be declared
- A `transfer` overload with 4 parameters `(from, to, amount, data)` must exist
- **Conformance warning**: if all 5 names are present but the `transfer`
  signature or `Transfer` event does not conform, the standard is NOT
  advertised and a warning explains what to fix
- **Near-miss warning**: if 3+ of 5 methods are present but not all, the compiler emits a warning listing the missing methods

For strict NEP-17 compliance, the `transfer` method returns `bool`, the `data`
parameter maps to Neo ABI `Any`, and token contracts call
`onNEP17Payment(from, amount, data)` after a successful transfer to a deployed
contract.

## NEP-11 Detection

All 7 mandatory NEP-11 methods must be present as public/external functions:

```
symbol, decimals, totalSupply, balanceOf, tokensOf, ownerOf, transfer
```

Additional rules:

- A `Transfer` event with 4 parameters must be declared
- A `transfer` overload with 3 parameters `(to, tokenId, data)` must exist
- **Conformance warning**: if all names are present but the `transfer`
  signature or `Transfer` event does not conform, the standard is NOT
  advertised and a warning explains what to fix
- **Near-miss warning**: `ownerOf` (a strong NFT signal) with an incomplete
  method set emits a warning listing the missing methods. Note that
  `transferFrom` is an ERC-721 method with no NEP-11 significance — an
  ERC-721-shaped contract is not advertised as NEP-11

For strict NEP-11 compliance, also check the common methods (`symbol`,
`decimals`, `totalSupply`, `balanceOf`, `tokensOf`, `transfer`), the
appropriate indivisible or divisible ownership methods, and receiver callback
behavior. `properties(tokenId)` is optional in the NEP-11 specification but is
included in the devpack's full NFT interface for metadata-rich tokens.

## NEP-24 Detection

Detection requires the standard's single mandatory method with its
3-parameter signature (method matching is case-insensitive):

- `royaltyInfo(tokenId, royaltyToken, salePrice)`

A `royaltyInfo` with a different arity produces an informational diagnostic
instead of a detection. `tokenURI` / `tokenUri` is ERC-721 metadata and does
NOT trigger NEP-24 — advertising the royalty standard without `royaltyInfo`
would make marketplaces call a nonexistent method.

For strict NEP-24 compliance, `royaltyInfo(tokenId, royaltyToken, salePrice)`
returns an array of `[royaltyRecipient, royaltyAmount]` pairs. The
`royaltyAmount` values are final amounts for the provided `salePrice`; basis
points are only an internal configuration pattern used by many implementations.

## Additional NEP Detection

The compiler also auto-detects these contract-lifecycle and callback standards:

- `NEP-22`: `update(nefFile, manifest, data)`
- `NEP-26`: `onNEP11Payment(from, amount, tokenId, data)`
- `NEP-27`: `onNEP17Payment(from, amount, data)`
- `NEP-29`: `_deploy(data, update)`
- `NEP-30`: `verify(...) -> bool`
- `NEP-31`: `destroy()`

## Explicit Manifest Declarations

Contracts can also declare standards via
`@custom:neo.manifest.supportedstandards`. Explicit declarations are validated
more strictly for NEP-17 and NEP-11: missing required methods or the wrong
`Transfer` event arity are treated as manifest errors instead of advisory
diagnostics.

::: tip Checking Detection Results
After compilation, inspect the manifest to verify detected standards:

```bash
neo-solc MyToken.sol -I devpack -O2 -o build/MyToken
cat build/MyToken/MyToken.manifest.json | jq '.supportedstandards'
# Expected: ["NEP-17"]
```

:::
