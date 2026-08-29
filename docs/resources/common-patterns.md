# Common Patterns

This section highlights common Solidity design patterns and explains how they are adapted for NeoVM using Neo DevPack for Solidity.

## The Withdrawal Pattern

On Ethereum, pushing funds to another address using `address.transfer()` is discouraged due to reentrancy risks and fixed gas limits. The recommended approach is a "pull" pattern where users call a `withdraw()` function.

On Neo N3, value transfer is handled via NEP-17 token transfers, which inherently use a callback pattern (`onNEP17Payment`).

**Neo Adaptation:**
Because NEP-17 transfers invoke the recipient's callback, reentrancy is still a theoretical possibility on Neo if the recipient is a contract. You should still utilize the Checks-Effects-Interactions pattern.

```solidity
import {NativeCalls} from "@neo/NativeCalls.sol";
import {Runtime} from "@neo/Runtime.sol";

contract PullPayment {
    mapping(address => uint256) public balances;

    function withdraw() public {
        address payee = Runtime.getCallingScriptHash();
        uint256 amount = balances[payee];
        
        require(amount > 0, "No funds to withdraw");
        
        // Effects
        balances[payee] = 0;
        
        // Interactions
        bool success = NativeCalls.gasTransfer(address(this), payee, amount, "");
        require(success, "Transfer failed");
    }
}
```

## Receiving Funds (`onNEP17Payment`)

::: tip 💡 NeoVM Difference
Ethereum contracts use `receive() external payable` or `fallback() external payable` to accept native Ether. **Neo contracts must implement `onNEP17Payment` to receive NEP-17 tokens (including GAS).**
:::

```solidity
import "contracts/NativeCalls.sol";
import "contracts/Syscalls.sol";

contract Treasury {
    uint256 public totalDeposits;

    /// @notice Equivalent to `receive() external payable`
    function onNEP17Payment(address from, uint256 amount, Any calldata data) external {
        from;
        data;
        // Only accept GAS
        require(Syscalls.getCallingScriptHash() == NativeCalls.GAS_CONTRACT, "Only GAS accepted");
        
        totalDeposits += amount;
    }
}
```

## Restricting Access

Access restriction is one of the most common patterns. On Neo, this relies on `Runtime.checkWitness()` rather than `msg.sender`.

```solidity
import {Runtime} from "@neo/Runtime.sol";

contract Owned {
    address public owner;

    constructor() {
        // Initialize owner during deployment
        owner = Runtime.getCallingScriptHash();
    }

    modifier onlyOwner() {
        require(Runtime.checkWitness(owner), "Unauthorized: Must be signed by owner");
        _;
    }

    function changeOwner(address newOwner) public onlyOwner {
        owner = newOwner;
    }
}
```

## Contract Upgrades

Proxy patterns (like Transparent Proxies or UUPS) using `delegatecall` are heavily used on Ethereum. NeoVM blocks `delegatecall` but offers a native upgrade syscall.

**Neo Adaptation:**
Use `ContractManagement.update` to replace the contract bytecode while preserving the storage state.

```solidity
import {ContractManagement} from "@neo/ContractManagement.sol";

contract Upgradeable {
    address public admin;

    constructor() {
        admin = Runtime.getCallingScriptHash();
    }

    function upgrade(bytes memory nefFile, bytes memory manifest) public {
        require(Runtime.checkWitness(admin), "Unauthorized");
        
        // Native in-place upgrade. Storage remains intact.
        ContractManagement.update(nefFile, manifest);
    }
}
```
