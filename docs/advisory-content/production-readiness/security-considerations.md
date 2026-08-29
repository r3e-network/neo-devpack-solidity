---
title: "Production Readiness Security Considerations"
description: "Security Considerations from Production Readiness."
---

# Production Readiness Security Considerations

[Back to Production Readiness](/advisory-content/production-readiness)

## Smart Contract Security

- **Reentrancy protection**: Use a `locked` state variable with a `nonReentrant` modifier.
- **Access control**: Use `msg.sender` checks (maps to `Runtime.GetCallingScriptHash()`).
- **Input validation**: Validate all function parameters with `require`.
- **Integer overflow**: Solidity 0.8 checked arithmetic is enforced outside `unchecked`; verify intentional wraparound paths explicitly.
- **Event logging**: Emit events for all state changes to enable off-chain monitoring.

## Neo-Specific Security

::: tip Neo Authorization Model
Neo uses witness-based authorization (`Runtime.checkWitness`) rather than EVM's `msg.sender`-only model. Always use witness checks for sensitive operations, even if your Solidity source uses `msg.sender` guards.
:::

- **Witness checks**: Use `Runtime.checkWitness(addr)` for authorization instead of relying solely on `msg.sender`.
- **Permission minimization**: Use `--deny-wildcard-contracts --deny-wildcard-methods` to enforce least-privilege permissions.
- **NEP-17 callbacks**: If your contract receives tokens, implement `onNEP17Payment` with proper validation.
- **Contract updates**: Protect the update mechanism with strict access control. Test the update path thoroughly.
- **Storage key collisions**: The compiler uses prefix-based storage keys. Be aware of key layout when upgrading contracts.

## External Audit

For high-value contracts:

1. Use external Solidity analysis tools (Slither, Mythril) on the source before compilation.
2. Review the generated NeoVM assembly for unexpected behavior.
3. Consider a professional audit of both the Solidity source and the generated Neo artifacts.
4. Test on TestNet with adversarial scenarios.
