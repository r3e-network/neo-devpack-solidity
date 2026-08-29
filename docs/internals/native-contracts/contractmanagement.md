---
title: "Native Contracts: ContractManagement"
description: "ContractManagement from Native Contracts."
---

# ContractManagement

[Back to Native Contracts](/internals/native-contracts)

Manages contract deployment, upgrades, and introspection. The embedded runtime implements the subset needed by compiler/runtime tests; production Neo N3 exposes additional management methods.

## Methods

| Method                    | Signature                         | Return          | Safe | Embedded runtime status                                  |
| ------------------------- | --------------------------------- | --------------- | :--: | -------------------------------------------------------- |
| `deploy`                  | `deploy(bytes,bytes)`             | `ContractState` |  ❌  | Implemented with an in-memory registry.                  |
| `deploy`                  | `deploy(bytes,bytes,bytes)`       | `ContractState` |  ❌  | Implemented; initialization data is accepted by the call. |
| `update`                  | `update(bytes,bytes)`             | `ContractState` |  ❌  | Implemented against the in-memory registry.              |
| `update`                  | `update(bytes,bytes,bytes)`       | `ContractState` |  ❌  | Implemented; migration data is accepted by the call.     |
| `getContract`             | `getContract(address)`            | `ContractState` |  ✅  | Implemented, including a self-contract fallback.         |
| `isContract`              | `isContract(address)`             | `bool`          |  ✅  | Implemented for the self contract and registry entries.  |
| `hasMethod`               | `hasMethod(address,string,uint8)` | `bool`          |  ✅  | Implemented using self method offsets or manifest data.  |
| `getMinimumDeploymentFee` | `getMinimumDeploymentFee()`       | `uint256`       |  ✅  | Implemented as Neo N3's canonical default fee value.     |

Production methods such as `destroy`, `getContractById`, `setMinimumDeploymentFee`, and `listContracts` are not implemented by the embedded runtime handler today.

## Code Example

```solidity
import "devpack/contracts/NativeCalls.sol";

contract Upgradeable {
    address private _owner;

    constructor() {
        _owner = msg.sender;
    }

    /// @dev Upgrade this contract in-place
    function upgrade(bytes memory newNef, bytes memory newManifest) public {
        require(Runtime.checkWitness(_owner), "not owner");
        NativeCalls.updateContract(newNef, newManifest);
        // After update, the new code executes immediately
    }

    /// @dev Check if another contract exists
    function contractExists(address target) public view returns (bool) {
        return NativeCalls.hasMethod(target, "name", 0);
    }
}
```

::: info
The devpack still exposes `NativeCalls.destroyContract()` and the compiler maps `selfdestruct(addr)` to production Neo N3 `ContractManagement.destroy()` with a warning. The embedded runtime used by compiler tests does not model the destructive state deletion; it focuses on deploy, update, and introspection behavior.
:::

---
