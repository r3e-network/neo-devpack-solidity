# Account (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/account/Account.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `686`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 686 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 664 | function 'panic' in 'Account' overrides 'AbstractSigner::panic' which is not marked 'virtual' |
| W121 | 14 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| RAW | 3 | warning: block.basefee auto-mapped to Policy.getFeePerByte() on Neo N3. Neo uses a fixed fee structure, not EIP-1559 base fees. |
| MANIFEST_WILDCARD_CONTRACT | 2 | contract 'AbstractSigner' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| VALIDATION_WARNING | 2 | abstract contract 'AbstractSigner' has 1 unimplemented function(s): [_rawSignatureValidation] |
| W111 | 1 | function 'onNEP17Payment' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/account/Account.sol`