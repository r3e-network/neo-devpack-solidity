# Examples

This folder contains reference Solidity contracts you can compile with the Neo DevPack for Solidity compiler (`neo-solc`).

Compiled artifacts are not committed. Use `examples/out/` (ignored by git) or `/tmp` for generated
`.nef` / `.manifest.json` outputs.

## Quick compile

```bash
target/release/neo-solc examples/TestContract.sol -I devpack -O2 -o /tmp/TestContract
```

## Included contracts

Most contracts in this folder compile to deployable Neo N3 artifacts today. Some are EVM-style
reference ports adapted for Neo N3.

### Compiles on Neo N3 today

- `TestContract.sol`: minimal storage getter/setter used by integration tests.
- `ERC20Token.sol`: ERC20-like token exercising NEP-17 detection.
- `ERC721Token.sol`: ERC721-style NFT example (Neo-adapted receiver checks; prefer NEP-11 examples for production).
- `Escrow.sol`: escrow sample.
- `GovernanceToken.sol`: governance token sample (Neo-adapted: `values[]` must be `0`; use NEP-17 transfers instead).
- `Lottery.sol`: lottery sample.
- `MultiSigWallet.sol`: multisig wallet sample (Neo-adapted; receives GAS via `onNEP17Payment`).
- `NameService.sol`: name service sample.
- `SimpleStorage.sol`: basic storage sample.
- `Staking.sol`: staking sample.
- `UniswapV2Pair.sol`: AMM pair sample (Neo-adapted; no native ETH-style value transfers).
- `new/Counter.sol`: counter with increment/decrement events.
- `new/Bank.sol`: banking contract with deposits, withdrawals, and transfers.
- `new/MultiSigWalletNEP17.sol`: multisig wallet for GAS/NEO via NEP-17 transfers.
- `new/Vault.sol`: GAS-only vault using `onNEP17Payment` + strict manifest-safe transfers.
- `new/UpgradeLifecycleShowcase.sol`: owner bootstrap + witness-gated upgrade/destroy lifecycle via `NativeCalls.updateContract`/`destroyContract`.
- `new/WitnessGuardShowcase.sol`: owner bootstrap + guardian authorization with `Runtime.checkWitness` and lock/unlock controls.
- `new/NFT.sol`: minimal NEP-11-style NFT implementation.
- `new/NeoInteropShowcase.sol`: Runtime/Storage/Syscalls/NativeCalls intrinsic showcase.
- `new/LowLevelCallShowcase.sol`: `address.call` / `address.staticcall` with `abi.encodeWith*` payloads.
- `new/EnumArrayShowcase.sol`: dynamic enum array allocation (`new EnumType[](n)`) and return behavior.
- `new/CustomErrorsShowcase.sol`: custom `error` definitions, `revert CustomError()`, error with parameters.
- `new/InheritanceShowcase.sol`: abstract contracts, multiple inheritance, `super`, `virtual`/`override`.
- `new/InterfaceShowcase.sol`: interface definition, implementation, interface inheritance, and NEP-17-style method surfaces.
- `new/ModifierShowcase.sol`: function modifiers, modifier chaining, modifier with arguments.
- `new/StructMappingShowcase.sol`: nested structs, structs in mappings, struct arrays, struct return values.
- `new/TypeCastingShowcase.sol`: explicit casts (`uint8→uint256`, `address→uint160`), `bytes` conversions, `abi.encode`/`decode`.
- `new/ConstantsImmutableShowcase.sol`: `constant`, `immutable`, compile-time evaluation.
- `new/BitwiseShowcase.sol`: bitwise AND/OR/XOR/NOT/shifts, bit packing, flags pattern.
- `new/TryCatchShowcase.sol`: `try/catch`, catch with error data, catch panic, catch bytes.
- `new/EventIndexedShowcase.sol`: events with indexed params, anonymous events, multi-topic events.
- `new/OracleShowcase.sol`: Neo N3 Oracle request/callback pattern using NativeCalls.
- `new/OracleRelayStrictShowcase.sol`: strict-manifest-safe oracle relay with request tracking and fixed `onOracleResponse` callback.
- `new/MultiStandardToken.sol`: NEP-17 + NEP-24 combined, royalty-bearing fungible token.

