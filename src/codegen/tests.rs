use super::*;
use crate::interop::interop_id_bytes;
use crate::solidity::analyse_source;
use crate::{
    frontend::VisibilityKind,
    solidity::{FunctionKind, FunctionMetadata, StateMutability, ParameterMetadata},
};

include!("tests/integration.rs");
include!("tests/helpers.rs");
include!("tests/neovm_limits.rs");
