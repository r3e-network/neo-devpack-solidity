# Mapping Parity and Limitations

Neo DevPack for Solidity is source-compatible with many Solidity 0.8.x patterns, but not
every EVM behavior has an exact Neo N3 equivalent.

## Fidelity Levels

| Level | Meaning | Examples |
| --- | --- | --- |
| Exact | Neo N3 provides equivalent semantics. | `block.timestamp`, `block.number`, `address(this)`, storage reads/writes. |
| Compatible with warning | Code compiles, but behavior is a Neo approximation. | Ether units, `block.coinbase`, `block.basefee`, `msg.data`, `msg.sig`. |
| Neo-specific replacement | Use a Neo pattern instead of the EVM pattern. | `Runtime.checkWitness` instead of approval-heavy account assumptions. |
| Unsupported | The EVM behavior cannot be represented safely. | `delegatecall`, true transient storage, fixed-point numbers. |

## Common Porting Risks

| Pattern | Risk | Neo-Specific Fix |
| --- | --- | --- |
| Relying on `tx.origin` for auth | Signer identity can be too broad for authorization. | Use `Runtime.checkWitness` on the specific account. |
| Assuming Ether balance semantics | Neo assets are NEP tokens and GAS, not native Ether. | Use GAS/NEP transfer APIs and callbacks. |
| Assuming EVM overflow behavior | NeoVM integers are arbitrary precision. | Use explicit modulo arithmetic for wrapping. |
| Renaming state variables in upgrades | Storage keys are name-derived. | Migrate data or preserve state variable names. |
| Iterating mappings | Solidity mappings do not expose keys; Neo storage keys do not add enumeration automatically. | Track inserted keys in arrays or explicit indexes. |
| Using delegate-call proxy patterns | NeoVM has no caller-storage execution mode. | Use `ContractManagement.update` / NEP-22 style upgrades. |

## Diagnostics

Approximate mappings should emit compiler diagnostics. Treat those diagnostics
as migration tasks, not as noise, before deploying to Neo N3 MainNet.

## More Detail

- [Parity and Limitations](/internals/parity-and-limitations)
- [Security Considerations](/advisory-content/security-considerations)
- [Calls and Assets](/mapping/calls-and-assets)
