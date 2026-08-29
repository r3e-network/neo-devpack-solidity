# ERC20Votes (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `4006`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 4004 |
| error | 2 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 3726 | function 'panic' in 'ERC20' overrides 'Context::panic' which is not marked 'virtual' |
| W121 | 135 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| INVALID_STORAGE_RETURN | 96 | function 'getAddressSlot' return value 'AddressSlot' uses 'storage' data location (treated as Any) |
| RETURN_TYPE_UNMAPPED | 27 | function 'toDelay' returns 'Delay', which may not map cleanly to Neo manifest types |
| VALIDATION_WARNING | 13 | function '_getFullAt' should return 3 values but expression does not match tuple |
| RAW | 2 | [info][NEP-17] NEP-17 `transfer` method has 2 parameter(s), spec expects 4. See STANDARDS_MAPPING.md for details. |
| W101 | 2 | function 'transfer' has 2 parameters (ERC-20 pattern). NEP-17 requires 4 parameters: transfer(from, to, amount, data). The `from` address is verified via Runtime.checkWitness() and `data` (type Any) is forwarded to the recipient's onNEP17Payment callback. |
| W103 | 2 | ERC-20 method(s) [approve, allowance, transferfrom] detected. These are not part of the NEP-17 spec; Neo uses Runtime.checkWitness() for authorization instead of the approve/allowance pattern. You may keep them as extensions, but they will not contribute to NEP-17 standard detection. |
| W113 | 2 | Contract has transfer function but no onNEP17Payment callback. Other contracts cannot send tokens to this contract. |
| W122 | 1 | duplicate state variable '_name' detected while flattening/merging contracts |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol`