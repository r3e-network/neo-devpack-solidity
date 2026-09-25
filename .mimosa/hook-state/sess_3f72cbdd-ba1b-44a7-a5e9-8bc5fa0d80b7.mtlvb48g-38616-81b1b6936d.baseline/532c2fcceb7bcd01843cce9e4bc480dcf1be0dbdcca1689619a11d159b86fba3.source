use super::*;

#[test]
fn non_builtin_libraries_are_merged_for_internal_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    library MathLib {
        function double(uint256 x) internal pure returns (uint256) {
            return x * 2;
        }
    }

    contract UsesLib {
        function f(uint256 x) public pure returns (uint256) {
            return MathLib.double(x);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn non_builtin_libraries_allow_external_static_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    library MathLib {
        function bump(uint256 x) external pure returns (uint256) {
            return x + 1;
        }
    }

    contract UsesLib {
        function f(uint256 x) public pure returns (uint256) {
            return MathLib.bump(x);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn non_builtin_libraries_allow_external_member_calls_via_using() {
    let source = r#"
    pragma solidity ^0.8.19;

    library MathLib {
        function bump(uint256 self, uint256 delta) external pure returns (uint256) {
            return self + delta;
        }
    }

    contract UsesLib {
        using MathLib for uint256;

        function f(uint256 x) public pure returns (uint256) {
            return x.bump(1);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn builtin_libraries_are_not_merged_into_contract_metadata() {
    let source = r#"
    pragma solidity ^0.8.19;

    library Syscalls {
        // Even if a Solidity source provides an implementation, `Syscalls.*` is treated as a
        // compiler intrinsic and should not be merged into the user contract.
        function getTime() internal view returns (uint256) {
            return 123;
        }
    }

    contract UsesSyscalls {
        function nowTs() public view returns (uint256) {
            return Syscalls.getTime();
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let method_names: Vec<_> = artifacts[0]
        .metadata
        .methods
        .iter()
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        method_names.contains(&"nowTs"),
        "expected user method to remain present"
    );
    assert!(
        !method_names.contains(&"getTime"),
        "expected Syscalls.getTime to not be merged into contract metadata"
    );
}

#[test]
fn using_for_typed_receiver_enforces_type_compatibility() {
    let source = r#"
    pragma solidity ^0.8.19;

    library UintLib {
        function bump(uint256 self, uint256 delta) internal pure returns (uint256) {
            return self + delta;
        }
    }

    contract UsesTypedUsing {
        using UintLib for uint256;

        function ok(uint256 x) public pure returns (uint256) {
            return x.bump(1);
        }

        function bad(bool flag) public pure returns (uint256) {
            return flag.bump(1);
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected receiver type mismatch");
    match err {
        CompileError::Ir(messages) => {
            assert!(
                messages
                    .iter()
                    .any(|diag| diag.message.contains("not available for receiver type")),
                "unexpected diagnostics: {messages:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn using_function_list_restricts_member_names() {
    let source = r#"
    pragma solidity ^0.8.19;

    library MathLib {
        function add(uint256 self, uint256 rhs) internal pure returns (uint256) {
            return self + rhs;
        }

        function sub(uint256 self, uint256 rhs) internal pure returns (uint256) {
            return self - rhs;
        }
    }

    contract UsesFunctionList {
        using {MathLib.add} for uint256;

        function ok(uint256 x) public pure returns (uint256) {
            return x.add(1);
        }

        function bad(uint256 x) public pure returns (uint256) {
            return x.sub(1);
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected function-list rejection");
    match err {
        CompileError::Ir(messages) => {
            assert!(
                messages.iter().any(|diag| diag
                    .message
                    .contains("not allowed by `using {...} for ...` function lists")),
                "unexpected diagnostics: {messages:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn member_style_library_calls_require_using_directive() {
    let source = r#"
    pragma solidity ^0.8.19;

    library MathLib {
        function add(uint256 self, uint256 rhs) internal pure returns (uint256) {
            return self + rhs;
        }
    }

    contract MissingUsing {
        function bad(uint256 x) public pure returns (uint256) {
            return x.add(1);
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected missing-using failure");
    match err {
        CompileError::Ir(messages) => {
            assert!(
                messages.iter().any(|diag| diag
                    .message
                    .contains("requires an explicit `using` directive")),
                "unexpected diagnostics: {messages:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}
