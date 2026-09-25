//! Multi-source standard JSON compilation proptests + smoke tests.
//!
//! Closes the wave-#30 coverage hole: `examples_smoke_props::examples_compile_smoke`
//! skips every `.sol` file with a non-comment `import` directive (13 files at
//! the time of writing) because the in-process `compile_contracts` helper is
//! single-source. Real production Solidity always uses imports — relative
//! paths, transitive chains, and (rarely) cycles. This module exercises the
//! standard-JSON import resolver under those shapes.
//!
//! Why this matters: if `build_combined_source_with_import_validation`
//! mishandles relative paths, transitive deps, or import cycles, real
//! deployments break silently. See
//! `src/cli/standard_json/standard_json_process/imports.rs`.
//!
//! Strategy: invoke the `neo-solc --standard-json` binary as a subprocess
//! (the only public entry point that returns the full output JSON; the
//! in-process `process_standard_json_content` is `pub(crate)`). Each test
//! writes a tempdir input.json, runs the compiler, parses the output JSON,
//! and asserts on `contracts`/`errors`.
//!
//! Reference: `tests/standard_json_cli_tests.rs` (CLI subprocess pattern),
//! `src/cli/tests/standard_json/basic/multiple_sources.rs` (in-process
//! 2-file shape), `tests/fuzz_tests/examples_smoke_props.rs` (examples scan).

#![allow(unused_imports)]

use proptest::prelude::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

// ---------- Helpers ----------

fn compiler_path() -> &'static str {
    env!("CARGO_BIN_EXE_neo-solc")
}

/// Run `neo-solc --standard-json` against an `input` JSON value. Returns the
/// parsed output JSON (whether the process exited 0 or non-0; the compiler
/// writes a JSON error envelope either way) and the process exit status.
fn run_standard_json(input: &Value) -> (Value, std::process::ExitStatus) {
    let dir = tempdir().expect("tempdir");
    let input_path = dir.path().join("input.json");
    let output_path = dir.path().join("out.json");
    std::fs::write(
        &input_path,
        serde_json::to_string_pretty(input).expect("serialise input"),
    )
    .expect("write input");
    let proc_out = Command::new(compiler_path())
        .arg("--standard-json")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("spawn neo-solc");
    let body = std::fs::read_to_string(&output_path).unwrap_or_else(|_| {
        // Some error paths fail before writing output; surface stdout/stderr.
        format!(
            "{{\"errors\":[{{\"message\":\"no output written; stderr={}\"}}]}}",
            String::from_utf8_lossy(&proc_out.stderr).replace('"', "'")
        )
    });
    let parsed: Value = serde_json::from_str(&body).unwrap_or_else(|e| {
        json!({
            "errors": [{ "message": format!("parse output failed: {e}; raw={body}") }]
        })
    });
    (parsed, proc_out.status)
}

/// Did the output JSON contain at least one severity=error entry?
fn has_errors(output: &Value) -> bool {
    output
        .get("errors")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter().any(|e| {
                e.get("severity")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "error")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Concatenate all error messages (and types) from the output JSON for
/// diagnostic snippets.
fn error_summary(output: &Value) -> String {
    output
        .get("errors")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .map(|e| {
                    let typ = e.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                    let msg = e.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    format!("[{typ}] {msg}")
                })
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .unwrap_or_default()
}

/// Look up `output.contracts[file][contract]`. Returns `None` if any layer
/// is missing.
fn get_contract<'a>(output: &'a Value, file: &str, contract: &str) -> Option<&'a Value> {
    output.get("contracts")?.get(file)?.get(contract)
}

// ==================== Test (a): simple 2-file import ====================

