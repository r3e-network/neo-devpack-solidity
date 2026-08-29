# Cleaning Up Variables

In Ethereum Solidity, variables occupying fewer than 256 bits in memory or storage must be carefully "cleaned" (masked) because the EVM stack operates strictly on 256-bit words. When a `uint8` is loaded, the compiler must guarantee the upper 248 bits are zeroed out before performing operations like `JUMPI` or comparisons.

## Cleanup on NeoVM

::: tip 💡 NeoVM Difference
**NeoVM uses strictly typed StackItems, not a flat 256-bit word stack.** Variable cleanup operations are largely unnecessary and omitted by the compiler.
:::

On Neo, an integer is an arbitrary-precision `BigInteger`. There is no concept of "upper bits" containing garbage data from a previous operation.

```solidity
uint8 a = 255;
uint256 b = a;
```

In EVM, this requires careful masking to ensure `b` does not inherit trailing bytes. In NeoVM, `a` is simply the `BigInteger` `255`. Assigning it to `b` carries over the exact numeric value with zero structural masking required.

### Struct and Array Cleanup

Similarly, when deleting data from storage, Ethereum must issue `SSTORE 0` to zero out the data slot. 

On NeoVM, `delete` operations compile directly to the `System.Storage.Delete` syscall, which entirely removes the Key-Value pair from the node's database, freeing up the state footprint efficiently.