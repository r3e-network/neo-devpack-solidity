# PositionManager (Uniswap V4 Periphery)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/PositionManager.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `6392`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 6389 |
| error | 3 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 4738 | function 'revertWith' in 'EIP712_v4' overrides 'ERC721::revertWith' which is not marked 'virtual' |
| W121 | 1557 | duplicate constant state variable 'ZERO_DELTA' detected while merging libraries |
| W111 | 23 | function 'revokeNonce' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 21 | function 'revokeNonce' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| INVALID_STORAGE_RETURN | 16 | function 'get' return value 'State' uses 'storage' data location (treated as Any) |
| MANIFEST_WILDCARD_CONTRACT | 16 | contract 'ERC721' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 7 | abstract contract 'ERC721' has 1 unimplemented function(s): [tokenURI] |
| RAW | 3 | [info][NEP-11] NEP-11 `Transfer` event has 3 parameter(s), expected 4. |
| W104 | 3 | function 'transferFrom' with 3 parameters (ERC-721 pattern) detected. NEP-11 uses transfer(to, tokenId, data) with 3 parameters instead. Authorization is via Runtime.checkWitness(owner), not msg.sender. |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| W114 | 3 | NFT contract (has ownerOf) but missing onNEP11Payment callback. Other contracts cannot send NFTs to this contract. |
| RETURN_TYPE_UNMAPPED | 2 | function 'positionInfo' returns 'Any', which may not map cleanly to Neo manifest types |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/PositionManager.sol`