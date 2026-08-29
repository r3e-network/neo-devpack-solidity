# AccessManager (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/access/manager/AccessManager.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `764`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 764 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 708 | function 'sendValue' in 'Multicall' overrides 'Context::sendValue' which is not marked 'virtual' |
| W121 | 30 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| RETURN_TYPE_UNMAPPED | 15 | function 'toDelay' returns 'Delay', which may not map cleanly to Neo manifest types |
| VALIDATION_WARNING | 8 | function '_getFullAt' should return 3 values but expression does not match tuple |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'AccessManager' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W111 | 1 | function 'execute' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 1 | function 'execute' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/access/manager/AccessManager.sol`