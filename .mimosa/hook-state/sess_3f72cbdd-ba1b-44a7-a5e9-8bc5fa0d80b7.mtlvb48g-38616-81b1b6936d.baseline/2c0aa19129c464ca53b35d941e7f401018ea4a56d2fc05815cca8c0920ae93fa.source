//! Canonical intermediate representation for Neo DevPack for Solidity.

use crate::solidity::{
    ContractMetadata, EnumMetadata, EventMetadata, FunctionKind as MetadataFunctionKind,
    FunctionMetadata, ParameterMetadata, SelectorRegistry, StateVariableMetadata,
};
use crate::storage_key::compute_state_slot;
use hex::decode as hex_decode;
use num_bigint::BigInt;
use num_traits::{One, ToPrimitive, Zero};
use sha3::{Digest, Keccak256};
use solang_parser::pt::{
    Expression, HexLiteral as PtHexLiteral, Identifier, Statement,
    StorageLocation as PtStorageLocation, StringLiteral as PtStringLiteral, Type as PtType,
};
use std::collections::{HashMap, HashSet};

mod ir_build;
pub(crate) mod ir_context;
mod ir_deploy;
mod ir_expressions;
mod ir_statements;
mod ir_types;

pub(crate) use ir_build::*;
pub(crate) use ir_context::*;
pub(crate) use ir_deploy::*;
pub(crate) use ir_expressions::*;
pub(crate) use ir_statements::*;
pub use ir_types::*;

pub use ir_context::builtin_intrinsic_surface;

#[cfg(test)]
mod tests;
