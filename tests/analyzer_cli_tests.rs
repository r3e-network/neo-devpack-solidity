use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn compiler_path() -> &'static str {
    env!("CARGO_BIN_EXE_neo-solc")
}

fn write_temp_contract(name: &str, source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join(name);
    std::fs::write(&path, source).expect("write source");
    (dir, path)
}

fn write_temp_contract_in_dir(
    dir: &tempfile::TempDir,
    name: &str,
    source: &str,
) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, source).expect("write source");
    path
}

#[test]
fn analyze_mode_reports_upgrade_findings() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UpgradeProbe {
        function risky(address target, uint256 height) public view returns (bytes4) {
            address origin = tx.origin;
            origin;
            blockhash(height);
            selfdestruct(payable(target));
            return msg.sig;
        }
    }
    "#;

    let (_dir, path) = write_temp_contract("UpgradeProbe.sol", source);

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: Value = serde_json::from_str(&stdout).expect("analyze report JSON");
    let file_report = &report["files"][0];

    assert_eq!(
        file_report["compileSuccess"],
        Value::Bool(false),
        "expected compile failure to be reflected in analyze mode: {stdout}"
    );

    let findings = file_report["findings"].as_array().expect("findings array");
    assert!(
        findings
            .iter()
            .any(|finding| finding["category"] == "auto_compatible"),
        "expected at least one auto-compatible finding: {stdout}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding["category"] == "manual_migration"),
        "expected at least one manual migration finding: {stdout}"
    );
}

#[test]
fn json_errors_preserve_library_visibility_warnings() {
    let source = r#"
    pragma solidity ^0.8.19;

    library BadLibrary {
        function broken(uint256 value) external pure returns (uint256) {
            return value + 1;
        }
    }
    "#;

    let (_dir, path) = write_temp_contract("BadLibrary.sol", source);

    let output = Command::new(compiler_path())
        .arg("--json-errors")
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected external library helper compilation to succeed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning[W124]"),
        "expected warning code in stderr: {stderr}"
    );
    assert!(
        stderr.contains(
            "is declared `external`; on NeoVM, user-defined library functions are inlined"
        ),
        "unexpected message: {stderr}"
    );
    assert!(
        stderr.contains("prefer `internal` visibility for library helpers"),
        "expected warning suggestion to be preserved in stderr output: {stderr}"
    );
}

#[test]
fn analyze_mode_with_multiple_sources_and_verbose_emits_valid_json() {
    let dir = tempdir().expect("tempdir");
    let first = write_temp_contract_in_dir(
        &dir,
        "First.sol",
        r#"
        pragma solidity ^0.8.19;
        contract First {
            function ping() public pure returns (uint256) { return 1; }
        }
        "#,
    );
    let second = write_temp_contract_in_dir(
        &dir,
        "Second.sol",
        r#"
        pragma solidity ^0.8.19;
        contract Second {
            function pong() public pure returns (uint256) { return 2; }
        }
        "#,
    );

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg("-v")
        .arg(&first)
        .arg(&second)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: Value = serde_json::from_str(&stdout).expect("analyze report JSON");
    let files = report["files"].as_array().expect("files array");
    assert_eq!(
        files.len(),
        2,
        "expected one report per source file: {stdout}"
    );
}

#[test]
fn analyze_mode_reports_findings_from_imported_sources() {
    let dir = tempdir().expect("tempdir");
    let helper = write_temp_contract_in_dir(
        &dir,
        "Helper.sol",
        r#"
        pragma solidity ^0.8.19;

        library Helper {
            function who() internal view returns (address) {
                return tx.origin;
            }
        }
        "#,
    );
    let main = write_temp_contract_in_dir(
        &dir,
        "Main.sol",
        r#"
        pragma solidity ^0.8.19;
        import "./Helper.sol";

        contract Main {
            function caller() public view returns (address) {
                return Helper.who();
            }
        }
        "#,
    );
    assert!(helper.exists());

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg(&main)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: Value = serde_json::from_str(&stdout).expect("analyze report JSON");
    let findings = report["files"][0]["findings"]
        .as_array()
        .expect("findings array");

    assert!(
        findings
            .iter()
            .any(|finding| finding["code"] == "SCAN_TX_ORIGIN"),
        "expected imported source findings to be analyzed: {stdout}"
    );
}

