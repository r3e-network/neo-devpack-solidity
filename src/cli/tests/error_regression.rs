//! Error-handling regression tests for the compiler CLI.
//!
//! These tests verify that invalid Solidity inputs are rejected with a
//! structured `CompileError` carrying an `NSH-XXXX` error code rather than
//! panicking or returning an opaque message.

use crate::cli::{compile_contracts, compile_contracts_with_options, CompileError, CompileOptions};
use crate::diagnostics::{Diagnostic, ErrorCode};

fn first_error_code(err: &CompileError) -> Option<ErrorCode> {
    let diags: Option<&[Diagnostic]> = match err {
        CompileError::Diagnostics(d) => Some(d),
        CompileError::Semantic(d) => Some(d),
        CompileError::Ir(d) => Some(d),
        CompileError::ParseErrors(d) => Some(d),
        CompileError::Manifest(d) => return Some(d.code),
        CompileError::Io { .. } | CompileError::Message(_) => None,
    };
    diags.and_then(|d| d.first()).map(|d| d.code)
}

#[test]
fn parse_error_carries_nsh1000() {
    let source = r#"
    pragma solidity ^0.8.19;
    contract Broken {
        function f( public pure returns (uint256) { return 1; }
    }
    "#;
    let err = compile_contracts(source, false, 0).expect_err("expected parse error");
    let code = first_error_code(&err).expect("expected a diagnostic code");
    assert!(
        matches!(
            code,
            ErrorCode::Nsh1000
                | ErrorCode::Nsh1001
                | ErrorCode::Nsh1002
                | ErrorCode::Nsh1003
                | ErrorCode::Nsh1004
                | ErrorCode::Nsh1005
        ),
        "parse error should carry an NSH-1xxx code, got {code:?}"
    );
}

#[test]
fn semantic_duplicate_signature_carries_nsh2000() {
    let source = r#"
    pragma solidity ^0.8.19;
    contract D {
        function foo(uint256 a) public {}
        function foo(uint256 a) public {}
    }
    "#;
    let err = compile_contracts(source, false, 0).expect_err("expected semantic error");
    let code = first_error_code(&err).expect("expected a diagnostic code");
    assert_eq!(
        code,
        ErrorCode::Nsh2000,
        "duplicate signature should be NSH-2000"
    );
}

#[test]
fn ir_invalid_storage_param_carries_nsh3000() {
    let source = r#"
    pragma solidity ^0.8.19;
    contract E {
        function foo(uint256[] storage x) public {}
    }
    "#;
    let err = compile_contracts(source, false, 0).expect_err("expected IR error");
    let code = first_error_code(&err).expect("expected a diagnostic code");
    assert!(
        matches!(
            code,
            ErrorCode::Nsh3000
                | ErrorCode::Nsh3001
                | ErrorCode::Nsh3002
                | ErrorCode::Nsh3003
                | ErrorCode::Nsh3004
                | ErrorCode::Nsh3005
        ),
        "IR error should carry an NSH-3xxx code, got {code:?}"
    );
}

#[test]
fn manifest_wildcard_permission_carries_nsh6000() {
    let source = r#"
    pragma solidity ^0.8.19;
    contract FullyDynamicCalls {
        function callAny(address target, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;
    let mut opts = CompileOptions::new(0, false);
    opts.deny_wildcard_permissions = true;
    let err =
        compile_contracts_with_options(source, false, opts).expect_err("expected manifest error");
    let code = first_error_code(&err).expect("expected a diagnostic code");
    assert!(
        matches!(
            code,
            ErrorCode::Nsh6000 | ErrorCode::Nsh6001 | ErrorCode::Nsh6002
        ),
        "manifest error should carry an NSH-6xxx code, got {code:?}"
    );
}
