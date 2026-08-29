---
title: "Manifest Specification: Permission Inference"
description: "Permission Inference from Manifest Specification."
---

# Permission Inference

[Back to Manifest Specification](/internals/contract-metadata)

The compiler performs static analysis on the compiled IR to infer the minimal set of permissions your contract needs. The goal is to emit the narrowest possible permissions — wildcards are used only when the compiler cannot determine the call target or method at compile time.

## How it works

1. **Native contract calls** — When your code calls a native contract (GAS, NEO, StdLib, CryptoLib, etc.), the compiler emits an explicit permission with the native contract's deterministic hash and the specific method name.

2. **Known contract address calls** — When `Syscalls.contractCall()` is invoked with a compile-time constant address (literal or `constant` variable), the compiler emits a permission for that specific contract hash.

3. **Known method name** — When the method name is a string literal, the compiler restricts the permission to that specific method.

4. **Dynamic target or method** — When the contract address or method name is a runtime variable, the compiler falls back to a wildcard for the unknown component.

5. **Self-calls** — Calls to `address(this)` are recognized and do not generate wildcard permissions, since Neo N3 always allows contracts to call themselves.

## Example: native contract call

A Solidity call to `NativeCalls.gasTransfer(from, to, amount, data)` generates:

```json
{
    "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
    "methods": ["transfer"]
}
```

## Example: dynamic target, static method

```solidity
interface IFace {
    function ping(uint256 value) external returns (uint256);
}

contract Caller {
    function callPing(address target, uint256 value) public returns (uint256) {
        return IFace(target).ping(value);
    }
}
```

Generates a wildcard contract restricted to the known method:

```json
{ "contract": "*", "methods": ["ping"] }
```

## Example: static target, dynamic method

```solidity
contract Caller {
    function callGas(string memory method) public returns (bytes memory) {
        return Syscalls.contractCall(NativeCalls.GAS_CONTRACT, method, abi.encode());
    }
}
```

Generates a specific contract with wildcard methods:

```json
{
    "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
    "methods": "*"
}
```

## Implicit native permissions

Certain Solidity constructs require native contract calls that are not visible in your source code:

| Construct                             | Native Permission                                             |
| ------------------------------------- | ------------------------------------------------------------- |
| `mapping` storage access              | `StdLib.serialize` + `CryptoLib.keccak256`                    |
| Struct field storage                  | `CryptoLib.keccak256` (+ `StdLib.serialize` for mapping keys) |
| `keccak256()`                         | `CryptoLib.keccak256`                                         |
| `ecrecover()`                         | `CryptoLib.recoverSecp256K1`                                  |
| `abi.encode()` / `abi.encodePacked()` | `StdLib.serialize`                                            |
| `abi.decode()`                        | `StdLib.deserialize`                                          |
| Cross-contract calls                  | `StdLib.serialize` + `StdLib.deserialize`                     |
| `block.number`                        | `Ledger.currentIndex`                                         |
| Contract deployment                   | `ContractManagement.deploy`                                   |
| `getContract()`                       | `ContractManagement.getContract` + `StdLib.serialize`         |
