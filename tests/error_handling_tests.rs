//! Error handling tests for compiler robustness.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_syntax_error_detected() {
    let source = r#"
    pragma solidity ^0.8.0;
    contract Bad { uint256 x = }
    "#;
    assert!(compile_contracts(source, false, 2).is_err());
}

#[test]
fn test_missing_pragma() {
    let source = r#"
    contract NoPragma { uint256 x; }
    "#;
    // Should still compile (pragma is optional)
    let result = compile_contracts(source, false, 2);
    // Should succeed since pragma is optional in neo-devpack-solidity
    assert!(
        result.is_ok(),
        "Compilation without pragma should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_unclosed_brace() {
    let source = r#"
    pragma solidity ^0.8.0;
    contract Bad { uint256 x;
    "#;
    assert!(compile_contracts(source, false, 2).is_err());
}
