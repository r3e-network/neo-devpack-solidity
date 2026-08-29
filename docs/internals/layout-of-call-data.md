# Layout of Call Data

In Ethereum Solidity, `calldata` is a special, non-modifiable, non-persistent data location where function arguments are stored. Reading from `calldata` is significantly cheaper than loading those arguments into `memory`. EVM uses the `CALLDATALOAD` opcode to slice data from this contiguous byte array.

## Calldata on NeoVM

::: tip 💡 NeoVM Difference
**NeoVM does not have a separate `calldata` region.** All method arguments are simply pushed onto the execution stack by the Neo node prior to the contract's execution.
:::

When the `neo-devpack-solidity` compiler encounters the `calldata` keyword in a Solidity contract, it treats the location identically to `memory`. 

```solidity
function process(bytes calldata inputData) external {
    // EVM: Accesses a specialized read-only execution region
    // NeoVM: The inputData is a standard ByteArray stack item loaded as a local variable
}
```

This means:
1. **Source Compatibility:** Existing Ethereum contracts that optimize for `calldata` will compile perfectly on Neo DevPack for Solidity.
2. **Runtime Execution:** At runtime, the data is just a standard stack item. There is no gas difference between `memory` and `calldata` on NeoVM.
3. **Modifiability:** While NeoVM stack items are technically mutable, the compiler still enforces Solidity's static analysis rules, preventing you from modifying a variable explicitly marked as `calldata`.