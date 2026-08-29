# ERC721Votes (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/extensions/ERC721Votes.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `4910`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 4908 |
| error | 2 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 4576 | function 'checkOnERC721Received' in 'ERC165' overrides 'Context::checkOnERC721Received' which is not marked 'virtual' |
| W121 | 165 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| INVALID_STORAGE_RETURN | 108 | function 'getAddressSlot' return value 'AddressSlot' uses 'storage' data location (treated as Any) |
| RETURN_TYPE_UNMAPPED | 30 | function 'toDelay' returns 'Delay', which may not map cleanly to Neo manifest types |
| VALIDATION_WARNING | 14 | function '_getFullAt' should return 3 values but expression does not match tuple |
| MANIFEST_WILDCARD_CONTRACT | 7 | contract 'Context' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| RAW | 2 | [info][NEP-11] NEP-11 `Transfer` event has 3 parameter(s), expected 4. |
| W104 | 2 | function 'transferFrom' with 3 parameters (ERC-721 pattern) detected. NEP-11 uses transfer(to, tokenId, data) with 3 parameters instead. Authorization is via Runtime.checkWitness(owner), not msg.sender. |
| W114 | 2 | NFT contract (has ownerOf) but missing onNEP11Payment callback. Other contracts cannot send NFTs to this contract. |
| W122 | 1 | duplicate state variable '_name' detected while flattening/merging contracts |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/extensions/ERC721Votes.sol`