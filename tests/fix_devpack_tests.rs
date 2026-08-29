//! Regression tests (devpack agent): the devpack `Storage`/`Syscalls` "Local"
//! storage API lowered to fictional `System.Storage.Local.*` syscalls that do
//! not exist in Neo N3's interop table (instant FAULT on real nodes), and
//! several documented `Storage` helpers (`batchPut`, `count`, `exists`,
//! `clearPrefix`, `getUsage`, ...) silently miscompiled to single raw
//! syscalls with the wrong arity and semantics. Those intrinsics were removed
//! so calls now fail compilation with a loud diagnostic; this suite pins both
//! the failures and the still-supported `Storage` surface.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn compiler_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_neo-solc"))
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn unique_temp_dir(scope: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seq = OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "neo-sol-fix-devpack-{scope}-{}-{nanos}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Compile an inline contract that imports the devpack from the repository
/// root (`import "devpack/..."` resolved via `-I <repo>`). Returns the
/// process output and the output directory; callers assert on
/// status/stderr/artifacts.
fn compile_inline(scope: &str, source: &str, extra_args: &[&str]) -> (Output, PathBuf) {
    let dir = unique_temp_dir(scope);
    let contract_path = dir.join("Probe.sol");
    std::fs::write(&contract_path, source).expect("write probe contract");

    let output = Command::new(compiler_path())
        .arg(&contract_path)
        .arg("-I")
        .arg(manifest_dir())
        .arg("-o")
        .arg(dir.join("Probe"))
        .args(extra_args)
        .output()
        .expect("failed to run compiler");
    (output, dir)
}

fn probe_contract(body: &str) -> String {
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "devpack/libraries/Storage.sol";
import "devpack/contracts/Syscalls.sol";

contract Probe {{
{body}
}}
"#
    )
}

fn assert_compile_fails_with(scope: &str, body: &str, expected_diagnostic: &str) {
    let (output, _dir) = compile_inline(scope, &probe_contract(body), &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "{scope}: expected compilation to fail, but it succeeded.\nstderr: {stderr}"
    );
    assert!(
        stderr.contains(expected_diagnostic),
        "{scope}: expected diagnostic containing {expected_diagnostic:?}, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Fictional System.Storage.Local.* syscalls (resolve.rs finding #1)
// ---------------------------------------------------------------------------

#[test]
fn storage_put_local_fails_compilation() {
    assert_compile_fails_with(
        "putlocal",
        r#"    function probe(bytes memory k, bytes memory v) public {
        Storage.putLocal(k, v);
    }"#,
        "unsupported builtin library call 'Storage.putLocal'",
    );
}

#[test]
fn storage_get_local_fails_compilation() {
    assert_compile_fails_with(
        "getlocal",
        r#"    function probe(bytes memory k) public view returns (bytes memory) {
        return Storage.getLocal(k);
    }"#,
        "unsupported builtin library call 'Storage.getLocal'",
    );
}

#[test]
fn storage_remove_local_fails_compilation() {
    assert_compile_fails_with(
        "removelocal",
        r#"    function probe(bytes memory k) public {
        Storage.removeLocal(k);
    }"#,
        "unsupported builtin library call 'Storage.removeLocal'",
    );
}

#[test]
fn syscalls_storage_put_local_fails_compilation() {
    assert_compile_fails_with(
        "sysputlocal",
        r#"    function probe(bytes memory k, bytes memory v) public {
        Syscalls.storagePutLocal(k, v);
    }"#,
        "unsupported builtin library call 'Syscalls.storagePutLocal'",
    );
}

// ---------------------------------------------------------------------------
// Miscompiling Storage helpers (resolve.rs finding #2)
// ---------------------------------------------------------------------------

#[test]
fn storage_batch_put_fails_compilation() {
    assert_compile_fails_with(
        "batchput",
        r#"    function probe(bytes[] memory ks, bytes[] memory vs) public {
        Storage.batchPut(ks, vs);
    }"#,
        "unsupported builtin library call 'Storage.batchPut'",
    );
}

#[test]
fn storage_count_fails_compilation() {
    assert_compile_fails_with(
        "count",
        r#"    function probe(bytes memory prefix) public view returns (uint256) {
        return Storage.count(prefix);
    }"#,
        "unsupported builtin library call 'Storage.count'",
    );
}

#[test]
fn storage_clear_prefix_fails_compilation() {
    assert_compile_fails_with(
        "clearprefix",
        r#"    function probe(bytes memory prefix) public {
        Storage.clearPrefix(prefix);
    }"#,
        "unsupported builtin library call 'Storage.clearPrefix'",
    );
}

#[test]
fn storage_exists_fails_compilation() {
    assert_compile_fails_with(
        "exists",
        r#"    function probe(bytes memory k) public view returns (bool) {
        return Storage.exists(k);
    }"#,
        "unsupported builtin library call 'Storage.exists'",
    );
}

#[test]
fn storage_get_usage_fails_compilation() {
    // Used to be a hardcoded PUSH0 stub that returned 0 for every contract.
    assert_compile_fails_with(
        "getusage",
        r#"    function probe() public view returns (uint256) {
        return Storage.getUsage();
    }"#,
        "unsupported builtin library call 'Storage.getUsage'",
    );
}

// ---------------------------------------------------------------------------
// The supported Storage surface still compiles, without fictional syscalls
// ---------------------------------------------------------------------------

#[test]
fn supported_storage_surface_compiles_without_local_syscalls() {
    let body = r#"    function write(bytes memory k, bytes memory v) public {
        Storage.put(k, v);
    }

    function read(bytes memory k) public view returns (bytes memory) {
        return Storage.get(k);
    }

    function erase(bytes memory k) public {
        Storage.remove(k);
    }"#;

    let (output, dir) = compile_inline("supported-asm", &probe_contract(body), &["-f", "assembly"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "supported Storage members failed to compile:\n{stderr}"
    );

    // Read the generated assembly artifact(s) and make sure no fictional
    // `System.Storage.Local.*` syscall survives anywhere.
    let mut asm_files = 0;
    for entry in std::fs::read_dir(&dir).expect("read output dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("asm") {
            continue;
        }
        asm_files += 1;
        let assembly = std::fs::read_to_string(&path).expect("read assembly artifact");
        assert!(
            !assembly.contains("System.Storage.Local"),
            "{} still references fictional System.Storage.Local syscalls:\n{assembly}",
            path.display()
        );
    }
    assert!(
        asm_files > 0,
        "no assembly artifact generated in {}",
        dir.display()
    );
}

#[test]
fn devpack_storage_library_compiles_standalone() {
    // The pruned devpack/libraries/Storage.sol must remain parseable and
    // compilable as an input file.
    let path = manifest_dir().join("devpack/libraries/Storage.sol");
    let dir = unique_temp_dir("storage-lib");
    let output = Command::new(compiler_path())
        .arg(&path)
        .arg("-I")
        .arg(manifest_dir().join("devpack"))
        .arg("-o")
        .arg(dir.join("StorageLib"))
        .output()
        .expect("failed to run compiler");
    assert!(
        output.status.success(),
        "devpack/libraries/Storage.sol no longer compiles:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
