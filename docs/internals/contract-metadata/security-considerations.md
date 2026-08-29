---
title: "Contract Metadata Security Considerations"
description: "Security Considerations from Manifest Specification."
---

# Contract Metadata Security Considerations

[Back to Manifest Specification](/internals/contract-metadata)

## Overview

- **Wildcard permissions** — `contract="*"` with `methods="*"` allows your contract to call any method on any contract. This is the maximum attack surface. Use `--deny-wildcard-permissions` in production builds.
- **Wildcard contracts** — `contract="*"` with specific methods still allows calling that method on any contract, which may include malicious ones. Use `--deny-wildcard-contracts` when possible.
- **Wildcard methods** — Specific contract with `methods="*"` allows calling any method on that contract. Prefer explicit method lists.
- **Trust field** — `trusts: "*"` means any contract on the network can invoke your contract's methods. Leave as `[]` unless your contract is a public utility.
- **Supported standards** — Verify that `supportedstandards` matches your intent. Declaring NEP-17 compliance without implementing the full spec can cause wallet and explorer integration failures.
- **Safe methods** — Ensure methods marked `safe: true` are genuinely read-only. A `view` function that modifies state through assembly or low-level calls will pass compilation but may behave unexpectedly on-chain.
- **Permission override files** — Treat override files as security-critical configuration. Review them in code review alongside your Solidity source.
