#[test]
fn runtime_notify_rejects_argument_count_mismatch() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UsesRuntimeNotify {
        event Ping(uint256 a);

        function ping() public {
            Runtime.notify("Ping", abi.encode(uint256(1), uint256(2)));
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected compilation failure");
    match err {
        CompileError::Ir(messages) => {
            assert!(
                messages.iter().any(|m| m.message.contains("expects 1 argument(s)")),
                "unexpected error messages: {messages:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn runtime_notify_rejects_argument_type_mismatch() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UsesRuntimeNotify {
        event Ping(uint256 a, uint256 b);

        function ping() public {
            Runtime.notify("Ping", abi.encode("hello", "world"));
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected compilation failure");
    match err {
        CompileError::Ir(messages) => {
            assert!(
                messages.iter().any(|m| m.message.contains("incompatible type")),
                "unexpected error messages: {messages:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn overloaded_events_are_rejected() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract OverloadedEvents {
        event Ping(uint256 a);
        event Ping(string b);

        function ping() public {
            emit Ping(uint256(1));
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected compilation failure");
    match err {
        CompileError::Diagnostics(diags) => {
            assert!(
                diags
                    .iter()
                    .any(|d| d.message.to_ascii_lowercase().contains("overloaded")),
                "unexpected diagnostics: {diags:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}
