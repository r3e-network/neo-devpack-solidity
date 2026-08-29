# OwnableUpgradeable (OpenZeppelin Upgradeable)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `4`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 4 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| INVALID_STORAGE_RETURN | 4 | function '_getInitializableStorage' return value 'InitializableStorage' uses 'storage' data location (treated as Any) |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol`