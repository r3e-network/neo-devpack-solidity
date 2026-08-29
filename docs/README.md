# Documentation Guide

The project documentation is served as a VitePress website from `docs/`.

## Local docs development

```bash
npm install
npm run docs:dev
```

## Build static docs

```bash
npm run docs:build
```

Output directory:

- `docs/.vitepress/dist`

## Documentation structure

- `docs/basics/` - introduction, installation, quickstart, examples, deployment, and testing
- `docs/language-description/` - Solidity syntax and behavior accepted by Neo DevPack for Solidity
- `docs/mapping/` - split EVM to NeoVM semantic mapping guides
- `docs/compiler/` - compiler usage, output analysis, fuzzing, and codegen notes
- `docs/internals/` - NeoVM storage, ABI, metadata, syscalls, runtime, and architecture references
- `docs/advisory-content/` - security, production readiness, troubleshooting, known bugs, and breaking changes
- `docs/standards-mirror/` - deployable ERC/EIP to Neo standards mirror
- `docs/additional-material/` - devpack, standards, import resolution, NatSpec, SMTChecker, and Yul references
- `docs/resources/` - style guide, contributing, patterns, resources, and keyword index

## Canonical technical references

- `docs/SOLIDITY_SUPPORT_MATRIX.md`
- `docs/RUNTIME_SPEC.md`
- `docs/ARCHITECTURE.md`
- `docs/ERROR_REFERENCE.md`
- `docs/NEO_VM_PARITY_TODO.md`

## Archived content

Historical documents are tracked under `docs/archive/`.
