# ERC721Upgradeable (OpenZeppelin Upgradeable)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1444`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1443 |
| error | 1 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 1360 | function 'checkOnERC721Received' in 'ContextUpgradeable' overrides 'Initializable::checkOnERC721Received' which is not marked 'virtual' |
| W121 | 70 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| INVALID_STORAGE_RETURN | 5 | function '_getInitializableStorage' return value 'InitializableStorage' uses 'storage' data location (treated as Any) |
| MANIFEST_WILDCARD_CONTRACT | 4 | contract 'Initializable' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W106 | 2 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| RAW | 1 | [info][NEP-11] NEP-11 `Transfer` event has 3 parameter(s), expected 4. |
| W104 | 1 | function 'transferFrom' with 3 parameters (ERC-721 pattern) detected. NEP-11 uses transfer(to, tokenId, data) with 3 parameters instead. Authorization is via Runtime.checkWitness(owner), not msg.sender. |
| W114 | 1 | NFT contract (has ownerOf) but missing onNEP11Payment callback. Other contracts cannot send NFTs to this contract. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol`