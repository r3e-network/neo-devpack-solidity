# Layout in Memory

Solidity reserves four 32-byte slots in memory for special purposes in the EVM, and relies on a linear memory array that expands as data is written.

::: tip 💡 NeoVM Difference
**NeoVM does not have a linear byte-array memory model.** Instead, memory in NeoVM is managed through an abstract evaluation stack consisting of structured `StackItem` objects (Integers, ByteArrays, Arrays, Maps). 
:::

## Stack-Based Memory Allocation

When you declare a `memory` variable in Neo DevPack for Solidity, the compiler allocates a local slot on the execution stack. 

```solidity
function doSomething() public {
    // EVM: allocates memory starting at the free memory pointer
    // NeoVM: creates a new Array stack item and stores it in a local variable slot
    uint256[] memory myMemoryArray = new uint256[](5);
}
```

### Structs in Memory

In memory, structs are represented as NeoVM `Array` types. The fields of the struct are accessed via their positional index within the array.

```solidity
struct Proposal {
    address creator;
    uint256 votes;
}

// In memory: represented as a NeoVM Array `[creator, votes]`
Proposal memory p = Proposal(msg.sender, 0);
```

### `calldata` vs `memory`

On Ethereum, `calldata` is a non-modifiable, non-persistent area where function arguments are stored, and behaves differently than `memory`. 

On NeoVM, **`calldata` is treated identically to `memory`**. The compiler accepts the `calldata` keyword to maintain strict source compatibility with Ethereum contracts, but internally, function arguments are simply read from the execution stack.