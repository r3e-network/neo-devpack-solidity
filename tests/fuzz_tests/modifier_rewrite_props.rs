//! Property-based tests for the modifier rewriting code path.
//!
//! Targets `src/solidity/analyse/modifiers/rewrite/expressions.rs` and
//! `src/solidity/analyse/modifiers/rewrite/statements.rs`. When a Solidity
//! function carries one or more `modifier`s, the compiler inlines the modifier
//! body around the function body, replacing the modifier's `_;` placeholder
//! with the function body. The two files above implement that rewrite for
//! expressions and statements respectively. The shapes covered here exercise
//! varied modifier shapes (multiple statements before/after `_;`, control flow
//! around `_;`, parameters consumed inside the body, modifier inheritance, and
//! a modifier-revert path observed via try/catch) so the per-arm `match`
//! branches in `rewrite_expression` / `rewrite_statement` are reached.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

/// One modifier shape used by `modifier_chain_rewrite_compiles`.
///
/// Each variant pins down a distinct `_;` placement that the rewriter must
/// handle: statements before, statements after, branches around, and a local
/// variable declaration before. The `params` field carries the modifier's
/// formal parameters (rendered into the modifier signature) and `args`
/// carries the call-site arguments (rendered into the function attachment).
#[derive(Debug, Clone)]
struct ModifierShape {
    name: String,
    params: String,
    args: String,
    body: String,
}

