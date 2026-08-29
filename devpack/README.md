# Neo N3 Devpack for Solidity

**Neo N3 smart contract devpack for the `neo-devpack-solidity` compiler**

> Important: `neo-devpack-solidity` treats `Runtime`, `Storage`, `Syscalls`, `Neo`, `NativeCalls` and `abi`
> as **compiler intrinsics** (built-in libraries). Their Solidity source is primarily for editor
> tooling (signatures/docs). The compiler lowers supported members directly to Neo N3 syscalls and
> native contract calls. Unsupported members will fail compilation with a diagnostic listing the
> supported intrinsics.

This devpack provides Solidity-facing interfaces for the Neo N3 features that `neo-devpack-solidity`
currently supports, plus complete example contracts (NEP-17/NEP-11) and reusable NEP lifecycle/callback interfaces.

## 🎯 Features

### ✅ Core Neo N3 Integration

- **Syscalls**: Runtime, crypto, storage, iterator, and contract syscalls (plus convenience helpers)
- **Native Contracts**: NEO, GAS, ContractManagement, Policy, Oracle, RoleManagement
- **Storage**: `Storage.get/put/remove/find` lowered to `System.Storage.*`
- **Events**: Solidity `emit` lowered to `System.Runtime.Notify` (Neo enforces ABI event name + parameter types)

### ✅ NEP Standards Support

- **NEP-17**: Fungible token standard (enhanced ERC-20)
- **NEP-11**: Non-fungible token standard (enhanced ERC-721)
- **NEP-24**: NFT royalty standard (`royaltyInfo`)
- **NEP-22/29/30/31**: Update/deploy/verify/destroy lifecycle interfaces
- **NEP-26/27**: NEP-11/NEP-17 payment callback interfaces
- **Custom NEPs**: Framework for extending additional standards

### ✅ ERC → NEP Migration

The compiler includes built-in diagnostics to help migrate Ethereum contracts to Neo N3:

- **ERC-20 → NEP-17**: Detects 2-param `transfer(to, amount)` and suggests 4-param NEP-17 form
- **ERC-721 → NEP-11**: Detects `transferFrom(from, to, tokenId)` and suggests NEP-11 `transfer(to, tokenId, data)`
- **approve/allowance**: Warns that these are not part of NEP-17 (Neo uses `Runtime.checkWitness()`)
- **receive()/fallback()**: Suggests `onNEP17Payment()` callback instead
- **supportsInterface()**: Notes that Neo uses manifest `supportedstandards` (auto-populated by compiler)
- **EVM compatibility adapters**: `contracts/compat/*` provides explicit payable, fallback dispatch, and factory lifecycle migration helpers

See [`standards/STANDARDS_MAPPING.md`](standards/STANDARDS_MAPPING.md) for the complete EIP↔NEP mapping
with method signatures, event mappings, migration checklists, and code examples.

> **ERC-20 `approve`/`allowance` on Neo N3:**
> The ERC-20 approve/allowance pattern **does not exist** in NEP-17. On Neo N3,
> authorization is handled by `Runtime.checkWitness(account)`, which verifies that
> the transaction was signed by the given account. If your Ethereum contract relies
> on `approve` + `transferFrom`, replace it with a direct `transfer(from, to, amount, data)`
> call where the caller proves ownership via witness verification. The NEP-17 base
> contract in this devpack includes `Runtime.checkWitness` checks in its `transfer`
> implementation. See `devpack/standards/NEP17.sol` line 236 for the reference pattern.

### ✅ Advanced Features

- **Cross-Contract Calls**: `Syscalls.contractCall{WithFlags}` and optional `CALLT` token emission
- **Upgradeable Contracts**: ContractManagement wrappers via `NativeCalls.*`

## 📚 Quick Start

### Installation

```bash
# Install Neo DevPack for Solidity with devpack
git clone https://github.com/r3e-network/neo-devpack-solidity.git
cd neo-devpack-solidity
make install

# Or add the devpack to an existing Hardhat 2 project
npm install --save-dev @neo-devpack-solidity/contracts hardhat@^2.28.6 @neo-devpack-solidity/hardhat-solc-neo @neo-devpack-solidity/hardhat-neo-deployer
```

The Hardhat plugins in this repository currently support Hardhat 2.28.x. Hardhat 3 requires a
separate plugin/runtime migration and is intentionally outside the supported peer range.

