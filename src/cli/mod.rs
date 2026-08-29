//! Neo DevPack for Solidity CLI Module
//!
//! Command-line interface for the Neo DevPack for Solidity compiler. This module provides
//! the main entry point for compiling Solidity contracts to NeoVM bytecode.
//!
//! # Features
//!
//! - Single file and multi-file compilation
//! - Standard JSON input/output (Solidity compiler compatible)
//! - NEF (Neo Executable Format) generation
//! - Manifest generation for Neo N3 deployment
//! - Multiple optimization levels (0-3)
//! - Verbose output
//!
//! # Submodules
//!
//! - `bytecode` - NeoVM bytecode generation from IR
//! - `standard_json` - Standard JSON compilation interface
//!
//! # Usage
//!
//! ```bash
//! neo-solc input.sol -o build/
//! neo-solc --standard-json < input.json > output.json
//! neo-solc --standard-json --input input.json --output output.json
//! ```

// The non-test CLI code imports what it needs inside each `cli_parts`
// submodule. These names are pulled into scope only for the `#[cfg(test)]
// mod tests;` tree below, whose modules reach them via `use super::*`.
// Gating them on `cfg(test)` keeps the release build free of unused imports.
#[cfg(test)]
use crate::ir;
#[cfg(test)]
use crate::solidity::{validate_contract, ContractMetadata, FunctionKind, FunctionMetadata};
#[cfg(test)]
use serde_json::{json, Value};
#[cfg(test)]
use sha3::{Digest, Keccak256};

#[cfg(test)]
pub(crate) use crate::codegen::generate_contract_bytecode;
#[cfg(test)]
pub(crate) use crate::interop::interop_id_bytes;
// Public re-export of the NeoVM bytecode disassembler so external tooling
// (fuzz targets, debugging consumers) can call it without touching
// `pub(crate)` internals. The function is total — any byte sequence
// produces a string without panicking. See
// `fuzz/fuzz_targets/fuzz_target_disasm.rs`.
pub use crate::codegen::disassemble_neovm_bytecode;
#[cfg(test)]
pub(crate) use crate::optimizer::optimize_ir;
mod standard_json;
pub(crate) use standard_json::*;

/// Fuzz-only public entry point for the Solidity standard-JSON input parser.
///
/// Accepts arbitrary bytes, converts to UTF-8 (returns `Err` on invalid
/// UTF-8 rather than panicking), and invokes the internal standard-JSON
/// processor with safe defaults (no output path, default options). The
/// function is total — it must never panic. Callers (the fuzz target)
/// ignore the returned `Result` and only watch for panics.
///
/// This wrapper exists so `fuzz/fuzz_targets/fuzz_target_standard_json.rs`
/// can exercise the standard-JSON parser without touching `pub(crate)`
/// internals. See that target for the libfuzzer harness.
pub fn fuzz_process_standard_json_content(input: &[u8]) -> Result<(), String> {
    let input_str = std::str::from_utf8(input)
        .map_err(|err| format!("standard-json fuzz input was not valid UTF-8: {err}"))?;
    let options = StandardJsonOptions {
        optimizer_level: 0,
        use_callt: false,
        deny_wildcard_permissions: false,
        deny_wildcard_contracts: false,
        deny_wildcard_methods: false,
        nef_source: None,
        manifest_permissions: None,
        contract_names: Vec::new(),
    };
    process_standard_json_content(input_str, None, options)
}

mod cli_parts;
#[cfg(test)]
pub(crate) use crate::manifest::{
    detect_supported_standards, solidity_to_manifest_type, StandardsDiagnosticLevel,
};
pub(crate) use cli_parts::cli_analyze::*;
pub use cli_parts::cli_compile::*;
pub use cli_parts::cli_defs::*;
pub(crate) use cli_parts::cli_deploy::*;
pub(crate) use cli_parts::cli_diagnostics::*;
pub(crate) use cli_parts::cli_output::*;
pub use cli_parts::cli_run::*;

#[cfg(test)]
mod tests;
