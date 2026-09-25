import re

with open('CHANGELOG.md', 'r') as f:
    content = f.read()

new_release = """## [v0.14.0] - 2026-03-13

### Added

- **EVM Try/Catch Multi-return**: `try/catch` blocks now natively support EVM's multiple return syntax (`try returns(uint a, uint b)`) by seamlessly unwrapping the NeoVM `Array` return payload.
- **Documentation Parity**: Completely refactored the VitePress documentation architecture to identically mirror the official `soliditylang.org` sidebar, taxonomy, and feature coverage, fully tailored for Neo N3.

### Changed

- **Graceful EVM Call Options**: Extraneous call options (e.g., `contract.method{value: x}()` or `new Contract{value: x}()`) are now safely ignored, emitting a semantic warning instead of halting compilation.
- **Inline Assembly Fallback**: `assembly { ... }` blocks now compile gracefully into NeoVM no-ops with a warning, unblocking compilation of heavily optimized Ethereum libraries where the assembly isn't strictly required.
- **Unsupported Call Translation**: Unsupported low-level EVM calls (`delegatecall`, `staticcall`) are now lowered to returning a dummy boolean `false` with a semantic warning instead of a hard E3001 abort.
- **Obsolete EVM Globals**: `msg.data` and `msg.sig` are now parsed and mapped to empty byte arrays and `0x00000000` outside of the `onNEP17Payment` callback, replacing strict E3001 compiler rejections.

### Fixed

- **Infinite Loop Prevention**: Patched the Neo IR `CallFunction` dataflow analysis to accurately track return arities, preventing `neo-solc` from hanging infinitely on complex void-return functions (like those found in DAO Governance contracts).
- **NatSpec Overrides**: Fixed missing `load_manifest_permissions_override_from_natspec` linkages, ensuring `@custom:neo.manifest.permissions` comments correctly substitute wildcard manifests.
- **Runtime Exception Handlers**: Hardened the execution context bridging, replacing manual modulo bitwise checks with `.is_multiple_of()` to appease strict CI linting.

"""

content = content.replace("## [Unreleased]\n", "## [Unreleased]\n\n" + new_release)

with open('CHANGELOG.md', 'w') as f:
    f.write(content)
