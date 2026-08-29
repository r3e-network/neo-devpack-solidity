---
title: "Devpack Overview: Core Contracts"
description: "Core Contracts from Devpack Overview."
---

# Core Contracts

[Back to Devpack Overview](/additional-material/neo-devpack)

## Syscalls.sol

Low-level Solidity wrappers for the supported Neo N3 syscall surface. This is the lowest-level devpack surface — every other contract and library ultimately delegates to `Syscalls`.

| Category  | Syscalls                                                                    | Examples                                             |
| --------- | --------------------------------------------------------------------------- | ---------------------------------------------------- |
| Storage   | `GetContext`, `Get`, `Put`, `Delete`, `Find`, `AsReadOnly`, `Local.*`       | `storageGet()`, `storagePut()`, `storageFind()`      |
| Runtime   | `CheckWitness`, `GetTime`, `GasLeft`, `Notify`, `Log`, `BurnGas`            | `checkWitness()`, `gasLeft()`, `notify()`            |
| Contract  | `Call`, `GetCallFlags`, `CreateStandardAccount`, `CreateMultisigAccount`    | `contractCall()`, `getCallFlags()`                   |
| Crypto    | `CheckSig`, `CheckMultisig` (via `System.Crypto.*`)                         | `checkSig()`, `checkMultisig()`                      |
| CryptoLib | SHA256, RIPEMD160, Keccak256, VerifyWithECDsa, Ed25519, BLS12-381, Murmur32 | `sha256()`, `verifyWithECDsa()`, `bls12381Pairing()` |
| StdLib    | serialize, deserialize, JSON, base64/58, hex, string ops, itoa/atoi         | `jsonSerialize()`, `base64Encode()`, `stringSplit()` |
| Iterator  | `Next`, `Value`                                                             | `iteratorNext()`, `iteratorValue()`                  |
| Advanced  | `GetRandom`, `GetNetwork`, `GetAddressVersion`, `GetInvocationCounter`      | `getCurrentRandom()`, `getNetwork()`                 |

Syscalls.sol also defines the core data structures used throughout the devpack:

| Struct                | Description                                                                                                           |
| --------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `Block`               | Trimmed block: hash, version, previousHash, merkleRoot, timestamp, nonce, index, primaryIndex, nextConsensus, txCount |
| `Transaction`         | Base transaction: hash, version, nonce, sender, systemFee, networkFee, validUntilBlock, script                        |
| `Witness`             | invocationScript + verificationScript                                                                                 |
| `Signer`              | account, scopes, allowedContracts, allowedGroups, rules                                                               |
| `WitnessRule`         | action (deny/allow) + condition bytes                                                                                 |
| `StorageContext`      | Contract storage context (id + isReadOnly)                                                                            |
| `Iterator`            | NeoVM iterator handle (id, hasNext, currentKey, currentValue)                                                         |
| `Notification`        | scriptHash, eventName, state array                                                                                    |
| `ContractStateNative` | id, updateCounter, hash, nef, manifest                                                                                |

::: tip
Prefer `NativeCalls` wrappers over raw `Syscalls.contractCall()` — they generate specific manifest permissions instead of wildcards.
:::

## NativeCalls.sol

Direct wrappers for all Neo N3 native contracts. Each function calls through `Syscalls.contractCall()` with a deterministic contract hash constant, enabling the compiler to infer exact manifest permissions.

| Native Contract    | Hash Constant         | Key Functions                                                                                         |
| ------------------ | --------------------- | ----------------------------------------------------------------------------------------------------- |
| NEO Token          | `NEO_CONTRACT`        | `neoBalanceOf()`, `neoTransfer()`, `vote()`, `getCandidates()`, `getAccountState()`, `unclaimedGas()` |
| GAS Token          | `GAS_CONTRACT`        | `gasBalanceOf()`, `gasTransfer()`, `gasTotalSupply()`                                                 |
| ContractManagement | `CONTRACT_MANAGEMENT` | `deployContract()`, `updateContract()`, `destroyContract()`, `getContract()`, `hasMethod()`           |
| Policy             | `POLICY_CONTRACT`     | `getFeePerByte()`, `getExecFeeFactor()`, `getStoragePrice()`, `blockAccount()`, `isBlocked()`         |
| Oracle             | `ORACLE_CONTRACT`     | `requestOracleData()`, `getOraclePrice()`, `setOraclePrice()`                                         |
| RoleManagement     | `ROLE_MANAGEMENT`     | `designateAsRole()`, `getDesignatedByRole()`                                                          |
| Notary             | `NOTARY_CONTRACT`     | `notaryVerify()`, `notaryBalanceOf()`, `notaryLockDepositUntil()`, `notaryWithdraw()`                 |
| Treasury           | `TREASURY_CONTRACT`   | `treasuryVerify()`, `treasuryOnNEP17Payment()`, `treasuryOnNEP11Payment()`                            |
| Ledger             | `LEDGER_CONTRACT`     | `currentIndex()`, `currentHash()`, `getBlock()`, `getTransaction()`                                   |

