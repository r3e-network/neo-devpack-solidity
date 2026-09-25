use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn compiler_path() -> &'static str {
    env!("CARGO_BIN_EXE_neo-solc")
}

#[test]
fn standard_json_cli_writes_json_error_for_unsupported_language() {
    let dir = tempdir().expect("tempdir");
    let input_path = dir.path().join("input.json");
    let output_path = dir.path().join("out.json");

    let input = serde_json::json!({
        "language": "Vyper",
        "sources": {
            "C.sol": { "content": "contract C {}" }
        },
        "settings": {}
    });
    std::fs::write(
        &input_path,
        serde_json::to_string_pretty(&input).expect("serialize input"),
    )
    .expect("write input");

    let output = Command::new(compiler_path())
        .arg("--standard-json")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("run compiler");

    assert!(
        !output.status.success(),
        "expected unsupported language to fail"
    );
    assert!(
        output_path.exists(),
        "expected CLI to write standard-json error output"
    );

    let json: Value = serde_json::from_str(
        &std::fs::read_to_string(&output_path).expect("read standard-json output"),
    )
    .expect("parse standard-json output");

    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.iter().any(|err| {
            err["code"] == "STANDARD_JSON_INPUT_ERROR"
                && err["message"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("Unsupported language")
        }),
        "expected standard-json input error payload: {json}"
    );
}

#[test]
fn standard_json_cli_writes_json_error_for_invalid_json_input() {
    let dir = tempdir().expect("tempdir");
    let input_path = dir.path().join("input.json");
    let output_path = dir.path().join("out.json");

    std::fs::write(&input_path, "{ this is not valid json").expect("write invalid input");

    let output = Command::new(compiler_path())
        .arg("--standard-json")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("run compiler");

    assert!(!output.status.success(), "expected invalid json to fail");
    assert!(
        output_path.exists(),
        "expected CLI to write standard-json parse error output"
    );

    let json: Value = serde_json::from_str(
        &std::fs::read_to_string(&output_path).expect("read standard-json output"),
    )
    .expect("parse standard-json output");

    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors
            .iter()
            .any(|err| err["code"] == "STANDARD_JSON_PARSE_ERROR"),
        "expected standard-json parse error payload: {json}"
    );
}
