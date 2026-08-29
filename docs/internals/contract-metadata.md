---
title: "Manifest Specification"
description: "Manifest Specification section index."
---

# Manifest Specification

Every Neo N3 smart contract is deployed as two artifacts: a `.nef` file (NeoVM bytecode) and a `.manifest.json` file (contract metadata). The manifest describes the contract's ABI, permissions, supported standards, and trust relationships. The neo-devpack-solidity compiler generates both artifacts automatically from your Solidity source code — no manual JSON authoring required.

## Sections

| Section |
| --- |
| [NEF Format](/internals/contract-metadata/nef-format) |
| [Manifest Structure](/internals/contract-metadata/manifest-structure) |
| [Standards Auto-Detection](/internals/contract-metadata/standards-auto-detection) |
| [Permission Inference](/internals/contract-metadata/permission-inference) |
| [Permission Hardening](/internals/contract-metadata/permission-hardening) |
| [NatSpec Manifest Overrides](/internals/contract-metadata/natspec-manifest-overrides) |
| [Manifest Warnings](/internals/contract-metadata/manifest-warnings) |
| [Complete Example](/internals/contract-metadata/complete-example) |
| [Security Considerations](/internals/contract-metadata/security-considerations) |
| [See Also](/internals/contract-metadata/see-also) |
