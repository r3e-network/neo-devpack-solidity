# Famous Solidity Contracts (Vendored Upstream Sources)

This directory stores upstream Solidity contract sources used by the Neo DevPack for Solidity compatibility audit.

What lives here:

- `sources/` — byte-identical vendored upstream package sources, **deps-stripped**:
  full dependency trees are not vendored, so many files here do not compile
  stand-alone. This tree exists for source-integrity tracking and auditing;
  compile results against it are NOT the compiler-capability signal.
- `samples/` — **33 self-contained famous contracts** (defi / nft / gamefi /
  zkp / infra-dao / patterns) that compile hermetically (no include paths, no
  external dependencies). See [`samples/README.md`](samples/README.md). All 33
  are asserted to compile by `cargo test --release --test famous_samples_compile`.

Measured coverage (the real capability signal):

The famous-contract evaluation compiles ~600 well-known contracts with their
**full dependency trees** supplied (npm/git). Result: **~87%** compile
end-to-end. Per corpus: zero-knowledge 100%, solmate 100%, solady 99%,
Uniswap v3 93%, OpenZeppelin 89%, Aave v3 85% (every deployable
implementation — Pool/PoolConfigurator/AToken — compiles).

Pointers:

- Source root: `third_party/famous-contracts/sources/`
- Hermetic sample corpus: `third_party/famous-contracts/samples/`
- Target list: `docs/data/famous-contracts-targets.json` (entries with `source: "npm"`)
- Vendor sync script: `node scripts/vendor_famous_contracts.js`
- Integrity check script: `node scripts/verify_famous_contract_sources.js`

Rules:

1. Vendored files must remain byte-identical to upstream package sources.
1. Do not hand-edit vendored contract files.
1. Update by re-running the vendor script, then run audit + verify scripts.

Licensing:

- Each vendored file keeps its upstream SPDX/license header.
- Respect upstream package licenses when redistributing.
