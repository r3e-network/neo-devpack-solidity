# ERC20Permit (OpenZeppelin)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `1765`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 1763 |
| error | 2 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 1620 | function 'tryRecover' in 'ERC20' overrides 'Context::tryRecover' which is not marked 'virtual' |
| W121 | 75 | duplicate constant state variable 'GENERIC' detected while merging libraries |
| INVALID_STORAGE_RETURN | 59 | function 'getAddressSlot' return value 'AddressSlot' uses 'storage' data location (treated as Any) |
| RAW | 2 | [info][NEP-17] NEP-17 `transfer` method has 2 parameter(s), spec expects 4. See STANDARDS_MAPPING.md for details. |
| W101 | 2 | function 'transfer' has 2 parameters (ERC-20 pattern). NEP-17 requires 4 parameters: transfer(from, to, amount, data). The `from` address is verified via Runtime.checkWitness() and `data` (type Any) is forwarded to the recipient's onNEP17Payment callback. |
| W103 | 2 | ERC-20 method(s) [approve, allowance, transferfrom] detected. These are not part of the NEP-17 spec; Neo uses Runtime.checkWitness() for authorization instead of the approve/allowance pattern. You may keep them as extensions, but they will not contribute to NEP-17 standard detection. |
| W113 | 2 | Contract has transfer function but no onNEP17Payment callback. Other contracts cannot send tokens to this contract. |
| VALIDATION_WARNING | 1 | 'using X for *' is supported; all library functions are available but type-specific filtering is not enforced at compile time |
| W108 | 1 | ERC-2612 permit pattern detected (7-parameter permit function). Neo N3 uses Runtime.checkWitness() for authorization; off-chain signature permits are not needed. |
| W122 | 1 | duplicate state variable '_name' detected while flattening/merging contracts |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol`