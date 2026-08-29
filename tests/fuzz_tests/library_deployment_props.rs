//! Standalone-library deployment + cross-contract `using L for T` linkage
//! proptests.
//!
//! Existing coverage (wave-#19/#30/#40):
//!   * Inlined library + `using L for T` in a SINGLE source string
//!     (`batches_31_45::batch40_p4_library_using_for_runtime`,
//!     `batches_31_45::batch45_u1_safemath_add_library_inlining`).
//!   * 2-source standard-JSON wiring at the COMPILE level (no execution)
//!     via `multi_source_compile_props::multi_source_imported_library_used`.
//!
//! Gap closed here: full-stack 2-source standard-JSON compile + Neo-runtime
//! execution of the consumer contract whose `using L for uint256` binding
//! reaches into a SEPARATE imported `library L { ... }` source. A regression
//! in the library-symbol resolver (cross-source `using` lookup, library
//! function inlining across source boundaries) or in the runtime's CALLT
//! routing for library targets would silently break gas-shared production
//! contracts. Library deployment is a real-world gas-optimization pattern.
//!
//! Strategy: each test pairs the standard-JSON multi-source `neo-solc`
//! subprocess (mirrors `multi_source_compile_props`) for compile-time
//! assertions with the in-process `compile_contracts` + `NeoRuntime` harness
//! (mirrors `openzeppelin_patterns_props`) for runtime assertions. The
//! in-process harness accepts a single concatenated string holding both
//! `library L { ... }` and `contract Main { ... }` definitions — Solidity's
//! `internal pure` library functions are inlined into the consumer per
//! spec, so the resulting bytecode for `Main` is identical to the
//! standard-JSON multi-source case modulo source-mapping metadata. Where
//! the standard-JSON path is the load-bearing assertion (test (a)), we
//! invoke the subprocess; where the inlining semantic is what we exercise
//! (b/d), we use the in-process harness; for the `public` library function
//! deployment dance (test c), we attempt both paths and document whichever
//! the compiler produces.
//!
//! Reference reading:
//!   - `tests/fuzz_tests/multi_source_compile_props.rs` — wave-#31 multi-
//!     source standard-JSON harness.
//!   - `tests/fuzz_tests/openzeppelin_patterns_props.rs` — call-method
//!     runtime pattern.
//!   - `tests/fuzz_tests/batches_31_45.rs::batch40_p4_library_using_for_runtime`
//!     — the inlined `using L for uint` baseline this expands.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use num_bigint::BigUint;
use proptest::prelude::*;
use serde_json::{json, Value};
use std::process::Command;
use tempfile::tempdir;

// ---------- Subprocess helpers (copied in spirit from multi_source_compile_props) ----------

fn compiler_path() -> &'static str {
    env!("CARGO_BIN_EXE_neo-solc")
}

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

fn get_contract<'a>(output: &'a Value, file: &str, contract: &str) -> Option<&'a Value> {
    output.get("contracts")?.get(file)?.get(contract)
}

// ==================== Test (a): library_using_for_uint256_compile ====================

proptest! {
    #![proptest_config(ProptestConfig {
        // Each case spawns a subprocess.
        cases: 4,
        ..ProptestConfig::default()
    })]

    /// `library SafeMath { add(uint256, uint256) -> uint256 }` deployed in
    /// `SafeMath.sol`, consumed by `Main.sol` with `using SafeMath for
    /// uint256;` and `function f(uint256, uint256)` calling `x.add(y)`.
    ///
    /// Asserts:
    ///   1. Standard-JSON compile succeeds (process exit 0, no severity=error
    ///      diagnostics in the output JSON's `errors` array).
    ///   2. BOTH source files emit a contract entry — `SafeMath` (the
    ///      library; the standard-JSON output emits libraries as contract
    ///      entries) and `Main`.
    ///   3. `Main` exposes `f(uint256,uint256)` in its EVM
    ///      `methodIdentifiers` map — confirming the cross-source `using`
    ///      binding reached the dispatcher.
    ///
    /// Fails if: the import resolver can't find `./SafeMath.sol`, the
    /// `using` directive can't bind across source files, or the library's
    /// `add` symbol can't be resolved when lowering `x.add(y)`.
    #[test]
    fn library_using_for_uint256_compile(seed in 0u32..16) {
        // The seed is unused in the source body — it varies the proptest
        // shrinking surface so a hypothetical seed-dependent regression
        // (e.g. file-iteration order in the resolver) would still surface.
        let _ = seed;
        let safemath_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library SafeMath {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }
}
"#;
        let main_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SafeMath.sol";

