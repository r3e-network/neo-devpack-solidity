//! ## Source-Level Solidity Analysis Pipeline
//!
//! This module is the **entry point** for Solidity source analysis. It takes
//! raw Solidity source text and produces validated `ContractMetadata` structs
//! ready for IR lowering and bytecode generation.
//!
//! The pipeline is split into focused submodules that each handle one stage:
//!
//! | Stage | Module | Description |
//! |-------|--------|-------------|
//! | 1 — Parse & classify | `classify` | Parse sources, separate primary (contract/abstract) from fallback (library/interface) |
//! | 2 — Library validation | `library_validation` | Pre-merge struct pools, normalize libraries, validate each |
//! | 3 — Sibling merge | `sibling_merge` | Merge sibling primary functions/modifiers/state/events/ctors into each primary (Task #83) |
//! | 4 — Type sharing | `type_sharing` | Cross-contract struct/enum namespace sharing |
//! | 5 — Inheritance & conversion | `convert_stage` | Flatten inheritance, expand modifiers, convert to metadata |
//!
//! ### Related tasks
//!
//! - **Task #83**: `new B()` sibling dispatch → merge target functions into caller
//! - **Task #115**: Interface casts → match interface methods to sibling primaries
//! - **Task #126**: `fallback()` as universal dispatcher for sibling merge
//! - **Task #194**: Low-level `.call()` selector resolution
//! - **Task #197**: Sibling state-variable merge
//! - **Task #198**: Sibling constructor inline
//! - **Task #206**: Transitive sibling reference closure

use super::*;

pub(crate) mod classify;
pub(crate) mod convert_stage;
pub(crate) mod library_validation;
pub(crate) mod sibling_merge;
pub(crate) mod type_sharing;

pub(crate) use classify::*;
pub(crate) use convert_stage::*;
pub(crate) use library_validation::*;
pub(crate) use sibling_merge::*;
pub(crate) use type_sharing::*;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn analyse_source(source: &str) -> Result<ContractMetadata, SolidityError> {
    let mut contracts = analyse_all_sources(source)?;
    Ok(contracts.swap_remove(0))
}

// ============================================================================
// analyse_all_sources — Main pipeline
// ============================================================================

pub fn analyse_all_sources(source: &str) -> Result<Vec<ContractMetadata>, SolidityError> {
    // -----------------------------------------------------------------------
    // Stage 1 — Parse sources and separate primary (contract/abstract) from
    // fallback (library/interface) contracts.
    // -----------------------------------------------------------------------
    let (mut primary, fallback, has_primary, pre_merge_contract_map, contract_types) =
        classify_contracts(source)?;

    // -------------------------------------------------------------------
    // Stage 2 — Validate and normalize user libraries before merging them
    // into primary contracts. Pre-populates a cross-library struct/enum
    // pool so cross-references between libraries resolve correctly.
    // -------------------------------------------------------------------
    let libraries = validate_libraries(has_primary, &fallback, &contract_types)?;

    // -------------------------------------------------------------------
    // Stage 3 — Sibling merge. When a primary contract references a sibling
    // primary, merge the sibling's functions/modifiers/state/events/ctors
    // into the referencing contract so self-dispatch works at runtime.
    // -------------------------------------------------------------------
    if has_primary {
        perform_sibling_merge(&mut primary, &pre_merge_contract_map)?;
    }

    // -------------------------------------------------------------------
    // Stage 4 — Cross-contract struct/enum namespace sharing.
    // Make non-inherited type definitions visible across compilation units
    // so expressions like `Enum.Operation.DelegateCall` resolve even when
    // the defining type lives in another contract file.
    // -------------------------------------------------------------------
    share_type_definitions(has_primary, &mut primary, &pre_merge_contract_map);

    // -------------------------------------------------------------------
    // Stage 5 — Build selector registry, flatten inheritance, expand
    // modifiers, and convert each contract to ContractMetadata.
    // -------------------------------------------------------------------
    let contract_map: std::collections::HashMap<String, ContractIR> = primary
        .iter()
        .chain(fallback.iter())
        .map(|contract| (contract.name.clone(), contract.clone()))
        .collect();

    let selected = if has_primary { primary } else { fallback };
    convert_all_contracts(
        selected,
        &contract_map,
        &contract_types,
        &libraries,
        has_primary,
    )
}