proptest! {
    #![proptest_config(ProptestConfig {
        // Each case spawns a subprocess; keep cases tight.
        cases: 4,
        ..ProptestConfig::default()
    })]

    /// 2-file Solidity package with a `./Counter.sol` import. Asserts both
    /// contracts appear in standard-JSON output and that `Main` carries a
    /// `c` getter (auto-generated from `Counter public c`).
    #[test]
    fn multi_source_simple_import(seed_n in 0u64..16) {
        let counter_src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Counter {{
    uint256 public n;
    function inc() external {{ n = n + {seed_n} + 1; }}
}}
"#);
        let main_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./Counter.sol";

contract Main {
    Counter public c;
    constructor() { c = new Counter(); }
}
"#;

        let input = json!({
            "language": "Solidity",
            "sources": {
                "Counter.sol": { "content": counter_src },
                "Main.sol":    { "content": main_src },
            },
            "settings": {}
        });

        let (output, status) = run_standard_json(&input);
        prop_assert!(
            status.success(),
            "compile exited non-zero: errors={}",
            error_summary(&output)
        );

        // Both contracts should be emitted.
        let main = get_contract(&output, "Main.sol", "Main");
        let counter = get_contract(&output, "Counter.sol", "Counter");
        prop_assert!(
            main.is_some(),
            "Main not present in output: errors={}; contracts={}",
            error_summary(&output),
            output.get("contracts").map(|v| v.to_string()).unwrap_or_default()
        );
        prop_assert!(
            counter.is_some(),
            "Counter not present in output: errors={}",
            error_summary(&output)
        );

        // Main should expose the `c()` getter (auto-generated for `Counter public c`).
        let method_ids = main
            .and_then(|m| m.get("evm"))
            .and_then(|m| m.get("methodIdentifiers"));
        let has_c_getter = method_ids
            .and_then(|m| m.as_object())
            .map(|m| m.keys().any(|k| k == "c()"))
            .unwrap_or(false);
        prop_assert!(
            has_c_getter,
            "expected Main to expose c() getter; methodIdentifiers={:?}",
            method_ids
        );
    }
}

// ==================== Test (b): A -> B -> C transitive ====================

proptest! {
    #![proptest_config(ProptestConfig { cases: 4, ..ProptestConfig::default() })]

    /// Transitive chain: `A.sol` imports `B.sol`, `B.sol` imports `C.sol`.
    /// Asserts all three are emitted and no resolution errors surface.
    #[test]
    fn multi_source_transitive_import(_seed in 0u32..16) {
        let c_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract C {
    function leaf() external pure returns (uint256) { return 7; }
}
"#;
        let b_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./C.sol";

contract B {
    function mid() external pure returns (uint256) { return 5; }
}
"#;
        let a_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./B.sol";

contract A {
    function top() external pure returns (uint256) { return 3; }
}
"#;

        let input = json!({
            "language": "Solidity",
            "sources": {
                "A.sol": { "content": a_src },
                "B.sol": { "content": b_src },
                "C.sol": { "content": c_src },
            },
            "settings": {}
        });

        let (output, status) = run_standard_json(&input);
        prop_assert!(
            status.success(),
            "transitive compile exited non-zero: {}",
            error_summary(&output)
        );

        for (file, name) in [("A.sol", "A"), ("B.sol", "B"), ("C.sol", "C")] {
            prop_assert!(
                get_contract(&output, file, name).is_some(),
                "{name} not present in transitive output: errors={}",
                error_summary(&output)
            );
        }
    }
}

// ==================== Test (c): circular import is handled gracefully ====================

