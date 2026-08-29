# V4Router (Uniswap V4 Periphery)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/V4Router.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1463`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1463 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 1072 | function 'revertWith' in 'SafeCallback' overrides 'ImmutableState::revertWith' which is not marked 'virtual' |
| W121 | 381 | duplicate constant state variable 'ZERO_DELTA' detected while merging libraries |
| MANIFEST_WILDCARD_CONTRACT | 5 | contract 'ImmutableState' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 5 | abstract contract 'SafeCallback' has 1 unimplemented function(s): [_unlockCallback] |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/V4Router.sol`