---
title: "Runtime Specification: Opcode Support"
description: "Opcode Support from Runtime Specification."
---

# Opcode Support

[Back to Runtime Specification](/internals/runtime-specification)

The embedded runtime implements the opcode subset documented below. Opcodes not handled by the runtime are rejected with an `Unsupported opcode` execution error.

## Constants

| Opcode           | Hex           | Gas | Description                      |
| ---------------- | ------------- | --- | -------------------------------- |
| `PUSHINT8`       | `0x00`        | 1   | Push 1-byte signed integer       |
| `PUSHINT16`      | `0x01`        | 1   | Push 2-byte signed integer       |
| `PUSHINT32`      | `0x02`        | 1   | Push 4-byte signed integer       |
| `PUSHINT64`      | `0x03`        | 1   | Push 8-byte signed integer       |
| `PUSHINT128`     | `0x04`        | 1   | Push 16-byte signed integer      |
| `PUSHINT256`     | `0x05`        | 1   | Push 32-byte signed integer      |
| `PUSHT`          | `0x08`        | 1   | Push `true`                      |
| `PUSHF`          | `0x09`        | 1   | Push `false`                     |
| `PUSHA`          | `0x0A`        | 1   | Push address (pointer)           |
| `PUSHNULL`       | `0x0B`        | 1   | Push null                        |
| `PUSHDATA1`      | `0x0C`        | 2   | Push data (1-byte length prefix) |
| `PUSHDATA2`      | `0x0D`        | 2   | Push data (2-byte length prefix) |
| `PUSHDATA4`      | `0x0E`        | 2   | Push data (4-byte length prefix) |
| `PUSHM1`         | `0x0F`        | 1   | Push -1                          |
| `PUSH0`-`PUSH16` | `0x10`-`0x20` | 1   | Push small integers 0 through 16 |

## Flow Control

| Opcode                    | Hex             | Gas | Description                                      |
| ------------------------- | --------------- | --- | ------------------------------------------------ |
| `NOP`                     | `0x21`          | 1   | No operation                                     |
| `JMP` / `JMP_L`           | `0x22` / `0x23` | 2   | Unconditional jump (short/long offset)           |
| `JMPIF` / `JMPIF_L`       | `0x24` / `0x25` | 2   | Jump if top is true                              |
| `JMPIFNOT` / `JMPIFNOT_L` | `0x26` / `0x27` | 2   | Jump if top is false                             |
| `JMPEQ` / `JMPEQ_L`       | `0x28` / `0x29` | 2   | Jump if top two are equal                        |
| `JMPNE` / `JMPNE_L`       | `0x2A` / `0x2B` | 2   | Jump if top two are not equal                    |
| `JMPGT` / `JMPGT_L`       | `0x2C` / `0x2D` | 2   | Jump if greater than                             |
| `JMPGE` / `JMPGE_L`       | `0x2E` / `0x2F` | 2   | Jump if greater or equal                         |
| `JMPLT` / `JMPLT_L`       | `0x30` / `0x31` | 2   | Jump if less than                                |
| `JMPLE` / `JMPLE_L`       | `0x32` / `0x33` | 2   | Jump if less or equal                            |
| `CALL` / `CALL_L`         | `0x34` / `0x35` | 512 | Call function (short/long offset)                |
| `CALLA`                   | `0x36`          | 512 | Call address on stack                            |
| `CALLT`                   | `0x37`          | 512 | Call method token (native contract optimization) |
| `ABORT`                   | `0x38`          | 1   | Abort execution unconditionally                  |
| `ASSERT`                  | `0x39`          | 1   | Abort if top is false                            |
| `THROW`                   | `0x3A`          | 1   | Throw exception                                  |
| `TRY` / `TRY_L`           | `0x3B` / `0x3C` | 1   | Begin try block (catch/finally offsets)          |
| `ENDTRY` / `ENDTRY_L`     | `0x3D` / `0x3E` | 1   | End try block                                    |
| `ENDFINALLY`              | `0x3F`          | 1   | End finally block (rethrows pending exception)   |
| `RET`                     | `0x40`          | 0   | Return from current context                      |
| `SYSCALL`                 | `0x41`          | 10  | Invoke system interop (4-byte hash follows)      |

## Stack Operations

| Opcode     | Hex    | Gas | Description                  |
| ---------- | ------ | --- | ---------------------------- |
| `DEPTH`    | `0x43` | 2   | Push stack depth             |
| `DROP`     | `0x45` | 2   | Remove top item              |
| `NIP`      | `0x46` | 2   | Remove second item           |
| `XDROP`    | `0x48` | 2   | Remove item at index N       |
| `CLEAR`    | `0x49` | 2   | Clear the stack              |
| `DUP`      | `0x4A` | 2   | Duplicate top item           |
| `OVER`     | `0x4B` | 2   | Copy second item to top      |
| `PICK`     | `0x4D` | 2   | Copy item at index N to top  |
| `TUCK`     | `0x4E` | 2   | Insert top item below second |
| `SWAP`     | `0x50` | 2   | Swap top two items           |
| `ROT`      | `0x51` | 2   | Rotate top three items       |
| `ROLL`     | `0x52` | 2   | Move item at index N to top  |
| `REVERSE3` | `0x53` | 2   | Reverse top 3 items          |
| `REVERSE4` | `0x54` | 2   | Reverse top 4 items          |
| `REVERSEN` | `0x55` | 2   | Reverse top N items          |

## Splice and Buffer

