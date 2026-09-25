//! Regression tests for gap `runtime-local`: the fictional
//! `System.Storage.Local.*` syscalls were removed from the bundled runtime
//! (they never existed on real Neo N3). A script that invokes them must now
//! FAULT gracefully — surface a `RuntimeError`, never panic — exactly like
//! any other unknown syscall on a real node. The compiler-side intrinsics
//! (`Syscalls.storage*Local`, `Storage.*Local`) were removed alongside, so
//! contracts referencing them are rejected at compile time instead of
//! emitting bytecode that can only fault.

use std::process::Command;

use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

/// Neo N3 interop id: first 4 bytes of SHA-256 over the syscall name.
fn syscall_id(name: &str) -> [u8; 4] {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

fn push_data(script: &mut Vec<u8>, data: &[u8]) {
    assert!(data.len() <= u8::MAX as usize, "push_data length overflow");
    script.push(0x0C); // PUSHDATA1
    script.push(data.len() as u8);
    script.extend_from_slice(data);
}

/// Run a raw script and return the error it faulted with (if any).
fn run_to_fault(code: &[u8]) -> Option<String> {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(code, &[]).expect("init");
    loop {
        match ctx.step() {
            Ok(state) => {
                if state.halted {
                    return None;
                }
            }
            Err(err) => return Some(err.to_string()),
        }
    }
}

/// `System.Storage.Local.Get` must fault as an unknown syscall (real-node
/// behavior), not read from storage and not panic.
#[test]
fn storage_local_get_faults_as_unknown_syscall() {
    let mut code = vec![];
    push_data(&mut code, b"some_key");
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&syscall_id("System.Storage.Local.Get"));
    code.push(0x40); // RET

    let fault = run_to_fault(&code).expect("System.Storage.Local.Get must FAULT");
    assert!(
        fault.contains("Unsupported syscall"),
        "expected unknown-syscall fault, got: {fault}"
    );
}

/// All four removed syscalls fault identically, regardless of operands.
#[test]
fn all_storage_local_syscalls_fault_as_unknown() {
    for name in [
        "System.Storage.Local.Get",
        "System.Storage.Local.Put",
        "System.Storage.Local.Delete",
        "System.Storage.Local.Find",
    ] {
        let mut code = vec![];
        push_data(&mut code, b"value");
        push_data(&mut code, b"key");
        code.push(0x41); // SYSCALL
        code.extend_from_slice(&syscall_id(name));
        code.push(0x40); // RET

        let fault =
            run_to_fault(&code).unwrap_or_else(|| panic!("{name} must FAULT, but it halted"));
        assert!(
            fault.contains("Unsupported syscall"),
            "{name}: expected unknown-syscall fault, got: {fault}"
        );
    }
}

/// A fault must not poison the syscall table for real syscalls: the genuine
/// `System.Storage.Get` path still works after the Local.* removal.
#[test]
fn real_storage_get_still_works() {
    let mut code = vec![];
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&syscall_id("System.Storage.GetContext"));
    push_data(&mut code, b"missing_key");
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&syscall_id("System.Storage.Get"));
    code.push(0x40); // RET

    assert!(
        run_to_fault(&code).is_none(),
        "System.Storage.Get must still execute"
    );
}

/// The compiler must reject `Syscalls.storageGetLocal` / `Storage.getLocal`
/// at compile time now that the intrinsics are gone — it must not silently
/// emit a syscall that faults on every node.
#[test]
fn compiler_rejects_local_storage_intrinsics() {
    let compiler = std::path::PathBuf::from(env!("CARGO_BIN_EXE_neo-solc"));
    assert!(compiler.exists(), "Compiler not found");

    let cases = [
        (
            "syscalls_storage_get_local",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract LocalSyscallUser {
    function readLocal(bytes memory key) public view returns (bytes memory) {
        return Syscalls.storageGetLocal(key);
    }
}
"#,
        ),
        (
            "storage_get_local",
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract LocalStorageUser {
    function readLocal(bytes memory key) public view returns (bytes memory) {
        return Storage.getLocal(key);
    }
}
"#,
        ),
    ];

    for (label, source) in cases {
        let dir = std::env::temp_dir().join(format!(
            "neo-sol-gap-runtime-local-{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let src_path = dir.join("LocalUser.sol");
        std::fs::write(&src_path, source).expect("write source");

        let output = Command::new(&compiler)
            .arg(&src_path)
            .arg("-o")
            .arg(dir.join("LocalUser"))
            .output()
            .expect("run compiler");

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output.status.success(),
            "{label}: compilation must fail, but it succeeded. Output:\n{combined}"
        );
        assert!(
            !combined.contains("Storage.Local"),
            "{label}: diagnostics must not advertise the removed syscalls. Output:\n{combined}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
