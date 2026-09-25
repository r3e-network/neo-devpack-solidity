//! Manifest generation module
//!
//! Builds Neo N3 contract manifests (ABI methods, events, permissions) and
//! handles manifest custom overrides. Extracted from `cli/cli_parts/cli_manifest/`
//! as part of Architecture Phase 1 (v0.29.1).

use crate::codegen;
use crate::frontend::VisibilityKind;
use crate::ir;
use crate::solidity::{ContractMetadata, EventMetadata, FunctionKind, FunctionMetadata};
use crate::type_system::NeoType;
use serde_json::{json, Value};
// `Digest` trait is consumed by `permissions/native.rs` via `use super::*`
// for `sha2::Sha256::digest()` trait method resolution.
#[allow(unused_imports)]
use sha3::{Digest, Keccak256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Manifest-specific error type. Mapped to `CompileError::Manifest` at the
/// CLI boundary so the manifest module doesn't depend on `crate::cli`.
#[derive(Debug)]
pub struct ManifestError(pub String);

impl From<String> for ManifestError {
    fn from(s: String) -> Self {
        ManifestError(s)
    }
}

/// Compiler identity constants. These are defined here (mirroring
/// `cli_parts/cli_defs.rs`) so the manifest module doesn't depend on
/// `crate::cli`. The values are identical and stable.
const COMPILER_ID: &str = concat!("neo-devpack-solidity-", env!("CARGO_PKG_VERSION"));
const COMPILER_EMAIL: &str = "Jimmy <jimmy@r3e.network>";

fn compiler_version_string_4() -> String {
    let mut parts = env!("CARGO_PKG_VERSION")
        .split('.')
        .map(|p| p.parse::<u32>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    format!("{major}.{minor}.{patch}.0")
}

mod build;
pub(crate) use build::*;
mod standards;
pub(crate) use standards::*;
mod permissions;
pub(crate) use permissions::*;

/// Map a Solidity type string to its Neo N3 manifest type name.
///
/// Moved here from `standard_json_output.rs` to break the bidirectional
/// dependency between `cli_manifest` and `standard_json`. Both the manifest
/// builder and the standard-JSON output use this function.
pub(crate) fn solidity_to_manifest_type(solidity_type: &str) -> &'static str {
    let ty = solidity_type.trim().to_ascii_lowercase();

    // Gap `nep11` — the devpack NeoVM iterator handle (`Syscalls.Iterator`,
    // also reachable as `Storage.Iterator`). Returning it from a public
    // method leaves the raw iterator stack item as the NeoVM return value,
    // so the manifest must declare `InteropInterface` (the NEP-11 spec type
    // for `tokensOf`/`tokens`). The builtin helper libraries are never
    // struct-merged into user contracts, so this type always arrives here as
    // a raw type string with `neo_type == None`.
    if matches!(
        ty.as_str(),
        "iterator" | "syscalls.iterator" | "storage.iterator"
    ) {
        return "InteropInterface";
    }

    // Array types must be checked FIRST (before checking base types). Any
    // trailing `]` is an array — both dynamic `T[]` and fixed-size `T[N]`
    // (which ends with `]` but not `[]`) — so `uint256[3]` returns "Array",
    // not "Integer".
    if ty.ends_with(']') {
        return "Array";
    }

    // Mapping types
    if ty.starts_with("mapping") {
        return "Map";
    }

    // Integer types (uint8-256, int8-256)
    if ty.starts_with("uint") || ty.starts_with("int") {
        return "Integer";
    }

    // Boolean
    if ty == "bool" || ty == "boolean" {
        return "Boolean";
    }

    // String
    if ty == "string" {
        return "String";
    }

    // Address types (Neo uses Hash160 for 20-byte addresses)
    if ty == "address" || ty == "address payable" || ty == "bytes20" || ty == "hash160" {
        return "Hash160";
    }

    // Hash types (must check before generic bytes handling)
    if ty == "bytes32" || ty == "hash256" {
        return "Hash256";
    }

    // Fixed-size byte arrays (bytes1-32)
    if ty == "bytes" {
        return "ByteArray";
    }
    if ty.starts_with("bytes") {
        // bytes1, bytes2, ..., bytes32 are fixed-size
        if let Some(size_str) = ty.strip_prefix("bytes") {
            if size_str.parse::<u8>().is_ok() {
                return if size_str == "32" {
                    "Hash256"
                } else {
                    "ByteArray"
                };
            }
        }
        return "ByteArray";
    }

    // Void/empty return type
    if ty == "void" || ty.is_empty() {
        return "Void";
    }

    // Struct and other complex types
    "Any"
}
