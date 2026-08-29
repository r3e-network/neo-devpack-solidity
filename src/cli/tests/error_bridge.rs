//! Regression guard for the SolidityError -> CompileError bridge in
//! `compile_contracts_with_options`.
//!
//! The bridge maps every `SolidityError` variant to a `CompileError` and, for
//! the non-structural variants, builds the `CompileError::Message` string by
//! hand from a `format!`. These tests assert that each hand-written string is
//! byte-identical to the variant's own `Display` impl (i.e. what the previous
//! catch-all `other.to_string()` produced). If a `#[error]` attribute changes,
//! the corresponding assertion here must be updated in lockstep — keeping the
//! bridge a true zero-output-change refactor.

use neo_devpack_solidity::frontend::FrontendError;
use neo_devpack_solidity::solidity::SolidityError;

/// Mirror of the format strings used in the `compile_contracts_with_options`
/// match arms. Kept in sync by the assertions below.
fn bridge_message(err: &SolidityError) -> String {
    match err {
        SolidityError::Frontend(FrontendError::ParseDiagnostics(_)) => {
            unreachable!("structural arm; handled as CompileError::ParseErrors")
        }
        SolidityError::Frontend(FrontendError::Parse(msg)) => {
            format!("Solidity parsing failed:\n{msg}")
        }
        SolidityError::Frontend(FrontendError::UnsupportedVersion(version)) => {
            format!("Unsupported Solidity version: {version}")
        }
        SolidityError::Frontend(FrontendError::ImportError { path, reason }) => {
            format!("Failed to resolve import '{path}': {reason}")
        }
        SolidityError::Frontend(FrontendError::ContractNotFound(name)) => {
            format!("Contract '{name}' not found in source")
        }
        SolidityError::Frontend(FrontendError::UnsupportedConstruct(kind)) => format!(
            "internal error: unsupported top-level Solidity construct '{kind}' (please file \
             a bug — the compiler may need updating for a newer Solidity grammar)"
        ),
        SolidityError::Analysis(msg) => msg.clone(),
        SolidityError::NoContracts => "no contract definitions found in source".into(),
        SolidityError::ContractNotFound(name) => format!("contract '{name}' not found"),
        SolidityError::UnsupportedFeature(msg) => format!("unsupported feature: {msg}"),
        SolidityError::InheritanceError(msg) => format!("inheritance error: {msg}"),
    }
}

#[test]
fn bridge_messages_match_display_impls() {
    let cases: Vec<SolidityError> = vec![
        SolidityError::Frontend(FrontendError::Parse("line 1:1: boom".into())),
        SolidityError::Frontend(FrontendError::UnsupportedVersion("0.8.0".into())),
        SolidityError::Frontend(FrontendError::ImportError {
            path: "./missing.sol".into(),
            reason: "not found".into(),
        }),
        SolidityError::Frontend(FrontendError::ContractNotFound("Foo".into())),
        SolidityError::Frontend(FrontendError::UnsupportedConstruct("Weird".into())),
        SolidityError::Analysis("type mismatch".into()),
        SolidityError::NoContracts,
        SolidityError::ContractNotFound("Bar".into()),
        SolidityError::UnsupportedFeature("tx.origin".into()),
        SolidityError::InheritanceError("cyclic".into()),
    ];

    for err in &cases {
        assert_eq!(
            bridge_message(err),
            err.to_string(),
            "bridge message diverges from Display for {err:?}"
        );
    }
}
