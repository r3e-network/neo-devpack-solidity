# UniswapV2Factory (Uniswap V2 Core)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Factory.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `26`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 24 |
| error | 2 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 14 | function 'add' in 'UniswapV2Pair' overrides 'UniswapV2ERC20::add' which is not marked 'virtual' |
| RAW | 2 | [info][NEP-17] NEP-17 `transfer` method has 2 parameter(s), spec expects 4. See STANDARDS_MAPPING.md for details. |
| W101 | 2 | function 'transfer' has 2 parameters (ERC-20 pattern). NEP-17 requires 4 parameters: transfer(from, to, amount, data). The `from` address is verified via Runtime.checkWitness() and `data` (type Any) is forwarded to the recipient's onNEP17Payment callback. |
| W103 | 2 | ERC-20 method(s) [approve, allowance, transferfrom] detected. These are not part of the NEP-17 spec; Neo uses Runtime.checkWitness() for authorization instead of the approve/allowance pattern. You may keep them as extensions, but they will not contribute to NEP-17 standard detection. |
| W108 | 2 | ERC-2612 permit pattern detected (7-parameter permit function). Neo N3 uses Runtime.checkWitness() for authorization; off-chain signature permits are not needed. |
| W113 | 2 | Contract has transfer function but no onNEP17Payment callback. Other contracts cannot send tokens to this contract. |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'UniswapV2Pair' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W121 | 1 | duplicate constant state variable 'Q112' detected while merging libraries |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Factory.sol`