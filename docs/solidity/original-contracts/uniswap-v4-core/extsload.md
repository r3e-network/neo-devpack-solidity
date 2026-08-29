# Extsload (Uniswap V4 Core)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@uniswap/v4-core/src/Extsload.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W130 | 1 | overloaded function 'extsload' with 1 parameter(s) uses Neo overload mangling; external callers must invoke the generated Neo method names |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@uniswap/v4-core/src/Extsload.sol`