| Opcode      | Hex    | Gas | Description                     |
| ----------- | ------ | --- | ------------------------------- |
| `NEWBUFFER` | `0x88` | 4   | Create new buffer of given size |
| `MEMCPY`    | `0x89` | 4   | Copy bytes between buffers      |
| `CAT`       | `0x8B` | 4   | Concatenate two byte arrays     |
| `SUBSTR`    | `0x8C` | 4   | Extract substring               |
| `LEFT`      | `0x8D` | 4   | Take left N bytes               |
| `RIGHT`     | `0x8E` | 4   | Take right N bytes              |

## Bitwise and Logic

| Opcode     | Hex    | Gas | Description           |
| ---------- | ------ | --- | --------------------- |
| `INVERT`   | `0x90` | 3   | Bitwise NOT           |
| `AND`      | `0x91` | 3   | Bitwise AND           |
| `OR`       | `0x92` | 3   | Bitwise OR            |
| `XOR`      | `0x93` | 3   | Bitwise XOR           |
| `EQUAL`    | `0x97` | 3   | Equality comparison   |
| `NOTEQUAL` | `0x98` | 3   | Inequality comparison |

## Numeric

| Opcode                     | Hex                      | Gas | Description                        |
| -------------------------- | ------------------------ | --- | ---------------------------------- |
| `SIGN`                     | `0x99`                   | 3   | Push sign (-1, 0, 1)               |
| `ABS`                      | `0x9A`                   | 3   | Absolute value                     |
| `NEGATE`                   | `0x9B`                   | 3   | Negate                             |
| `INC` / `DEC`              | `0x9C` / `0x9D`          | 3   | Increment / decrement              |
| `ADD` / `SUB`              | `0x9E` / `0x9F`          | 3   | Addition / subtraction             |
| `MUL` / `DIV` / `MOD`      | `0xA0` / `0xA1` / `0xA2` | 5   | Multiplication / division / modulo |
| `POW`                      | `0xA3`                   | 8   | Exponentiation                     |
| `SQRT`                     | `0xA4`                   | 6   | Integer square root                |
| `MODMUL` / `MODPOW`        | `0xA5` / `0xA6`          | 8   | Modular multiply / modular power   |
| `SHL` / `SHR`              | `0xA8` / `0xA9`          | 3   | Shift left / shift right           |
| `NOT`                      | `0xAA`                   | 2   | Logical NOT                        |
| `BOOLAND` / `BOOLOR`       | `0xAB` / `0xAC`          | 2   | Logical AND / OR                   |
| `NZ`                       | `0xB1`                   | 2   | Non-zero test                      |
| `NUMEQUAL` / `NUMNOTEQUAL` | `0xB3` / `0xB4`          | 3   | Numeric equality                   |
| `LT` / `LE` / `GT` / `GE`  | `0xB5`-`0xB8`            | 3   | Comparison operators               |
| `MIN` / `MAX`              | `0xB9` / `0xBA`          | 3   | Minimum / maximum                  |
| `WITHIN`                   | `0xBB`                   | 3   | Range check (a <= x < b)           |

## Compound and Collections

| Opcode                                  | Hex                      | Gas | Description                   |
| --------------------------------------- | ------------------------ | --- | ----------------------------- |
| `PACKMAP`                               | `0xBE`                   | 4   | Pack key-value pairs into map |
| `PACKSTRUCT`                            | `0xBF`                   | 4   | Pack items into struct        |
| `PACK` / `UNPACK`                       | `0xC0` / `0xC1`          | 4   | Pack/unpack array             |
| `NEWARRAY0` / `NEWARRAY` / `NEWARRAY_T` | `0xC2` / `0xC3` / `0xC4` | 4   | Create arrays                 |
| `NEWSTRUCT0` / `NEWSTRUCT`              | `0xC5` / `0xC6`          | 4   | Create structs                |
| `NEWMAP`                                | `0xC8`                   | 4   | Create empty map              |
| `SIZE`                                  | `0xCA`                   | 4   | Get collection/string size    |
| `HASKEY`                                | `0xCB`                   | 4   | Check if key exists           |
| `KEYS` / `VALUES`                       | `0xCC` / `0xCD`          | 4   | Get map keys / values         |
| `PICKITEM`                              | `0xCE`                   | 4   | Get item by index/key         |
| `APPEND`                                | `0xCF`                   | 4   | Append to array               |
| `SETITEM`                               | `0xD0`                   | 4   | Set item by index/key         |
| `REVERSEITEMS`                          | `0xD1`                   | 4   | Reverse collection in place   |
| `REMOVE`                                | `0xD2`                   | 4   | Remove item by index/key      |
| `CLEARITEMS`                            | `0xD3`                   | 4   | Clear all items               |
| `POPITEM`                               | `0xD4`                   | 4   | Pop last item from array      |

## Type Operations

| Opcode    | Hex    | Gas | Description                    |
| --------- | ------ | --- | ------------------------------ |
| `ISNULL`  | `0xD8` | 2   | Check if top is null           |
| `ISTYPE`  | `0xD9` | 2   | Check if top matches type code |
| `CONVERT` | `0xDB` | 2   | Convert top to specified type  |

Supported type codes:

| Code   | Type             |
| ------ | ---------------- |
| `0x20` | Boolean          |
| `0x21` | Integer          |
| `0x28` | ByteString       |
| `0x30` | Buffer           |
| `0x40` | Array            |
| `0x41` | Struct           |
| `0x48` | Map              |
| `0x60` | InteropInterface |

## Extensions

| Opcode      | Hex    | Gas | Description         |
| ----------- | ------ | --- | ------------------- |
| `ABORTMSG`  | `0xE0` | 1   | Abort with message  |
| `ASSERTMSG` | `0xE1` | 1   | Assert with message |