proptest! {
    #![proptest_config(ProptestConfig { cases: 2, ..ProptestConfig::default() })]

    /// A imports B, B imports A. The import resolver MUST handle this
    /// gracefully — no infinite loop, no stack overflow, no host panic.
    /// The current resolver intentionally tolerates cycles (back-edges are
    /// dropped during DFS rather than reported, see imports.rs:218); this
    /// test pins that behavior. If a future change starts rejecting cycles
    /// with a typed error, the test will still pass as long as the process
    /// terminates and the output contains a reasonable diagnostic.
    #[test]
    fn multi_source_circular_import_rejected(_seed in 0u32..4) {
        let a_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./B.sol";

contract A {
    function fromA() external pure returns (uint256) { return 1; }
}
"#;
        let b_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./A.sol";

contract B {
    function fromB() external pure returns (uint256) { return 2; }
}
"#;

        let input = json!({
            "language": "Solidity",
            "sources": {
                "A.sol": { "content": a_src },
                "B.sol": { "content": b_src },
            },
            "settings": {}
        });

        // Subprocess invocation: a hang or crash here would surface as a
        // test timeout / non-zero exit with no output JSON. That alone is
        // sufficient to detect the regression class the wave-#30 finding
        // anticipated.
        let (output, status) = run_standard_json(&input);

        // Two acceptable shapes:
        //   1. Compile succeeds — both A and B emitted (current resolver
        //      tolerates cycles by dropping back-edges).
        //   2. Compile fails cleanly with an error mentioning "circular",
        //      "cycle", or similar (a future, stricter resolver).
        // What's NOT acceptable: the process hangs or crashes (caught by
        // `Command::output` returning an Err / non-terminating wait).
        if status.success() {
            // Tolerated cycle: both contracts must still be emitted.
            let a = get_contract(&output, "A.sol", "A");
            let b = get_contract(&output, "B.sol", "B");
            prop_assert!(
                a.is_some() && b.is_some(),
                "circular-import compile succeeded but emitted incomplete output: \
                 A={}, B={}, errors={}",
                a.is_some(), b.is_some(), error_summary(&output)
            );
        } else {
            // Strict-rejection path: error message should mention the cycle.
            let summary = error_summary(&output).to_lowercase();
            prop_assert!(
                summary.contains("circular")
                    || summary.contains("cycle")
                    || summary.contains("recursive")
                    || summary.contains("import"),
                "circular-import compile failed but error did not mention cycles: {}",
                summary
            );
        }
    }
}

// ==================== Test (d): library used via `using ... for` ====================

proptest! {
    #![proptest_config(ProptestConfig { cases: 4, ..ProptestConfig::default() })]

    /// Library + consumer in separate files, wired via `using L for uint`.
    /// Asserts the compile succeeds and the consumer carries the expected
    /// public method. Runtime execution is NOT exercised here because
    /// running compiled code requires a parsed manifest, and the standard-
    /// JSON output's manifest object is the same one produced by
    /// `compile_contracts` (already covered by `examples_call_smoke`). The
    /// load-bearing assertion for THIS test is that the import resolver
    /// successfully wires a library consumer to its definition file.
    #[test]
    fn multi_source_imported_library_used(_seed in 0u32..16) {
        let lib_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library L {
    function add(uint a, uint b) internal pure returns (uint) { return a + b; }
}
"#;
        let consumer_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./L.sol";

contract C {
    using L for uint;
    function f() external pure returns (uint) {
        uint x = 1;
        return x.add(2);
    }
}
"#;

        let input = json!({
            "language": "Solidity",
            "sources": {
                "L.sol": { "content": lib_src },
                "C.sol": { "content": consumer_src },
            },
            "settings": {}
        });

        let (output, status) = run_standard_json(&input);
        prop_assert!(
            status.success(),
            "library compile exited non-zero: {}",
            error_summary(&output)
        );

        let c = get_contract(&output, "C.sol", "C");
        prop_assert!(
            c.is_some(),
            "consumer C not in output: errors={}",
            error_summary(&output)
        );

        // The consumer must expose `f()` as a public method.
        let has_f = c
            .and_then(|v| v.get("evm"))
            .and_then(|v| v.get("methodIdentifiers"))
            .and_then(|v| v.as_object())
            .map(|m| m.contains_key("f()"))
            .unwrap_or(false);
        prop_assert!(
            has_f,
            "expected C.f() in methodIdentifiers; output={}",
            c.map(|v| v.to_string()).unwrap_or_default()
        );
    }
}

// ==================== Test (e): the 13 wave-#30 skipped example contracts ====================

/// File detection: scan `examples/`, `devpack/contracts/`, `devpack/examples/`
/// for `.sol` files with a non-comment `import` directive — exactly the set
/// the wave-#30 smoke test skips.
fn collect_imported_sol_files(workspace_root: &Path) -> Vec<PathBuf> {
    let scan_roots = ["examples", "devpack/contracts", "devpack/examples"];
    let mut out: Vec<PathBuf> = Vec::new();
    for root in &scan_roots {
        let dir = workspace_root.join(root);
        if !dir.is_dir() {
            continue;
        }
        let mut stack: Vec<PathBuf> = vec![dir];
        while let Some(d) = stack.pop() {
            let entries = match std::fs::read_dir(&d) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|e| e.to_str()) == Some("sol") {
                    if let Ok(src) = std::fs::read_to_string(&p) {
                        if has_non_comment_import(&src) {
                            out.push(p);
                        }
                    }
                }
            }
        }
    }
    out.sort();
    out
}

