# Pool (Aave V3)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/pool/Pool.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1410`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1409 |
| error | 1 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 924 | function 'setLtv' in 'IncentivizedERC20' overrides 'Context::setLtv' which is not marked 'virtual' |
| W121 | 476 | duplicate constant state variable 'BORROWING_MASK' detected while merging libraries |
| MANIFEST_WILDCARD_CONTRACT | 5 | contract 'VersionedInitializable' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| RAW | 1 | [info][NEP-17] NEP-17 `transfer` method has 2 parameter(s), spec expects 4. See STANDARDS_MAPPING.md for details. |
| VALIDATION_WARNING | 1 | abstract contract 'VersionedInitializable' has 1 unimplemented function(s): [getRevision] |
| W101 | 1 | function 'transfer' has 2 parameters (ERC-20 pattern). NEP-17 requires 4 parameters: transfer(from, to, amount, data). The `from` address is verified via Runtime.checkWitness() and `data` (type Any) is forwarded to the recipient's onNEP17Payment callback. |
| W103 | 1 | ERC-20 method(s) [approve, allowance, transferfrom] detected. These are not part of the NEP-17 spec; Neo uses Runtime.checkWitness() for authorization instead of the approve/allowance pattern. You may keep them as extensions, but they will not contribute to NEP-17 standard detection. |
| W113 | 1 | Contract has transfer function but no onNEP17Payment callback. Other contracts cannot send tokens to this contract. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/pool/Pool.sol`