//! Solidity frontend integration using `solang-parser`.
//!
//! This module parses Solidity source code into a light-weight intermediate
//! representation that can be consumed by later compiler stages.

use solang_parser::{
    diagnostics::Diagnostic,
    parse,
    pt::{
        Base, Comment, ContractDefinition, ContractPart, ContractTy, EnumDefinition,
        EventDefinition, Expression, FunctionAttribute, FunctionDefinition, FunctionTy, Identifier,
        Loc, Mutability, ParameterList, SourceUnitPart, Statement, StorageLocation,
        StructDefinition, Using, UsingFunction, UsingList, VariableAttribute, VariableDefinition,
        Visibility,
    },
};
use std::collections::HashMap;
use thiserror::Error;

mod frontend_convert;
mod frontend_diagnostics;
mod frontend_errors;
mod frontend_guarded_parse;
mod frontend_ir;
mod frontend_parse;

pub(crate) use frontend_convert::*;
pub(crate) use frontend_diagnostics::*;
pub(crate) use frontend_errors::*;
pub(crate) use frontend_guarded_parse::*;
pub(crate) use frontend_ir::*;
pub use frontend_parse::*;

#[cfg(test)]
mod tests;