### Basic Usage

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

import "@neo-devpack-solidity/contracts/standards/NEP17.sol";

contract MyToken is NEP17 {
    constructor() NEP17("My Token", "MTK", 8, 0, 1_000_000 * 10**8) {}

    event Minted(address indexed to, uint256 amount);

    function mint(address to, uint256 amount) public onlyOwner withWitness {
        _mint(to, amount);
        emit Minted(to, amount);
    }
}
```

## ✅ Validation Workflow

```bash
npm run build
npm run test
npm run test:integration
```

- `build`: compiles devpack contracts with `hardhat neo-compile`.
- `test`: compile-smoke pass with forced Neo recompilation.
- `test:integration`: recompiles, then runs artifact-level integration checks (build-info, manifest, ABI, permissions).
- Integration tests intentionally validate Neo artifacts and compiler outputs, not EVM `ethers` deployment flows.

## 🏗️ Architecture

### Core Components

```
devpack/
├── contracts/           # Core framework contracts
│   ├── FrameworkBase.sol # Base framework (minimally-permissioned)
│   ├── Framework.sol    # Extended framework (dynamic calls; requires wildcard permissions)
│   ├── OracleService.sol # Oracle native-contract wrapper (optional)
│   ├── NativeContracts.sol # Canonical native script-hash constants
│   ├── NativeCalls.sol # Native contract interfaces
│   └── Syscalls.sol    # System call mappings
├── standards/          # NEP standard implementations
│   ├── NEP17.sol      # Fungible tokens
│   ├── NEP11.sol      # Non-fungible tokens
│   └── NEP24.sol      # NFT royalty standard
├── libraries/          # Utility libraries
│   ├── Neo.sol        # Neo blockchain utilities
│   ├── Storage.sol    # Advanced storage operations
│   └── Runtime.sol    # Runtime services
└── examples/           # Complete contract examples
    ├── CompleteNEP17Token.sol # Full NEP-17 token (strict-manifest compatible)
    ├── CompleteNEP11NFT.sol   # NEP-11 NFT (strict-manifest compatible; see STANDARDS_MAPPING.md for manifest-type deviations)
    └── VaultPattern.sol       # ERC-4626-style vault adapted for Neo N3
