#[test]
fn integer_type_constructors_truncate_like_solidity() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CastU8FromBig {
        function run() public pure returns (uint256) {
            return uint8(300);
        }
    }

    contract CastU8FromNeg {
        function run() public pure returns (uint256) {
            return uint8(uint256(-1));
        }
    }

    contract CastI8FromU8 {
        function run() public pure returns (int256) {
            return int8(255);
        }
    }

    contract CastI8Wraps {
        function run() public pure returns (int256) {
            return int8(-129);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");

    let u8_from_big = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "CastU8FromBig")
        .expect("expected CastU8FromBig artifact");
    let result = execute_bytecode(&u8_from_big.bytecode);
    assert!(result.is_success(), "expected CastU8FromBig to succeed");
    assert_eq!(result.return_data, 44i64.to_le_bytes().to_vec());

    let u8_from_neg = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "CastU8FromNeg")
        .expect("expected CastU8FromNeg artifact");
    let result = execute_bytecode(&u8_from_neg.bytecode);
    assert!(result.is_success(), "expected CastU8FromNeg to succeed");
    assert_eq!(result.return_data, 255i64.to_le_bytes().to_vec());

    let i8_from_u8 = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "CastI8FromU8")
        .expect("expected CastI8FromU8 artifact");
    let result = execute_bytecode(&i8_from_u8.bytecode);
    assert!(result.is_success(), "expected CastI8FromU8 to succeed");
    assert_eq!(result.return_data, (-1i64).to_le_bytes().to_vec());

    let i8_wraps = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "CastI8Wraps")
        .expect("expected CastI8Wraps artifact");
    let result = execute_bytecode(&i8_wraps.bytecode);
    assert!(result.is_success(), "expected CastI8Wraps to succeed");
    assert_eq!(result.return_data, 127i64.to_le_bytes().to_vec());
}

#[test]
fn explicit_tuple_return_emits_abi_encoded_bytes() {
    // Task #64 canonical shape check: `return (uint256(5), uint256(7));`
    // on an external function must produce 64 bytes of EVM-canonical
    // ABI encoding — two BE-padded 32-byte slots containing 5 and 7.
    let source = r#"
    pragma solidity ^0.8.19;
    contract C {
        function f() external pure returns (uint256, uint256) {
            return (uint256(5), uint256(7));
        }
    }
    "#;
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected f() to succeed: exc={:?}", result.exception);

    let mut expected = vec![0u8; 64];
    expected[24..32].copy_from_slice(&5u64.to_be_bytes());
    expected[56..64].copy_from_slice(&7u64.to_be_bytes());
    assert_eq!(result.return_data, expected,
        "expected 64-byte BE-packed ABI encoding for (5, 7); got {:?}",
        result.return_data);
}

#[test]
fn internal_multi_return_is_destructurable_by_caller() {
    // Task #64 semantics preservation check: Internal functions with
    // tuple returns must STILL produce a StackItem::Array so intra-contract
    // callers can destructure via `ArrayGet`. The abi-encode conversion
    // only applies to externally-callable (Public/External) functions,
    // where the main-frame RET would otherwise leak the Array as JSON.
    let source = r#"
    pragma solidity ^0.8.19;

    contract C {
        function foo() internal pure returns (uint256, uint256) {
            return (uint256(5), uint256(7));
        }
        function bar() external pure returns (uint256) {
            (uint256 x, uint256 y) = foo();
            return x + y;
        }
    }
    "#;
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected bar() to succeed: exc={:?}", result.exception);
    // bar() returns a single uint256 = 5 + 7 = 12, which hits the single-
    // return (non-tuple) path — LE 8-byte Integer encoding.
    assert_eq!(result.return_data, 12i64.to_le_bytes().to_vec(),
        "internal tuple destructure must round-trip to x+y=12");
}

