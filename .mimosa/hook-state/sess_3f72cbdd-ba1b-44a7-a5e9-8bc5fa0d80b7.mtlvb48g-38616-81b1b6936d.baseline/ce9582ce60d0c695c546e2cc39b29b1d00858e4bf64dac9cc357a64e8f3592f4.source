//! Solidity analysis sub-modules for contract metadata extraction.
//!
//! Each sub-module handles a specific phase of Solidity contract analysis:
//! - `inheritance` — C3 linearization, flattening, interface collection
//! - `modifiers` — modifier expansion, rewriting, and application
//! - `siblings` — sibling contract merge detection
//! - `merge_helpers` — state variable type normalization for merge checks

// Re-export types and functions needed by the sub-modules (they use `use super::*`).
// This includes cross-module function references:
// - `inheritance/flatten.rs` calls `rewrite_statement` from `modifiers/rewrite`
// - `modifiers/expand.rs` calls `base_last_name` from `inheritance/helpers`
// - `modifiers/constructors.rs` calls `inheritance_contract_chain` from `inheritance/flatten`
pub(crate) use crate::frontend::{
    ContractIR, ContractKind, EnumIR, ErrorIR, EventIR, FunctionIR, ParameterIR, StateVariableIR,
    StructIR, VisibilityKind,
};
pub(crate) use crate::solidity::SolidityError;
pub(crate) use solang_parser::pt::{
    Base, CatchClause, Expression, FunctionTy, Identifier, Loc, NamedArgument, Parameter, Statement,
};

pub(crate) mod inheritance;
pub(crate) mod merge_helpers;
pub(crate) mod modifiers;
pub(crate) mod siblings;