### Famous DeFi/Web3 Contracts (`famous/`)

Ports of iconic Ethereum DeFi protocols adapted for Neo N3. See [`famous/README.md`](famous/README.md) for details.

- `famous/WGAS.sol`: Wrapped GAS (WETH9-style) — NEP-17 wrapped native token.
- `famous/FlashLoan.sol`: Aave V2-style flash loan pool with 0.09% fee.
- `famous/SimpleAMM.sol`: Uniswap V2-style constant-product AMM with LP shares.
- `famous/TokenVesting.sol`: OpenZeppelin VestingWallet-style linear vesting with cliff.
- `famous/SimpleLending.sol`: Compound-style lending pool with 150% collateral ratio.
- `famous/SimpleDAO.sol`: Compound Governor-style DAO with staking, voting, and timelock.

### Standards Compliance & Compiler Diagnostics

The compiler automatically detects NEP standard compliance and emits actionable warnings
for Ethereum-style patterns that need adaptation:

| Pattern Detected                           | Compiler Warning                                                |
| ------------------------------------------ | --------------------------------------------------------------- |
| `transfer(to, amount)` (2 params)          | Suggests NEP-17 4-param form `transfer(from, to, amount, data)` |
| `transferFrom(from, to, tokenId)`          | Suggests NEP-11 `transfer(to, tokenId, data)`                   |
| `approve` / `allowance` on token contracts | Notes these are not in NEP-17 spec                              |
| `receive()` / `fallback()`                 | Suggests `onNEP17Payment()` callback                            |
| `supportsInterface(bytes4)`                | Notes Neo uses manifest `supportedstandards`                    |

**Near-miss detection**: If a contract has 3+ of 5 required NEP-17 methods, the compiler
warns about the missing methods. Similar hints exist for NEP-11 and NEP-26.

**Event validation**: When a standard is detected, the compiler checks for the required
`Transfer` event with the correct parameter count.

For the complete EIP↔NEP mapping, see [`../devpack/standards/STANDARDS_MAPPING.md`](../devpack/standards/STANDARDS_MAPPING.md).

### Notes

- Prefer `new/NFT.sol` or `devpack/examples/CompleteNEP11NFT.sol` for a production NEP-11-style NFT on Neo N3.

To compile all new samples:

```bash
for f in examples/new/*.sol; do base=$(basename "$f" .sol); cargo run --quiet -- "$f" -o examples/out/$base; done
```

To run the strict compatibility sweep (devpack + `examples/new`) in one command:

```bash
bash examples/test_strict_compatibility_sweep.sh
```

The sweep excludes intentionally failing diagnostic fixtures (for example fixed-point
or unsupported EVM-only samples) and validates strict-safe contracts only. It also
checks that the remaining excluded negative fixtures still fail under strict flags.

To enforce strict manifest permissions for only `examples/new` samples:

```bash
for f in examples/new/*.sol; do
  cargo run --quiet -- "$f" -o /tmp/examples-strict     --deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods
done
```

Manifests in `examples/out/` (ignored by git) will describe supported standards and ABI entries for inspection.

### Neo-Express smoke scripts (strict-safe highlights)

From the repository root, run:

```bash
# Run all three new showcase smokes
./examples/test_neoxp_new_showcases_smoke.sh

# Or run individually
./examples/test_neoxp_upgrade_lifecycle_smoke.sh
./examples/test_neoxp_witness_guard_smoke.sh
./examples/test_neoxp_oracle_relay_smoke.sh
```

These scripts compile with strict manifest flags, deploy on a temporary Neo-Express chain, and run deterministic invoke checks for each showcase contract.