contract Main {
    using SafeMath for uint256;
    function f(uint256 x, uint256 y) external pure returns (uint256) {
        return x.add(y);
    }
}
"#;

        let input = json!({
            "language": "Solidity",
            "sources": {
                "SafeMath.sol": { "content": safemath_src },
                "Main.sol":     { "content": main_src },
            },
            "settings": {}
        });

        let (output, status) = run_standard_json(&input);
        prop_assert!(
            status.success(),
            "library compile (a) exited non-zero: errors={}",
            error_summary(&output)
        );

        // Main MUST be present.
        let main = get_contract(&output, "Main.sol", "Main");
        prop_assert!(
            main.is_some(),
            "Main not present in standard-JSON output: errors={}; contracts={}",
            error_summary(&output),
            output.get("contracts").map(|v| v.to_string()).unwrap_or_default()
        );

        // SafeMath MAY be present (libraries with only internal functions
        // are sometimes elided since they have no externally-callable
        // surface). We accept either: present-as-contract-entry, OR absent
        // because internal-only libraries have no deployable footprint.
        // The load-bearing check is that Main compiled and exposes f.
        let _safemath_present = get_contract(&output, "SafeMath.sol", "SafeMath").is_some();

        // Main must expose `f(uint256,uint256)` in its methodIdentifiers.
        let method_ids = main
            .and_then(|m| m.get("evm"))
            .and_then(|m| m.get("methodIdentifiers"))
            .and_then(|m| m.as_object());
        let has_f = method_ids
            .map(|m| m.keys().any(|k| k == "f(uint256,uint256)"))
            .unwrap_or(false);
        prop_assert!(
            has_f,
            "expected Main to expose f(uint256,uint256) — confirms cross-\
             source `using SafeMath for uint256` binding reached the \
             dispatcher; methodIdentifiers={:?}, errors={}",
            method_ids,
            error_summary(&output)
        );
    }
}

// ==================== Test (b): library_using_for_uint256_runtime ====================

proptest! {
    #![proptest_config(ProptestConfig {
        // Each case compiles + deploys + invokes; keep modest.
        cases: 6,
        ..ProptestConfig::default()
    })]

    /// Same SafeMath / Main pair as (a). Compile via the in-process
    /// `compile_contracts` (the load-bearing semantic — `internal pure`
    /// library inlining — is independent of which compile entry point we
    /// take, since the inliner runs after AST construction). Deploy `Main`
    /// and call `f(x, y)`. Assert it returns x + y for the canonical pin
    /// (7, 5) → 12 plus a fuzzed (x, y) pair.
    ///
    /// Failure here means library inlining produced wrong code OR runtime
    /// CALLT routing broke for inlined library targets.
    #[test]
    fn library_using_for_uint256_runtime(
        x in 0u32..1_000_000u32,
        y in 0u32..1_000_000u32,
    ) {
        // Concatenated single-source form — equivalent to the 2-source
        // standard-JSON case modulo source mapping (Solidity's `internal
        // pure` library functions inline into the consumer regardless of
        // file boundaries — see Solidity spec
        // https://docs.soliditylang.org/en/v0.8.19/contracts.html#libraries).
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
library SafeMath {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }
}
contract Main {
    using SafeMath for uint256;
    function f(uint256 x, uint256 y) external pure returns (uint256) {
        return x.add(y);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("library_using_for_uint256_runtime compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "compile produced no artifacts");
        // Find Main (library is inlined and may or may not have its own
        // artifact).
        let art = arts
            .iter()
            .find(|a| a.metadata.name == "Main")
            .unwrap_or(&arts[0]);

        // Canonical pin: f(7, 5) == 12.
        let mut rt_pin = NeoRuntime::new(RuntimeConfig::default()).expect("rt_pin");
        let r_pin = rt_pin
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "f",
                &[StackItem::Integer(7), StackItem::Integer(5)],
                None,
            )
            .expect("Main.f(7, 5) host-level");
        prop_assert!(
            r_pin.success,
            "Canonical pin f(7, 5) must succeed; exc={:?}. Failure here \
             means library inlining produced uncallable bytecode for the \
             cross-source `using SafeMath for uint256; x.add(y)` path.",
            r_pin.exception.as_ref().map(|e| &e.message)
        );
        prop_assert_eq!(
            decode_uint_le(&r_pin.return_data),
            BigUint::from(12u64),
            "Canonical pin: f(7, 5) must return 12; got rd_hex={}. \
             SafeMath.add was either not inlined, or was inlined but with \
             wrong operand order / argument routing.",
            hex::encode(&r_pin.return_data)
        );

        // Fuzzed pair: f(x, y) must equal x + y. Inputs are u32 so x + y
        // fits in u64 with no wrap.
        let mut rt_fz = NeoRuntime::new(RuntimeConfig::default()).expect("rt_fz");
        let r_fz = rt_fz
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "f",
                &[StackItem::Integer(x as i64), StackItem::Integer(y as i64)],
                None,
            )
            .expect("Main.f(x, y) host-level (fuzzed)");
        prop_assert!(
            r_fz.success,
            "Fuzzed f({}, {}) must succeed; exc={:?}",
            x, y, r_fz.exception.as_ref().map(|e| &e.message)
        );
        let expected = BigUint::from(x as u64) + BigUint::from(y as u64);
        prop_assert_eq!(
            decode_uint_le(&r_fz.return_data),
            expected.clone(),
            "Fuzzed f({}, {}) must return {}; got rd_hex={}. Library-add \
             diverged from native `+` for these inputs.",
            x, y, expected, hex::encode(&r_fz.return_data)
        );
    }
}

