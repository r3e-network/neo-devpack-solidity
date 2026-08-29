# Standards Mapping

Neo DevPack for Solidity maps Ethereum standards to Neo N3 at three different
layers:

- **Neo standard compliance**: the ABI shapes defined by NEP specifications.
- **Compiler detection**: the heuristic that populates manifest
  `supportedstandards` and emits near-miss diagnostics.
- **Migration examples**: deployable Solidity and Neo C# pairs in the standards
  mirror.

Keep these layers separate when auditing a contract. A contract can look
ERC-like and compile successfully without being a canonical NEP implementation.

## Standard Equivalents

| Ethereum Standard | Neo Equivalent | Canonical Migration Rule |
| --- | --- | --- |
| ERC-20 | NEP-17 | Use `transfer(from, to, amount, data)`, `Runtime.checkWitness(from)`, a 3-field `Transfer` event, and `onNEP17Payment` for token receipts. |
| ERC-721 | NEP-11 | Use `transfer(to, tokenId, data)`, `tokensOf(owner)`, and ByteString-compatible token IDs. The devpack commonly uses `bytes32`; official NEP-11 allows IDs up to 64 bytes. |
| ERC-2981 | NEP-24 | Return an array of `[royaltyRecipient, royaltyAmount]` pairs. `royaltyAmount` is the absolute amount for the passed `salePrice`, not a percentage. |
| ERC-1155 | No single NEP | Split into NEP-17 contracts, NEP-11 contracts, or a manual NEP-11-divisible-style storage surface depending on whether each token ID is fungible, NFT-like, or fractional. |
| EIP-165 | Manifest `supportedstandards` | Read the deployed manifest instead of calling `supportsInterface(bytes4)`. |
| ERC-2612 / EIP-712 permit | Witness-scoped transaction authorization | Prefer transaction witnesses and `Runtime.checkWitness`. If a contract still accepts off-chain signed messages, keep explicit nonce/deadline replay protection. |
| EIP-1967 / proxy upgrades | NEP-22 / NEP-29 / NEP-31 | Prefer Neo in-place update, deploy/update callback, and optional destroy lifecycle instead of proxy storage slots. |
| ERC-721 receiver | NEP-26 | Implement `onNEP11Payment(from, amount, tokenId, data)` and verify the caller when only specific NFT contracts are accepted. |
| ERC-677 / ERC-1363 hooks | NEP-27 | Implement `onNEP17Payment(from, amount, data)` and verify the caller when only specific token contracts are accepted. |

## Manifest Detection

The compiler inspects public/external methods and events and populates
`supportedstandards` in the manifest. Detection requires the standard's
method set plus a conformant `transfer` signature and `Transfer` event;
ERC-shaped near-misses are flagged with diagnostics instead of advertised:

| Standard | Current Auto-Detection Signal | Compliance Notes |
| --- | --- | --- |
| NEP-17 | `symbol`, `decimals`, `totalSupply`, `balanceOf`, a 4-parameter `transfer(from, to, amount, data)`, a 3-parameter `Transfer` event, and no `ownerOf` | Canonical NEP-17 also requires a Boolean `transfer` return. A 2-parameter ERC-20 `transfer` produces a warning instead of a claim. |
| NEP-11 | `symbol`, `decimals`, `totalSupply`, `balanceOf`, `tokensOf`, `ownerOf`, a 3-parameter `transfer(to, tokenId, data)`, and a 4-parameter `Transfer` event | `transferFrom` is ERC-721, not NEP-11 — ERC-721-shaped contracts are not advertised. `properties` is optional in the NEP but included by the devpack full interface. |
| NEP-24 | A 3-parameter `royaltyInfo(tokenId, royaltyToken, salePrice)` | `tokenURI` / `tokenUri` is ERC-721 metadata and does not trigger NEP-24. `royaltyInfo` must return final royalty amounts; devpack helpers may store basis points internally. |
| NEP-22 / 26 / 27 / 29 / 30 / 31 | Signature-based lifecycle/callback methods | These are additive. A token contract can advertise both a token standard and receiver/lifecycle standards. |

## Deployed ERC/EIP Mirror

The [ERC / EIP to Neo Mirror](/standards-mirror/) contains deployable Solidity
and Neo C# pairs for representative standards. Use it when you need executable
side-by-side examples instead of only semantic guidance.

The mirror is a validation matrix, not a promise that every pair is already
all-green. Always read the [Coverage Matrix](/standards-mirror/coverage-matrix)
and the latest [TestNet Results](/standards-mirror/deployments/RESULTS) before
treating a sample as production-ready.

## Contract Review Checklist

- Confirm the ABI shape against the NEP, not only against ERC method names.
- Confirm manifest `supportedstandards` matches the contract's real ABI and
  event surface.
- Check receiver callbacks (`onNEP17Payment`, `onNEP11Payment`) reject
  unexpected caller contract hashes when the contract only accepts specific
  assets.
- Treat approvals, permits, `safeTransferFrom`, proxies, `receive`, `fallback`,
  and `selfdestruct` as compatibility surfaces that need explicit Neo migration.
- For royalty contracts, ensure `royaltyInfo` returns computed amounts for the
  provided `salePrice`; basis points are an internal configuration detail.
- For deployed mirror pages, compare Solidity and Neo C# behavior against the
  checked-in assertion snapshot.

## More Detail

- [Standards and Contracts](/additional-material/neo-standards)
- [ERC / EIP to Neo Mirror](/standards-mirror/)
- [Coverage Matrix](/standards-mirror/coverage-matrix)
- [Calls and Assets](/mapping/calls-and-assets)
