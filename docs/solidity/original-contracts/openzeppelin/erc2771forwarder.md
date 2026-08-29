# ERC2771Forwarder (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/metatx/ERC2771Forwarder.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1130`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1130 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 1020 | function 'tryRecover' in 'ERC2771Context' overrides 'Context::tryRecover' which is not marked 'virtual' |
| INVALID_STORAGE_RETURN | 59 | function 'getAddressSlot' return value 'AddressSlot' uses 'storage' data location (treated as Any) |
| W121 | 45 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| W111 | 2 | function 'execute' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 2 | function 'execute' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'ERC2771Forwarder' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 1 | 'using X for *' is supported; all library functions are available but type-specific filtering is not enforced at compile time |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/metatx/ERC2771Forwarder.sol`