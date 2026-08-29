//! CLI sub-modules for the Neo DevPack for Solidity compiler.
//!
//! This module groups the CLI implementation files using standard Rust
//! module declarations for clean IDE navigation and rust-analyzer support.

use crate::cli::{
    build_upgrade_report, compile_contracts_with_options, compiler_version_string_4,
    contract_output_prefix, emit_error, emit_error_with_suggestion, emit_warning,
    emit_warning_with_suggestion, ensure_deploy_stub, ensure_output_dir,
    load_manifest_permissions_override, print_upgrade_reports, process_standard_json,
    process_standard_json_content, sanitize_contract_name, standard_json, write_assembly_file,
    write_json_file, write_manifest_file, write_nef_file, write_upgrade_reports,
    CompilationArtifacts, CompileError, CompileOptions, StandardJsonOptions, COMPILER_EMAIL,
    COMPILER_ID,
};
use crate::codegen;
use crate::codegen::generate_contract_bytecode;
use crate::frontend::VisibilityKind;
use crate::ir;
use crate::manifest::build_manifest;
use crate::neo::{build_nef_with_tokens, clamp_nef_source_with_flag, NEF_SOURCE_MAX_BYTES};
use crate::optimizer::optimize_ir;
use crate::semantic_model::build_semantic_model;
// These imports are consumed by child modules via `use super::*`.
// The compiler's unused-import detection doesn't track through glob
// imports, so `#[allow(unused_imports)]` suppresses false positives.
#[allow(unused_imports)]
use crate::solidity::{
    analyse_all_sources, validate_contract, ContractMetadata, DiagnosticSeverity, FunctionKind,
    FunctionMetadata, NatspecDoc, ParameterMetadata, StateMutability,
};
use crate::type_system::NeoType;
use serde_json::{json, Value};
use sha3::{Digest, Keccak256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// Re-export the unified diagnostic types so every CLI sub-module can render
// errors with stable NSH-XXXX codes without importing them individually.
pub use crate::diagnostics::{Diagnostic, ErrorCode, Severity};

pub(crate) mod cli_analyze;
pub(crate) mod cli_compile;
pub(crate) mod cli_defs;
pub(crate) mod cli_deploy;
pub(crate) mod cli_diagnostics;
pub(crate) mod cli_output;
pub(crate) mod cli_run;
