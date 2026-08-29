# PoolAddressesProvider (Aave V3)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/PoolAddressesProvider.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `289`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 289 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 264 | function 'isContract' in 'Ownable' overrides 'Context::isContract' which is not marked 'virtual' |
| W111 | 9 | function 'fallback' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 9 | function 'fallback' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| W105 | 5 | function 'fallback' has no effect on Neo N3. Use onNEP17Payment(address from, uint256 amount, bytes data) to handle incoming token payments. |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'PoolAddressesProvider' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 1 | abstract contract 'Proxy' has 1 unimplemented function(s): [_implementation] |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/PoolAddressesProvider.sol`