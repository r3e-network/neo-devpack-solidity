# Multicall (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/utils/Multicall.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `16`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 16 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 16 | function 'sendValue' in 'Multicall' overrides 'Context::sendValue' which is not marked 'virtual' |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/utils/Multicall.sol`