```

## 🧪 Additional Strict-Safe Examples

Beyond `devpack/examples/*`, the repository includes focused feature samples in `examples/new/*`:

- `UpgradeLifecycleShowcase.sol`: owner+witness gated `NativeCalls.updateContract`/`destroyContract` lifecycle
- `WitnessGuardShowcase.sol`: role-based access control and emergency locks with `Runtime.checkWitness`
- `OracleRelayStrictShowcase.sol`: oracle request/callback relay using fixed `onOracleResponse` callback

All are designed to compile with strict manifest flags (`--deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods`).

## 📖 API Reference

### Supported Intrinsics

`neo-devpack-solidity` lowers calls to a supported subset of these libraries:

#### Runtime (`devpack/libraries/Runtime.sol`)

| Method                       | Neo Syscall                   | Description                    |
| ---------------------------- | ----------------------------- | ------------------------------ |
| `checkWitness(address)`      | `System.Runtime.CheckWitness` | Verify transaction signer      |
| `notify(string, ...)`        | `System.Runtime.Notify`       | Emit Neo event                 |
| `notifyIndexed(string, ...)` | `System.Runtime.Notify`       | Emit event with indexed params |
| `gasLeft()`                  | `System.Runtime.GasLeft`      | Remaining GAS for execution    |
| `burnGas(uint256)`           | `System.Runtime.BurnGas`      | Burn GAS (anti-spam)           |
| `log(string)`                | `System.Runtime.Log`          | Debug log message              |
| `getTime()`                  | `System.Runtime.GetTime`      | Current block timestamp        |
| `getTrigger()`               | `System.Runtime.GetTrigger`   | Invocation trigger type        |

#### Storage (`devpack/libraries/Storage.sol`)

| Method                                          | Neo Syscall                          | Description               |
| ----------------------------------------------- | ------------------------------------ | ------------------------- |
| `get(bytes)`                                    | `System.Storage.Get`                 | Read value by key         |
| `put(bytes, bytes)`                             | `System.Storage.Put`                 | Write key-value pair      |
| `remove(bytes)`                                 | `System.Storage.Delete`              | Delete key                |
| `find(bytes)`                                   | `System.Storage.Find`                | Iterate keys by prefix    |
| `getContext()` / `getReadOnlyContext()`         | `System.Storage.Get[ReadOnly]Context`| Storage context handles   |
| `asReadOnly(context)`                           | `System.Storage.AsReadOnly`          | Read-only context         |
| `putContractMetadata(name, version, author, extra)` | `System.Storage.Put`            | Store contract metadata   |

These are the only supported `Storage` members. Earlier devpack revisions also
documented typed helpers (`putUint256`, `getString`, ...), batch operations
(`batchPut`, `batchGet`, `batchDelete`), prefix utilities (`count`,
`findValues`, `findKeys`, `clearPrefix`, `exists`), and a `putLocal`/`getLocal`
"local storage" API. None of these had faithful Neo N3 lowerings (the `*Local`
family mapped to syscalls that do not exist on Neo N3), so they were removed
and now fail compilation. Implement such helpers in contract code on top of
`put`/`get`/`remove`/`find`; note Neo N3 storage is already contract-private.

#### Syscalls (`devpack/contracts/Syscalls.sol`)

| Method                                                 | Neo Syscall                             | Description                |
| ------------------------------------------------------ | --------------------------------------- | -------------------------- |
| `contractCall(address, string, bytes)`                 | `System.Contract.Call`                  | Cross-contract call        |
| `contractCallWithFlags(address, string, bytes, uint8)` | `System.Contract.Call`                  | Call with CallFlags        |
| `createStandardAccount(bytes)`                         | `System.Contract.CreateStandardAccount` | Derive address from pubkey |
| `createMultisigAccount(uint8, bytes[])`                | `System.Contract.CreateMultisigAccount` | Create multisig address    |
| `sha256(bytes)`                                        | `CryptoLib.sha256`                      | SHA-256 hash               |
| `ripemd160(bytes)`                                     | `CryptoLib.ripemd160`                   | RIPEMD-160 hash            |
| `base64Encode(bytes)` / `base64Decode(string)`         | `StdLib.base64Encode/Decode`            | Base64 encoding            |
| `jsonSerialize(bytes)` / `jsonDeserialize(string)`     | `StdLib.jsonSerialize/Deserialize`      | JSON encoding              |

#### NativeContracts (`devpack/contracts/NativeContracts.sol`)

Canonical Neo N3 native contract/script-hash constants:
`NEO_CONTRACT`, `GAS_CONTRACT`, `CONTRACT_MANAGEMENT`, `POLICY_CONTRACT`,
`ORACLE_CONTRACT`, `ROLE_MANAGEMENT`, `NOTARY_CONTRACT`, `TREASURY_CONTRACT`,
`LEDGER_CONTRACT`, `CRYPTO_LIB`, `STD_LIB`.

Note: values are listed in Neo RPC big-endian form.

#### NativeCalls (`devpack/contracts/NativeCalls.sol`)

| Category               | Key Methods                                                          |
| ---------------------- | -------------------------------------------------------------------- |
| **NEO token**          | `neoBalanceOf`, `neoTransfer`, `neoTotalSupply`, `neoDecimals`       |
| **GAS token**          | `gasBalanceOf`, `gasTransfer`, `gasTotalSupply`, `gasDecimals`       |
| **ContractManagement** | `deployContract`, `updateContract`, `destroyContract`, `getContract` |
| **Policy**             | `getGasPrice`, `getStoragePrice`, `getFeePerByte`, `isBlocked`       |
| **Oracle**             | `requestOracleData` (callback-based)                                 |
| **RoleManagement**     | `getDesignatedByRole`                                                |

#### EVM Compatibility Adapters (`devpack/contracts/compat/`)

| Adapter                     | Purpose                                                                      |
| --------------------------- | ---------------------------------------------------------------------------- |
| `EVMNativeAssetAdapter`     | Bridges payable/msg.value-style deposits to `onNEP17Payment` callbacks       |
| `EVMFallbackDispatcher`     | Exposes explicit selector routing instead of implicit EVM fallback dispatch   |
| `EVMContractFactory`        | Wraps ContractManagement deploy/update/destroy for CREATE-style lifecycles    |

#### NEP Standards (`devpack/standards/`)

| Standard | File        | Description                                                                          |
| -------- | ----------- | ------------------------------------------------------------------------------------ |
| NEP-17   | `NEP17.sol` | Fungible token (symbol, decimals, totalSupply, balanceOf, transfer + Transfer event) |
| NEP-11   | `NEP11.sol` | Non-fungible token with non-divisible and divisible variants                         |
| NEP-24   | `NEP24.sol` | NFT royalty standard (royaltyInfo per token or default)                              |
| NEP-22   | `NEP22.sol` | Contract update interface (`update(nefFile, manifest, data)`)                        |
| NEP-26   | `NEP26.sol` | NEP-11 receiver callback interface (`onNEP11Payment`)                                |
| NEP-27   | `NEP27.sol` | NEP-17 receiver callback interface (`onNEP17Payment`)                                |
| NEP-29   | `NEP29.sol` | Deployment callback interface (`_deploy(data, update)`)                              |
| NEP-30   | `NEP30.sol` | Verification callback interface (`verify()`)                                          |
| NEP-31   | `NEP31.sol` | Contract destroy interface (`destroy()`)                                              |

> Note: These files are primarily for Solidity tooling ergonomics (types/signatures/docs) over
> compiler intrinsics. Most members compile standalone, but callback/function-pointer helpers
> currently revert with explicit `unsupported` errors because callback invocation lowering is not
> implemented yet. Use direct `Runtime`/`Storage`/`Syscalls`/`NativeCalls` intrinsic calls in
> production contracts (see `devpack/examples/*.sol` and `examples/new/NeoInteropShowcase.sol`).

## 🔐 Manifest Permissions (Dynamic Calls)

Neo N3 manifests must declare cross-contract call permissions. The compiler can infer many direct calls
(native contracts like `GAS.transfer`, fixed `System.Contract.Call` sites), but **fully dynamic calls**
(unknown contract hash and/or method name at compile time) may require wildcard permissions.

**Recommended options:**

- Prefer `FrameworkBase.sol` / `NativeCalls.*` helpers to avoid dynamic calls in production code.
- Enforce strict manifests in CI with:
  - `--deny-wildcard-permissions` (blocks `{"contract":"*","methods":"*"}`)
  - `--deny-wildcard-contracts` (blocks any `contract:"*"`)
  - `--deny-wildcard-methods` (blocks any `methods:"*"`)
- Strict-mode check for devpack + strict-safe showcase contracts:
  ```bash
  bash examples/test_strict_compatibility_sweep.sh
  ```
- If you only want `devpack/examples/*`:
  ```bash
  for f in devpack/examples/*.sol; do
    cargo run --quiet -- "$f" -o /tmp/devpack-strict       --deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods
  done
  ```
- If you intentionally need dynamic calls, provide an explicit allowlist:

```json
// permissions.json
[
  {
    "contract": "0x0000000000000000000000000000000000000000",
    "methods": ["transfer"]
  }
]
```

```bash
neo-solc MyContract.sol -I devpack -o build/MyContract \\
  --manifest-permissions permissions.json \\
  --manifest-permissions-mode replace-wildcards \\
  --deny-wildcard-contracts --deny-wildcard-methods
```

## 🏗️ Constructors & Deployment Data

Neo N3 deployment invokes the contract’s `_deploy(data, update)` method.
`neo-devpack-solidity` injects `_deploy` automatically.

- If your Solidity contract has **no constructor parameters**, you can deploy normally.
- If your Solidity contract has a **parameterised constructor**, pass constructor arguments via
  `_deploy.data` as an **array of arguments**:
  - Neo-Express / CLI tooling: pass a **JSON-encoded array string** (e.g. `[7]`, `["hello", 1]`).
  - SDKs that support StackItems directly may pass an **Array** value.
  - Contract-to-contract deploy flows may pass **StdLib.serialize(...) bytes** (for example `abi.encode(...)`).

This compiler attempts to decode constructor deployment data via `StdLib.jsonDeserialize`, then
falls back to `StdLib.deserialize` (binary serialization), and finally uses `data` as-is when native
calls throw. The generated manifest will include (and on-chain execution will require) permissions
for the StdLib methods used (`jsonDeserialize` / `deserialize`).
