# ACLManager (Aave V3)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/ACLManager.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `489`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 489 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W121 | 455 | duplicate constant state variable '_HEX_SYMBOLS' detected while merging libraries |
| W200 | 30 | function 'toString' in 'ERC165' overrides 'Context::toString' which is not marked 'virtual' |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'ACLManager' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/ACLManager.sol`