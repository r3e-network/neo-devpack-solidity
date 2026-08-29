//! Neo Runtime Module
//!
//! Complete runtime integration layer providing EVM compatibility on NeoVM,
//! state management, storage operations, and execution environment.
//!
//! The `spec` submodule (opcode/syscall/native-contract lookup tables) is
//! always compiled: the `--format assembly` disassembler
//! (`src/cli/bytecode/bytecode_disasm/disassemble.rs`) depends on it for the
//! default toolchain. The simulator proper (`bridge`, `execution`, `state`,
//! `storage`, `types`, `runtime_parts`) is only exercised by tests and fuzz
//! targets, so it is gated behind the `runtime` feature (on by default). Pass
//! `--no-default-features` for a leaner binary that skips the ~17K-LOC NeoVM
//! execution simulator entirely.

// Always available — lookup tables shared with the disassembler.
pub mod spec;

// Simulator — only needed for tests/fuzz.
#[cfg(feature = "runtime")]
pub mod bridge;
#[cfg(feature = "runtime")]
pub mod execution;
#[cfg(feature = "runtime")]
pub mod state;
#[cfg(feature = "runtime")]
pub mod storage;
#[cfg(feature = "runtime")]
pub mod types;

#[cfg(feature = "runtime")]
use hex;

#[cfg(feature = "runtime")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "runtime")]
use std::collections::HashMap;
#[cfg(feature = "runtime")]
use thiserror::Error;

#[cfg(feature = "runtime")]
mod runtime_parts;

#[cfg(feature = "runtime")]
pub use runtime_parts::*;

#[cfg(feature = "runtime")]
#[cfg(test)]
mod tests;