#[test]
fn return_without_value_returns_named_return_variable() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ReturnNamedHarness {
        function value() public pure returns (uint256 x) {
            x = 7;
            return;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected execution to succeed");
    assert_eq!(result.return_data, 7i64.to_le_bytes().to_vec());
}

#[test]
fn implicit_multi_return_is_abi_encoded_bytes() {
    // Task #64: implicit end-of-function multi-return no longer packs the
    // return slots into a StackItem::Array that leaks as serde_json at
    // main-frame RET. It now routes through the same `abiEncode` handler
    // as `abi.encode(...)`, producing 2 * 32 = 64 BE-packed bytes.
    let source = r#"
    pragma solidity ^0.8.19;

    contract MultiReturnHarness {
        function values() public pure returns (uint256 a, uint256 b) {
            a = 1;
            b = 2;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected execution to succeed");

    let mut expected = vec![0u8; 64];
    expected[24..32].copy_from_slice(&1u64.to_be_bytes());
    expected[56..64].copy_from_slice(&2u64.to_be_bytes());
    assert_eq!(result.return_data, expected,
        "expected 2 * 32-byte BE-packed slots post-Task-#64");
}

#[test]
fn explicit_return_without_value_packs_multi_returns_as_abi_encoded_bytes() {
    // Task #64: same behavior as the implicit case — explicit `return;`
    // on a multi-return function now hands the declared return locals to
    // `abiEncode` instead of building a StackItem::Array.
    let source = r#"
    pragma solidity ^0.8.19;

    contract MultiReturnHarness {
        function values() public pure returns (uint256 a, uint256 b) {
            a = 1;
            b = 2;
            return;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected execution to succeed");

    let mut expected = vec![0u8; 64];
    expected[24..32].copy_from_slice(&1u64.to_be_bytes());
    expected[56..64].copy_from_slice(&2u64.to_be_bytes());
    assert_eq!(result.return_data, expected,
        "expected 2 * 32-byte BE-packed slots post-Task-#64");
}

#[test]
fn named_return_array_defaults_to_empty_array() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract DefaultArrayReturnHarness {
        function values() public pure returns (uint256[] memory out) {
            // No assignment; rely on Solidity default initialization.
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected execution to succeed");

    // Externally-callable array returns are abi-encoded into canonical
    // bytes (`offset=0x20 || length=0`) — the implicit named-return path
    // now matches the explicit `return out;` shape instead of leaking the
    // raw StackItem::Array as serde_json.
    let mut expected = vec![0u8; 64];
    expected[31] = 0x20;
    assert_eq!(
        result.return_data, expected,
        "expected abi-encoded empty dynamic array (offset || length=0)"
    );
}

#[test]
fn named_return_struct_defaults_fields_like_solidity() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract DefaultStructReturnHarness {
        struct S {
            uint256 a;
            bool b;
        }

        function value() public pure returns (S memory out) {
            // No assignment; rely on Solidity default initialization.
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected execution to succeed");

    let json: Value =
        serde_json::from_slice(&result.return_data).expect("expected JSON-encoded StackItem");
    assert_eq!(json["type"], "Array");
    assert_eq!(
        json["value"].as_array().map(|v| v.len()),
        Some(2),
        "expected struct to lower to a 2-element Array"
    );
    assert_eq!(json["value"][0]["type"], "Integer");
    assert_eq!(json["value"][0]["value"], 0);
    assert_eq!(json["value"][1]["type"], "Boolean");
    assert_eq!(json["value"][1]["value"], false);
}

#[test]
fn enum_dynamic_array_allocation_is_supported() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract EnumArrayHarness {
        enum Stage {
            Pending,
            Active,
            Closed
        }

        function values() public pure returns (Stage[] memory out) {
            out = new Stage[](2);
            out[0] = Stage.Pending;
            out[1] = Stage.Closed;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "EnumArrayHarness")
        .expect("expected EnumArrayHarness artifact");

    let result = execute_bytecode(&artifact.bytecode);
    assert!(result.is_success(), "expected execution to succeed");

    // Externally-callable array returns are abi-encoded into canonical
    // bytes: `offset=0x20 || length=2 || 0 || 2` (the implicit named-return
    // path now matches the explicit `return out;` shape).
    let mut expected = vec![0u8; 128];
    expected[31] = 0x20;
    expected[63] = 2;
    expected[95] = 0; // Stage.Pending
    expected[127] = 2; // Stage.Closed
    assert_eq!(
        result.return_data, expected,
        "expected abi-encoded Stage[] (offset || length || elements)"
    );
}
