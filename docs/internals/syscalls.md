---
title: "Syscalls"
description: "Syscalls section index."
---

# Syscalls

NeoVM syscalls are the interface between smart contract bytecode and the Neo N3 blockchain runtime. Each syscall is identified by a 4-byte hash derived from SHA-256 of its name string. The neo-devpack-solidity compiler lowers Solidity constructs to these syscalls automatically.

When the NeoVM encounters a `SYSCALL` opcode, it reads the next 4 bytes as the syscall ID, looks up the corresponding handler in the interop service table, and dispatches execution. The compiler handles this translation transparently — you write Solidity, and the correct syscall sequence is emitted in the NEF output.

## Sections

| Section |
| --- |
| [Overview](/internals/syscalls/overview) |
| [Syscall Categories](/internals/syscalls/syscall-categories) |
| [Storage Syscalls](/internals/syscalls/storage-syscalls) |
| [Runtime Syscalls](/internals/syscalls/runtime-syscalls) |
| [Contract Syscalls](/internals/syscalls/contract-syscalls) |
| [Crypto Syscalls](/internals/syscalls/crypto-syscalls) |
| [Iterator Syscalls](/internals/syscalls/iterator-syscalls) |
| [Gas Cost Reference](/internals/syscalls/gas-cost-reference) |
| [Solidity to Syscall Mapping](/internals/syscalls/solidity-to-syscall-mapping) |
| [Devpack Wrapper Reference](/internals/syscalls/devpack-wrapper-reference) |
| [See Also](/internals/syscalls/see-also) |
