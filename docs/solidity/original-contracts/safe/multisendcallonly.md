# MultiSendCallOnly (Safe)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/libraries/MultiSendCallOnly.sol`
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
| W111 | 1 | function 'multiSend' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 1 | function 'multiSend' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/libraries/MultiSendCallOnly.sol`