# UniswapV4DeployerCompetition (Uniswap V4 Periphery)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/UniswapV4DeployerCompetition.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `3`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 3 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W117 | 2 | function 'competitionDeadline' appears to be time-sensitive. block.timestamp on Neo N3 is deterministic but can be affected by block production timing. |
| MANIFEST_WILDCARD_CONTRACT | 1 | contract 'UniswapV4DeployerCompetition' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/UniswapV4DeployerCompetition.sol`