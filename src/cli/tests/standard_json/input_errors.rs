#[test]
fn standard_json_reports_missing_content_error() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "Missing.sol": { "urls": ["ipfs://example"] }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0]["type"],
        Value::String("MissingSourceContent".into())
    );
    assert_eq!(errors[0]["severity"], Value::String("error".into()));
    assert_eq!(
        errors[0]["code"],
        Value::String("MISSING_SOURCE_CONTENT".into())
    );
}

#[test]
fn standard_json_rejects_non_solidity_language() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");

    let input_json = json!({
        "language": "Vyper",
        "sources": {
            "C.sol": { "content": "contract C {}" }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    let result = process_standard_json(
        input_path.to_str().unwrap(),
        None,
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    );
    assert!(
        result
            .err()
            .map(|msg| msg.to_ascii_lowercase().contains("unsupported language"))
            .unwrap_or(false),
        "expected unsupported language error"
    );
}

#[test]
fn standard_json_reports_no_contracts() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "Empty.sol": { "content": "" }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["type"], Value::String("NoContracts".into()));
    assert_eq!(errors[0]["severity"], Value::String("error".into()));
    assert_eq!(errors[0]["code"], Value::String("NO_CONTRACTS".into()));
}

#[test]
fn standard_json_reports_missing_import_error() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import "./Missing.sol";

    contract A {
        function ok() public pure returns (uint256) {
            return 1;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    assert!(
        errors.iter().any(|e| {
            e.get("type").and_then(Value::as_str) == Some("MissingImport")
                && e["code"] == "MISSING_IMPORT"
        }),
        "expected MissingImport error"
    );
}

#[test]
fn standard_json_reports_duplicate_contract_name_import_closure() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source_a = r#"
    pragma solidity ^0.8.19;
    import "./B.sol";

    contract Duplicate {
        function fromA() public pure returns (uint256) {
            return 1;
        }
    }
    "#;

    let source_b = r#"
    pragma solidity ^0.8.19;

    contract Duplicate {
        function fromB() public pure returns (uint256) {
            return 2;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source_a },
            "B.sol": { "content": source_b }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed with diagnostics");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");
    let errors = output["errors"].as_array().expect("errors array");

    assert!(
        errors.iter().any(|error| {
            error["code"] == "STANDARD_JSON_OUTPUT_ERROR"
                && error["sourceLocation"]["file"] == "A.sol"
                && error["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("duplicate contract names"))
        }),
        "expected duplicate import-closure diagnostic: {output}"
    );
    assert!(
        output["contracts"]["A.sol"]["Duplicate"].is_null(),
        "A.sol Duplicate must not be emitted from an ambiguous import closure: {output}"
    );
}

#[test]
fn standard_json_accepts_import_aliasing_for_dependency_resolution() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import { Lib as MathLib } from "./Lib.sol";

    contract A {
        function ok() public pure returns (uint256) {
            return Lib.add(1, 2);
        }
    }
    "#;

    let lib = r#"
    pragma solidity ^0.8.19;

    library Lib {
        function add(uint256 a, uint256 b) internal pure returns (uint256) {
            return a + b;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source },
            "Lib.sol": { "content": lib }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    assert!(
        !errors
            .iter()
            .any(|e| e.get("type").and_then(Value::as_str) == Some("UnsupportedImportSyntax")),
        "alias import should not emit UnsupportedImportSyntax: {errors:?}"
    );
    assert!(
        output["contracts"]["A.sol"]["A"].is_object(),
        "expected compiled contract artifact for A.sol/A"
    );
}

#[test]
fn standard_json_ir_errors_preserve_code_and_suggestion() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract C {
        function f() public pure returns (address) {
            return super;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "C.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    let error = errors
        .iter()
        .find(|err| err["type"] == "IrGeneration")
        .expect("IR generation error");

    assert_eq!(error["code"], "NSH-3000");
    assert!(
        error["suggestions"][0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("use super.methodName()"),
        "expected IR suggestion to be preserved: {error:?}"
    );
}

#[test]
fn standard_json_manifest_errors_include_code() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract FullyDynamicCalls {
        function callAny(address target, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "C.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    let error = errors
        .iter()
        .find(|err| err["type"] == "ManifestGeneration")
        .expect("manifest generation error");

    assert_eq!(error["code"], "NSH-6001");
}

#[test]
fn standard_json_generic_errors_include_code() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract Broken {
        function f( public pure returns (uint256) {
            return 1;
       
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "Broken.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    // A syntax error now surfaces as one or more structured ParseError entries,
    // each carrying a precise sourceLocation (start/end), instead of a single
    // opaque Generic blob — matching solc's per-diagnostic standard-JSON output.
    let error = errors
        .iter()
        .find(|err| err["type"] == "ParseError")
        .expect("parse error");

    assert_eq!(error["code"], "NSH-1000");
    let loc = &error["sourceLocation"];
    assert!(
        loc.get("start").and_then(Value::as_u64).is_some()
            && loc.get("end").and_then(Value::as_u64).is_some(),
        "ParseError sourceLocation must carry byte start/end offsets, got {loc}"
    );
}

#[test]
fn standard_json_accepts_global_symbol_import_for_dependency_resolution() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import * as LibNS from "./Lib.sol";

    contract A {
        function ok() public pure returns (uint256) {
            return Lib.add(1, 2);
        }
    }
    "#;

    let lib = r#"
    pragma solidity ^0.8.19;

    library Lib {
        function add(uint256 a, uint256 b) internal pure returns (uint256) {
            return a + b;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source },
            "Lib.sol": { "content": lib }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    assert!(
        !errors
            .iter()
            .any(|e| e.get("type").and_then(Value::as_str) == Some("UnsupportedImportSyntax")),
        "global-symbol import should not emit UnsupportedImportSyntax: {errors:?}"
    );
    assert!(
        output["contracts"]["A.sol"]["A"].is_object(),
        "expected compiled contract artifact for A.sol/A"
    );
    // The alias-resolution limitation is surfaced LOUDLY (a warning), not left
    // as a silent bare-name bind: a global-symbol namespace must emit an
    // IMPORT_ALIAS_BY_NAME warning so collisions can be caught by the author.
    assert!(
        errors.iter().any(|e| {
            e.get("severity").and_then(Value::as_str) == Some("warning")
                && e.get("code").and_then(Value::as_str) == Some("IMPORT_ALIAS_BY_NAME")
        }),
        "global-symbol import must emit an IMPORT_ALIAS_BY_NAME warning: {errors:?}"
    );
}

#[test]
fn standard_json_resolves_aliased_symbol_binding() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import { Lib as MathLib } from "./Lib.sol";

    contract A {
        function ok() public pure returns (uint256) {
            return MathLib.add(1, 2);
        }
    }
    "#;

    let lib = r#"
    pragma solidity ^0.8.19;

    library Lib {
        function add(uint256 a, uint256 b) internal pure returns (uint256) {
            return a + b;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source },
            "Lib.sol": { "content": lib }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    assert!(
        output["contracts"]["A.sol"]["A"].is_object(),
        "expected compiled contract artifact for A.sol/A"
    );
}

#[test]
fn standard_json_resolves_wildcard_namespace_symbol_binding() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import * as LibNS from "./Lib.sol";

    contract A {
        function ok() public pure returns (uint256) {
            return LibNS.Lib.add(1, 2);
        }
    }
    "#;

    let lib = r#"
    pragma solidity ^0.8.19;

    library Lib {
        function add(uint256 a, uint256 b) internal pure returns (uint256) {
            return a + b;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source },
            "Lib.sol": { "content": lib }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    assert!(
        output["contracts"]["A.sol"]["A"].is_object(),
        "expected compiled contract artifact for A.sol/A"
    );
}

#[test]
fn standard_json_resolves_wildcard_namespace_type_cast_binding() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import * as LibNS from "./IFoo.sol";

    contract A {
        function ping(address target) public {
            LibNS.IFoo(target).foo();
        }
    }
    "#;

    let iface = r#"
    pragma solidity ^0.8.19;

    interface IFoo {
        function foo() external;
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source },
            "IFoo.sol": { "content": iface }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    assert!(
        output["contracts"]["A.sol"]["A"].is_object(),
        "expected compiled contract artifact for A.sol/A"
    );
}

#[test]
fn standard_json_resolves_wildcard_namespace_selector_binding() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    import * as LibNS from "./IFoo.sol";

    contract A {
        function selectorOfFoo() public pure returns (bytes4) {
            return LibNS.IFoo.foo.selector;
        }
    }
    "#;

    let iface = r#"
    pragma solidity ^0.8.19;

    interface IFoo {
        function foo() external;
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source },
            "IFoo.sol": { "content": iface }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    assert!(
        output["contracts"]["A.sol"]["A"].is_object(),
        "expected compiled contract artifact for A.sol/A"
    );
}
