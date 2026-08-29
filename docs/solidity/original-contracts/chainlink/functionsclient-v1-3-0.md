# FunctionsClient_v1_3_0 (Chainlink)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_3_0/FunctionsClient.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `2`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 2 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'FunctionsClient' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 1 | abstract contract 'FunctionsClient' has 1 unimplemented function(s): [_fulfillRequest] |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_3_0/FunctionsClient.sol`