# Structure of a Contract

Smart contracts in Solidity are similar to classes in object-oriented languages. Each contract can contain declarations of State Variables, Functions, Function Modifiers, Events, Errors, Struct Types and Enum Types.

## State Variables

State variables are values which are permanently stored in contract storage. On NeoVM, state variables map directly to Neo `Storage` operations using deterministic key derivation based on the variable name.

```solidity
contract SimpleStorage {
    uint256 storedData; // State variable mapped to Neo Storage
}
```

## Functions

Functions are the executable units of code. Functions are usually defined inside a contract, but they can also be defined outside of contracts. External and public functions automatically generate entries in the Neo manifest's ABI `methods` array.

```solidity
contract SimpleAuction {
    function bid() public payable { // Method entry generated in manifest
        // ...
    }
}
```

## Function Modifiers

Function modifiers can be used to amend the semantics of functions in a declarative way. On Neo, modifiers often encapsulate authorization logic using `Runtime.checkWitness()`.

```solidity
contract Mutex {
    address public owner;

    modifier onlyOwner() {
        require(Runtime.checkWitness(owner), "Unauthorized");
        _;
    }
    
    function restricted() public onlyOwner {
        // ...
    }
}
```

## Events

Events are convenience interfaces with the NeoVM `Runtime.Notify` logging facilities. All arguments provided to an event are compiled into the notification's state array.

```solidity
contract SimpleAuction {
    event HighestBidIncreased(address bidder, uint amount);

    function bid() public payable {
        // ...
        emit HighestBidIncreased(msg.sender, msg.value); // Maps to Runtime.Notify
    }
}
```

## Errors

Errors allow you to define descriptive names and data for failure situations. Errors are compiled down to NeoVM `ABORTMSG` or `THROW` instructions, preserving the error information for the transaction receipt.

```solidity
error NotEnoughFunds(uint requested, uint available);

contract Token {
    function transfer(address to, uint amount) public {
        uint balance = balances[msg.sender];
        if (balance < amount)
            revert NotEnoughFunds(amount, balance);
        // ...
    }
}
```

## Struct Types

Structs are custom defined types that can group several variables. When stored in a state variable on NeoVM, structs are serialized into a single binary blob using `StdLib.serialize()`.

```solidity
struct Voter {
    uint weight;
    bool voted;
    address delegate;
    uint vote;
}
```

## Enum Types

Enums can be used to create custom types with a finite set of 'constant values'. On NeoVM, enums are backed by `BigInteger` at runtime and integer constants at compile time.

```solidity
enum State { Created, Locked, Inactive }
```
