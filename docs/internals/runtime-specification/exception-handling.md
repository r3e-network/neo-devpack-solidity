---
title: "Runtime Specification: Exception Handling"
description: "Exception Handling from Runtime Specification."
---

# Exception Handling

[Back to Runtime Specification](/internals/runtime-specification)

## TRY / CATCH / FINALLY Opcodes

NeoVM supports structured exception handling:

- `TRY` begins a try block, specifying catch and/or finally offsets
- `THROW` raises an exception (top of stack becomes the exception object)
- `ENDTRY` exits the try block normally, jumping to the end offset
- `ENDFINALLY` exits the finally block; if an exception is pending, it is rethrown

## Exception Propagation

1. When `THROW` is executed, the VM searches the current context's exception handler stack
2. If a catch handler is found, execution jumps to the catch offset with the exception on the stack
3. If a finally handler exists, it executes before propagation continues
4. If no handler is found in the current context, the exception propagates to the caller
5. `ENDFINALLY` rethrows any pending exception that was not handled by the catch block

## Gas Effects

Gas continues to be consumed during exception handling. The `TRY`, `ENDTRY`, and `ENDFINALLY` opcodes each cost 1 gas unit. Stack unwinding during propagation does not have additional gas cost beyond the opcodes executed.
