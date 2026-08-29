//! Standard JSON Compilation Interface
//!
//! Implements the Solidity compiler standard JSON input/output format for
//! compatibility with existing tooling (Hardhat, Foundry, etc.).
//!
//! # Input Format
//!
//! ```json
//! {
//!   "language": "Solidity",
//!   "sources": {
//!     "Contract.sol": { "content": "..." }
//!   },
//!   "settings": {
//!     "optimizer": { "enabled": true, "runs": 200 }
//!   }
//! }
//! ```
//!
//! # Output Format
//!
//! Returns compiled contracts with bytecode, ABI, and metadata in JSON format
//! compatible with standard Solidity compiler output.

use super::{
    compile_contracts_with_options, compiler_version_string_4, CompilationArtifacts,
    CompileOptions, ManifestPermissionsOverride, COMPILER_ID,
};
use neo_devpack_solidity::frontend::{parse_source, ContractKind, VisibilityKind};
use neo_devpack_solidity::neo::{
    build_nef_with_tokens, clamp_nef_source_with_flag, NEF_SOURCE_MAX_BYTES,
};
use neo_devpack_solidity::solidity::{
    ContractMetadata, DiagnosticSeverity, FunctionKind, StateMutability,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha3::{Digest, Keccak256};
use std::collections::{HashMap, VecDeque};
use std::fs;

mod standard_json_abi;
mod standard_json_diagnostics;
mod standard_json_helpers;
mod standard_json_output;
mod standard_json_process;
mod standard_json_types;

pub(crate) use standard_json_abi::*;
pub(crate) use standard_json_diagnostics::*;
pub(crate) use standard_json_helpers::*;
pub(crate) use standard_json_output::*;
pub(crate) use standard_json_process::*;
pub(crate) use standard_json_types::*;
