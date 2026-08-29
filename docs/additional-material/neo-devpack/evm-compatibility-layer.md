---
title: "Devpack Overview: EVM Compatibility Layer"
description: "Compatibility adapters for Solidity contracts that rely on EVM-only semantics."
---

# EVM Compatibility Layer

[Back to Devpack Overview](/additional-material/neo-devpack)

Neo N3 is not an EVM chain. Some Solidity constructs compile directly, some are
lowered with warnings, and some have no safe one-to-one mapping. The devpack
compatibility layer provides explicit Neo-native adapters for the most common
EVM migration gaps.

## Files

```solidity
import "contracts/compat/EVMNativeAssetAdapter.sol";
import "contracts/compat/EVMFallbackDispatcher.sol";
import "contracts/compat/EVMContractFactory.sol";
```

| EVM pattern | Neo-compatible path | Adapter |
| --- | --- | --- |
| `receive()`, payable deposits, `msg.value` | Receive GAS/NEP-17 assets through `onNEP17Payment(from, amount, data)` | `EVMNativeAssetAdapter` |
| Unknown selector `fallback()` routing | Expose an explicit `dispatch(bytes4 selector, bytes data)` method | `EVMFallbackDispatcher` |
| `CREATE` / factory deployment | Deploy NEF + manifest through `ContractManagement.deploy` | `EVMContractFactory._deployLikeCreate` |
| CREATE2 salt tracking | Treat salt as app metadata; do not derive deterministic EVM addresses | `EVMContractFactory._deployLikeCreate(..., salt, labelHash)` |
| `selfdestruct` cleanup | Call Neo ContractManagement `destroy` when the contract owner/admin authorizes it | `EVMContractFactory._destroyLikeSelfdestruct` |
| `address.code.length` style checks | Query ContractManagement `isContract` | `EVMContractFactory._evmIsContract` |

## Payable / msg.value

Use `EVMNativeAssetAdapter` when an Ethereum contract expected Ether through
`receive()`, payable functions, or `msg.value`.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "contracts/compat/EVMNativeAssetAdapter.sol";

contract Treasury is EVMNativeAssetAdapter {
    uint256 public totalReceived;

    function _onEVMValue(
        address token,
        address from,
        uint256 amount,
        bytes memory data
    ) internal override {
        token;
        from;
        data;
        require(amount > 0, "zero value");
        totalReceived += amount;
    }
}
```

The adapter exports a NEP-27-compliant `onNEP17Payment(address,uint256,Any)`
method and unwraps `Any` into `bytes` for the internal hook. It records the last
token, sender, amount, and data length through `lastEVMValue*` getters. The
token contract is read from Neo's calling script hash, which is the reliable
source during a NEP-17 callback.

## Explicit Fallback Dispatch

Neo manifests contain named methods. There is no implicit "unknown selector"
entrypoint equivalent to EVM fallback dispatch. Use `EVMFallbackDispatcher` and
make selector routing intentional:

```solidity
contract Router is EVMFallbackDispatcher {
    bytes4 private constant PING = bytes4(hex"11111111");
    uint256 public hits;

    function _dispatch(bytes4 selector, bytes memory data)
        internal
        override
        returns (bytes memory)
    {
        require(selector == PING, "unsupported selector");
        hits += data.length;
        return "";
    }
}
```

When writing selector constants, prefer `bytes4(hex"....")`; it keeps byte
ordering explicit across Solidity, Neo ABI manifests, and off-chain tooling.

## Factory Deployment

`EVMContractFactory` wraps ContractManagement for contracts that previously used
`new`, CREATE, CREATE2, or selfdestruct-based lifecycle flows.

```solidity
contract Factory is EVMContractFactory {
    address public lastCreated;

    function deploy(bytes memory nef, bytes memory manifest, bytes memory data) public {
        lastCreated = _deployLikeCreate(
            nef,
            manifest,
            data,
            bytes32(0),
            keccak256(bytes("example"))
        );
    }
}
```

Neo contract hashes are derived by Neo's deployment rules, not by EVM CREATE2
preimage rules. If a protocol relied on deterministic CREATE2 addresses, store
the salt and returned contract hash in application state and expose a resolver.

## No Safe Direct Equivalent

Some EVM features should not be hidden behind a compatibility shim:

| EVM feature | Neo migration |
| --- | --- |
| `delegatecall` proxy execution | Use `ContractManagement.update` for in-place upgrades; keep storage layout stable. |
| `tx.origin` authorization | Use `Runtime.checkWitness(account)` and explicit signer policies. |
| Raw EVM bytecode introspection | Use manifest methods, supported standards, and ContractManagement queries. |
| `blockhash`, EVM gas price/base fee assumptions | Use Neo Ledger/Policy/native contract data where available; treat the semantics as different. |

## Verification

The compatibility layer is covered by a Neo-Express deployment smoke test:

```bash
NEO_SOLC="$PWD/target/release/neo-solc" bash examples/test_neoxp_evm_compat_smoke.sh
```

The test compiles a contract importing all three adapters, deploys it to a fresh
Neo-Express chain, transfers GAS into the contract to exercise
`onNEP17Payment`, reads the recorded amount, and runs an explicit fallback
dispatch that updates state.
