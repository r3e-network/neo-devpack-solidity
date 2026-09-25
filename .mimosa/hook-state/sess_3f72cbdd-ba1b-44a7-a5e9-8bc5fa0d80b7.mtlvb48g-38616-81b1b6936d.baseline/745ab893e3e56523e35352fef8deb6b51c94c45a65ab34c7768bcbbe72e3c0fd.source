//! Compiler determinism / reproducibility proptests.
//!
//! Reproducible builds are foundational for verification: an auditor must be
//! able to take a Solidity source, run the compiler, and obtain bytecode that
//! is byte-identical to what was deployed. Non-determinism — `HashMap`
//! iteration order leaking into emitted code, time-of-day strings in error
//! messages, unstable sort ordering — silently produces different bytecode
//! for identical inputs and breaks that contract.
//!
//! This module fuzzes two related determinism properties:
//!
//!   1. `compile_determinism_same_process` — compile the same source three
//!      times in a row at the same opt level, assert the NEF byte sequence
//!      and the manifest JSON byte sequence match across all three runs.
//!      A single byte differing is a real bug; the failure path captures
//!      the offending source, the diverging byte offset, and the two values
//!      so the regression can be triaged immediately.
//!
//!   2. `compile_opt_level_independence_for_pure_constants` — the four opt
//!      levels (0..=3) legitimately produce different NEF bytes (that is,
//!      after all, the point of optimization), but the contract's *public
//!      API* — the manifest's `abi.methods[].name` list and each method's
//!      parameter shape — is opt-level-invariant. A change there means the
//!      optimizer is mutating the ABI, which would silently break callers.

#![allow(dead_code)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use proptest::prelude::*;

/// Reuse the `compile_runtime_roundtrip` simple-type set, but trimmed to the
/// types whose canonical-default literal compiles cleanly under all four
/// opt levels for a `pure` function body. (The full set already does, but
/// keeping this enum local avoids a cross-module dependency on a private
/// type that could legitimately drift.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetType {
    Uint256,
    Bool,
    Address,
    Bytes32,
}

impl DetType {
    fn solidity(self) -> &'static str {
        match self {
            DetType::Uint256 => "uint256",
            DetType::Bool => "bool",
            DetType::Address => "address",
            DetType::Bytes32 => "bytes32",
        }
    }

    fn default_literal(self) -> &'static str {
        match self {
            DetType::Uint256 => "42",
            DetType::Bool => "true",
            DetType::Address => "address(0)",
            DetType::Bytes32 => "bytes32(0)",
        }
    }
}

fn det_type_strategy() -> impl Strategy<Value = DetType> {
    prop_oneof![
        Just(DetType::Uint256),
        Just(DetType::Bool),
        Just(DetType::Address),
        Just(DetType::Bytes32),
    ]
}

/// One declared method, structurally similar to `compile_runtime_roundtrip`'s
/// `MethodSpec` but local to this module so changes there can't quietly
/// alter the determinism harness's input shape.
#[derive(Debug, Clone)]
struct DetMethod {
    name: String,
    params: Vec<DetType>,
    ret: DetType,
}

impl DetMethod {
    fn render(&self) -> String {
        let mut params = String::new();
        for (i, ty) in self.params.iter().enumerate() {
            if i > 0 {
                params.push_str(", ");
            }
            params.push_str(ty.solidity());
            params.push_str(&format!(" p{i}"));
        }
        format!(
            "    function {name}({params}) external pure returns ({ret}) {{\n\
             \x20       return {lit};\n\
             \x20   }}\n",
            name = self.name,
            params = params,
            ret = self.ret.solidity(),
            lit = self.ret.default_literal(),
        )
    }
}

/// Build a self-contained Solidity source from an opt-level-stable shape.
/// `state_vars` are `uint256 public sX;` (auto-generates `sX()` getters in
/// the ABI). `methods` are pure constant returners.
fn build_source(n_state_vars: usize, methods: &[DetMethod]) -> String {
    let mut src = String::new();
    src.push_str("// SPDX-License-Identifier: MIT\n");
    src.push_str("pragma solidity ^0.8.19;\n");
    src.push_str("contract C {\n");
    for i in 0..n_state_vars {
        src.push_str(&format!("    uint256 public s{i};\n"));
    }
    for m in methods {
        src.push_str(&m.render());
    }
    src.push_str("}\n");
    src
}

/// Locate the first divergence between two byte sequences. Returns
/// `(offset, a_byte, b_byte)` or `None` if equal (and same length).
fn first_byte_divergence(a: &[u8], b: &[u8]) -> Option<(usize, Option<u8>, Option<u8>)> {
    let max = a.len().max(b.len());
    for i in 0..max {
        let av = a.get(i).copied();
        let bv = b.get(i).copied();
        if av != bv {
            return Some((i, av, bv));
        }
    }
    None
}

