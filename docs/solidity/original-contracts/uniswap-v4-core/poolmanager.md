# PoolManager (Uniswap V4 Core)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@uniswap/v4-core/src/PoolManager.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `2558`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 2558 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 2088 | function 'revertWith' in 'ProtocolFees' overrides 'Owned::revertWith' which is not marked 'virtual' |
| W121 | 424 | duplicate constant state variable 'ZERO_DELTA' detected while merging libraries |
| INVALID_STORAGE_RETURN | 13 | function 'get' return value 'State' uses 'storage' data location (treated as Any) |
| MANIFEST_WILDCARD_CONTRACT | 8 | contract 'NoDelegateCall' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W130 | 4 | overloaded function 'extsload' with 1 parameter(s) uses Neo overload mangling; external callers must invoke the generated Neo method names |
| W102 | 3 | function 'transfer' has 3 parameters, but NEP-17 requires 4: transfer(from, to, amount, data). The `data` parameter (type Any) is forwarded to the recipient's onNEP17Payment callback. |
| W103 | 3 | ERC-20 method(s) [approve, allowance, transferfrom] detected. These are not part of the NEP-17 spec; Neo uses Runtime.checkWitness() for authorization instead of the approve/allowance pattern. You may keep them as extensions, but they will not contribute to NEP-17 standard detection. |
| W106 | 3 | function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. Neo uses the manifest 'supportedstandards' array for interface detection, which the compiler populates automatically. |
| W113 | 3 | Contract has transfer function but no onNEP17Payment callback. Other contracts cannot send tokens to this contract. |
| W123 | 3 | public state variable 'balanceOf' conflicts with a function of the same name |
| VALIDATION_WARNING | 2 | abstract contract 'ProtocolFees' has 2 unimplemented function(s): [_isUnlocked, _getPool] |
| W111 | 2 | function 'settle' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@uniswap/v4-core/src/PoolManager.sol`