# DataFeedsCache (Chainlink)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/data-feeds/DataFeedsCache.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `185`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 185 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 180 | function 'sendValue' in 'ConfirmedOwner' overrides 'ConfirmedOwnerWithProposal::sendValue' which is not marked 'virtual' |
| MANIFEST_WILDCARD_CONTRACT | 4 | contract 'ConfirmedOwnerWithProposal' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W106 | 1 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/data-feeds/DataFeedsCache.sol`