Reference constants for CryptoLib (`CRYPTO_LIB`) and StdLib (`STD_LIB`) are also defined, but their methods are exposed through `Syscalls.sol` directly (e.g., `Syscalls.sha256()`, `Syscalls.base64Encode()`).

```solidity
// Transfer GAS — generates permission: {"contract":"0xd2a4...","methods":["transfer"]}
bool ok = NativeCalls.gasTransfer(from, to, amount, "");

// Query NEO governance
NativeCalls.NeoCandidate[] memory candidates = NativeCalls.getCandidates();
uint256 unclaimed = NativeCalls.unclaimedGas(account, NativeCalls.currentIndex());
```

## FrameworkBase.sol

Base contract providing common infrastructure for Neo N3 Solidity contracts:

- **Ownership** — `transferOwnership()`, `renounceOwnership()`, `onlyOwner` modifier
- **Initialization** — `whenInitialized` modifier, version tracking
- **Witness-based access control** — `withWitness` modifier (calls `Runtime.checkWitness`)
- **Gas limit guards** — `withGasLimit(minGas)` modifier (calls `Runtime.gasLeft`)
- **Contract lifecycle** — `upgradeContract()` with NEF + manifest update
- **Events** — `FrameworkInitialized`, `OwnershipTransferred`, `ContractUpgraded`, `EmergencyStop`

```solidity
contract MyToken is FrameworkBase {
    function adminAction() public onlyOwner withWitness withGasLimit(20000000) {
        // Only owner, witness-verified, with at least 0.2 GAS remaining
    }
}
```

## Framework.sol

Extended framework that inherits `FrameworkBase`. Previously exposed a fully dynamic `callContract(address, string, bytes)` surface, but this forced wildcard permissions in the manifest (`{"contract":"*","methods":"*"}`).

::: warning Strict-Manifest Mode
`Framework.callContract()` is intentionally disabled — it reverts at runtime. Use explicit `NativeCalls.*` or `Syscalls.contractCall(KNOWN_HASH, ...)` wrappers instead. If truly dynamic dispatch is unavoidable, compile with `--manifest-permissions` to supply explicit overrides.
:::

## OracleService.sol

Convenience wrapper around the Neo N3 Oracle native contract:

- `request(url, filter, userData, gasForResponse)` — issue an oracle data request, returns a `requestId`
- `oracleCallback(url, userData, code, result)` — receives the native Oracle callback (restricted to `onlyOracleNative`)
- Forwards responses to the original requester via `IOracleServiceReceiver.onOracleResponse()`
- Uses a fixed callback method name to avoid wildcard manifest permissions
- Tracks request metadata (requester, URL, filter, timestamps, completion status)

```solidity
contract MyOracle is IOracleServiceReceiver {
    OracleService private _oracle;

    function fetchPrice(string calldata url) external {
        _oracle.request(url, "$.price", "", 100000000);
    }

    function onOracleResponse(
        uint256 requestId, uint256 code, bytes calldata result, bytes calldata userData
    ) external override {
        // Handle oracle response
    }
}
```

## NEP17Rescue.sol

Optional extension for NEP-17 tokens that enables emergency recovery of accidentally sent native tokens (NEO and GAS only). In strict-manifest mode, rescuing arbitrary NEP-17 tokens is not supported because it would require wildcard contract permissions.

```solidity
contract MyToken is NEP17Rescue {
    // Inherits emergencyTokenRecovery(token, to, amount, data)
    // Restricted to NEO_CONTRACT and GAS_CONTRACT targets
}
```
