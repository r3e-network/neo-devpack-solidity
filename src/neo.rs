//! Neo N3 Utilities Module
//!
//! Provides Neo N3 blockchain-specific utilities for NEF (Neo Executable Format)
//! generation and method tokens.
//!
//! # Key Components
//!
//! - [`MethodToken`](crate::neo::MethodToken) - Cross-contract call optimization tokens
//! - [`build_nef_with_tokens`](crate::neo::build_nef_with_tokens) - NEF file generation with method tokens
//! - [`clamp_nef_source_with_flag`](crate::neo::clamp_nef_source_with_flag) - Source code embedding in NEF
//!
//! # NEF Format
//!
//! The Neo Executable Format (NEF) is the standard container for Neo N3 smart
//! contracts, containing bytecode, metadata, and optional method tokens.

mod build;
mod constants;
mod contract_hash;
mod encoding;
mod method_token;
mod source;

// Re-export public API
pub use build::{build_nef, build_nef_with_tokens, parse_nef, ParsedNef};
pub use constants::{
    MAX_CALL_FLAGS, MAX_METHOD_TOKENS, MAX_SCRIPT_SIZE, MAX_TOKEN_METHOD_LENGTH, NEF_HEADER_SIZE,
    NEF_MAGIC, NEF_SOURCE_MAX_BYTES,
};
pub use contract_hash::{compute_contract_hash, format_uint160_hex_be, parse_uint160_hex_be};
pub use method_token::MethodToken;
pub use source::{clamp_nef_source, clamp_nef_source_with_flag};

#[cfg(test)]
mod tests;
