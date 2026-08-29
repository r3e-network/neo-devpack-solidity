# Contributing

Neo DevPack for Solidity is an open-source project, and we welcome contributions from the community! Whether you're fixing bugs in the Rust compiler, improving the EVM-to-NeoVM semantic mapping, or writing documentation, your help is appreciated.

## How to Report Issues

If you find a bug, an unhandled EVM feature, or have a feature request, please open an issue on the [Neo DevPack for Solidity GitHub Repository](https://github.com/r3e-network/neo-devpack-solidity/issues).

When reporting an issue, please include:
1. The exact version of `neo-solc` you are using (`neo-solc --version`).
2. The Solidity source code that triggered the issue.
3. The exact command-line arguments used.
4. The expected behavior (usually what standard EVM solc does) vs the actual behavior on NeoVM.

## Workflow for Pull Requests

We use the standard GitHub Pull Request workflow:
1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/my-new-feature`).
3. Commit your changes (`git commit -am 'Add some feature'`).
4. Push to the branch (`git push origin feature/my-new-feature`).
5. Create a new Pull Request.

## Running the Compiler Tests

Before submitting a PR, ensure the layered Rust, runtime, conformance, E2E, fuzz, and tooling suites that cover your change still pass.

To run the Rust compiler workspace tests:
```bash
cargo test --workspace
```

To run the full production suite (including Node.js tooling tests):
```bash
make test-all
```

## Documentation Style Guide

This documentation site is built using VitePress. When contributing to the docs:
*   Ensure your page fits within the existing taxonomy (Basics, Language Description, Internals, etc.).
*   Use the `::: tip 💡 NeoVM Difference` blockquote specifically to highlight areas where Neo's execution model diverges from Ethereum.
*   Check for dead links locally before submitting your PR by running `npm run docs:build`.
