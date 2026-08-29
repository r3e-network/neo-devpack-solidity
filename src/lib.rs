//! Neo DevPack for Solidity Library
//!
//! A Solidity to NeoVM bytecode compiler for Neo N3 smart contracts.
//!
//! # Public API
//!
//! - [`cli`] — compiler entry point (`compile_contracts`, `disassemble_neovm_bytecode`)
//! - [`neo`] — NEF/manifest building and parsing utilities
//!
//! All other modules are `#[doc(hidden)]` — internal/test-facing, not part
//! of the documented public contract.
//!
//! Author: Jimmy <jimmy@r3e.network>

// Allow referring to this crate by name (`neo_devpack_solidity::...`) internally.
extern crate self as neo_devpack_solidity;

// === Public API (documented) ===

/// Compiler diagnostics: `Diagnostic`, `ErrorCode`, `SourceSpan`, `Severity`,
/// `Suggestion`, and `CompilerResult`.
///
/// Unified error representation used across the parser, semantic analysis,
/// IR lowering, optimizer, codegen, and manifest stages. Every user-facing
/// error carries a stable `NSH-XXXX` code and an optional source location.
pub mod diagnostics;

/// Compiler CLI: `compile_contracts`, `disassemble_neovm_bytecode`, standard-JSON.
pub mod cli;

/// NEF/manifest building, parsing, method tokens, contract hashes.
pub mod neo;

/// Canonical Neo N3 VM [`OpCode`](opcode::OpCode) definitions.
///
/// Single source of truth for every NeoVM opcode byte used by the
/// compiler, runtime simulator, and disassembler. See [`opcode`] for
/// the enum and the helper constructors for indexed opcodes
/// (`PUSH0..PUSH16`, `LDLOC0..LDLOC6`, `PUSHDATA1/2/4`, …).
pub mod opcode;

// === Internal / test-facing (hidden from docs) ===

#[doc(hidden)]
pub mod codegen;
#[doc(hidden)]
pub mod frontend;
#[doc(hidden)]
pub mod interop;
#[doc(hidden)]
pub mod ir;
#[doc(hidden)]
pub mod manifest;
#[doc(hidden)]
pub mod optimizer;
#[doc(hidden)]
pub mod runtime;
#[doc(hidden)]
pub mod semantic_model;
#[doc(hidden)]
pub mod solidity;
#[doc(hidden)]
pub mod storage_key;
#[doc(hidden)]
pub mod type_system;
#[doc(hidden)]
pub mod utils;

/// Compiler version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