#[test]
fn analyze_mode_writes_report_to_output_file() {
    let source = r#"
    pragma solidity ^0.8.19;
    contract OutputProbe {
        function risky() public view returns (address) {
            return tx.origin;
        }
    }
    "#;

    let (dir, path) = write_temp_contract("OutputProbe.sol", source);
    let report_path = dir.path().join("report.json");

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg("-o")
        .arg(&report_path)
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        report_path.exists(),
        "expected analyze report file to exist"
    );

    let report_text = std::fs::read_to_string(&report_path).expect("read report");
    let report: Value = serde_json::from_str(&report_text).expect("analyze report JSON");
    assert_eq!(report["files"][0]["contracts"][0], "OutputProbe");
}

#[test]
fn analyze_mode_errors_when_contract_filter_matches_nothing() {
    let source = r#"
    pragma solidity ^0.8.19;
    contract Alpha {
        function ping() public pure returns (uint256) { return 1; }
    }
    "#;

    let (_dir, path) = write_temp_contract("Alpha.sol", source);

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg("--contract")
        .arg("Missing")
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        !output.status.success(),
        "expected missing contract filter to fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no matching contract(s) found for --contract Missing"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn analyze_mode_respects_deny_wildcard_permissions_flag() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract FullyDynamicCalls {
        function callAny(address target, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;

    let (_dir, path) = write_temp_contract("FullyDynamicCalls.sol", source);

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg("--deny-wildcard-permissions")
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: Value = serde_json::from_str(&stdout).expect("analyze report JSON");
    let file_report = &report["files"][0];

    assert_eq!(
        file_report["compileSuccess"],
        Value::Bool(false),
        "expected deny-wildcard-permissions to affect analyze mode: {stdout}"
    );
    assert!(
        file_report["findings"]
            .as_array()
            .expect("findings array")
            .iter()
            .any(|finding| finding["message"]
                .as_str()
                .unwrap_or_default()
                .contains("full wildcard manifest permissions")),
        "expected manifest wildcard failure to appear in analyze findings: {stdout}"
    );
}

#[test]
fn analyze_mode_respects_manifest_permission_overrides() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract FullyDynamicCalls {
        function callAny(address target, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;

    let (dir, path) = write_temp_contract("FullyDynamicCalls.sol", source);
    let permissions_path = dir.path().join("permissions.json");
    std::fs::write(
        &permissions_path,
        r#"[{"contract":"0x0102030405060708090a0b0c0d0e0f1011121314","methods":["ping"]}]"#,
    )
    .expect("write permissions");

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg("--deny-wildcard-permissions")
        .arg("--manifest-permissions")
        .arg(&permissions_path)
        .arg("--manifest-permissions-mode")
        .arg("replace-wildcards")
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: Value = serde_json::from_str(&stdout).expect("analyze report JSON");
    let file_report = &report["files"][0];

    assert_eq!(
        file_report["compileSuccess"],
        Value::Bool(true),
        "expected manifest override to keep analyze compilation successful: {stdout}"
    );
}

#[test]
fn analyze_mode_reports_manifest_review_for_exported_overloads() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract OverloadedApi {
        function ping(uint256 value) public pure returns (uint256) {
            return value;
        }

        function ping(uint256 value, uint256 extra) public pure returns (uint256) {
            return value + extra;
        }
    }
    "#;

    let (_dir, path) = write_temp_contract("OverloadedApi.sol", source);

    let output = Command::new(compiler_path())
        .arg("--analyze")
        .arg(&path)
        .output()
        .expect("run compiler");

    assert!(
        output.status.success(),
        "expected analyze mode to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: Value = serde_json::from_str(&stdout).expect("analyze report JSON");
    let findings = report["files"][0]["findings"]
        .as_array()
        .expect("findings array");

    assert!(
        findings.iter().any(|finding| {
            finding["code"] == "SCAN_EXPORTED_OVERLOADS" && finding["category"] == "manifest_review"
        }),
        "expected overload manifest-review finding: {stdout}"
    );
}
