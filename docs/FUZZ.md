# Fuzz Testing Compatibility Entry

## Canonical Guide

The maintained fuzzing guide is
[`docs/compiler/fuzz-testing.md`](./compiler/fuzz-testing.md).

This root-level docs file is kept as a stable compatibility entry point for
older links. Keeping the detailed guide in one place avoids drift between the
VitePress compiler docs and historical root documentation.

Quick commands:

```bash
cargo test --test fuzz_tests
bash scripts/run_fuzz_suite.sh deep
cargo +nightly fuzz list
```
