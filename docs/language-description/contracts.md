# Contracts

Contracts in Solidity are similar to classes in object-oriented languages. Each contract can contain declarations of State Variables, Functions, Function Modifiers, Events, Errors, Struct Types and Enum Types. Furthermore, contracts can inherit from other contracts.

## Creating Contracts

When a contract is created, its constructor (a function declared with the `constructor` keyword) is executed once.

```solidity
contract MyToken {
    string public name;
    address public owner;

    constructor(string memory _name) {
        name = _name;
        owner = msg.sender;
    }
}
```

::: tip 💡 NeoVM Difference: _deploy
On NeoVM, the concept of a "constructor" maps to the `_deploy(data, update)` entry point. This method is called automatically by Neo's `ContractManagement` service when the contract is deployed or updated on the blockchain.

When compiled to NeoVM, the `constructor` parameters are serialized and passed through the `data` array parameter of the `_deploy` method. The compiler automatically handles the deserialization and initialization.
:::

## Visibility and Getters

Solidity has four types of visibilities for functions and state variables:
* `external`: Part of the contract interface (Neo Manifest ABI). Can be called from other contracts via message calls.
* `public`: Part of the contract interface (Neo Manifest ABI). Can be called internally or externally.
* `internal`: Not part of the ABI. Can only be accessed internally or by derived contracts.
* `private`: Not part of the ABI. Can only be accessed by the contract they are defined in.

Getter functions for `public` state variables are automatically generated and exposed in the Neo manifest as `safe: true` methods.

## Function Modifiers

Modifiers can be used to easily change the behavior of functions. For example, they can automatically check a condition prior to executing the function. Modifiers are inheritable properties of contracts and may be overridden by derived contracts.

```solidity
contract Mutex {
    bool locked;
    modifier noReentrancy() {
        require(!locked, "Reentrant call.");
        locked = true;
        _;
        locked = false;
    }

    function f() public noReentrancy returns (uint) {
        return 7;
    }
}
```
Modifiers act the same on NeoVM as they do on EVM, seamlessly expanding via inlining during compilation.

## Constant and Immutable State Variables

State variables can be declared as `constant` or `immutable`. In both cases, the variables cannot be modified after the contract has been constructed.

* **`constant`**: The value has to be fixed at compile-time.
* **`immutable`**: The value can be assigned during construction time, but cannot be changed afterwards.

* **NeoVM Mapping:** Constants are deeply inlined by the compiler. Immutable variables are stored into standard Neo `Storage` but with checks preventing assignment outside the `constructor` / `_deploy` context.

## Inheritance

Solidity supports multiple inheritance including polymorphism.

Multiple inheritance is resolved using C3 linearization, exactly identical to Ethereum Solidity. The `virtual` and `override` keywords work identically.

```solidity
contract Base {
    function greet() public virtual returns (string memory) {
        return "Hello";
    }
}

contract Derived is Base {
    function greet() public override returns (string memory) {
        return super.greet();
    }
}
```

## Abstract Contracts and Interfaces

* **Abstract Contracts:** Contracts are abstract if at least one of their functions lacks an implementation.
* **Interfaces:** Interfaces are similar to abstract contracts, but they cannot have any functions implemented, and they cannot inherit from other interfaces.

Both features are perfectly supported on NeoVM. Interfaces are crucial for executing cross-contract calls using `System.Contract.Call`.

## Libraries

Libraries are similar to contracts, but their purpose is that they are deployed only once at a specific address and their code is reused using the `DELEGATECALL` feature of the EVM.

::: tip 💡 NeoVM Difference: No Delegatecall
Because NeoVM does not support `delegatecall` (each contract has entirely isolated storage and logic contexts), user-defined libraries are compiled by inlining their functions into the consuming contract.

`external` and `public` library functions are accepted for source compatibility, but the compiler normalizes them to internal-call lowering and emits warnings. Separate deployable/linkable library semantics are not available on Neo N3.
:::

## Using For

The directive `using A for B;` can be used to attach library functions (from the library `A`) to any type (`B`) in the context of a contract. These functions will receive the object they are called on as their first parameter.

```solidity
library Math {
    function add(uint a, uint b) internal pure returns (uint) {
        return a + b;
    }
}

contract C {
    using Math for uint;
    function f() public pure returns (uint) {
        uint x = 5;
        return x.add(7);
    }
}
```
This is fully supported and works effectively on NeoVM, provided the library functions are marked `internal`.
