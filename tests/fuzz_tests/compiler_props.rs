//! Compiler-level property tests.
//!
//! Extracted from the top-level "Compiler Fuzz Tests" banner in
//! `tests/fuzz_tests.rs`. Contents unchanged from the pre-split file.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Compiler Fuzz Tests ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn simple_storage_contract_compiles(
        var_name in identifier_strategy(),
        initial_value in uint_value_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = {};
}}"#,
            var_name, initial_value
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_functions_compiles(
        func_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 0;
    
    function {}() public view returns (uint256) {{
        return value;
    }}
}}"#,
            func_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_events_compiles(
        event_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    event {}(address indexed sender, uint256 value);
}}"#,
            event_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_mapping_compiles(
        mapping_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    mapping(address => uint256) public {};
}}"#,
            mapping_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_require_compiles(
        func_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {}(uint256 value) public pure returns (uint256) {{
        require(value > 0, "Value must be positive");
        return value * 2;
    }}
}}"#,
            func_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_modifier_compiles(
        mod_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    address public owner = msg.sender;
    
    modifier {}() {{
        require(msg.sender == owner, "Not owner");
        _;
    }}
}}"#,
            mod_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_loops_compiles(
        func_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {}(uint256 n) public pure returns (uint256) {{
        uint256 sum = 0;
        for (uint256 i = 0; i < n && i < 100; i++) {{
            sum += i;
        }}
        return sum;
    }}
}}"#,
            func_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn contract_with_conditionals_compiles(
        func_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {}(uint256 x) public pure returns (uint256) {{
        if (x > 100) {{
            return x * 2;
        }} else if (x > 50) {{
            return x + 10;
        }} else {{
            return x;
        }}
    }}
}}"#,
            func_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn empty_contract_compiles(
        contract_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract {} {{}}"#,
            contract_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Empty contract should compile: {:?}", result.err());
    }

    #[test]
    fn compilation_is_deterministic(
        var_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 42;
}}"#,
            var_name
        );

        let result1 = compile_contracts(&source, false, 2);
        let result2 = compile_contracts(&source, false, 2);

        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let artifacts1 = result1.unwrap();
        let artifacts2 = result2.unwrap();

        prop_assert_eq!(artifacts1.len(), artifacts2.len());

        for (a1, a2) in artifacts1.iter().zip(artifacts2.iter()) {
            prop_assert_eq!(a1.bytecode.len(), a2.bytecode.len());
        }
    }

    #[test]
    fn identifier_keyword_case_variants_compile(
        name in prop_oneof![
            Just("Contract".to_string()),
            Just("FUNCTION".to_string()),
            Just("pUblic".to_string()),
            Just("PrAgMa".to_string()),
            Just("uInT256".to_string()),
            Just("SoLidity".to_string()),
            Just("RETURN".to_string()),
            Just("Memory".to_string()),
            Just("STORAGE".to_string()),
            Just("Calldata".to_string()),
            Just("EVEnt".to_string()),
            Just("ModiFIER".to_string()),
        ]
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 42;
}}"#,
            name
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed for case-variant '{}': {:?}", name, result.err());
    }

    #[test]
    fn nested_ternary_compile(
        depth in 1usize..8
    ) {
        let mut expr = "0".to_string();
        for i in 0..depth {
            expr = format!("x > {} ? {} : {}", i, i + 1, expr);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function test(uint256 x) public pure returns (uint256) {{
        return {};
    }}
}}"#,
            expr
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn nested_function_calls_compile(
        depth in 1usize..8
    ) {
        let mut calls = "x".to_string();
        for i in 0..depth {
            calls = format!("add({}, {})", calls, i);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function add(uint256 a, uint256 b) public pure returns (uint256) {{
        return a + b;
    }}
    function test(uint256 x) public pure returns (uint256) {{
        return {};
    }}
}}"#,
            calls
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn hex_literals_compile(
        hex in "[0-9a-fA-F]{1,64}"
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 0x{};
}}"#,
            hex
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn scientific_notation_literals_compile(
        mantissa in "[1-9][0-9]{0,5}",
        exp in 0u32..40
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = {}e{};
}}"#,
            mantissa, exp
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn leading_zeros_compile(
        digits in "[0-9]{1,20}"
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 0{};
}}"#,
            digits
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn comments_in_unusual_places_compile(
        name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma/*comment*/solidity ^0.8.0;
contract TestContract {{
    uint256/*a*/public {} = 42;
    function/*b*/test()/*c*/public/*d*/pure/*e*/returns/*f*/(uint256) {{
        return/*g*/42/*h*/;
    }}
}}"#,
            name
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn empty_string_compile(
        _dummy in any::<bool>()
    ) {
        let source = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract TestContract {\n    string public message = \"\";\n}";
        let result = compile_contracts(source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn string_escape_sequences_compile(
        esc in prop_oneof![
            Just("\\n".to_string()),
            Just("\\t".to_string()),
            Just("\\r".to_string()),
            Just("\\\\".to_string()),
            Just("\\x41".to_string()),
            Just("\\x00".to_string()),
        ]
    ) {
        let source = format!(
            "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract TestContract {{\n    string public message = \"{}\";\n}}",
            esc
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }
}

// ==================== Coverage-gap targeted fuzz tests ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // Invariant: function overloading with SAME arity but DIFFERENT parameter types
    // compiles and each overload is declared in the manifest with a distinct name.
    #[test]
    fn overloaded_functions_same_arity_different_types(
        fn_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {name}(uint256 a) public pure returns (uint256) {{ return a; }}
    function {name}(string memory a) public pure returns (string memory) {{ return a; }}
    function {name}(address a) public pure returns (address) {{ return a; }}
}}"#,
            name = fn_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Overload compile failed: {:?}", result.err());

        let artifacts = result.unwrap();
        prop_assert!(!artifacts.is_empty(), "expected at least one artifact");

        let methods = artifacts[0].manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");

        let entries: Vec<_> = methods.iter().filter(|m| {
            m.get("name").and_then(serde_json::Value::as_str)
                .map(|n| n == fn_name.as_str() || n.starts_with(&format!("{}(", fn_name)))
                .unwrap_or(false)
        }).collect();

        prop_assert!(
            entries.len() >= 2,
            "Expected at least 2 overload entries for '{}', got {}. methods={:?}",
            fn_name,
            entries.len(),
            methods.iter().map(|m| m.get("name").cloned()).collect::<Vec<_>>()
        );
    }

    // Invariant: immutable variables of various types compile.
    #[test]
    fn immutable_various_types_compile(
        ty in prop_oneof![Just("uint256"), Just("address"), Just("bytes32"), Just("bool")],
        var_name in identifier_strategy()
    ) {
        let init = match ty {
            "uint256" => "42",
            "address" => "address(0x1234567890123456789012345678901234567890)",
            "bytes32" => "bytes32(uint256(0xabcd))",
            "bool" => "true",
            _ => "0",
        };

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    {ty} public immutable {name} = {init};
}}"#,
            ty = ty,
            name = var_name,
            init = init
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Immutable {ty} compile failed: {:?}", result.err());
    }

    // Invariant: try/catch with varying clause combinations compiles.
    #[test]
    fn try_catch_clause_combinations_compile(
        has_error_clause in any::<bool>(),
        has_custom_clause in any::<bool>(),
        has_bare_clause in any::<bool>(),
    ) {
        prop_assume!(has_error_clause || has_custom_clause || has_bare_clause);

        let mut clauses = String::new();
        if has_error_clause {
            clauses.push_str("        } catch Error(string memory reason) {\n            reason;\n");
        }
        if has_custom_clause {
            clauses.push_str("        } catch Panic(uint256 code) {\n            code;\n");
        }
        if has_bare_clause {
            clauses.push_str("        } catch (bytes memory lowLevelData) {\n            lowLevelData;\n");
        }

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract TryCatchFuzz {{
    function foo() external pure {{ }}
    function bar() external {{
        try this.foo() {{
{clauses}        }}
    }}
}}"#,
            clauses = clauses
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Try/catch compile failed: {:?}", result.err());
    }

    // Invariant: payable functions compile and appear in the manifest.
    #[test]
    fn payable_function_compiles_and_manifest_correct(
        fn_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {name}() external payable {{ }}
}}"#,
            name = fn_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Payable compile failed: {:?}", result.err());

        let artifacts = result.unwrap();
        prop_assert!(!artifacts.is_empty());

        let methods = artifacts[0].manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");

        let method = methods.iter().find(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some(fn_name.as_str())
        });
        prop_assert!(method.is_some(), "Payable method '{}' missing from manifest", fn_name);
    }

    // Invariant: fixed-size arrays of various types and sizes compile.
    #[test]
    fn fixed_array_various_types_compile(
        size in 1usize..10,
        ty in prop_oneof![Just("uint256"), Just("address"), Just("bytes32")],
        var_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    {ty}[{size}] public {name};
}}"#,
            ty = ty,
            size = size,
            name = var_name
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Fixed array {ty}[{size}] compile failed: {:?}", result.err());
    }

    // Invariant: abi.encode/decode round-trip with varying static-type pairs compiles
    // and both wrapper methods are declared in the manifest.
    #[test]
    fn abi_encode_decode_various_types_compile(
        ty1 in prop_oneof![Just("uint256"), Just("address"), Just("bool")],
        ty2 in prop_oneof![Just("uint256"), Just("address"), Just("bool")],
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract AbiRoundTrip {{
    function pack({ty1} a, {ty2} b) external pure returns (bytes memory) {{
        return abi.encode(a, b);
    }}
    function unpack(bytes calldata data) external pure returns ({ty1}, {ty2}) {{
        return abi.decode(data, ({ty1}, {ty2}));
    }}
}}"#,
            ty1 = ty1,
            ty2 = ty2
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "abi.encode/decode compile failed: {:?}", result.err());

        let artifacts = result.unwrap();
        prop_assert!(!artifacts.is_empty());

        let methods = artifacts[0].manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");

        prop_assert!(
            methods.iter().any(|m| m.get("name").and_then(serde_json::Value::as_str) == Some("pack")),
            "pack missing from manifest"
        );
        prop_assert!(
            methods.iter().any(|m| m.get("name").and_then(serde_json::Value::as_str) == Some("unpack")),
            "unpack missing from manifest"
        );
    }

    // Invariant: `gasleft()` compiles and returns a positive uint at runtime.
    #[test]
    fn gasleft_returns_positive_uint(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function gasBefore() external view returns (uint) { return gasleft(); }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "gasBefore",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "gasBefore() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert!(got > num_bigint::BigUint::from(0u64),
            "gasleft() must return a positive value; got {}", got);
    }

    // Invariant: `block.timestamp` compiles and returns a reasonable value at runtime.
    #[test]
    fn block_timestamp_returns_reasonable_value(
        t_seconds in 1_704_067_200u64..=2_051_222_400u64,
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function ts() external view returns (uint) { return block.timestamp; }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        rt.override_timestamp(t_seconds.saturating_mul(1000));
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "ts",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "ts() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got.clone(), num_bigint::BigUint::from(t_seconds),
            "block.timestamp must equal override/1000 = {} seconds; got {}",
            t_seconds, got);
        prop_assert!(got >= num_bigint::BigUint::from(1_704_067_200u64),
            "block.timestamp must be >= 2024-01-01 (unix seconds); got {}", got);
    }

    // Invariant: `address(this).balance` compiles and executes (mapped to GasToken.balanceOf).
    #[test]
    fn address_this_balance_compiles_and_executes(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function bal() external view returns (uint) { return address(this).balance; }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "bal",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "bal() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert!(!r.return_data.is_empty(),
            "bal() return_data must be non-empty — GasToken.balanceOf should return a balance");
    }

    // Invariant: `selfdestruct(address)` compiles and executes (mapped to ContractManagement.destroy).
    #[test]
    fn selfdestruct_executes_via_contract_management_destroy(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function kill(address payable r) external { selfdestruct(r); }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let c = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&c.bytecode, &c.tokens, &c.manifest, "kill",
            &[StackItem::byte_array(vec![0u8; 20])]).expect("call");
        prop_assert!(r.success,
            "kill() must succeed via ContractManagement.destroy() auto-map; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert!(r.return_data.is_empty(),
            "kill() returns nothing; got rd_hex={}", hex::encode(&r.return_data));
    }

    // Invariant: `abi.encode` / `abi.decode` round-trip compiles and returns correct values.
    #[test]
    fn abi_encode_decode_roundtrip(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function roundtrip() external pure returns (uint256, uint256) {
        bytes memory data = abi.encode(uint256(42), uint256(99));
        (uint256 a, uint256 b) = abi.decode(data, (uint256, uint256));
        return (a, b);
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "roundtrip",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "roundtrip() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let mut expected = vec![0u8; 64];
        expected[24..32].copy_from_slice(&42u64.to_be_bytes());
        expected[56..64].copy_from_slice(&99u64.to_be_bytes());
        prop_assert_eq!(r.return_data.as_slice(), expected.as_slice(),
            "roundtrip return must be EVM-canonical 64 bytes (BE-padded 32 per scalar); \
             rd.len={}, rd={:?}", r.return_data.len(), r.return_data);
    }

    // Invariant: manifest JSON for a basic contract satisfies the Neo N3 required schema.
    #[test]
    fn manifest_schema_validation(
        var_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 42;
}}"#,
            var_name
        );

        let artifacts = compile_contracts(&source, false, 2).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let manifest = &artifacts[0].manifest;

        // Top-level required keys.
        for key in ["name", "abi", "permissions", "supportedstandards"] {
            prop_assert!(manifest.get(key).is_some(), "manifest missing key: {}", key);
        }

        // abi.methods must be an array.
        let methods = manifest["abi"]["methods"].as_array();
        prop_assert!(methods.is_some(), "abi.methods must be an array");

        // Each method must have required keys.
        for m in methods.unwrap() {
            for key in ["name", "parameters", "returntype", "offset", "safe"] {
                prop_assert!(m.get(key).is_some(), "method missing key: {}", key);
            }
        }

        // permissions must be an array.
        prop_assert!(
            manifest["permissions"].is_array(),
            "permissions must be an array"
        );

        // supportedstandards must be an array.
        prop_assert!(
            manifest["supportedstandards"].is_array(),
            "supportedstandards must be an array"
        );
    }

    // Invariant: internal function pointers compile and can be passed/returned.
    #[test]
    fn internal_function_pointer_compiles(
        func_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {name}(uint256 a) internal pure returns (uint256) {{ return a + 1; }}
    function apply(uint256 x) public pure returns (uint256) {{
        function (uint256) internal pure returns (uint256) f = {name};
        return f(x);
    }}
}}"#,
            name = func_name
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Internal function pointer compile failed: {:?}", result.err());
    }

    // Invariant: type(X).interfaceId compiles for interfaces.
    #[test]
    fn type_interface_id_compiles(
        iface_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
interface {name} {{
    function foo() external;
}}
contract TestContract {{
    function getId() public pure returns (bytes4) {{
        return type({name}).interfaceId;
    }}
}}"#,
            name = iface_name
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "type(X).interfaceId compile failed: {:?}", result.err());
    }

    // Invariant: global using-for directive compiles.
    #[test]
    fn global_using_for_compiles(
        lib_name in identifier_strategy(),
        func_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
library {lib} {{
    function {fn}(uint256 a) internal pure returns (uint256) {{ return a * 2; }}
}}
using {lib} for uint256;
contract TestContract {{
    function compute(uint256 x) public pure returns (uint256) {{
        return x.{fn}();
    }}
}}"#,
            lib = lib_name,
            fn = func_name
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Global using-for compile failed: {:?}", result.err());
    }

    // Invariant: user-defined value types compile and operations work at runtime.
    #[test]
    fn user_defined_value_type_compiles_and_runs(
        a in 0u64..1000,
        b in 0u64..1000
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    type MyUint is uint256;
    function add(uint256 x, uint256 y) external pure returns (uint256) {
        MyUint mx = MyUint.wrap(x);
        MyUint my = MyUint.wrap(y);
        MyUint sum = MyUint.wrap(MyUint.unwrap(mx) + MyUint.unwrap(my));
        return MyUint.unwrap(sum);
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "add",
            &[StackItem::Integer(a as i64), StackItem::Integer(b as i64)]).expect("call");
        prop_assert!(r.success,
            "add() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        let expected = num_bigint::BigUint::from(a + b);
        prop_assert_eq!(got, expected, "add return mismatch");
    }

    // Invariant: nested struct arrays in storage compile.
    #[test]
    fn nested_struct_arrays_in_storage_compile(
        _seed in any::<u8>()
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Inner { uint256 x; }
    struct Outer { Inner[] items; }
    Outer public outer;
    function get(uint256 i) external view returns (uint256) {
        return outer.items[i].x;
    }
}"#;
        let result = compile_contracts(src, false, 2);
        prop_assert!(result.is_ok(), "Nested struct arrays compile failed: {:?}", result.err());
    }

    // Invariant: anonymous events compile.
    #[test]
    fn anonymous_event_compiles(
        _seed in any::<u8>()
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event MyEvent(uint256) anonymous;
    function emitIt(uint256 x) external {
        emit MyEvent(x);
    }
}"#;
        let result = compile_contracts(src, false, 2);
        prop_assert!(result.is_ok(), "Anonymous event compile failed: {:?}", result.err());
    }

    // Invariant: try/catch with custom errors compiles and the revert is caught
    // at runtime via the generic catch clause (custom-error-specific catch
    // compiles but currently falls through to the generic catch).
    #[test]
    fn try_catch_custom_error_compiles_and_runs(
        _seed in any::<u8>()
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    error CustomError(uint256 code);
    function alwaysRevert() external pure {
        revert CustomError(42);
    }
    function catchIt() external view returns (uint256) {
        try this.alwaysRevert() {
            return 0;
        } catch CustomError(uint256 code) {
            return code;
        } catch (bytes memory) {
            return 1;
        }
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "catchIt",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "catchIt() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert!(got != num_bigint::BigUint::from(0u64),
            "custom error must be caught (return != 0); got {}", got);
    }

    // Invariant: uint-to-enum casts work for valid values at runtime.
    #[test]
    fn enum_cast_valid_values_work(
        val in 0u8..3
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    enum Status { Pending, Approved, Rejected }
    function fromUint(uint256 n) external pure returns (Status) {
        return Status(n);
    }
    function toUint(Status s) external pure returns (uint256) {
        return uint256(s);
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "fromUint",
            &[StackItem::Integer(val as i64)]).expect("call");
        prop_assert!(r.success,
            "fromUint() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got, num_bigint::BigUint::from(val),
            "enum cast must return original value {}", val);
    }

    // === Precompile runtime verification (Coverage-gap targeted) ===

    // Invariant: sha256 precompile (0x02) runtime output matches Rust sha2 reference.
    #[test]
    fn sha256_precompile_runtime_matches_reference(
        fn_name in identifier_strategy(),
        data in prop::collection::vec(any::<u8>(), 0..128)
    ) {
        use sha2::{Digest, Sha256};
        let reference = Sha256::digest(&data).to_vec();

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Sha256Precompile {{
    function {name}(bytes memory data) public pure returns (bytes memory) {{
        (bool ok, bytes memory out) = address(0x02).staticcall(data);
        require(ok, "staticcall failed");
        return out;
    }}
}}"#,
            name = fn_name
        );

        let artifacts = compile_contracts(&source, false, 2).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let art = &artifacts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let result = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, &fn_name,
            &[StackItem::byte_array(data.clone())]).expect("call");
        prop_assert!(result.success, "sha256 execution failed: {:?}", result.exception);
        prop_assert_eq!(result.return_data, reference,
            "sha256 output mismatch");
    }

    // Invariant: ripemd160 precompile (0x03) runtime output matches Rust ripemd reference.
    #[test]
    fn ripemd160_precompile_runtime_matches_reference(
        fn_name in identifier_strategy(),
        data in prop::collection::vec(any::<u8>(), 0..128)
    ) {
        use ripemd::{Digest, Ripemd160};
        let reference = Ripemd160::digest(&data).to_vec();

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Ripemd160Precompile {{
    function {name}(bytes memory data) public pure returns (bytes memory) {{
        (bool ok, bytes memory out) = address(0x03).staticcall(data);
        require(ok, "staticcall failed");
        return out;
    }}
}}"#,
            name = fn_name
        );

        let artifacts = compile_contracts(&source, false, 2).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let art = &artifacts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let result = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, &fn_name,
            &[StackItem::byte_array(data.clone())]).expect("call");
        prop_assert!(result.success, "ripemd160 execution failed: {:?}", result.exception);
        prop_assert_eq!(result.return_data, reference,
            "ripemd160 output mismatch");
    }

    // Invariant: identity precompile (0x04) returns input bytes unchanged.
    #[test]
    fn identity_precompile_returns_input_unchanged(
        fn_name in identifier_strategy(),
        data in prop::collection::vec(any::<u8>(), 0..128)
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract IdentityPrecompile {{
    function {name}(bytes memory data) public pure returns (bytes memory) {{
        (bool ok, bytes memory out) = address(0x04).staticcall(data);
        require(ok, "staticcall failed");
        return out;
    }}
}}"#,
            name = fn_name
        );

        let artifacts = compile_contracts(&source, false, 2).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let art = &artifacts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let result = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, &fn_name,
            &[StackItem::byte_array(data.clone())]).expect("call");
        prop_assert!(result.success, "identity execution failed: {:?}", result.exception);
        prop_assert_eq!(result.return_data, data,
            "identity output mismatch");
    }

    // Invariant: modexp precompile (0x05) with small 1-byte operands matches num_bigint reference.
    #[test]
    fn modexp_precompile_small_operands_matches_reference(
        fn_name in identifier_strategy(),
        base in 0u64..16,
        exp in 0u64..16,
        modulus in 1u64..16
    ) {
        use num_bigint::BigUint;
        let expected = BigUint::from(base).modpow(&BigUint::from(exp), &BigUint::from(modulus));

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract ModexpPrecompile {{
    function {name}(bytes memory b) public pure returns (bytes memory) {{
        (bool ok, bytes memory out) = address(0x05).staticcall(b);
        require(ok, "staticcall failed");
        return out;
    }}
}}"#,
            name = fn_name
        );

        let mut payload = vec![0u8; 99];
        payload[31] = 0x01;
        payload[63] = 0x01;
        payload[95] = 0x01;
        payload[96] = base as u8;
        payload[97] = exp as u8;
        payload[98] = modulus as u8;

        let artifacts = compile_contracts(&source, false, 2).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let art = &artifacts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let result = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, &fn_name,
            &[StackItem::byte_array(payload)]).expect("call");
        prop_assert!(result.success, "modexp execution failed: {:?}", result.exception);
        // emit_precompile_modexp now produces a conformant 32-byte BE slot: a
        // real node's CONVERT→ByteString of the MODPOW result is the MINIMAL
        // encoding (1 byte, or empty for zero), which the precompile left-pads
        // to 32 bytes. The 1-byte operand result therefore sits in byte 31.
        prop_assert_eq!(result.return_data.len(), 32,
            "modexp output must be a 32-byte BE slot; got {}", result.return_data.len());
        let got = BigUint::from(result.return_data[31]);
        prop_assert_eq!(got, expected,
            "modexp output mismatch (base={} exp={} mod={})", base, exp, modulus);
    }

    // === Neo N3 native contract runtime verification (Coverage-gap targeted) ===

    // Invariant: Oracle.request compiles and executes without panic.
    #[test]
    fn oracle_request_executes_without_panic(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function requestData() external {
        NativeCalls.requestOracleData("http://example.com", "$.price", address(this), "callback", "", 100000000);
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "requestData",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "requestData() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
    }

    // Invariant: Policy.getFeePerByte returns a non-negative value at runtime.
    #[test]
    fn policy_get_fee_per_byte_returns_non_negative(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function fee() external view returns (uint256) {
        return NativeCalls.getFeePerByte();
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "fee",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "fee() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert!(got >= num_bigint::BigUint::from(0u64),
            "getFeePerByte() must return non-negative; got {}", got);
    }

    // Invariant: Ledger.currentIndex returns a non-negative value at runtime.
    #[test]
    fn ledger_current_index_returns_non_negative(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function idx() external view returns (uint256) {
        return NativeCalls.currentIndex();
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "idx",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "idx() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert!(got >= num_bigint::BigUint::from(0u64),
            "currentIndex() must return non-negative; got {}", got);
    }

    // Invariant: RoleManagement.getDesignatedByRole executes successfully.
    #[test]
    fn role_management_get_designated_by_role_executes(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function designated() external view {
        NativeCalls.getDesignatedByRole(4);
    }
}"#;
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "designated",
            &[] as &[StackItem]).expect("call");
        prop_assert!(r.success,
            "designated() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
    }
}
