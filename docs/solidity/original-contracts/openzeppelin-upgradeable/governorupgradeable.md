# GovernorUpgradeable (OpenZeppelin Upgradeable)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/GovernorUpgradeable.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `3302`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 3302 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 3150 | function 'panic' in 'EIP712Upgradeable' overrides 'Initializable::panic' which is not marked 'virtual' |
| W121 | 126 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| INVALID_STORAGE_RETURN | 11 | function '_getInitializableStorage' return value 'InitializableStorage' uses 'storage' data location (treated as Any) |
| MANIFEST_WILDCARD_CONTRACT | 6 | contract 'Initializable' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W111 | 3 | function 'onNEP17Payment' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W106 | 2 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| W116 | 2 | function 'execute' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| VALIDATION_WARNING | 1 | abstract contract 'GovernorUpgradeable' has 9 unimplemented function(s): [_quorumReached, _voteSucceeded, _getVotes, _countVote, clock, CLOCK_MODE, votingDelay, votingPeriod, quorum] |
| W117 | 1 | function 'proposalDeadline' appears to be time-sensitive. block.timestamp on Neo N3 is deterministic but can be affected by block production timing. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/GovernorUpgradeable.sol`