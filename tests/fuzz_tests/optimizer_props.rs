//! Optimizer-level property tests.
//!
//! Verifies that optimization levels preserve semantics, NEF/Manifest correctness,
//! and compilation determinism.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::neo::{build_nef_with_tokens, parse_nef};
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

/// Compile `source` at the given optimization level and call `method` with `args`.
fn compile_and_call(
    source: &str,
    opt_level: u8,
    method: &str,
    args: &[StackItem],
) -> Result<neo_devpack_solidity::runtime::ExecutionResult, String> {
    let artifacts = compile_contracts(source, false, opt_level)
        .map_err(|e| format!("compile failed at O{opt_level}: {e:?}"))?;
    if artifacts.is_empty() {
        return Err("no artifacts".to_string());
    }
    let art = &artifacts[0];
    let mut rt =
        NeoRuntime::new(RuntimeConfig::default()).map_err(|e| format!("runtime failed: {e:?}"))?;
    rt.call_method(&art.bytecode, &art.tokens, &art.manifest, method, args)
        .map_err(|e| format!("call_method failed: {e:?}"))
}

/// Assert that two execution results are semantically equivalent (success,
/// return data, exception shape, storage state changes, and notification
/// logs). M-TEST3 fix — the previous version compared only `return_data`,
/// which made a peephole pass that reorders PUT and Notify invisible to the
/// optimizer differential.
fn assert_results_equivalent(
    a: &neo_devpack_solidity::runtime::ExecutionResult,
    b: &neo_devpack_solidity::runtime::ExecutionResult,
) {
    assert_eq!(a.success, b.success, "success mismatch between O0 and O3");
    assert_eq!(
        a.return_data, b.return_data,
        "return_data mismatch between O0 and O3"
    );
    match (&a.exception, &b.exception) {
        (None, None) => {}
        (Some(ae), Some(be)) => {
            assert_eq!(
                ae.exception_type, be.exception_type,
                "exception type mismatch between O0 and O3"
            );
            assert_eq!(
                ae.message, be.message,
                "exception message mismatch between O0 and O3"
            );
        }
        _ => panic!(
            "exception presence mismatch: a={:?}, b={:?}",
            a.exception, b.exception
        ),
    }

    // M-TEST3 fix — compare storage state changes (order + content).
    assert_eq!(
        a.state_changes.len(),
        b.state_changes.len(),
        "state_changes count mismatch between O0 and O3: a={:?}, b={:?}",
        a.state_changes,
        b.state_changes
    );
    for (i, (ac, bc)) in a
        .state_changes
        .iter()
        .zip(b.state_changes.iter())
        .enumerate()
    {
        assert_eq!(ac, bc, "state_changes[{i}] mismatch between O0 and O3");
    }

    // M-TEST3 fix — compare notification logs (order + content). A
    // peephole pass that reorders emit + storage write is observable
    // on-chain and was previously invisible.
    assert_eq!(
        a.logs.len(),
        b.logs.len(),
        "logs count mismatch between O0 and O3: a={:?}, b={:?}",
        a.logs,
        b.logs
    );
    for (i, (al, bl)) in a.logs.iter().zip(b.logs.iter()).enumerate() {
        assert_eq!(al, bl, "logs[{i}] mismatch between O0 and O3");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // ------------------------------------------------------------------
    // 1. Optimizer semantic equivalence — pure arithmetic with branching
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_semantic_equivalence_pure_arith(
        func_name in identifier_strategy(),
        x in any::<u32>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {func_name}(uint256 a) public pure returns (uint256) {{
        uint256 b = a * 3 + 7;
        uint256 c = b / 2;
        if (c > 1000) {{
            c = c - 1000;
        }}
        return c;
    }}
}}"#,
            func_name = func_name
        );

        let r0 = compile_and_call(&source, 0, &func_name, &[StackItem::Integer(x as i64)]);
        let r3 = compile_and_call(&source, 3, &func_name, &[StackItem::Integer(x as i64)]);

        prop_assert!(r0.is_ok(), "O0 execution failed: {:?}", r0.err());
        prop_assert!(r3.is_ok(), "O3 execution failed: {:?}", r3.err());
        assert_results_equivalent(&r0.unwrap(), &r3.unwrap());
    }

    // ------------------------------------------------------------------
    // 2. Optimizer semantic equivalence — bounded loops + conditionals
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_semantic_equivalence_loop_conditional(
        func_name in identifier_strategy(),
        n in 0u32..50u32,
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {func_name}(uint256 n) public pure returns (uint256) {{
        uint256 sum = 0;
        for (uint256 i = 0; i < n && i < 50; i++) {{
            if (i % 2 == 0) {{
                sum += i;
            }} else {{
                sum += i * 2;
            }}
        }}
        return sum;
    }}
}}"#,
            func_name = func_name
        );

        let r0 = compile_and_call(&source, 0, &func_name, &[StackItem::Integer(n as i64)]);
        let r3 = compile_and_call(&source, 3, &func_name, &[StackItem::Integer(n as i64)]);

        prop_assert!(r0.is_ok(), "O0 execution failed: {:?}", r0.err());
        prop_assert!(r3.is_ok(), "O3 execution failed: {:?}", r3.err());
        assert_results_equivalent(&r0.unwrap(), &r3.unwrap());
    }

    // ------------------------------------------------------------------
    // 3. Optimizer semantic equivalence — recursion (factorial)
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_semantic_equivalence_recursion(
        func_name in identifier_strategy(),
        n in 0u32..15u32,
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function {func_name}(uint256 n) public pure returns (uint256) {{
        if (n <= 1) return 1;
        return n * {func_name}(n - 1);
    }}
}}"#,
            func_name = func_name
        );

        let r0 = compile_and_call(&source, 0, &func_name, &[StackItem::Integer(n as i64)]);
        let r3 = compile_and_call(&source, 3, &func_name, &[StackItem::Integer(n as i64)]);

        prop_assert!(r0.is_ok(), "O0 execution failed: {:?}", r0.err());
        prop_assert!(r3.is_ok(), "O3 execution failed: {:?}", r3.err());
        assert_results_equivalent(&r0.unwrap(), &r3.unwrap());
    }

    // ------------------------------------------------------------------
    // 4. Optimizer semantic equivalence — internal function calls
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_semantic_equivalence_internal_calls(
        func_name in identifier_strategy(),
        x in any::<u32>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function helper(uint256 a) internal pure returns (uint256) {{
        return a * 3 + 1;
    }}
    function {func_name}(uint256 x) public pure returns (uint256) {{
        return helper(x) + helper(x + 1);
    }}
}}"#,
            func_name = func_name
        );

        let r0 = compile_and_call(&source, 0, &func_name, &[StackItem::Integer(x as i64)]);
        let r3 = compile_and_call(&source, 3, &func_name, &[StackItem::Integer(x as i64)]);

        prop_assert!(r0.is_ok(), "O0 execution failed: {:?}", r0.err());
        prop_assert!(r3.is_ok(), "O3 execution failed: {:?}", r3.err());
        assert_results_equivalent(&r0.unwrap(), &r3.unwrap());
    }

    // ------------------------------------------------------------------
    // 5. Optimizer semantic equivalence — modifier with require
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_semantic_equivalence_modifier(
        func_name in identifier_strategy(),
        mod_name in identifier_strategy(),
        x in 1u32..100u32,
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    modifier {mod_name}(uint256 min) {{
        require(min >= 1, "min too small");
        _;
    }}
    function {func_name}(uint256 x) public {mod_name}(x) pure returns (uint256) {{
        return x + 1;
    }}
}}"#,
            func_name = func_name,
            mod_name = mod_name
        );

        let r0 = compile_and_call(&source, 0, &func_name, &[StackItem::Integer(x as i64)]);
        let r3 = compile_and_call(&source, 3, &func_name, &[StackItem::Integer(x as i64)]);

        prop_assert!(r0.is_ok(), "O0 execution failed: {:?}", r0.err());
        prop_assert!(r3.is_ok(), "O3 execution failed: {:?}", r3.err());
        assert_results_equivalent(&r0.unwrap(), &r3.unwrap());
    }

    // ------------------------------------------------------------------
    // 6. Optimizer semantic equivalence — storage variables + events
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_semantic_equivalence_storage_and_events(
        func_name in identifier_strategy(),
        event_name in identifier_strategy(),
        init_value in uint_value_strategy(),
        add_value in any::<u32>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = {init};
    event {event_name}(uint256 oldValue, uint256 newValue);

    function {func_name}(uint256 x) public returns (uint256) {{
        uint256 oldValue = value;
        value = oldValue + x;
        emit {event_name}(oldValue, value);
        return value;
    }}
}}"#,
            func_name = func_name,
            event_name = event_name,
            init = init_value
        );

        let artifacts0 = compile_contracts(&source, false, 0).expect("O0 compile");
        let artifacts3 = compile_contracts(&source, false, 3).expect("O3 compile");
        prop_assert!(!artifacts0.is_empty());
        prop_assert!(!artifacts3.is_empty());

        // Pin a non-zero `contract_account` so the runtime skips its
        // bytecode-derived address path (`RIPEMD160(SHA256(script))` — see
        // execution_impl_part1_init.rs). O0 and O3 necessarily produce
        // different bytecode, so without this pin the `LogEntry.address`
        // would differ by construction across opt levels — a harness
        // artifact, not an optimizer bug. With the pin, `address` is a
        // stable literal for both runtimes and the full `LogEntry` equality
        // (address + topics + data) is meaningful: any real Notify
        // reordering still surfaces as a topics/data/order mismatch.
        let rt_config = RuntimeConfig {
            contract_account: "0x1111111111111111111111111111111111111111".to_string(),
            ..Default::default()
        };
        let mut rt0 = NeoRuntime::new(rt_config.clone()).expect("rt0");
        let mut rt3 = NeoRuntime::new(rt_config).expect("rt3");

        let res0 = rt0.call_method(
            &artifacts0[0].bytecode, &artifacts0[0].tokens,
            &artifacts0[0].manifest, &func_name,
            &[StackItem::Integer(add_value as i64)],
        );
        let res3 = rt3.call_method(
            &artifacts3[0].bytecode, &artifacts3[0].tokens,
            &artifacts3[0].manifest, &func_name,
            &[StackItem::Integer(add_value as i64)],
        );

        prop_assert!(res0.is_ok(), "O0 call failed: {:?}", res0.err());
        prop_assert!(res3.is_ok(), "O3 call failed: {:?}", res3.err());
        assert_results_equivalent(&res0.unwrap(), &res3.unwrap());
    }

    // ------------------------------------------------------------------
    // 7. NEF correctness — magic header, checksum, tokens, round-trip
    // ------------------------------------------------------------------
    #[test]
    fn nef_format_validity(
        var_name in identifier_strategy(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 42;
    function add(uint256 a, uint256 b) public pure returns (uint256) {{
        return a + b;
    }}
}}"#,
            var_name
        );

        let artifacts = compile_contracts(&source, false, 3).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let art = &artifacts[0];

        let nef = build_nef_with_tokens(&art.bytecode, "neo-devpack-solidity-fuzz", "", &art.tokens)
            .expect("NEF build");

        // Magic header
        prop_assert!(nef.starts_with(b"NEF3"), "NEF must start with NEF3 magic");
        prop_assert!(nef.len() > 4, "NEF must be larger than checksum");

        // Checksum = sha256(sha256(prefix))[..4]
        let prefix = &nef[..nef.len() - 4];
        let stored_trailer = &nef[nef.len() - 4..];
        let first = Sha256::digest(prefix);
        let second = Sha256::digest(first);
        prop_assert_eq!(stored_trailer, &second[..4], "NEF checksum invalid");

        // Parse round-trip
        let parsed = parse_nef(&nef).expect("parse_nef must succeed");
        prop_assert_eq!(parsed.script, art.bytecode.clone(), "script mismatch after NEF parse");
        prop_assert_eq!(parsed.tokens.len(), art.tokens.len(), "token count mismatch");

        for (orig, out) in art.tokens.iter().zip(parsed.tokens.iter()) {
            prop_assert_eq!(orig.hash, out.hash, "token hash mismatch");
            prop_assert_eq!(&orig.method, &out.method, "token method mismatch");
            prop_assert_eq!(
                orig.parameters_count, out.parameters_count,
                "token parameters_count mismatch"
            );
            prop_assert_eq!(
                orig.has_return_value, out.has_return_value,
                "token has_return_value mismatch"
            );
            prop_assert_eq!(orig.call_flags, out.call_flags, "token call_flags mismatch");
        }
    }

    // ------------------------------------------------------------------
    // 8. Manifest correctness — required fields + ABI consistency
    // ------------------------------------------------------------------
    #[test]
    fn manifest_correctness(
        contract_name in identifier_strategy(),
        func_name in identifier_strategy(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract {name} {{
    uint256 public stored = 1;
    event ValueChanged(uint256 newValue);

    function {func}(uint256 x) public pure returns (uint256) {{
        return x * 2;
    }}
}}"#,
            name = contract_name,
            func = func_name
        );

        let artifacts = compile_contracts(&source, false, 2).expect("compile");
        prop_assert!(!artifacts.is_empty());
        let manifest = &artifacts[0].manifest;

        // Required top-level keys
        for key in ["name", "abi", "permissions", "supportedstandards", "groups"] {
            prop_assert!(
                manifest.get(key).is_some(),
                "manifest missing required top-level key: {key}"
            );
        }

        // ABI structure
        let abi = manifest.get("abi").expect("abi object");
        let methods = abi.get("methods")
            .and_then(|m| m.as_array())
            .expect("abi.methods must be an array");
        let events = abi.get("events")
            .and_then(|e| e.as_array())
            .expect("abi.events must be an array");

        prop_assert!(!methods.is_empty(), "manifest abi.methods must not be empty");

        // Our function should be declared
        let has_func = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some(func_name.as_str())
        });
        prop_assert!(has_func, "function '{}' missing from manifest methods", func_name);

        // Event should be declared
        let has_event = events.iter().any(|e| {
            e.get("name").and_then(serde_json::Value::as_str) == Some("ValueChanged")
        });
        prop_assert!(has_event, "event 'ValueChanged' missing from manifest events");

        // Each method has consistent shape
        for method in methods {
            prop_assert!(method.get("name").is_some(), "method missing name");
            prop_assert!(method.get("parameters").is_some(), "method missing parameters");
            prop_assert!(method.get("returntype").is_some(), "method missing returntype");
            if let Some(offset) = method.get("offset") {
                prop_assert!(offset.is_number(), "method offset must be a number");
            }
        }

        // Each event has consistent shape
        for event in events {
            prop_assert!(event.get("name").is_some(), "event missing name");
            prop_assert!(event.get("parameters").is_some(), "event missing parameters");
        }

        // JSON round-trip
        let as_string = serde_json::to_string(manifest).expect("manifest must serialize");
        let reparsed: serde_json::Value = serde_json::from_str(&as_string)
            .expect("manifest must reparse");
        prop_assert_eq!(&reparsed, manifest, "manifest JSON round-trip was not lossless");
    }

    // ------------------------------------------------------------------
    // 9. Determinism — same source compiled twice produces identical output
    // ------------------------------------------------------------------
    #[test]
    fn optimizer_determinism(
        var_name in identifier_strategy(),
        opt_level in 0u8..=3u8,
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 123;
    function compute(uint256 a, uint256 b) public pure returns (uint256) {{
        uint256 c = a + b;
        if (c > 100) {{
            c = c * 2;
        }}
        return c;
    }}
}}"#,
            var_name
        );

        let result1 = compile_contracts(&source, false, opt_level);
        let result2 = compile_contracts(&source, false, opt_level);

        prop_assert!(result1.is_ok(), "first compile failed: {:?}", result1.err());
        prop_assert!(result2.is_ok(), "second compile failed: {:?}", result2.err());

        let arts1 = result1.unwrap();
        let arts2 = result2.unwrap();
        prop_assert_eq!(arts1.len(), arts2.len(), "artifact count mismatch");

        for (a1, a2) in arts1.iter().zip(arts2.iter()) {
            prop_assert_eq!(
                &a1.bytecode, &a2.bytecode,
                "bytecode differed for opt_level={}", opt_level
            );
            prop_assert_eq!(
                &a1.manifest, &a2.manifest,
                "manifest differed for opt_level={}", opt_level
            );
            prop_assert_eq!(
                a1.tokens.len(), a2.tokens.len(),
                "token count differed for opt_level={}", opt_level
            );
            for (t1, t2) in a1.tokens.iter().zip(a2.tokens.iter()) {
                prop_assert_eq!(t1.hash, t2.hash, "token hash mismatch");
                prop_assert_eq!(&t1.method, &t2.method, "token method mismatch");
                prop_assert_eq!(
                    t1.parameters_count, t2.parameters_count,
                    "token parameters_count mismatch"
                );
                prop_assert_eq!(
                    t1.has_return_value, t2.has_return_value,
                    "token has_return_value mismatch"
                );
                prop_assert_eq!(t1.call_flags, t2.call_flags, "token call_flags mismatch");
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    // ------------------------------------------------------------------
    // 10. Optimizer semantic equivalence — arithmetic pipeline across all 4 levels
    // ------------------------------------------------------------------
    // Compiles a non-trivial arithmetic pipeline at O0/O1/O2/O3 and asserts
    // the return_data is identical at every level. A divergence would be an
    // optimizer bug.
    #[test]
    fn optimizer_semantic_equivalence_arithmetic_pipeline(
        a in any::<u32>(),
        b in any::<u32>(),
        c in any::<u32>(),
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {
    function compute(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {
        unchecked {
            return (a + b) * c + (a ^ b) - (a % (c + 1));
        }
    }
}"#
        .to_string();

        let args = [
            StackItem::Integer(a as i64),
            StackItem::Integer(b as i64),
            StackItem::Integer(c as i64),
        ];

        let r0 = compile_and_call(&source, 0, "compute", &args);
        let r1 = compile_and_call(&source, 1, "compute", &args);
        let r2 = compile_and_call(&source, 2, "compute", &args);
        let r3 = compile_and_call(&source, 3, "compute", &args);

        prop_assert!(r0.is_ok(), "O0 execution failed: {:?}", r0.err());
        prop_assert!(r1.is_ok(), "O1 execution failed: {:?}", r1.err());
        prop_assert!(r2.is_ok(), "O2 execution failed: {:?}", r2.err());
        prop_assert!(r3.is_ok(), "O3 execution failed: {:?}", r3.err());

        let out0 = r0.unwrap().return_data;
        let out1 = r1.unwrap().return_data;
        let out2 = r2.unwrap().return_data;
        let out3 = r3.unwrap().return_data;

        prop_assert_eq!(&out0, &out1, "return_data mismatch between O0 and O1");
        prop_assert_eq!(&out1, &out2, "return_data mismatch between O1 and O2");
        prop_assert_eq!(&out2, &out3, "return_data mismatch between O2 and O3");
    }
}

// ----------------------------------------------------------------------------
// Manifest event wire-shape fidelity
//
// Generates a contract with N events (N in 1..=4), each with 1..=4 parameters
// drawn from a small set of Solidity ABI types and a randomized `indexed`
// flag. Compiles, then asserts that the Neo manifest's `abi.events` array
// contains each declared event exactly once, declared with the TRUTHFUL EVM
// wire shape the compiled bytecode hands to System.Runtime.Notify:
//
//   [topic0: ByteArray, <one ByteArray per indexed param>, data: ByteArray]
//
// Post-Basilisk Neo nodes fault any notification whose state-item count
// mismatches the manifest declaration, so the declaration MUST track the
// wire shape — not the Solidity parameter list. (NEP-17/NEP-11 `Transfer`
// declarations are the exception: they are emitted natively and keep their
// declared names/types — none of the generated `EvN` events match that
// signature; the native declaration is pinned by gap_events_native_tests.)
// ----------------------------------------------------------------------------

/// One Solidity ABI type the test can sample.
#[derive(Clone, Debug)]
struct ParamType {
    /// Solidity source-level type, e.g. "uint256"
    solidity: &'static str,
}

fn param_type_strategy() -> impl Strategy<Value = ParamType> {
    prop_oneof![
        Just(ParamType {
            solidity: "uint256"
        }),
        Just(ParamType { solidity: "int256" }),
        Just(ParamType {
            solidity: "address"
        }),
        Just(ParamType { solidity: "bool" }),
        Just(ParamType {
            solidity: "bytes32"
        }),
        Just(ParamType { solidity: "string" }),
        Just(ParamType { solidity: "bytes" }),
    ]
}

/// (parameter type, indexed?)
fn event_param_strategy() -> impl Strategy<Value = (ParamType, bool)> {
    (param_type_strategy(), any::<bool>())
}

/// One event: its parameter list (1..=4 entries).
fn event_decl_strategy() -> impl Strategy<Value = Vec<(ParamType, bool)>> {
    proptest::collection::vec(event_param_strategy(), 1..=4)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn manifest_event_parameter_type_fidelity(
        // 1..=4 events, each a list of (type, indexed)
        events in proptest::collection::vec(event_decl_strategy(), 1..=4),
    ) {
        // Generate stable, unique event names: Ev0, Ev1, ...
        // Param names are also fixed (p0, p1, ...) to avoid identifier-collision
        // noise; the property under test is *type fidelity*, not naming.
        let mut event_decls = String::new();
        for (i, params) in events.iter().enumerate() {
            event_decls.push_str("    event Ev");
            event_decls.push_str(&i.to_string());
            event_decls.push('(');
            for (j, (pt, indexed)) in params.iter().enumerate() {
                if j > 0 { event_decls.push_str(", "); }
                event_decls.push_str(pt.solidity);
                if *indexed {
                    // `indexed` is only legal for value types; all 7 types we
                    // sample accept it in Solidity 0.8.x. (string/bytes get
                    // hashed when indexed but the declaration is still valid.)
                    event_decls.push_str(" indexed");
                }
                event_decls.push_str(" p");
                event_decls.push_str(&j.to_string());
            }
            event_decls.push_str(");\n");
        }

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract EvTypes {{
{events}
    function noop() public pure returns (uint256) {{ return 0; }}
}}"#,
            events = event_decls,
        );

        let artifacts = compile_contracts(&source, false, 2)
            .map_err(|e| TestCaseError::fail(format!("compile failed: {e:?}")))?;
        prop_assert!(!artifacts.is_empty(), "no artifacts produced");
        let manifest = &artifacts[0].manifest;

        let manifest_events = manifest
            .get("abi")
            .and_then(|abi| abi.get("events"))
            .and_then(|e| e.as_array())
            .expect("manifest abi.events must be an array");

        // 1) Each declared event appears exactly once.
        for (i, _) in events.iter().enumerate() {
            let name = format!("Ev{i}");
            let occurrences = manifest_events
                .iter()
                .filter(|e| e.get("name").and_then(|n| n.as_str()) == Some(name.as_str()))
                .count();
            prop_assert_eq!(
                occurrences, 1,
                "event '{}' should appear exactly once in manifest abi.events (found {})",
                name, occurrences
            );
        }

        // 2) Per-event: the declared shape must be the EVM wire shape
        //    `[topic0, <indexed...>, data]`, all ByteArray. Indexed params
        //    keep their declared `pJ` names; non-indexed params fold into
        //    the trailing abi.encoded `data` slot.
        for (i, params) in events.iter().enumerate() {
            let name = format!("Ev{i}");
            let event_obj = manifest_events
                .iter()
                .find(|e| e.get("name").and_then(|n| n.as_str()) == Some(name.as_str()))
                .expect("event located above");

            let manifest_params = event_obj
                .get("parameters")
                .and_then(|p| p.as_array())
                .expect("event parameters must be an array");

            let indexed_names: Vec<String> = params
                .iter()
                .enumerate()
                .filter(|(_, (_, indexed))| *indexed)
                .map(|(j, _)| format!("p{j}"))
                .collect();
            let expected_names: Vec<String> = std::iter::once("topic0".to_string())
                .chain(indexed_names)
                .chain(std::iter::once("data".to_string()))
                .collect();

            prop_assert_eq!(
                manifest_params.len(), expected_names.len(),
                "event '{}' must declare the EVM wire shape [topic0, indexed..., data]: \
                 manifest has {} parameter(s), wire shape has {} \
                 (source decl: {:?})",
                name, manifest_params.len(), expected_names.len(), params
            );

            for (j, expected_name) in expected_names.iter().enumerate() {
                let m_param = &manifest_params[j];
                let m_name = m_param
                    .get("name")
                    .and_then(|n| n.as_str())
                    .expect("manifest event parameter must have a 'name' string");
                prop_assert_eq!(
                    m_name, expected_name.as_str(),
                    "event '{}' param[{}] name mismatch: expected '{}', got '{}'",
                    name, j, expected_name, m_name
                );
                let m_type = m_param
                    .get("type")
                    .and_then(|t| t.as_str())
                    .expect("manifest event parameter must have a 'type' string");
                prop_assert_eq!(
                    m_type, "ByteArray",
                    "event '{}' param[{}] ('{}') must be ByteArray (32-byte topic slot / \
                     keccak hash / abi.encoded data); got '{}'",
                    name, j, expected_name, m_type
                );

                // Per Neo N3 manifest spec, event params must NOT carry the
                // Solidity `indexed` field. (Already covered by a unit test,
                // but cheap to assert here and protects against regressions
                // when type-mapping code is touched.)
                prop_assert!(
                    m_param.get("indexed").is_none(),
                    "event '{}' param[{}] manifest must not include 'indexed'",
                    name, j
                );
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Optimizer 4-level differential on randomly-generated pure expressions
//
// Background: the structured cargo-fuzz target
// (`fuzz/fuzz_targets/fuzz_target_structured_sol.rs`) generates random
// Solidity but only compiles each iteration at *one* random opt-level per
// run, so opt-level divergence on randomly-generated source is invisible to
// that harness. The hand-written tests above (cases 1..=10) all hard-code
// one source per test.
//
// This block fills the gap: a proptest-grammar-driven differential where
// the *source itself* is randomly generated, and every case is compiled at
// O0/O1/O2/O3 with the same input. Any divergence (success flag,
// return_data, exception type) at any pair of levels is a real optimizer
// bug and surfaces as a counterexample.
//
// Grammar (independent from cargo-fuzz / `arbitrary` — proptest uses
// different traits):
//
//   Expr ::= Lit          // small uint256 literal
//          | Var          // one of: a, b, c
//          | Bin Op Expr Expr     // arithmetic / bitwise
//          | Shift Sh Expr Sh     // shift by a small fixed amount
//          | Cmp Cmp Expr Expr    // comparison (lifted to uint256)
//          | Tern Expr Expr Expr  // ternary (cond ? t : e)
//
//   Op  ::= + | - | * | / | % | & | | | ^
//   Sh  ::= << | >>     (rhs = small u8 in 0..32, never panics)
//   Cmp ::= == | != | < | <= | > | >=
//
// Constraints:
//   - Depth-bounded (≤4 nesting levels) so each case finishes <100ms.
//   - Pure: no storage, events, external calls, or state changes.
//   - All arithmetic is wrapped in `unchecked { ... }` so overflow does
//     NOT introduce optimizer-orthogonal Panic(0x11) noise. Div / mod
//     guards against zero RHS by replacing with `(rhs | 1)` at the source
//     level, eliminating Panic(0x12) noise too. Goal is to maximize the
//     fraction of cases that exercise the *successful return-data* path,
//     where a divergence is unambiguous evidence of an optimizer bug.
// ----------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum DOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
}

impl DOp {
    fn sym(&self) -> &'static str {
        match self {
            DOp::Add => "+",
            DOp::Sub => "-",
            DOp::Mul => "*",
            DOp::Div => "/",
            DOp::Mod => "%",
            DOp::And => "&",
            DOp::Or => "|",
            DOp::Xor => "^",
        }
    }
    /// True if this op needs a non-zero RHS to avoid Panic(0x12).
    fn needs_nonzero_rhs(&self) -> bool {
        matches!(self, DOp::Div | DOp::Mod)
    }
}

#[derive(Clone, Debug)]
enum DCmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl DCmp {
    fn sym(&self) -> &'static str {
        match self {
            DCmp::Eq => "==",
            DCmp::Ne => "!=",
            DCmp::Lt => "<",
            DCmp::Le => "<=",
            DCmp::Gt => ">",
            DCmp::Ge => ">=",
        }
    }
}

#[derive(Clone, Debug)]
enum DShift {
    Shl,
    Shr,
}

impl DShift {
    fn sym(&self) -> &'static str {
        match self {
            DShift::Shl => "<<",
            DShift::Shr => ">>",
        }
    }
}

#[derive(Clone, Debug)]
enum DExpr {
    Lit(u32),
    /// Variable index 0..=2 → a / b / c (the three formal parameters).
    Var(u8),
    Bin(DOp, Box<DExpr>, Box<DExpr>),
    Shift(DShift, Box<DExpr>, u8 /* 0..32 */),
    Cmp(DCmp, Box<DExpr>, Box<DExpr>),
    Tern(Box<DExpr>, Box<DExpr>, Box<DExpr>),
}

impl DExpr {
    /// Render Solidity source for this expression. Wraps every binary op in
    /// parentheses to sidestep precedence subtleties — the property under
    /// test is *opt-level fidelity*, not Solidity precedence.
    ///
    /// `Div` / `Mod` rewrite their RHS as `((rhs) | 1)` to guarantee a
    /// non-zero divisor without changing the value when the RHS is already
    /// odd-and-nonzero, which is the common case for the small literals we
    /// generate. (`x | 1` is always odd and never zero, so Panic(0x12) is
    /// statically impossible.) Comparison results are coerced to `uint256`
    /// via the `(... ? 1 : 0)` ternary so the outer expression always has
    /// type uint256.
    fn render(&self, out: &mut String) {
        match self {
            DExpr::Lit(n) => {
                out.push_str(&format!("uint256({})", n));
            }
            DExpr::Var(i) => {
                let v = match i % 3 {
                    0 => "a",
                    1 => "b",
                    _ => "c",
                };
                out.push_str(v);
            }
            DExpr::Bin(op, l, r) => {
                out.push('(');
                l.render(out);
                out.push(' ');
                out.push_str(op.sym());
                out.push(' ');
                if op.needs_nonzero_rhs() {
                    // Force RHS != 0 to dodge Panic(0x12).
                    out.push_str("((");
                    r.render(out);
                    out.push_str(") | 1)");
                } else {
                    r.render(out);
                }
                out.push(')');
            }
            DExpr::Shift(s, lhs, amt) => {
                // Bound shift amount to 0..32 so it stays well under the
                // 255 guard and within EVM-/Solidity-defined behavior.
                out.push('(');
                lhs.render(out);
                out.push(' ');
                out.push_str(s.sym());
                out.push_str(&format!(" {}", amt % 32));
                out.push(')');
            }
            DExpr::Cmp(op, l, r) => {
                // Lift bool to uint256 so any sub-expr is uint256-typed.
                out.push_str("((");
                l.render(out);
                out.push(' ');
                out.push_str(op.sym());
                out.push(' ');
                r.render(out);
                out.push_str(") ? uint256(1) : uint256(0))");
            }
            DExpr::Tern(c, t, e) => {
                // Cast cond to bool via `!= 0`. Both arms are uint256.
                out.push_str("((");
                c.render(out);
                out.push_str(" != 0) ? ");
                t.render(out);
                out.push_str(" : ");
                e.render(out);
                out.push(')');
            }
        }
    }
}

/// Leaf-only strategy — used at depth 0 of the recursive grammar.
fn dexpr_leaf_strategy() -> impl Strategy<Value = DExpr> {
    prop_oneof![
        // Bug #16 (resolved): unchecked uint256 Add/Sub/Mul now widens to a
        // >8-byte unsigned-magnitude ByteArray in the IR, so the runtime takes
        // the wide-BigInt path at all optimizer levels (no narrow i64 strict
        // overflow fault). Literals re-widened to the full u32 domain.
        any::<u32>().prop_map(DExpr::Lit),
        (0u8..=2).prop_map(DExpr::Var),
    ]
}

/// Recursive expression strategy bounded by `depth` (0 = leaf only).
/// Depth ≤ 4 keeps rendered source small and proptest cases <100 ms.
fn dexpr_strategy() -> impl Strategy<Value = DExpr> {
    let leaf = dexpr_leaf_strategy();
    leaf.prop_recursive(
        4,  // levels deep
        32, // total nodes across the tree
        4,  // collection size (here: branch fan-out hint)
        |inner| {
            let op = prop_oneof![
                Just(DOp::Add),
                Just(DOp::Sub),
                Just(DOp::Mul),
                Just(DOp::Div),
                Just(DOp::Mod),
                Just(DOp::And),
                Just(DOp::Or),
                Just(DOp::Xor),
            ];
            let cmp = prop_oneof![
                Just(DCmp::Eq),
                Just(DCmp::Ne),
                Just(DCmp::Lt),
                Just(DCmp::Le),
                Just(DCmp::Gt),
                Just(DCmp::Ge),
            ];
            let shift = prop_oneof![Just(DShift::Shl), Just(DShift::Shr)];
            prop_oneof![
                (op, inner.clone(), inner.clone()).prop_map(|(o, l, r)| DExpr::Bin(
                    o,
                    Box::new(l),
                    Box::new(r)
                )),
                (shift, inner.clone(), 0u8..32u8).prop_map(|(s, l, amt)| DExpr::Shift(
                    s,
                    Box::new(l),
                    amt
                )),
                (cmp, inner.clone(), inner.clone()).prop_map(|(c, l, r)| DExpr::Cmp(
                    c,
                    Box::new(l),
                    Box::new(r)
                )),
                (inner.clone(), inner.clone(), inner.clone()).prop_map(|(c, t, e)| DExpr::Tern(
                    Box::new(c),
                    Box::new(t),
                    Box::new(e)
                )),
            ]
        },
    )
}

/// Render a complete contract whose single external view function evaluates
/// the generated expression on three uint256 parameters and returns the
/// result. Every binop is inside `unchecked` so overflow yields the EVM
/// wrapping value rather than a panic — the assertion target is
/// optimizer-level fidelity, not Solidity-mode panics.
fn render_diff_contract(expr: &DExpr) -> String {
    let mut body = String::new();
    expr.render(&mut body);
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract D {{
    function f(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {{
        unchecked {{
            return {body};
        }}
    }}
}}"#,
        body = body
    )
}

/// One execution result reduced to the three fields that matter for an
/// opt-level differential. Comparing this projection avoids tripping on
/// gas_used / metadata fields that are *expected* to differ across
/// optimization levels.
#[derive(Debug, PartialEq, Eq)]
struct DiffOutcome {
    success: bool,
    return_data: Vec<u8>,
    /// `None` when no exception; otherwise the canonical exception type
    /// name. Message text is excluded because optimization can affect
    /// formatting of stack-trace-derived messages without changing the
    /// classification.
    exception_kind: Option<&'static str>,
}

impl DiffOutcome {
    fn from_result(r: &neo_devpack_solidity::runtime::ExecutionResult) -> Self {
        let exception_kind = r.exception.as_ref().map(|e| e.exception_type.as_str());
        DiffOutcome {
            success: r.success,
            return_data: r.return_data.clone(),
            exception_kind,
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // ------------------------------------------------------------------
    // 11. Optimizer 4-level differential on randomly-generated pure
    //     arithmetic / bitwise / shift / comparison expressions
    // ------------------------------------------------------------------
    // Generates a random uint256-typed expression tree (depth ≤ 4) over
    // three formal parameters a / b / c, wraps it in a one-function
    // contract, then compiles at O0/O1/O2/O3 and asserts that all four
    // produce the same {success, return_data, exception classification}
    // for the same input. Any pairwise divergence is a real optimizer
    // bug.
    //
    // This is the proptest analogue of what the structured cargo-fuzz
    // target *cannot* see: that target picks one random opt-level per
    // iteration, so per-source O0-vs-O3 disagreement is invisible there.
    #[test]
    fn optimizer_four_level_differential_random_expr(
        expr in dexpr_strategy(),
        // Bug #16 (resolved): full u32 domain. The IR now widens unchecked
        // uint256 Add/Sub/Mul operands to the wide-BigInt path so all four
        // optimizer levels agree even when narrow-path i64 would have overflowed.
        a in any::<u32>(),
        b in any::<u32>(),
        c in any::<u32>(),
    ) {
        let source = render_diff_contract(&expr);
        let args = [
            StackItem::Integer(a as i64),
            StackItem::Integer(b as i64),
            StackItem::Integer(c as i64),
        ];

        let r0 = compile_and_call(&source, 0, "f", &args);
        let r1 = compile_and_call(&source, 1, "f", &args);
        let r2 = compile_and_call(&source, 2, "f", &args);
        let r3 = compile_and_call(&source, 3, "f", &args);

        // If every level fails to compile/execute *the same way*, that's
        // not an optimizer bug — it's a frontend property. Skip those.
        // But if some succeed and others don't, that *is* a bug — fail.
        let oks = [r0.is_ok(), r1.is_ok(), r2.is_ok(), r3.is_ok()];
        let any_ok = oks.iter().any(|x| *x);
        let all_ok = oks.iter().all(|x| *x);
        prop_assert!(
            !any_ok || all_ok,
            "compile/exec succeeded at some opt levels but not others — opt-level-dependent compile bug.\n\
             O0={:?} O1={:?} O2={:?} O3={:?}\nsource:\n{}",
            r0.as_ref().err(), r1.as_ref().err(), r2.as_ref().err(), r3.as_ref().err(),
            source
        );
        if !all_ok {
            // All four failed at the frontend or runtime in the same way;
            // not interesting for opt-diff.
            return Ok(());
        }

        let o0 = DiffOutcome::from_result(&r0.unwrap());
        let o1 = DiffOutcome::from_result(&r1.unwrap());
        let o2 = DiffOutcome::from_result(&r2.unwrap());
        let o3 = DiffOutcome::from_result(&r3.unwrap());

        // Pairwise compare against O0 — transitivity gives full equivalence
        // and the failure message names the diverging level directly.
        prop_assert_eq!(
            &o0, &o1,
            "O0 vs O1 divergence — optimizer bug.\nsource:\n{}\nargs: a={} b={} c={}",
            source, a, b, c
        );
        prop_assert_eq!(
            &o0, &o2,
            "O0 vs O2 divergence — optimizer bug.\nsource:\n{}\nargs: a={} b={} c={}",
            source, a, b, c
        );
        prop_assert_eq!(
            &o0, &o3,
            "O0 vs O3 divergence — optimizer bug.\nsource:\n{}\nargs: a={} b={} c={}",
            source, a, b, c
        );
    }
}