fn shape_strategy(idx: usize) -> impl Strategy<Value = ModifierShape> {
    // Use the index to keep modifier names unique within a chain; proptest
    // shrinks to the same shape every time and we still get variety from the
    // `prop_oneof` selector below.
    let name = format!("mod{}", idx);
    prop_oneof![
        // Single require before `_;`, no params.
        Just(ModifierShape {
            name: name.clone(),
            params: String::new(),
            args: String::new(),
            body: r#"require(true, "ok"); _;"#.to_string(),
        }),
        // Multiple statements before `_;`, parameter consumed in body.
        Just(ModifierShape {
            name: name.clone(),
            params: "uint256 x".to_string(),
            args: "42".to_string(),
            body: "uint256 t = x + 1; require(t > 0, \"pre\"); _;".to_string(),
        }),
        // `_;` followed by a post-action that reads a parameter (post-rewrite
        // the placeholder is replaced by the function body and the trailing
        // statements remain — exercises the "multiple statements after `_;`"
        // path in `rewrite_statement` for the enclosing `Block`).
        Just(ModifierShape {
            name: name.clone(),
            params: "uint256 y".to_string(),
            args: "7".to_string(),
            body: "_; uint256 _post = y * 2; require(_post >= y, \"post\");".to_string(),
        }),
        // Control flow around `_;`: the placeholder lives inside an `if`-then
        // branch, the else branch reverts. Exercises `Statement::If` rewriting.
        Just(ModifierShape {
            name: name.clone(),
            params: "bool flag".to_string(),
            args: "true".to_string(),
            body: "if (flag) { _; } else { revert(\"flag\"); }".to_string(),
        }),
        // Local-variable definition before `_;`, no params. Exercises
        // `Statement::VariableDefinition` and `Expression::MemberAccess`.
        Just(ModifierShape {
            name,
            params: String::new(),
            args: String::new(),
            body: "uint256 _ts = block.timestamp; require(_ts >= 0, \"ts\"); _;".to_string(),
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    // ==================================================================
    // (a) Chained modifiers — assert the contract compiles successfully at
    //     all 4 opt levels and the function selector is present in the
    //     manifest.
    // ==================================================================
    #[test]
    fn modifier_chain_rewrite_compiles(
        n in 1usize..=4,
        s1 in shape_strategy(1),
        s2 in shape_strategy(2),
        s3 in shape_strategy(3),
        s4 in shape_strategy(4),
    ) {
        let shapes: Vec<&ModifierShape> = [&s1, &s2, &s3, &s4].into_iter().take(n).collect();

        let mut mod_decls = String::new();
        let mut attachments = String::new();
        for shape in &shapes {
            mod_decls.push_str(&format!(
                "    modifier {}({}) {{ {} }}\n",
                shape.name, shape.params, shape.body
            ));
            attachments.push(' ');
            if shape.params.is_empty() {
                attachments.push_str(&shape.name);
            } else {
                attachments.push_str(&format!("{}({})", shape.name, shape.args));
            }
        }

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract TestContract {{
{decls}
    function f() public{attach} returns (uint256) {{
        return 1;
    }}
}}"#,
            decls = mod_decls,
            attach = attachments,
        );

        for level in 0u8..=3u8 {
            let result = compile_contracts(&source, false, level);
            prop_assert!(
                result.is_ok(),
                "compile failed at O{} for n={} chain: {:?}\n--- source ---\n{}",
                level, n, result.err(), source
            );
            let arts = result.unwrap();
            prop_assert!(!arts.is_empty(), "no artifacts produced at O{}", level);

            // Selector for `f` must be reachable in the manifest.
            let methods = arts[0].manifest["abi"]["methods"]
                .as_array()
                .expect("abi.methods array");
            let found = methods.iter().any(|m| {
                m.get("name").and_then(serde_json::Value::as_str) == Some("f")
            });
            prop_assert!(found, "function 'f' missing from manifest at O{}", level);
        }
    }

    // ==================================================================
    // (b) Modifier-inline runtime assertion — the modifier increments a
    //     storage counter before `_;`. After N calls the counter must equal
    //     N. Proves the modifier's pre-statements actually execute on every
    //     call and that the placeholder substitution didn't drop them.
    // ==================================================================
    #[test]
    fn modifier_inline_with_runtime_assertion(
        calls in 1u8..=5,
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counter;
    modifier bump() {
        counter = counter + 1;
        _;
    }
    function tick() external bump returns (uint256) {
        return counter;
    }
    function read() external view returns (uint256) {
        return counter;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("modifier_inline compile: {:?}", e));
        prop_assert!(!arts.is_empty());
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        for i in 1..=calls {
            let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "tick",
                &[] as &[StackItem]).expect("tick() host call");
            prop_assert!(r.success,
                "tick() iteration {} must succeed; exc={:?}",
                i, r.exception.as_ref().map(|e| &e.message));
            let got = decode_uint_le(&r.return_data);
            prop_assert_eq!(got.clone(), num_bigint::BigUint::from(i as u64),
                "after call #{} counter must equal {}; got {}", i, i, got);
        }

        // Independent read() to confirm the persisted value matches.
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "read",
            &[] as &[StackItem]).expect("read() host call");
        prop_assert!(r.success, "read() must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got, num_bigint::BigUint::from(calls as u64),
            "final read() must equal call count {}", calls);
    }

    // ==================================================================
    // (c) Modifier inherited from a base contract — Derived's function uses
    //     Base.mod(). The rewriter must resolve the inherited symbol and
    //     inline the body cleanly.
    // ==================================================================
    #[test]
    fn modifier_with_inheritance(
        mod_name in identifier_strategy(),
        fn_name in identifier_strategy(),
    ) {
        // Disambiguate from "f", "g", and any pre-occupied identifier in the
        // template — also avoid collision between `mod_name` and `fn_name`.
        prop_assume!(mod_name != fn_name);

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Base {{
    uint256 public guard;
    modifier {mname}() {{
        require(guard == 0, "locked");
        guard = 1;
        _;
        guard = 0;
    }}
}}
contract Derived is Base {{
    function {fname}() external {mname} returns (uint256) {{
        return 7;
    }}
}}"#,
            mname = mod_name,
            fname = fn_name,
        );

        let result = compile_contracts(&source, false, 2);
        prop_assert!(
            result.is_ok(),
            "inherited modifier compile failed: {:?}\n--- source ---\n{}",
            result.err(), source,
        );

        // Manifest must expose the Derived function (the artifact for the
        // deployed contract is the last in the list — Derived is the
        // top-level contract).
        let arts = result.unwrap();
        prop_assert!(!arts.is_empty());
        let derived = arts.iter().find(|a| {
            a.manifest.get("name").and_then(serde_json::Value::as_str) == Some("Derived")
        }).unwrap_or(&arts[arts.len() - 1]);
        let methods = derived.manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");
        let found = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some(fn_name.as_str())
        });
        prop_assert!(found,
            "function '{}' missing from Derived manifest; methods={:?}",
            fn_name,
            methods.iter()
                .filter_map(|m| m.get("name").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
        );
    }

    // ==================================================================
    // (d) Modifier-revert path observed via try/catch — the modifier's
    //     `revert` reaches the caller correctly. The catch arm must fire,
    //     the wrapper must report the captured failure as a non-zero
    //     return value.
    // ==================================================================
    #[test]
    fn modifier_revert_path_caught(
        revert_msg in "[a-zA-Z]{1,12}"
    ) {
        // Single-quote a fixed message rather than the proptest one — proptest
        // generates ASCII letters here; concat into the contract literally.
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    modifier alwaysFail() {{
        revert("{msg}");
        _;
    }}
    function blocked() external alwaysFail returns (uint256) {{
        return 99;
    }}
    function caller() external returns (uint256) {{
        try this.blocked() returns (uint256 v) {{
            return v;
        }} catch Error(string memory /*reason*/) {{
            return 1;
        }} catch (bytes memory /*lowLevelData*/) {{
            return 2;
        }}
    }}
}}"#,
            msg = revert_msg,
        );

        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("modifier_revert_path compile: {:?}\n--- source ---\n{}", e, src));
        prop_assert!(!arts.is_empty());
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "caller",
            &[] as &[StackItem]).expect("caller() host call");
        prop_assert!(r.success,
            "caller() must succeed (the revert is caught inside try/catch); exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        // The catch arm must have fired — either the typed Error(string) arm
        // (returns 1) or the generic bytes arm (returns 2). Anything else
        // (0 = success path of try, or 99 = blocked() body executed) means
        // the revert path through the modifier did NOT propagate correctly.
        prop_assert!(
            got == num_bigint::BigUint::from(1u64) || got == num_bigint::BigUint::from(2u64),
            "modifier revert must be caught; expected 1 or 2, got {} (rd_hex={})",
            got, hex::encode(&r.return_data),
        );
    }
}
