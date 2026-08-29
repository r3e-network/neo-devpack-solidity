# AutomationRegistry2_3 (Chainlink)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/automation/v2_3/AutomationRegistry2_3.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `4150`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 4150 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 4104 | function 'toUint248' in 'ConfirmedOwner' overrides 'ConfirmedOwnerWithProposal::toUint248' which is not marked 'virtual' |
| RAW | 15 | warning: tx.gasprice auto-mapped to Policy.getFeePerByte() on Neo N3. Neo fees are determined by script size and syscall costs. |
| MANIFEST_WILDCARD_CONTRACT | 10 | contract 'OCR2Abstract' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 6 | abstract contract 'OCR2Abstract' has 4 unimplemented function(s): [setConfig, latestConfigDetails, latestConfigDigestAndEpoch, transmit] |
| W105 | 5 | function 'fallback' has no effect on Neo N3. Use onNEP17Payment(address from, uint256 amount, bytes data) to handle incoming token payments. |
| W111 | 5 | function 'fallback' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 5 | function 'fallback' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/automation/v2_3/AutomationRegistry2_3.sol`