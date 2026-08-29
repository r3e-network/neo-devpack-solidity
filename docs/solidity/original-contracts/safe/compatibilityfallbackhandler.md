# CompatibilityFallbackHandler (Safe)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/handler/CompatibilityFallbackHandler.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `185`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 185 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 168 | function 'mul' in 'ISignatureValidator' overrides 'ISignatureValidatorConstants::mul' which is not marked 'virtual' |
| MANIFEST_WILDCARD_CONTRACT | 3 | contract 'GuardManager' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| RAW | 3 | warning: tx.origin has different semantics on Neo N3. Neo uses multi-signature witnesses instead of a single origin. Consider using msg.sender or Runtime.CheckWitness() instead. |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| W111 | 3 | function 'onNEP17Payment' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W105 | 2 | function 'fallback' has no effect on Neo N3. Use onNEP17Payment(address from, uint256 amount, bytes data) to handle incoming token payments. |
| VALIDATION_WARNING | 1 | abstract contract 'ISignatureValidator' has 1 unimplemented function(s): [isValidSignature] |
| W116 | 1 | function 'execTransaction' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| W130 | 1 | overloaded function 'isValidSignature' with 2 parameter(s) uses Neo overload mangling; external callers must invoke the generated Neo method names |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/handler/CompatibilityFallbackHandler.sol`