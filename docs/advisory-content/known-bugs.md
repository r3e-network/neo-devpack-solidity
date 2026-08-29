# List of Known Bugs

This page tracks known security-relevant bugs in Neo DevPack for Solidity itself. It is not the upstream Solidity `bugs.json` registry: Neo DevPack for Solidity uses its own Rust compiler pipeline, `solang-parser` frontend parsing, NeoVM lowering, manifest generation, and embedded runtime.

Contract source verification tools and deployment reviewers should treat a finding here as release-blocking for affected compiler versions. If a contract was compiled with an affected version after a fixed version was available, recompile and redeploy the contract unless the issue is documented as non-impacting for that contract's feature set.

## Neo DevPack for Solidity Bugs

Because Neo DevPack for Solidity uses `solang-parser` for frontend syntax analysis and emits NeoVM instead of EVM bytecode, upstream Solidity compiler bugs do not automatically apply. Review upstream advisories only when the issue affects Solidity source interpretation rather than EVM-specific code generation.

Neo DevPack for Solidity maintains separate tracking for bugs in EVM-to-NeoVM semantic mapping, bytecode generation, manifest inference, diagnostics, and embedded-runtime behavior.

Currently, there are no known security-critical bugs in the `0.18.x` release lineage.

Any future bugs discovered that affect the execution behavior of deployed `.nef` bytecode will be tracked here and in the GitHub issue tracker under the `security` label.
