//! NeoVM Bytecode Generation Module
//!
//! Converts the intermediate representation (IR) to NeoVM bytecode. This module
//! handles the low-level emission of NeoVM opcodes and manages function dispatch,
//! storage operations, and syscall generation.
//!
//! # Architecture
//!
//! The bytecode generator works in several phases:
//! 1. **Function dispatch** - Generates entry point with method selector routing
//! 2. **IR lowering** - Converts IR instructions to NeoVM opcodes
//! 3. **Optimization** - Applies peephole optimizations to reduce bytecode size
//! 4. **Patching** - Resolves function call targets and jump addresses
//!
//! # NeoVM Opcodes
//!
//! Key opcodes used:
//! - `0x00-0x20` - Push operations (PUSH0, PUSHINT8, etc.)
//! - `0x40` - RET (return from function)
//! - `0x41` - SYSCALL (invoke system call)
//! - `0x45-0x4D` - Control flow (JMP, JMPIF, CALL, etc.)
//!
//! # Storage Model
//!
//! Storage keys are computed using SHA-256 hashing of variable names and
//! mapping keys, following Neo N3 storage conventions.

use crate::frontend::VisibilityKind;
use crate::ir::{self, LiteralValue, ValueType};
use crate::solidity::{ContractMetadata, FunctionKind, FunctionMetadata};

#[cfg(test)]
use crate::solidity::NatspecDoc;
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};
use std::collections::HashMap;

mod bytecode_builtins;
mod bytecode_core;
mod bytecode_disasm;
mod bytecode_emit_ir;
mod bytecode_helpers;

pub(crate) use bytecode_builtins::*;
pub(crate) use bytecode_core::*;
pub(crate) use bytecode_emit_ir::*;
pub(crate) use bytecode_helpers::*;

pub use bytecode_disasm::disassemble_neovm_bytecode;

#[cfg(test)]
pub(crate) mod tests;
