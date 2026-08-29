# GovernorVotesUpgradeable (OpenZeppelin Upgradeable)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/extensions/GovernorVotesUpgradeable.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `5822`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 5822 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 5520 | function 'panic' in 'EIP712Upgradeable' overrides 'Initializable::panic' which is not marked 'virtual' |
| W121 | 210 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| RETURN_TYPE_UNMAPPED | 39 | function 'toDelay' returns 'Delay', which may not map cleanly to Neo manifest types |
| INVALID_STORAGE_RETURN | 16 | function '_getInitializableStorage' return value 'InitializableStorage' uses 'storage' data location (treated as Any) |
| VALIDATION_WARNING | 15 | function '_getFullAt' should return 3 values but expression does not match tuple |
| MANIFEST_WILDCARD_CONTRACT | 7 | contract 'Initializable' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W111 | 6 | function 'onNEP17Payment' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 4 | function 'execute' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| W117 | 2 | function 'proposalDeadline' appears to be time-sensitive. block.timestamp on Neo N3 is deterministic but can be affected by block production timing. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/extensions/GovernorVotesUpgradeable.sol`