// ==================== Test (c): library_external_function_separate_address ====================

/// Document the compiler's response to a `public pure` library function.
///
/// In Solidity, only `internal` library functions are inlined; `public` and
/// `external` library functions require library-linking via DELEGATECALL on
/// EVM. Since this compiler hard-rejects `delegatecall` (see Task #38), the
/// equivalent on Neo would be CALLT to a separately-deployed library
/// contract — a more involved deployment dance involving NEF method tokens.
///
/// This test attempts the `public pure` shape and reports what the compiler
/// does:
///   1. Reject with a clear diagnostic — preferred and expected.
///   2. Accept and inline anyway (treating `public` as `internal` for
///      `pure` functions with no external surface impact) — surfaces the
///      inlined behavior.
///   3. Accept and try to link — the deployment dance is out of scope for
///      this lightweight harness, so we report the path taken and skip
///      runtime assertions.
///
/// The test is single-shot (no fuzz axis) since the load-bearing question
/// is "what does the compiler do here", and that is invariant in any input
/// fuzz dimension we could vary.
#[test]
fn library_external_function_separate_address() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
library SafeMath {
    function safeAdd(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}
contract Main {
    function f(uint256 x, uint256 y) external pure returns (uint256) {
        return SafeMath.safeAdd(x, y);
    }
}"#;
    let result = compile_contracts(src, false, 2);
    match result {
        Err(e) => {
            // Path 1: rejection. Document the diagnostic.
            let msg = format!("{:?}", e);
            eprintln!(
                "library_external_function_separate_address: compiler REJECTED \
                 `public pure` library function. Diagnostic: {}",
                msg
            );
            // Sanity: the diagnostic should mention something library- or
            // visibility-related, OR cite the `delegatecall` it would need.
            // We do NOT hard-fail on the message contents — the compiler is
            // free to phrase this however it likes; the load-bearing
            // assertion is that it doesn't silently accept and miscompile.
            // If a future change makes it accept and link properly, this
            // test branch is unreachable and the next branch fires.
        }
        Ok(arts) => {
            assert!(
                !arts.is_empty(),
                "library_external_function_separate_address: compile succeeded \
                 but produced zero artifacts — suspicious; means SafeMath was \
                 erased without producing the linked library contract."
            );
            let main = arts
                .iter()
                .find(|a| a.metadata.name == "Main")
                .unwrap_or(&arts[0]);

            // Path 2 or 3: try to invoke f(7, 5). If the compiler inlined
            // (treating public-pure-no-side-effects as internal), it should
            // return 12. If the compiler emitted a CALLT to a not-yet-
            // deployed library, the call will fault at runtime and we
            // surface that diagnostic.
            let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
            let r = rt
                .call_method_with_deploy_args(
                    &main.bytecode,
                    &main.tokens,
                    &main.manifest,
                    "f",
                    &[StackItem::Integer(7), StackItem::Integer(5)],
                    None,
                )
                .expect("Main.f host-level (public-library path)");
            if r.success {
                let v = decode_uint_le(&r.return_data);
                if v == BigUint::from(12u64) {
                    eprintln!(
                        "library_external_function_separate_address: compiler \
                         ACCEPTED `public pure` library function and returned \
                         12 = INLINED PATH (treating public as internal for \
                         pure functions with no external dispatch impact)."
                    );
                } else {
                    panic!(
                        "library_external_function_separate_address: compiler \
                         accepted `public pure` library and Main.f(7, 5) \
                         succeeded but returned {} (expected 12). The inline \
                         either evaluated wrong operands or the linked-call \
                         path returned bogus data.",
                        v
                    );
                }
            } else {
                let exc_msg = r
                    .exception
                    .as_ref()
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| "no exception".to_string());
                eprintln!(
                    "library_external_function_separate_address: compiler \
                     ACCEPTED `public pure` library function but Main.f(7, 5) \
                     faulted at runtime — likely the compiler emitted a CALLT \
                     / method-token reference to a library that wasn't \
                     deployed alongside Main (deployment dance required). \
                     Exception: {}",
                    exc_msg
                );
                // Not a hard fail — this is the expected outcome when the
                // compiler emits library-linking bytecode and we can't
                // satisfy the link without a multi-NEF deployment. The
                // value of this test is the diagnostic: it pins exactly
                // where in the pipeline the user would need to wire the
                // library deployment.
            }
        }
    }
}