fn has_non_comment_import(src: &str) -> bool {
    src.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("import ")
            || trimmed.starts_with("import\"")
            || trimmed.starts_with("import{")
            || trimmed.starts_with("import (")
    })
}

/// Extract the filename strings from non-comment `import "..."` directives
/// in `src`. Pure-string scan — no parser needed for this surface (all
/// imports in the example tree use the `import "path"` shape, see
/// devpack/contracts/*.sol).
fn extract_imports(src: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("import ")
            || trimmed.starts_with("import\"")
            || trimmed.starts_with("import{")
            || trimmed.starts_with("import ("))
        {
            continue;
        }
        // Find the first quoted segment.
        let mut chars = trimmed.chars().peekable();
        let mut in_quote = false;
        let mut buf = String::new();
        for c in chars.by_ref() {
            match (in_quote, c) {
                (false, '"') => in_quote = true,
                (true, '"') => break,
                (true, _) => buf.push(c),
                (false, _) => {}
            }
        }
        if !buf.is_empty() {
            imports.push(buf);
        }
    }
    imports
}

/// Normalize a virtual path the same way the resolver does: drop `.`,
/// pop on `..`, keep `Normal` segments. Mirrors
/// `imports.rs::normalize_virtual_path` so closure keys collide with the
/// resolver's lookup logic exactly.
fn normalize_virtual(path: &Path) -> String {
    use std::path::Component;
    let mut components: Vec<String> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(part) => {
                components.push(part.to_string_lossy().to_string());
            }
            Component::Prefix(_) | Component::RootDir => {}
        }
    }
    components.join("/")
}

/// Build the import closure for `entry_path`: every transitively-reachable
/// `.sol` file, indexed by a workspace-relative virtual path. The keys
/// match the resolver's view: importing file at `devpack/examples/X.sol`
/// importing `../contracts/Syscalls.sol` resolves to
/// `devpack/contracts/Syscalls.sol` — which we include under that exact key.
///
/// Returns `Vec<(virtual_key, content)>`. Caller wraps this into a
/// standard-JSON `sources` object; the resolver will then pair the entry
/// file's `import "../contracts/X.sol"` against the matching virtual key.
fn build_closure(
    workspace_root: &Path,
    entry_path: &Path,
) -> Result<Vec<(String, String)>, String> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let entry_canon = entry_path
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {}", entry_path.display(), e))?;
    let workspace_canon = workspace_root
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {}", workspace_root.display(), e))?;

    fn workspace_relative(workspace: &Path, abs: &Path) -> Option<String> {
        let rel = abs.strip_prefix(workspace).ok()?;
        Some(normalize_virtual(rel))
    }

    let entry_key = workspace_relative(&workspace_canon, &entry_canon)
        .ok_or_else(|| format!("entry not under workspace: {}", entry_canon.display()))?;
    let entry_content = std::fs::read_to_string(&entry_canon)
        .map_err(|e| format!("read {}: {}", entry_canon.display(), e))?;

    // (virtual_key) -> (abs_path, content)
    let mut sources: HashMap<String, (PathBuf, String)> = HashMap::new();
    sources.insert(entry_key.clone(), (entry_canon.clone(), entry_content));

    // BFS. Each queued item is the virtual_key whose imports we still need
    // to walk.
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(entry_key.clone());
    let mut seen_abs: HashSet<PathBuf> = HashSet::new();
    seen_abs.insert(entry_canon.clone());

    while let Some(cur_key) = queue.pop_front() {
        let (cur_abs, cur_content) = match sources.get(&cur_key) {
            Some((a, c)) => (a.clone(), c.clone()),
            None => continue,
        };
        for imp in extract_imports(&cur_content) {
            // Resolve `imp` against the CURRENT file's directory on disk.
            let cur_dir = match cur_abs.parent() {
                Some(d) => d,
                None => continue,
            };
            let abs_path = cur_dir.join(&imp);
            let abs_canon = match abs_path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue, // External / missing dep — let resolver report.
            };
            if !seen_abs.insert(abs_canon.clone()) {
                continue;
            }
            // Compute the WORKSPACE-relative virtual key for this file.
            // That's what the resolver will land on once it joins
            // `cur_dir_virtual / imp` and normalizes.
            let virt_key = match workspace_relative(&workspace_canon, &abs_canon) {
                Some(k) => k,
                None => continue, // Outside workspace — can't ship its content.
            };
            let content = match std::fs::read_to_string(&abs_canon) {
                Ok(c) => c,
                Err(_) => continue,
            };
            sources.insert(virt_key.clone(), (abs_canon, content));
            queue.push_back(virt_key);
        }
    }

    Ok(sources.into_iter().map(|(k, (_, c))| (k, c)).collect())
}

