# Inline Assembly

Solidity defines an assembly language that can be used without Solidity. This assembly language can also be used as "inline assembly" inside Solidity source code. 

::: tip 💡 NeoVM Difference
In Ethereum, inline assembly gives you direct access to the EVM stack and opcodes (e.g., `sload`, `sstore`, `call`). **On NeoVM, the compiler lowers a limited Yul compatibility subset and warns on anything outside that subset** because the opcodes and stack architecture fundamentally differ. Treat assembly blocks as portability scaffolding, not as EVM-opcode-level access.
:::

## How Neo DevPack for Solidity Handles Inline Assembly

When the Neo DevPack for Solidity compiler encounters an `assembly { ... }` block, it parses
Yul syntax and lowers a documented compatibility subset to NeoVM IR. This is not
EVM opcode parity: unsupported Yul constructs fall back to a compatibility
warning and emit no assembly logic for that block.

The supported subset is meant for common library idioms, not arbitrary EVM
assembly. It includes:

- Yul locals and assignments (`let x := ...`, `x := ...`)
- Reads from Solidity locals and parameters
- `mstore`, `mload`, and `return` over a bounded scratch memory buffer
- `tstore` and `tload` transient storage helpers
- `returndatasize()` and guarded `returndatacopy(...)`
- Arithmetic, bitwise, shift, and comparison helpers such as `add`, `sub`,
  `mul`, `div`, `mod`, `and`, `or`, `xor`, `not`, `shl`, `shr`, `lt`, `gt`,
  `eq`, and `iszero`
- Simple `if`, `for`, `switch`, and nested block forms when every nested
  expression is also in the supported subset

Unsupported EVM-only operations such as `sload`, `sstore`, raw `call`, and
`create` do not map directly to NeoVM. Rewrite those paths with Solidity,
devpack libraries, or explicit Neo native-contract/syscall wrappers.

```solidity
function loadWord() public pure returns (uint256) {
    assembly {
        mstore(0, 42)
        return(0, 32)
    }
}
```

## NeoVM Alternatives

If your contract requires Neo-specific low-level operations, prefer Neo's native
intrinsics and Syscalls directly.

For example, to execute a cross-contract call with dynamic parameters or specific execution flags, use the `Syscalls` library rather than EVM's `call` opcode.

```solidity
import {Syscalls, CallFlags} from "@neo/Syscalls.sol";

function callAnotherContract(address target, string memory method) public {
    // Replaces the need for low-level assembly calls
    Syscalls.contractCall(target, method, CallFlags.All, new any[](0));
}
```

For advanced mathematics, cryptography, or memory manipulation, the Neo Devpack exposes a comprehensive suite of native libraries designed explicitly for the NeoVM environment.
