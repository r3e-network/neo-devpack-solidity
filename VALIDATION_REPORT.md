# Neo DevPack for Solidity Validation Report

This file is a historical validation-report entry point. Earlier revisions
described point-in-time test counts and quality metrics; those values are no
longer the current source of truth.

Current validation references:

- [README.md](./README.md) for the maintained project status and command list.
- [docs/resources/contributing.md](./docs/resources/contributing.md) for current
  test-suite guidance.
- [docs/SOLIDITY_SUPPORT_MATRIX.md](./docs/SOLIDITY_SUPPORT_MATRIX.md) for the
  audited Solidity feature matrix.
- [docs/FUZZ.md](./docs/FUZZ.md) for fuzzing guidance and corpus layout.
- [docs/internals/parity-and-limitations.md](./docs/internals/parity-and-limitations.md)
  for current NeoVM runtime limits.

To produce a fresh validation snapshot, run the relevant gate for the change:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
npm run docs:check
npm run docs:build
```