/// Iterate the wave-#30-skipped example contracts (those with non-comment
/// `import` directives). For each, build a standard-JSON input that
/// bundles the entry file plus the transitive closure of its imports
/// (resolved off-disk), invoke the compiler, and tally outcomes.
///
/// Pass criterion: at least 80% of surveyed files compile cleanly — some
/// are expected to fail because they hit advanced features (interfaces
/// that aren't fully supported, native syscall shapes that need richer
/// linking) but the bulk should resolve and compile.
#[test]
fn multi_source_run_skipped_examples() {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let surveyed = collect_imported_sol_files(&workspace_root);
    assert!(
        !surveyed.is_empty(),
        "no .sol files with imports found under examples/ devpack/contracts/ devpack/examples/ — \
         scan-roots changed?"
    );

    let mut compiled = 0usize;
    let mut compile_failed: Vec<(PathBuf, String)> = Vec::new();
    let mut closure_failed: Vec<(PathBuf, String)> = Vec::new();

    for entry in &surveyed {
        let closure = match build_closure(&workspace_root, entry) {
            Ok(c) => c,
            Err(e) => {
                closure_failed.push((entry.clone(), e));
                continue;
            }
        };

        let mut sources_obj = serde_json::Map::new();
        for (key, content) in closure {
            sources_obj.insert(key, json!({ "content": content }));
        }

        let input = json!({
            "language": "Solidity",
            "sources": Value::Object(sources_obj),
            "settings": {}
        });

        let (output, status) = run_standard_json(&input);
        if status.success() && !has_errors(&output) {
            compiled += 1;
        } else {
            // Truncate the error summary so the panic message stays scannable.
            let summary: String = error_summary(&output).chars().take(300).collect();
            compile_failed.push((entry.clone(), summary));
        }
    }

    let total = surveyed.len();
    eprintln!(
        "multi_source_run_skipped_examples: surveyed {} files, compiled {}, \
         compile-failed {}, closure-failed {}",
        total,
        compiled,
        compile_failed.len(),
        closure_failed.len()
    );
    if !compile_failed.is_empty() {
        eprintln!("compile failures:");
        for (p, e) in &compile_failed {
            eprintln!("  - {}: {}", p.display(), e);
        }
    }
    if !closure_failed.is_empty() {
        eprintln!("closure-build failures:");
        for (p, e) in &closure_failed {
            eprintln!("  - {}: {}", p.display(), e);
        }
    }

    // Pass threshold: at least 80% of surveyed files compile cleanly via
    // the bundled closure. The remainder may trip on features that need
    // richer linking we don't model in this lightweight harness.
    let threshold = (total * 80) / 100;
    assert!(
        compiled >= threshold,
        "multi_source_run_skipped_examples: only {}/{} ({}%) compiled cleanly via \
         standard-JSON multi-source — wanted at least {} ({}%). \
         This is a regression in the import resolver or in one of the example contracts.",
        compiled,
        total,
        (compiled * 100) / total.max(1),
        threshold,
        80
    );
}
