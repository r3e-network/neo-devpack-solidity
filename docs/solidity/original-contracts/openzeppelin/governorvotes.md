# GovernorVotes (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/extensions/GovernorVotes.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `3843`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 3843 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 3582 | function 'panic' in 'ERC165' overrides 'Context::panic' which is not marked 'virtual' |
| W121 | 135 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| INVALID_STORAGE_RETURN | 68 | function 'getAddressSlot' return value 'AddressSlot' uses 'storage' data location (treated as Any) |
| RETURN_TYPE_UNMAPPED | 24 | function 'toDelay' returns 'Delay', which may not map cleanly to Neo manifest types |
| VALIDATION_WARNING | 11 | function '_getFullAt' should return 3 values but expression does not match tuple |
| MANIFEST_WILDCARD_CONTRACT | 6 | contract 'EIP712' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W111 | 6 | function 'onNEP17Payment' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 4 | function 'execute' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| W117 | 2 | function 'proposalDeadline' appears to be time-sensitive. block.timestamp on Neo N3 is deterministic but can be affected by block production timing. |
| W122 | 2 | duplicate state variable '_name' detected while flattening/merging contracts |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/extensions/GovernorVotes.sol`