/// Extract the public-API surface of a manifest: a sorted list of
/// `(method_name, parameter_shape)` tuples. `parameter_shape` is the JSON
/// of the manifest's `parameters` array (which encodes both name and type
/// for each parameter), serialized as a string for stable comparison.
fn manifest_api_signature(manifest: &serde_json::Value) -> Vec<(String, String)> {
    let methods = manifest
        .get("abi")
        .and_then(|v| v.get("methods"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut sig: Vec<(String, String)> = methods
        .iter()
        .map(|m| {
            let name = m
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            let params = m
                .get("parameters")
                .map(|p| serde_json::to_string(p).unwrap_or_default())
                .unwrap_or_default();
            (name, params)
        })
        .collect();
    sig.sort();
    sig
}

/// Build a small contract from the proptest-supplied shape. Shared between
/// both proptests so the same generator is exercised for the same-process
/// determinism check and the cross-opt-level public-API check.
fn build_methods(n_methods: usize, first_name: &str) -> Vec<DetMethod> {
    let mut methods: Vec<DetMethod> = Vec::with_capacity(n_methods);
    methods.push(DetMethod {
        name: first_name.to_string(),
        params: Vec::new(),
        ret: DetType::Uint256,
    });
    for i in 1..n_methods {
        methods.push(DetMethod {
            name: format!("m{i}"),
            params: Vec::new(),
            ret: DetType::Uint256,
        });
    }
    // Fill in arity + return type per method using a deterministic schedule
    // (mirrors the pattern in compile_runtime_roundtrip.rs so each method
    // has different shape => more surface for non-determinism to leak
    // through).
    for (i, spec) in methods.iter_mut().enumerate() {
        let arity = i % 3; // 0, 1, 2
        let ret_idx = i % 4;
        spec.ret = match ret_idx {
            0 => DetType::Uint256,
            1 => DetType::Bool,
            2 => DetType::Address,
            _ => DetType::Bytes32,
        };
        spec.params = (0..arity)
            .map(|j| {
                let p_idx = (i * 3 + j * 2) % 4;
                match p_idx {
                    0 => DetType::Uint256,
                    1 => DetType::Bool,
                    2 => DetType::Address,
                    _ => DetType::Bytes32,
                }
            })
            .collect();
    }
    methods
}

/// Resolve a possibly-colliding fuzzed method name against the synthetic
/// names used by `build_methods` and the auto-getter names produced by the
/// state variables. Mirrors `compile_runtime_roundtrip`'s collision-dodge
/// logic so the determinism test never spuriously fails on a duplicate-
/// signature compile error.
fn resolve_first_name(method_name: &str, n_state_vars: usize) -> String {
    let collides_with_state_var = method_name
        .strip_prefix('s')
        .and_then(|tail| tail.parse::<usize>().ok())
        .map(|i| i < n_state_vars)
        .unwrap_or(false);
    let collides_with_synth_method = matches!(method_name, "m1" | "m2" | "m3");
    if collides_with_state_var || collides_with_synth_method {
        format!("f_{method_name}")
    } else {
        method_name.to_string()
    }
}

proptest! {
    // 16 cases: each case runs 3 compiles, so at 16 cases this is 48 compiles
    // per shrink-pass-free run. Bumping higher would cost wall-clock without
    // proportional coverage gain (the search space here is shape, not value).
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Compile the same source 3 times in the same process at the same opt
    /// level. Assert that:
    ///   * all 3 NEF bytecode payloads (`art.bytecode`) are byte-identical
    ///   * all 3 token vectors are equal
    ///   * all 3 manifest JSON serializations are byte-identical
    /// A single byte mismatch is a real bug.
    #[test]
    fn compile_determinism_same_process(
        n_state_vars in 0usize..=3,
        n_methods in 1usize..=3,
        method_name in identifier_strategy(),
        opt_level in 0u8..=3,
    ) {
        let first = resolve_first_name(&method_name, n_state_vars);
        let methods = build_methods(n_methods, &first);
        let source = build_source(n_state_vars, &methods);

        let compile_once = || -> (Vec<u8>, Vec<u8>, String) {
            let arts = compile_contracts(&source, false, opt_level)
                .unwrap_or_else(|e| panic!(
                    "determinism: compile failed (opt={opt_level}):\n{source}\nerr: {e:?}"
                ));
            assert!(!arts.is_empty(), "determinism: empty artifact list");
            let art = &arts[0];
            // Serialize tokens via bincode-equivalent: each MethodToken has
            // hash/method/parameter_count/has_return_value/call_flags fields,
            // all of which are byte-stable; we serialize via Debug for a
            // simple, deterministic textual form.
            let tokens_repr = format!("{:?}", art.tokens).into_bytes();
            // Manifest is serde_json::Value: serializing with the default
            // `to_string` writes object keys in insertion order (since
            // `serde_json::Map` preserves insertion order by default), so
            // a deterministic builder => a deterministic output string.
            let manifest_str = serde_json::to_string(&art.manifest)
                .expect("manifest must serialize");
            (art.bytecode.clone(), tokens_repr, manifest_str)
        };

        let (bc1, tk1, mf1) = compile_once();
        let (bc2, tk2, mf2) = compile_once();
        let (bc3, tk3, mf3) = compile_once();

        // ---- bytecode determinism --------------------------------------------
        if let Some((off, a, b)) = first_byte_divergence(&bc1, &bc2) {
            panic!(
                "DETERMINISM BUG: bytecode differs between compile #1 and #2 \
                 at byte offset {off}: 0x{a:02x?} vs 0x{b:02x?}\n\
                 opt_level={opt_level} n_state_vars={n_state_vars} n_methods={n_methods}\n\
                 source:\n{source}",
                a = a, b = b
            );
        }
        if let Some((off, a, b)) = first_byte_divergence(&bc1, &bc3) {
            panic!(
                "DETERMINISM BUG: bytecode differs between compile #1 and #3 \
                 at byte offset {off}: 0x{a:02x?} vs 0x{b:02x?}\n\
                 opt_level={opt_level} n_state_vars={n_state_vars} n_methods={n_methods}\n\
                 source:\n{source}",
                a = a, b = b
            );
        }

        // ---- token-list determinism -----------------------------------------
        prop_assert_eq!(&tk1, &tk2, "token list differs between compile #1 and #2");
        prop_assert_eq!(&tk1, &tk3, "token list differs between compile #1 and #3");

        // ---- manifest determinism --------------------------------------------
        if let Some((off, a, b)) = first_byte_divergence(mf1.as_bytes(), mf2.as_bytes()) {
            panic!(
                "DETERMINISM BUG: manifest JSON differs between compile #1 and #2 \
                 at byte offset {off}: {a:?} vs {b:?}\n\
                 opt_level={opt_level} n_state_vars={n_state_vars} n_methods={n_methods}\n\
                 manifest #1:\n{mf1}\n\nmanifest #2:\n{mf2}\n\nsource:\n{source}",
                a = a.map(|c| c as char),
                b = b.map(|c| c as char),
            );
        }
        if let Some((off, a, b)) = first_byte_divergence(mf1.as_bytes(), mf3.as_bytes()) {
            panic!(
                "DETERMINISM BUG: manifest JSON differs between compile #1 and #3 \
                 at byte offset {off}: {a:?} vs {b:?}\n\
                 opt_level={opt_level} n_state_vars={n_state_vars} n_methods={n_methods}\n\
                 manifest #1:\n{mf1}\n\nmanifest #3:\n{mf3}\n\nsource:\n{source}",
                a = a.map(|c| c as char),
                b = b.map(|c| c as char),
            );
        }
    }
}

proptest! {
    // 8 cases × 4 opt levels = 32 compiles per run. The shape generator is
    // deliberately small here — the assertion is structural (public-API
    // invariance), not value-driven.
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// For a contract that compiles cleanly at every opt level, the manifest's
    /// `abi.methods[].name` list and each method's `parameters` shape are
    /// opt-level-invariant. Bytecode legitimately differs across opt levels;
    /// the public API does not.
    #[test]
    fn compile_opt_level_independence_for_pure_constants(
        n_state_vars in 0usize..=2,
        n_methods in 1usize..=3,
        method_name in identifier_strategy(),
    ) {
        let first = resolve_first_name(&method_name, n_state_vars);
        let methods = build_methods(n_methods, &first);
        let source = build_source(n_state_vars, &methods);

        // Compile at all four opt levels.
        let mut sigs: Vec<Vec<(String, String)>> = Vec::with_capacity(4);
        for opt in 0u8..=3 {
            let arts = compile_contracts(&source, false, opt)
                .unwrap_or_else(|e| panic!(
                    "opt-independence: compile failed at opt={opt}:\n{source}\nerr: {e:?}"
                ));
            prop_assert!(
                !arts.is_empty(),
                "opt-independence: opt={} produced empty artifact list", opt
            );
            sigs.push(manifest_api_signature(&arts[0].manifest));
        }

        // The opt-level-0 signature is the reference; assert each higher
        // opt level matches it. If any do not, the optimizer is mutating
        // the ABI, which would silently break callers between deploys.
        let ref_sig = &sigs[0];
        for (opt, s) in sigs.iter().enumerate().skip(1) {
            prop_assert_eq!(
                ref_sig,
                s,
                "ABI public-surface differs between opt=0 and opt={}\nsource:\n{}",
                opt,
                source
            );
        }
    }
}
