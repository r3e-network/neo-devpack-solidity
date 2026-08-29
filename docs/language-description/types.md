# Types

Solidity is a statically typed language, which means that the type of each variable (state and local) needs to be specified. Solidity provides several elementary types which can be combined to form complex types.

In addition, types can interact with each other in expressions containing operators. For a quick reference of the various operators, see [Order of Precedence of Operators](cheatsheet#order-of-precedence-of-operators).

The concept of "undefined" or "null" values does not exist in Solidity, but newly declared variables always have a default value dependent on its type. To handle any unexpected values, you should use the revert function to revert the whole transaction, or return a tuple with a second `bool` value denoting success.

::: tip 💡 NeoVM Difference
While Solidity types carry specific constraints (e.g., `uint8` vs `uint256`), NeoVM relies heavily on its own fundamental types (`BigInteger`, `ByteArray`, `Boolean`, `Array`, `Map`). The Neo DevPack for Solidity compiler enforces Solidity's type rules during compilation and automatically lowers them to the appropriate NeoVM representations.
:::

For a shorter reader guide to the same mapping rules, see
[Types and Values](/mapping/types-and-values) and
[Storage and Mappings](/mapping/storage-and-mappings).

---

## Value Types

The following types are also called value types because variables of these types will always be passed by value, i.e. they are always copied when they are used as function arguments or in assignments.

### Booleans

`bool`: The possible values are constants `true` and `false`.

**Operators:**
* `!` (logical negation)
* `&&` (logical conjunction, "and")
* `||` (logical disjunction, "or")
* `==` (equality)
* `!=` (inequality)

The operators `||` and `&&` apply the common short-circuiting rules. This means that in the expression `f(x) || g(y)`, if `f(x)` evaluates to `true`, `g(y)` will not be evaluated even if it may have side-effects.

* **NeoVM Mapping:** Maps directly to the NeoVM `Boolean` stack item.

### Integers

`int` / `uint`: Signed and unsigned integers of various sizes. Keywords `uint8` to `uint256` in steps of `8` (unsigned of 8 up to 256 bits) and `int8` to `int256`. `uint` and `int` are aliases for `uint256` and `int256`, respectively.

**Operators:**
* Comparisons: `<=`, `<`, `==`, `!=`, `>=`, `>` (evaluate to `bool`)
* Bit operators: `&`, `|`, `^` (bitwise exclusive or), `~` (bitwise negation)
* Shift operators: `<<` (left shift), `>>` (right shift)
* Arithmetic operators: `+`, `-`, unary `-` (only for signed integers), `*`, `/`, `%` (modulo), `**` (exponentiation)

::: tip 💡 NeoVM Difference: Checked Fixed-Width Semantics
NeoVM represents integers as arbitrary-precision `BigInteger` values internally, but Neo DevPack for Solidity enforces Solidity 0.8 fixed-width arithmetic semantics at the compiler/runtime boundary:

1. **Checked arithmetic:** Outside `unchecked`, operations such as `uint8(255) + 1` revert with `Panic(0x11)`.
2. **Unchecked arithmetic:** Inside `unchecked`, supported fixed-width arithmetic suppresses overflow guards and wraps, so `uint8(255) + 1` becomes `0`.
3. **Other panics still apply:** `unchecked` does not suppress division-by-zero or modulo-by-zero checks.
:::

### Fixed Point Numbers

`fixed` / `ufixed`: Signed and unsigned fixed point numbers of various sizes.

::: danger 🚫 Unsupported Feature
Fixed point numbers are not fully supported by mainline Solidity and are completely **unsupported** by Neo DevPack for Solidity. Use scaled integer arithmetic instead (e.g., multiply by `10^18`).
:::

### Address

The address type comes in two largely identical flavors:
* `address`: Holds a 20 byte value (size of an Ethereum address or Neo script hash).
* `address payable`: Same as `address`, but with the additional members `transfer` and `send`.

**Operators:**
* `<=`, `<`, `==`, `!=`, `>=`, `>`

::: tip 💡 NeoVM Difference: The Address Type
On NeoVM, `address` maps to a `UInt160` (the 20-byte script hash of an account or contract in little-endian format).

While `address payable` is parsed by the compiler, **direct value transfers via `.transfer()` and `.send()` emit a warning** and are mapped to `GAS.transfer()`. Neo does not have a native "Ether" concept attached to addresses; all value transfers use the NEP-17 token standard.
:::

#### Members of Addresses

* `balance` and `transfer`: 
  On Ethereum, you can query the balance of an address using the property `balance` and send Ether to a payable address using the `transfer` function.
  On NeoVM, `address.balance` is auto-mapped to `GAS.balanceOf(address)`. `address.transfer` is auto-mapped to `GAS.transfer(this, address, amount, data)`. Both emit warnings because they are approximations.

* `call`, `staticcall`, `delegatecall`, and `callcode`:
  `address.call` maps directly to `System.Contract.Call` and is the standard way to do cross-contract invocations.
  `address.staticcall` maps to `System.Contract.Call` with read-only flags.
  **`address.delegatecall` and `callcode` are blocked.** EVM uses these operations to execute code in the caller's storage context. NeoVM does not support that storage model. Use `ContractManagement.update` for contract upgrades.

### Contract Types

Every contract defines its own type. You can implicitly convert contracts to contracts they inherit from. Contracts can be explicitly converted to and from the `address` type.

```solidity
Token t = Token(0x123...);
```

### Fixed-size byte arrays

The value types `bytes1`, `bytes2`, `bytes3`, ..., `bytes32` hold a sequence of bytes from one to up to 32. `byte` is an alias for `bytes1`.

**Operators:**
* Comparisons: `<=`, `<`, `==`, `!=`, `>=`, `>` (evaluate to `bool`)
* Bit operators: `&`, `|`, `^` (bitwise exclusive or), `~` (bitwise negation)
* Shift operators: `<<` (left shift), `>>` (right shift)
* Index access: If `x` is of type `bytesI`, then `x[k]` for `0 <= k < I` returns the `k` th byte (read-only).

* **NeoVM Mapping:** Maps to a fixed-length NeoVM `ByteArray`. `bytes32` translates specifically to the Neo ABI type `Hash256` and is often used for NEP-11 NFT token IDs.

### Literals

#### Address Literals
Hexadecimal literals that pass the address checksum test, for example `0xdCad3a6d3569DF655070DEd06cb7A1b2Ccd1D3AF` are of `address` payable type.

#### Rational and Integer Literals
Integer literals are formed from a sequence of numbers in the range 0-9. They are evaluated to arbitrary precision in the compiler.
Fractional literals are not supported on NeoVM.

#### String Literals and Hex Literals
String literals are written with either double or single-quotes (`"foo"` or `'bar'`).
Hex literals are prefixed with the keyword `hex` and enclosed in double or single-quotes (`hex"001122FF"`).

### Enums

Enums are one way to create a user-defined type in Solidity. They are explicitly convertible to and from all integer types but implicit conversion is not allowed.

```solidity
enum ActionChoices { GoLeft, GoRight, GoStraight, SitStill }
ActionChoices choice = ActionChoices.GoStraight;
```
* **NeoVM Mapping:** Enums are backed by `uint8` and are represented as `BigInteger` at runtime on NeoVM.

### User-Defined Value Types

A user-defined value type allows creating a zero cost abstraction over an elementary value type. This is similar to an alias, but with stricter type requirements.

```solidity
type UFixed256x18 is uint256;
```
These map to the underlying type (`uint256` -> `BigInteger`) on NeoVM.

---

## Reference Types

Values of reference type can be modified through multiple different names. Contrast this with value types where you get an independent copy whenever a variable of value type is used. Because of that, reference types have to be handled more carefully.

### Data location

Every reference type has an additional annotation, the "data location", about where it is stored. There are three data locations: `memory`, `storage` and `calldata`.

* **`storage`** - The location where state variables are stored. Lifetime is limited to the contract's existence.
* **`memory`** - Lifetime is limited to an external function call.
* **`calldata`** - Special data location that contains the function arguments.

::: tip 💡 NeoVM Difference: calldata vs memory
On Ethereum, `calldata` is a non-modifiable, non-persistent area where function arguments are stored, and behaves differently than `memory`. 

On NeoVM, **`calldata` is treated identically to `memory`**. The compiler accepts the `calldata` keyword to maintain strict source compatibility with Ethereum contracts, but internally, both map to items on the NeoVM execution stack.
:::

### Arrays

Arrays can have a compile-time fixed size or a dynamic size.

```solidity
uint256[] dynamicArray;
uint256[5] fixedArray;
```

#### Array Members
* **`length`**: Arrays have a `length` member that contains their number of elements.
* **`push()` / `push(x)`**: Dynamic storage arrays and `bytes` (not `string`) have a `push` member that you can use to append a zero-initialized element at the end of the array, or append a given element.
* **`pop()`**: Dynamic storage arrays and `bytes` (not `string`) have a `pop` member that you can use to remove an element from the end of the array.

* **NeoVM Mapping:** Memory arrays map to the NeoVM `Array` type. Storage arrays map to deterministic key-value prefix operations in Neo `Storage`.

### Structs

Solidity provides a way to define new types in the form of structs.

```solidity
struct Funder {
    address addr;
    uint amount;
}
```

::: tip 💡 NeoVM Difference: Struct Storage
On NeoVM, structs map to an `Array` containing the struct's fields in memory. 

When written to `storage`, the entire struct is serialized as a single binary blob via `StdLib.serialize()`. Because struct storage is atomic on NeoVM, reading a single field from storage requires deserializing the entire struct. If you only ever access individual fields, consider using separate state variables to save GAS.
:::

---

## Mapping Types

Mapping types use the syntax `mapping(_KeyType => _ValueType)` and variables of mapping type are allowed in state variable arrays, structs, or as state variables themselves.

* `_KeyType` can be any built-in value type, `bytes`, `string`, or any contract or enum type. Other user-defined or complex types, such as mappings, structs or array types are not allowed.
* `_ValueType` can be any type, including mappings, arrays and structs.

```solidity
mapping(address => uint256) public balances;
```

* **NeoVM Mapping:** Mappings do not have a length or a concept of a key or value being "set". On Neo, mappings compile to iterative `SHA256` hashing operations. 
  `balances[addr]` computes `SHA256(serialize(addr) || SHA256("balances"))` and looks up the resulting key in Neo's global `System.Storage.Get` trie.

## Iterable Mappings

You cannot iterate over mappings, i.e. you cannot enumerate their keys. It is possible, though, to implement a data structure on top of them and iterate over that. On NeoVM, this is often done by pairing a `mapping` with an `array` that tracks the inserted keys.
