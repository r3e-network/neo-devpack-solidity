# Expressions and Control Structures

## Control Structures

Most of the control structures known from curly-brace languages are available in Solidity:

There is: `if`, `else`, `while`, `do`, `for`, `break`, `continue`, `return`, with the usual semantics known from C or JavaScript.

Solidity also supports exception handling in the form of `try`/`catch` statements, but only for external function calls and contract creation calls.

* **NeoVM Mapping:** All basic control structures lower perfectly to NeoVM's jump instructions (`JMP`, `JMPIF`, `JMPIFNOT`).

## Function Calls

### Internal Function Calls

Functions of the current contract can be called directly ("internally"), also recursively. 

```solidity
function g(uint a) public pure returns (uint ret) { return f(); }
function f() internal pure returns (uint ret) { return g(7) + f(); }
```
These map to standard `CALL` instructions within the generated NeoVM script, utilizing the same execution stack.

### External Function Calls

The expressions `this.g(8);` and `c.g(2);` (where `c` is a contract instance) are also valid function calls, but this time, the function will be called "externally", via a message call.

```solidity
interface InfoFeed {
    function info() external payable returns (uint ret);
}

contract Consumer {
    InfoFeed feed;
    function doCall() public {
        feed.info();
    }
}
```

External calls compile down to the NeoVM syscall `System.Contract.Call`. 

### Function Call Options (Value & Gas)

On Ethereum, you can append options to an external call to forward Ether or set a gas limit: `feed.info{value: 10, gas: 800}()`.

::: tip 💡 NeoVM Difference: Call Options Ignored
Because NeoVM handles value transfers strictly via the NEP-17 standard and lacks per-call gas limits, **these call options are gracefully ignored.** 

The compiler emits a warning notifying you that the options were dropped, but compilation succeeds. To actually transfer value on Neo, you must execute `NativeCalls.gasTransfer(...)` before or after your contract call.
:::

## Creating Contracts via `new`

A contract can create a new contract using the `new` keyword. The full code of the contract being created has to be known when the creating contract is compiled, so recursive creation-dependencies are not possible.

```solidity
D newD = new D(arg);
```

::: tip 💡 NeoVM Difference: Dynamic Deployment
Creating contracts dynamically via `new Contract()` does not deploy a Neo contract. The current compiler keeps this construct source-compatible by inlining/simulating constructor-like logic when the target contract is available and returning a zero-address placeholder.

NeoVM handles deployment differently than EVM. To deploy a child contract, use Neo's `ContractManagement.deploy(nef, manifest, data)` intrinsic directly, passing the compiled NEF bytecode and manifest.
:::

## Order of Evaluation of Expressions

The evaluation order of expressions is not specified (more formally, the order in which the children of one node in the expression tree are evaluated is not specified, but they are of course evaluated before the node itself). It is only guaranteed that statements are executed in order and short-circuiting for boolean expressions is done.

## Assignment

### Destructuring Assignments and Returning Multiple Values

Solidity internally allows tuple types, i.e. a list of objects of potentially different types whose number is a constant at compile-time. Those tuples can be used to return multiple values at the same time.

```solidity
function f() public pure returns (uint, bool, uint) {
    return (7, true, 2);
}

function g() public {
    // Destructuring assignment
    (uint x, bool b, uint y) = f();
}
```

* **NeoVM Mapping:** Tuples are mapped to NeoVM `Array` stack items. The compiler automatically unpacks the array upon destructuring.

## Scoping and Declarations

A variable which is declared will have an initial default value whose byte-representation is all zeros. The "default values" of variables are the typical "zero-state" of whatever the type is. For example, the default value for a `bool` is `false`.

Scoping rules in Solidity follow those of C99 (variables are visible from the point right after their declaration until the end of the smallest `{ }`-block that contains the declaration).

## Error Handling: Assert, Require, Revert and Exceptions

Solidity uses state-reverting exceptions to handle errors. Such an exception undoes all changes made to the state in the current call (and all its sub-calls) and flags an error to the caller.

### `assert` and `require`

The convenience functions `assert` and `require` can be used to check for conditions and throw an exception if the condition is not met.

*   `require` should be used to ensure valid conditions, such as inputs, or contract state variables are met, or to validate return values from calls to external contracts.
*   `assert` should only be used to test for internal errors, and to check invariants.

On EVM, `assert` failures consume all remaining gas and produce a Panic error. 
* **NeoVM Mapping:** On NeoVM, `assert` simply aborts execution — there is no gas penalty distinction between `require` and `assert`. Both map to NeoVM `ASSERT` or `ASSERTMSG` opcodes.

### `revert`

The `revert` statement can be used to flag an error and revert the current call. You can provide a string message or a custom error.

```solidity
error Unauthorized(address caller);

function restricted() public {
    if (msg.sender != owner) {
        revert Unauthorized(msg.sender);
        // NeoVM: ABORTMSG with "Unauthorized" and caller arg
    }
}
```

### `try`/`catch`

A failure in an external call can be caught using a try/catch statement.

```solidity
try target.riskyCall() returns (uint256 result) {
    // Success path
    processResult(result);
} catch Error(string memory reason) {
    // String exception
    emit CallFailed(reason);
} catch (bytes memory lowLevelData) {
    // Catch-all for other exceptions
    emit CallFailedRaw(lowLevelData);
}
```

* **NeoVM Mapping:** Maps to NeoVM `TRY`/`ENDTRY` structured exception handling. Note that NeoVM exceptions are untyped, so the compiler uses `ISTYPE` runtime guards to route by stack item type to the correct catch block. Multiple return values (`try returns(uint a, uint b)`) are fully supported; they map to a single NeoVM `Array` return which the compiler automatically unpacks via destructuring.
