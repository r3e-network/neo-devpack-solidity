# VRFCoordinatorV2_5 (Chainlink)

## Audit Snapshot

- Status: ✅ pass
- Source type: `npm`
- Source path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/vrf/dev/VRFCoordinatorV2_5.sol`
- Primary issue: No primary issue recorded.
- Audit corpus size: 98 contracts

## Diagnostics

Total diagnostics captured: `475`

### By Severity

| Severity | Count |
| --- | ---: |
| warning | 475 |

### Most Common Codes

| Code | Count | Example |
| --- | ---: | --- |
| W200 | 450 | function '_add' in 'ConfirmedOwner' overrides 'ConfirmedOwnerWithProposal::_add' which is not marked 'virtual' |
| W121 | 9 | duplicate constant state variable 'EXTRA_ARGS_V1_TAG' detected while merging libraries |
| RAW | 6 | warning: blockhash() auto-mapped to Ledger.getBlockHash() on Neo N3. Returns the block hash for the given index. |
| RETURN_TYPE_UNMAPPED | 3 | function 's_config' returns 'Any', which may not map cleanly to Neo manifest types |
| MANIFEST_WILDCARD_CONTRACT | 2 | contract 'SubscriptionAPI' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error. |
| W111 | 2 | function 'fundSubscriptionWithNative' is marked `payable`, but Neo N3 has no native coin transfer; the modifier is accepted for compatibility but has no effect. Use onNEP17Payment(address, uint256, bytes) to handle incoming NEP-17 token payments. |
| W116 | 2 | function 'fundSubscriptionWithNative' has payable modifier which has no effect on Neo N3. Use onNEP17Payment callback to receive token payments. |
| VALIDATION_WARNING | 1 | abstract contract 'VRFConsumerBaseV2Plus' has 1 unimplemented function(s): [fulfillRandomWords] |

Source diagnostic payload: `docs/data/famous-contracts-audit-results.json`.

## References

- Global audit report: [Famous Contracts on NeoVM](/solidity/famous-contracts-neo-audit)
- Per-contract index: [Original Famous Contracts](/solidity/original-contracts/)
- Upstream contract path: `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/vrf/dev/VRFCoordinatorV2_5.sol`