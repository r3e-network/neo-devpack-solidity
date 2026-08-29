---
title: "Runtime Specification: Event and Notification System"
description: "Event and Notification System from Runtime Specification."
---

# Event and Notification System

[Back to Runtime Specification](/internals/runtime-specification)

## Runtime.Notify Semantics

The `System.Runtime.Notify` syscall emits a notification event consisting of:

- **Event name**: A string identifying the event type
- **State array**: An array of stack items containing the event data

In Solidity, `emit` statements are compiled to `Runtime.Notify` calls. The event name corresponds to the Solidity event name, and the indexed/non-indexed parameters are packed into the state array.

```solidity
// Solidity
emit Transfer(from, to, amount);

// Compiles to NeoVM:
// PUSH3 args onto stack, PACK into array
// PUSHDATA "Transfer"
// SYSCALL System.Runtime.Notify
```

## Runtime.Log

`System.Runtime.Log` emits a simple string log message. This is used for debugging and does not appear in on-chain event logs.
