//! Canonical Neo N3 VM opcode definitions.
//!
//! The variant names follow the canonical Neo N3 opcode spelling
//! (e.g. `JMP_L`, `JMPIF_L`, `NEWARRAY_T`, `LDSFLD0`..`LDSFLD6`,
//! `PUSH0`..`PUSH16`). These deliberately mix digits, underscores,
//! and uppercase letters to match the spec, so we silence the
//! camel-case lint locally instead of mangling the names.
#![allow(non_camel_case_types)]
//!
//! This module is the **single source of truth** for every NeoVM opcode
//! byte value used anywhere in the compiler, runtime simulator, and
//! disassembler. Code that emits or interprets NeoVM bytecode must go
//! through [`OpCode`](crate::opcode::OpCode) rather than hardcoding byte literals — that way
//! renumbering, validation, gas accounting, and disassembler output
//! stay in lockstep with the spec.
//!
//! # Layout
//!
//! Opcodes fall into two categories:
//!
//! * **Fixed** opcodes ([`OpCode::ADD`](crate::opcode::OpCode::ADD), [`OpCode::JMP`](crate::opcode::OpCode::JMP), …) have a
//!   single, named variant and a single byte value.
//! * **Indexed** opcode families ([`OpCode::PUSH0`](crate::opcode::OpCode::PUSH0)..[`OpCode::PUSH16`](crate::opcode::OpCode::PUSH16),
//!   [`OpCode::LDLOC0`](crate::opcode::OpCode::LDLOC0)..[`OpCode::LDLOC6`](crate::opcode::OpCode::LDLOC6), [`OpCode::LDLOC`](crate::opcode::OpCode::LDLOC),
//!   [`OpCode::LDSFLD0`](crate::opcode::OpCode::LDSFLD0).., [`OpCode::LDARG0`](crate::opcode::OpCode::LDARG0).., [`OpCode::STLOC0`](crate::opcode::OpCode::STLOC0)..,
//!   [`OpCode::STSFLD0`](crate::opcode::OpCode::STSFLD0).., [`OpCode::STARG0`](crate::opcode::OpCode::STARG0)..) share a byte
//!   prefix/suffix. Use the [`OpCode::push_small`](crate::opcode::OpCode::push_small), [`OpCode::ldloc`](crate::opcode::OpCode::ldloc),
//!   [`OpCode::stloc`](crate::opcode::OpCode::stloc), [`OpCode::ldarg`](crate::opcode::OpCode::ldarg), [`OpCode::starg`](crate::opcode::OpCode::starg),
//!   [`OpCode::ldsfld`](crate::opcode::OpCode::ldsfld), [`OpCode::stsfld`](crate::opcode::OpCode::stsfld) constructors instead of
//!   computing `0x10 + n`.
//!
//! # Variants for data-bearing opcodes
//!
//! [`OpCode::PUSHDATA1`](crate::opcode::OpCode::PUSHDATA1) / [`OpCode::PUSHDATA2`](crate::opcode::OpCode::PUSHDATA2) / [`OpCode::PUSHDATA4`](crate::opcode::OpCode::PUSHDATA4)
//! and [`OpCode::PUSHINT8`](crate::opcode::OpCode::PUSHINT8)..[`OpCode::PUSHINT256`](crate::opcode::OpCode::PUSHINT256) are all distinct
//! opcodes that the VM dispatches on. [`OpCode::push_data`](crate::opcode::OpCode::push_data) and
//! [`OpCode::push_int`](crate::opcode::OpCode::push_int) select the right one based on operand size.
//!
//! # Usage
//!
//! ```
//! use neo_devpack_solidity::opcode::OpCode;
//!
//! let mut script = Vec::new();
//! script.push(OpCode::ABORT.byte());
//! script.push(OpCode::PUSH0.byte());
//! assert_eq!(script, vec![0x38, 0x10]);
//! ```

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpCode {
    // === Constants / Push (0x00–0x20) ===
    /// Push a signed 8-bit integer.
    PUSHINT8 = 0x00,
    /// Push a signed 16-bit integer.
    PUSHINT16 = 0x01,
    /// Push a signed 32-bit integer.
    PUSHINT32 = 0x02,
    /// Push a signed 64-bit integer.
    PUSHINT64 = 0x03,
    /// Push a signed 128-bit integer.
    PUSHINT128 = 0x04,
    /// Push a signed 256-bit integer.
    PUSHINT256 = 0x05,
    /// Push `true`.
    PUSHT = 0x08,
    /// Push `false`.
    PUSHF = 0x09,
    /// Push the current instruction pointer's address.
    PUSHA = 0x0A,
    /// Push `null`.
    PUSHNULL = 0x0B,
    /// Push a `≤ u8::MAX` byte string with a 1-byte length prefix.
    PUSHDATA1 = 0x0C,
    /// Push a `≤ u16::MAX` byte string with a 2-byte length prefix.
    PUSHDATA2 = 0x0D,
    /// Push a `≤ u32::MAX` byte string with a 4-byte length prefix.
    PUSHDATA4 = 0x0E,
    /// Push `-1` as an integer.
    PUSHM1 = 0x0F,
    /// Push the integer `0` (also `PUSH0`).
    PUSH0 = 0x10,
    /// Push the integer `1`.
    PUSH1 = 0x11,
    /// Push the integer `2`.
    PUSH2 = 0x12,
    /// Push the integer `3`.
    PUSH3 = 0x13,
    /// Push the integer `4`.
    PUSH4 = 0x14,
    /// Push the integer `5`.
    PUSH5 = 0x15,
    /// Push the integer `6`.
    PUSH6 = 0x16,
    /// Push the integer `7`.
    PUSH7 = 0x17,
    /// Push the integer `8`.
    PUSH8 = 0x18,
    /// Push the integer `9`.
    PUSH9 = 0x19,
    /// Push the integer `10`.
    PUSH10 = 0x1A,
    /// Push the integer `11`.
    PUSH11 = 0x1B,
    /// Push the integer `12`.
    PUSH12 = 0x1C,
    /// Push the integer `13`.
    PUSH13 = 0x1D,
    /// Push the integer `14`.
    PUSH14 = 0x1E,
    /// Push the integer `15`.
    PUSH15 = 0x1F,
    /// Push the integer `16`.
    PUSH16 = 0x20,

    // === Flow control (0x21–0x41) ===
    /// No operation.
    NOP = 0x21,
    /// Unconditional relative jump (1-byte offset).
    JMP = 0x22,
    /// Unconditional relative jump (4-byte offset).
    JMP_L = 0x23,
    /// Conditional jump if top of stack is truthy (1-byte offset).
    JMPIF = 0x24,
    /// Conditional jump if top of stack is truthy (4-byte offset).
    JMPIF_L = 0x25,
    /// Conditional jump if top of stack is falsy (1-byte offset).
    JMPIFNOT = 0x26,
    /// Conditional jump if top of stack is falsy (4-byte offset).
    JMPIFNOT_L = 0x27,
    /// Jump if the top two stack items are equal (1-byte offset).
    JMPEQ = 0x28,
    /// Jump if the top two stack items are equal (4-byte offset).
    JMPEQ_L = 0x29,
    /// Jump if the top two stack items are not equal (1-byte offset).
    JMPNE = 0x2A,
    /// Jump if the top two stack items are not equal (4-byte offset).
    JMPNE_L = 0x2B,
    /// Jump if the second item is greater than the top (1-byte offset).
    JMPGT = 0x2C,
    /// Jump if the second item is greater than the top (4-byte offset).
    JMPGT_L = 0x2D,
    /// Jump if the second item is greater than or equal to the top (1-byte offset).
    JMPGE = 0x2E,
    /// Jump if the second item is greater than or equal to the top (4-byte offset).
    JMPGE_L = 0x2F,
    /// Jump if the second item is less than the top (1-byte offset).
    JMPLT = 0x30,
    /// Jump if the second item is less than the top (4-byte offset).
    JMPLT_L = 0x31,
    /// Jump if the second item is less than or equal to the top (1-byte offset).
    JMPLE = 0x32,
    /// Jump if the second item is less than or equal to the top (4-byte offset).
    JMPLE_L = 0x33,
    /// Call the address at the top of the stack (1-byte offset).
    CALL = 0x34,
    /// Call the address at the top of the stack (4-byte offset).
    CALL_L = 0x35,
    /// Call the address on top of the stack as a tail call.
    CALLA = 0x36,
    /// Call a typed method by 2-byte index into the current script's method table.
    CALLT = 0x37,
    /// Abort execution.
    ABORT = 0x38,
    /// Assert the top of the stack is truthy, otherwise abort.
    ASSERT = 0x39,
    /// Throw the top of the stack as an exception.
    THROW = 0x3A,
    /// Try/catch (1-byte offset).
    TRY = 0x3B,
    /// Try/catch (4-byte offset).
    TRY_L = 0x3C,
    /// End of a `try` block (1-byte offset).
    ENDTRY = 0x3D,
    /// End of a `try` block (4-byte offset).
    ENDTRY_L = 0x3E,
    /// End of a `finally` block.
    ENDFINALLY = 0x3F,
    /// Return from the current execution context.
    RET = 0x40,
    /// Invoke a system call identified by the next 4 bytes (interop ID).
    SYSCALL = 0x41,

    // === Stack ops (0x43–0x55) ===
    /// Push the current evaluation stack depth.
    DEPTH = 0x43,
    /// Drop the top of the stack.
    DROP = 0x45,
    /// Remove the second-to-top stack item.
    NIP = 0x46,
    /// Drop the stack item at the given 1-byte index.
    XDROP = 0x48,
    /// Clear the evaluation stack.
    CLEAR = 0x49,
    /// Duplicate the top of the stack.
    DUP = 0x4A,
    /// Copy the second-to-top stack item to the top.
    OVER = 0x4B,
    /// Copy the item at the given 1-byte index to the top.
    PICK = 0x4D,
    /// Copy the top of the stack to the position below the second item.
    TUCK = 0x4E,
    /// Swap the top two stack items.
    SWAP = 0x50,
    /// Rotate the top three stack items.
    ROT = 0x51,
    /// Roll the top `n+1` stack items by 1 (1-byte count).
    ROLL = 0x52,
    /// Reverse the order of the top 3 stack items.
    REVERSE3 = 0x53,
    /// Reverse the order of the top 4 stack items.
    REVERSE4 = 0x54,
    /// Reverse the order of the top `n+1` stack items (1-byte count).
    REVERSEN = 0x55,

    // === Slot / local / argument ops (0x56–0x87) ===
    /// Initialize static field slots (1-byte count).
    INITSSLOT = 0x56,
    /// Initialize static-field + local + argument slots (3 1-byte counts).
    INITSLOT = 0x57,
    /// Load static field 0.
    LDSFLD0 = 0x58,
    /// Load static field 1.
    LDSFLD1 = 0x59,
    /// Load static field 2.
    LDSFLD2 = 0x5A,
    /// Load static field 3.
    LDSFLD3 = 0x5B,
    /// Load static field 4.
    LDSFLD4 = 0x5C,
    /// Load static field 5.
    LDSFLD5 = 0x5D,
    /// Load static field 6.
    LDSFLD6 = 0x5E,
    /// Load static field by 1-byte index.
    LDSFLD = 0x5F,
    /// Store into static field 0.
    STSFLD0 = 0x60,
    /// Store into static field 1.
    STSFLD1 = 0x61,
    /// Store into static field 2.
    STSFLD2 = 0x62,
    /// Store into static field 3.
    STSFLD3 = 0x63,
    /// Store into static field 4.
    STSFLD4 = 0x64,
    /// Store into static field 5.
    STSFLD5 = 0x65,
    /// Store into static field 6.
    STSFLD6 = 0x66,
    /// Store into static field by 1-byte index.
    STSFLD = 0x67,
    /// Load local variable 0.
    LDLOC0 = 0x68,
    /// Load local variable 1.
    LDLOC1 = 0x69,
    /// Load local variable 2.
    LDLOC2 = 0x6A,
    /// Load local variable 3.
    LDLOC3 = 0x6B,
    /// Load local variable 4.
    LDLOC4 = 0x6C,
    /// Load local variable 5.
    LDLOC5 = 0x6D,
    /// Load local variable 6.
    LDLOC6 = 0x6E,
    /// Load local variable by 1-byte index.
    LDLOC = 0x6F,
    /// Store into local variable 0.
    STLOC0 = 0x70,
    /// Store into local variable 1.
    STLOC1 = 0x71,
    /// Store into local variable 2.
    STLOC2 = 0x72,
    /// Store into local variable 3.
    STLOC3 = 0x73,
    /// Store into local variable 4.
    STLOC4 = 0x74,
    /// Store into local variable 5.
    STLOC5 = 0x75,
    /// Store into local variable 6.
    STLOC6 = 0x76,
    /// Store into local variable by 1-byte index.
    STLOC = 0x77,
    /// Load function argument 0.
    LDARG0 = 0x78,
    /// Load function argument 1.
    LDARG1 = 0x79,
    /// Load function argument 2.
    LDARG2 = 0x7A,
    /// Load function argument 3.
    LDARG3 = 0x7B,
    /// Load function argument 4.
    LDARG4 = 0x7C,
    /// Load function argument 5.
    LDARG5 = 0x7D,
    /// Load function argument 6.
    LDARG6 = 0x7E,
    /// Load function argument by 1-byte index.
    LDARG = 0x7F,
    /// Store into function argument 0.
    STARG0 = 0x80,
    /// Store into function argument 1.
    STARG1 = 0x81,
    /// Store into function argument 2.
    STARG2 = 0x82,
    /// Store into function argument 3.
    STARG3 = 0x83,
    /// Store into function argument 4.
    STARG4 = 0x84,
    /// Store into function argument 5.
    STARG5 = 0x85,
    /// Store into function argument 6.
    STARG6 = 0x86,
    /// Store into function argument by 1-byte index.
    STARG = 0x87,

    // === Buffer / string (0x88–0x8E) ===
    /// Allocate a new empty buffer.
    NEWBUFFER = 0x88,
    /// Copy `count` bytes from one buffer to another (3 1-byte operands).
    MEMCPY = 0x89,
    /// Concatenate two byte strings.
    CAT = 0x8B,
    /// Substring (2 1-byte operands).
    SUBSTR = 0x8C,
    /// Left-most bytes (1-byte count).
    LEFT = 0x8D,
    /// Right-most bytes (1-byte count).
    RIGHT = 0x8E,

    // === Bitwise / logic (0x90–0x98) ===
    /// Bitwise NOT (one's complement) on the top of the stack.
    INVERT = 0x90,
    /// Bitwise AND of the top two stack items.
    AND = 0x91,
    /// Bitwise OR of the top two stack items.
    OR = 0x92,
    /// Bitwise XOR of the top two stack items.
    XOR = 0x93,
    /// Logical equality of the top two stack items.
    EQUAL = 0x97,
    /// Logical inequality of the top two stack items.
    NOTEQUAL = 0x98,

    // === Numeric (0x99–0xAC) ===
    /// Push `-1`, `0`, or `1` according to the sign of the top of the stack.
    SIGN = 0x99,
    /// Replace the top of the stack with its absolute value.
    ABS = 0x9A,
    /// Negate the top of the stack.
    NEGATE = 0x9B,
    /// Increment the top of the stack by 1.
    INC = 0x9C,
    /// Decrement the top of the stack by 1.
    DEC = 0x9D,
    /// Add the top two stack items.
    ADD = 0x9E,
    /// Subtract the top of the stack from the second item.
    SUB = 0x9F,
    /// Multiply the top two stack items.
    MUL = 0xA0,
    /// Integer divide the second item by the top.
    DIV = 0xA1,
    /// Integer modulo of the second item by the top.
    MOD = 0xA2,
    /// Raise the second item to the power of the top.
    POW = 0xA3,
    /// Integer square root of the top of the stack.
    SQRT = 0xA4,
    /// Modular multiplication (operand order: bottom × top % modulus).
    MODMUL = 0xA5,
    /// Modular exponentiation.
    MODPOW = 0xA6,
    /// Shift the second item left by the top.
    SHL = 0xA8,
    /// Shift the second item right by the top.
    SHR = 0xA9,
    /// Logical NOT of the top of the stack.
    NOT = 0xAA,
    /// Boolean AND of the top two stack items.
    BOOLAND = 0xAB,
    /// Boolean OR of the top two stack items.
    BOOLOR = 0xAC,

    // === Comparisons (0xB1–0xBB) ===
    /// Push `true` if the top of the stack is non-zero.
    NZ = 0xB1,
    /// Numeric equality of the top two stack items.
    NUMEQUAL = 0xB3,
    /// Numeric inequality of the top two stack items.
    NUMNOTEQUAL = 0xB4,
    /// Push `true` if the second item is less than the top.
    LT = 0xB5,
    /// Push `true` if the second item is less than or equal to the top.
    LE = 0xB6,
    /// Push `true` if the second item is greater than the top.
    GT = 0xB7,
    /// Push `true` if the second item is greater than or equal to the top.
    GE = 0xB8,
    /// Push the minimum of the top two stack items.
    MIN = 0xB9,
    /// Push the maximum of the top two stack items.
    MAX = 0xBA,
    /// Push `true` if the third item is within `[min, max)` of the second.
    WITHIN = 0xBB,

    // === Compound / collections (0xBE–0xD4) ===
    /// Pack a map from a 1-byte count of key/value pairs on the stack.
    PACKMAP = 0xBE,
    /// Pack a struct from a 1-byte count of items on the stack.
    PACKSTRUCT = 0xBF,
    /// Pack an array from a 1-byte count of items on the stack.
    PACK = 0xC0,
    /// Unpack an array/struct onto the stack.
    UNPACK = 0xC1,
    /// Push a new empty array.
    NEWARRAY0 = 0xC2,
    /// Push a new array of items already on the stack (1-byte count).
    NEWARRAY = 0xC3,
    /// Push a new array of a specific stack-item type (1-byte type tag).
    NEWARRAY_T = 0xC4,
    /// Push a new empty struct.
    NEWSTRUCT0 = 0xC5,
    /// Push a new struct of items already on the stack (1-byte count).
    NEWSTRUCT = 0xC6,
    /// Push a new empty map.
    NEWMAP = 0xC8,
    /// Push the size of the top-of-stack collection / buffer / string.
    SIZE = 0xCA,
    /// Push `true` if the map has the given key (top).
    HASKEY = 0xCB,
    /// Push all keys of the map on top of the stack as an array.
    KEYS = 0xCC,
    /// Push all values of the map on top of the stack as an array.
    VALUES = 0xCD,
    /// Push the item at the given 1-byte index of the top array/struct/map.
    PICKITEM = 0xCE,
    /// Append the second item to the array on top of the stack.
    APPEND = 0xCF,
    /// Set the value of the map at the given key.
    SETITEM = 0xD0,
    /// Reverse the items of the top array/struct in place.
    REVERSEITEMS = 0xD1,
    /// Remove the entry with the given key from the top map.
    REMOVE = 0xD2,
    /// Clear all items from the top collection.
    CLEARITEMS = 0xD3,
    /// Pop the last item of the top array and push it.
    POPITEM = 0xD4,

    // === Types (0xD8–0xDB) ===
    /// Push `true` if the top of the stack is `null`.
    ISNULL = 0xD8,
    /// Push `true` if the top of the stack has the given 1-byte type tag.
    ISTYPE = 0xD9,
    /// Convert the top of the stack to the given 1-byte type tag.
    CONVERT = 0xDB,

    // === Extensions (0xE0–0xE1) ===
    /// Abort with a 1-byte-prefixed message.
    ABORTMSG = 0xE0,
    /// Assert-with-message; same as `ASSERT` but with a payload.
    ASSERTMSG = 0xE1,
}

mod conversion;
mod methods;
