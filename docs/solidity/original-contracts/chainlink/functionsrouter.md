# FunctionsRouter (Chainlink)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsRouter.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1183`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1183 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 1176 | function 'isContract' in 'ConfirmedOwner' overrides 'ConfirmedOwnerWithProposal::isContract' which is not marked 'virtual' |
| MANIFEST_WILDCARD_CONTRACT | 6 | contract 'ConfirmedOwnerWithProposal' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 1 | abstract contract 'FunctionsSubscriptions' has 5 unimplemented function(s): [_getMaxConsumers, _getSubscriptionDepositDetails, _onlySenderThatAcceptedToS, _onlyRouterOwner, _whenNotPaused] |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsRouter.sol`