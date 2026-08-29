# VariableDebtToken (Aave V3)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/tokenization/VariableDebtToken.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `2183`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 2179 |
| error | 4 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W121 | 1520 | duplicate constant state variable 'WAD' detected while merging libraries |
| W200 | 640 | function 'toUint224' in 'EIP712Base' overrides 'VersionedInitializable::toUint224' which is not marked 'virtual' |
| MANIFEST_WILDCARD_CONTRACT | 4 | contract 'IncentivizedERC20' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| RAW | 4 | [info][NEP-17] NEP-17 `transfer` method has 2 parameter(s), spec expects 4. See STANDARDS_MAPPING.md for details. |
| W101 | 4 | function 'transfer' has 2 parameters (ERC-20 pattern). NEP-17 requires 4 parameters: transfer(from, to, amount, data). The `from` address is verified via Runtime.checkWitness() and `data` (type Any) is forwarded to the recipient's onNEP17Payment callback. |
| W103 | 4 | ERC-20 method(s) [approve, allowance, transferfrom] detected. These are not part of the NEP-17 spec; Neo uses Runtime.checkWitness() for authorization instead of the approve/allowance pattern. You may keep them as extensions, but they will not contribute to NEP-17 standard detection. |
| W113 | 4 | Contract has transfer function but no onNEP17Payment callback. Other contracts cannot send tokens to this contract. |
| VALIDATION_WARNING | 3 | abstract contract 'VersionedInitializable' has 1 unimplemented function(s): [getRevision] |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/tokenization/VariableDebtToken.sol`