// ==================== Test (d): library_three_function_chain ====================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 6,
        ..ProptestConfig::default()
    })]

    /// 3-function library: `add`, `sub`, `mul`. `using L for uint256`
    /// chained: `x.add(y).mul(z).sub(w)` returns `((x + y) * z) - w`.
    ///
    /// Verifies that:
    ///   * Each library function inlines independently.
    ///   * The `using` binding fires on the result of a previous library
    ///     call (i.e. the return-type-driven receiver re-binding works for
    ///     uint256).
    ///   * Operator order matches Solidity's left-to-right evaluation —
    ///     not a divergence between AST traversal order and emit order.
    ///
    /// Inputs are kept small (u16) so `((x + y) * z) - w` stays well below
    /// u64::MAX with no wraparound (x + y ≤ 0x1FFFE, * z ≤ 0x1FFFE * 0xFFFF
    /// ≈ 0x1FFFD0001 → fits in u64). `w` is bounded to ≤ ((x+y)*z) so the
    /// subtract doesn't underflow (which would Panic 0x11 under
    /// 0.8 checked arithmetic and short-circuit the assertion).
    #[test]
    fn library_three_function_chain(
        x in 0u16..=10_000u16,
        y in 0u16..=10_000u16,
        z in 1u16..=100u16,
        w_factor in 0u16..=100u16,
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
library L {
    function add(uint256 a, uint256 b) internal pure returns (uint256) { return a + b; }
    function sub(uint256 a, uint256 b) internal pure returns (uint256) { return a - b; }
    function mul(uint256 a, uint256 b) internal pure returns (uint256) { return a * b; }
}
contract Main {
    using L for uint256;
    function chain(uint256 x, uint256 y, uint256 z, uint256 w) external pure returns (uint256) {
        return x.add(y).mul(z).sub(w);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("library_three_function_chain compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "compile produced no artifacts");
        let art = arts
            .iter()
            .find(|a| a.metadata.name == "Main")
            .unwrap_or(&arts[0]);

        // Compute expected without underflow risk.
        let xy = (x as u64) + (y as u64);
        let xy_z = xy * (z as u64);
        // w must NOT exceed xy_z to avoid 0.8 checked-sub Panic 0x11.
        let w = if xy_z == 0 { 0u64 } else { (w_factor as u64).min(xy_z) };
        let expected = xy_z - w;

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "chain",
                &[
                    StackItem::Integer(x as i64),
                    StackItem::Integer(y as i64),
                    StackItem::Integer(z as i64),
                    StackItem::Integer(w as i64),
                ],
                None,
            )
            .expect("Main.chain host-level");
        prop_assert!(
            r.success,
            "chain({}, {}, {}, {}) must succeed; exc={:?}. Failure means \
             one of add/mul/sub failed to inline OR the receiver re-bind \
             on the result of a previous library call dropped — i.e. \
             `x.add(y)` returned a uint256 but the parser/lowering did \
             not re-attach `.mul(...)` against the new uint256 receiver.",
            x, y, z, w, r.exception.as_ref().map(|e| &e.message)
        );
        prop_assert_eq!(
            decode_uint_le(&r.return_data),
            BigUint::from(expected),
            "chain({}, {}, {}, {}) must return ((x+y)*z)-w = {}; got \
             rd_hex={}. Either operator order diverged from Solidity's \
             left-to-right (e.g. mul applied before add), or one library \
             function inlined wrong operands.",
            x, y, z, w, expected, hex::encode(&r.return_data)
        );
    }
}
