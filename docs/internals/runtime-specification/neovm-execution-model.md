---
title: "Runtime Specification: NeoVM Execution Model"
description: "NeoVM Execution Model from Runtime Specification."
---

# NeoVM Execution Model

[Back to Runtime Specification](/internals/runtime-specification)

The Neo Virtual Machine (NeoVM) is a stack-based virtual machine designed for executing smart contracts on the Neo N3 blockchain. The embedded runtime in `neo-solc` faithfully emulates this execution model for local testing.

## Stack Machine Architecture

NeoVM uses two stacks for computation:

- **Evaluation Stack** -- the primary operand stack. All instructions consume operands from and push results to this stack. Stack items can be integers (arbitrary precision), byte arrays, booleans, arrays, structs, maps, or interop handles.
- **Alt Stack** -- an auxiliary stack for temporary storage. Items can be moved between the evaluation stack and alt stack using `TOALTSTACK` and `FROMALTSTACK`.

## Slots

NeoVM provides three categories of indexed variable slots, initialized per execution context:

| Slot Type    | Opcodes                                                      | Purpose                       |
| ------------ | ------------------------------------------------------------ | ----------------------------- |
| **Local**    | `LDLOC0`-`LDLOC6`, `LDLOC`, `STLOC0`-`STLOC6`, `STLOC`       | Function-local variables      |
| **Argument** | `LDARG0`-`LDARG6`, `LDARG`, `STARG0`-`STARG6`, `STARG`       | Function parameters           |
| **Static**   | `LDSFLD0`-`LDSFLD6`, `LDSFLD`, `STSFLD0`-`STSFLD6`, `STSFLD` | Contract-level static storage |

Slots are allocated with `INITSLOT` (locals + arguments) or `INITSSLOT` (static slots). Each slot holds a single stack item of any type.

## Execution Contexts

Each function call creates a new execution context containing:

- Its own evaluation stack frame
- Local and argument slots (sized by `INITSLOT`)
- An instruction pointer into the script bytecode
- Exception handling state (try/catch/finally frames)

The `CALL` instruction pushes a new context; `RET` pops it and returns control to the caller.
