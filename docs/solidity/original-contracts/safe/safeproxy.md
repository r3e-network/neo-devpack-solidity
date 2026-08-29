# SafeProxy (Safe)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/proxies/SafeProxy.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `3`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 3 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W105 | 1 | function 'fallback' has no effect on Neo N3. Use onNEP17Payment(address from, uint256 amount, bytes data) to handle incoming token payments. |
| W111 | 1 | function 'fallback' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 1 | function 'fallback' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/proxies/SafeProxy.sol`