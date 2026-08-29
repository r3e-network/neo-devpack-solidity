use super::*;
use neo_devpack_solidity::frontend::VisibilityKind;
use neo_devpack_solidity::solidity::{
    analyse_source, Diagnostic, DiagnosticSeverity, EventMetadata, EventParameter, NatspecDoc,
    ParameterMetadata, SelectorRegistry, StateMutability, StateVariableMetadata,
};
use neo_devpack_solidity::type_system::NeoType;
use sha3::{Digest, Keccak256};

include!("metadata/identifiers.rs");
include!("metadata/standards.rs");
include!("metadata/deploy.rs");
include!("metadata/output_and_storage.rs");
include!("metadata/state_variables.rs");
include!("metadata/erc_nep_patterns.rs");
