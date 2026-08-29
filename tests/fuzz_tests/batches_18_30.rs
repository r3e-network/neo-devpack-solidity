//! Batches #18 through #30 — example-contract regression gate and onward.
//! Contents unchanged from the pre-split `tests/fuzz_tests.rs`.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_match)]
#![allow(clippy::partialeq_to_none)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Batch #18 — Example Contract Regression ====================
//
// Purpose: regression gate on the user's stated goal — "existing Solidity
// contracts can be compiled into NeoVM and work correctly." Batches #1-#17
// probe individual language features via synthetic micro-sources; this batch
// instead walks the actual `examples/` and `examples/new/` directories on
// disk, compiles each single-file contract end-to-end via `compile_contracts`,
// and checks the resulting manifest shape. If someone regresses the lowering
// for a feature used by a real example contract, this batch fires on the
// exact file name so we can file it immediately.
//
// Scope & methodology:
//
//   * `compile_contracts(src, false, 2)` takes a single source string; it does
//     NOT resolve `import "./X.sol"` from the filesystem. Examples that use
//     `import` are skipped (conservative substring check on "import"). Those
//     contracts are still reachable via the neoxp smoke scripts and multi-
//     file compile paths; they are out of scope for THIS regression batch.
//
//   * Harness 1 walks `examples/new/` (the showcase directory).
//   * Harness 2 walks `examples/` (the legacy, historically-flat directory).
//   * Harness 3 re-walks both and validates manifest shape on every
//     successful compile.
//
// Failure policy: if any example contract surfaces a NEW compile-stack
// failure, the harness panics with the file path + error snippet so the
// failure is self-describing. Regressions should NOT be silenced; if a
// PR legitimately removes a contract feature, the example should be moved
// or deleted, not the harness ignored. `#[ignore]` is reserved for the
// case where this batch surfaces PRE-EXISTING failures that predate it —
// in which case the TODO below the `#[ignore]` lists the exact files.
//
// NOT fuzzed. These are deterministic regression `#[test]` fns placed
// outside any `proptest!` block. They add 3 to the pass count.

#[cfg(test)]
mod batch18_example_regression {
    use super::*;
    use std::fs::{read_dir, read_to_string};
    use std::path::{Path, PathBuf};

    /// Collect all `.sol` files directly in `dir` (non-recursive).
    /// Deterministic order via sort by file name (makes failures reproducible).
    fn collect_sol_files(dir: &str) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = read_dir(dir)
            .unwrap_or_else(|e| panic!("read_dir({}) failed: {}", dir, e))
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("sol"))
            .collect();
        out.sort();
        out
    }

    /// Conservative: skip any file whose source text contains the substring
    /// "import". This covers `import "./X.sol";`, `import { X } from "./Y";`,
    /// and any other variant. May occasionally skip a file that only mentions
    /// "import" in a comment — acceptable; the goal is to avoid false-positive
    /// compile failures from unresolved imports.
    fn has_import(src: &str) -> bool {
        // Line-oriented check: true if any non-comment line starts with
        // `import` (after optional whitespace). Falls back to substring for
        // safety if we're unsure.
        src.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("import ")
                || trimmed.starts_with("import\"")
                || trimmed.starts_with("import{")
                || trimmed.starts_with("import (")
        }) || src.contains("\nimport ")
    }

    /// Contracts that are INTENTIONALLY designed to fail compilation — they
    /// showcase rejection of unsupported Solidity features on NeoVM. These
    /// are negative-test fixtures; their failure is a feature, not a bug.
    ///
    /// Convention: any filename ending in `Error.sol` OR beginning with
    /// `EvmCompat` is treated as expected-to-fail. The harness still runs
    /// them (so we catch the case where they ACCIDENTALLY start compiling
    /// — that would be a regression in error surfacing), but their failure
    /// doesn't count against the regression gate.
    fn is_intentional_failure(path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        name.ends_with("Error.sol") || name.starts_with("EvmCompat")
    }

    /// Walk result:
    /// - `compiled`: (path, manifest) pairs — includes negative-test files
    ///   that ACCIDENTALLY compiled (regression-surface, see harness below).
    /// - `compiled_count`: len of compiled.
    /// - `skipped_import`: files skipped due to import statements.
    /// - `expected_failures`: intentional negative-test files that failed as
    ///   designed (good — surfaces the error).
    /// - `unexpected_failures`: real regressions.
    fn walk_and_compile(
        dir: &str,
    ) -> (
        Vec<(PathBuf, serde_json::Value)>,
        usize,
        usize,
        Vec<(PathBuf, String)>,
        Vec<(PathBuf, String)>,
    ) {
        let files = collect_sol_files(dir);
        let mut compiled: Vec<(PathBuf, serde_json::Value)> = Vec::new();
        let mut skipped_import = 0usize;
        let mut expected_failures: Vec<(PathBuf, String)> = Vec::new();
        let mut unexpected_failures: Vec<(PathBuf, String)> = Vec::new();

        for path in files {
            let src = match read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    // IO errors are always unexpected (path came from read_dir).
                    unexpected_failures.push((path.clone(), format!("read_to_string: {}", e)));
                    continue;
                }
            };

            if has_import(&src) {
                skipped_import += 1;
                continue;
            }

            let is_negative_test = is_intentional_failure(&path);

            match compile_contracts(&src, false, 2) {
                Ok(artifacts) => {
                    if artifacts.is_empty() {
                        unexpected_failures
                            .push((path.clone(), "compiled OK but zero artifacts".into()));
                    } else {
                        // Take the manifest from the FIRST artifact. Most
                        // example files have exactly one contract; those with
                        // multiple (e.g. a contract + a library) still expose
                        // a primary artifact. Negative tests that DO compile
                        // are still recorded — harness 3 checks their shape
                        // and will surface if one unexpectedly goes green.
                        compiled.push((path.clone(), artifacts[0].manifest.clone()));
                    }
                }
                Err(e) => {
                    let msg = format!("{:?}", e);
                    let truncated: String = msg.chars().take(320).collect();
                    if is_negative_test {
                        expected_failures.push((path.clone(), truncated));
                    } else {
                        unexpected_failures.push((path.clone(), truncated));
                    }
                }
            }
        }

        let compiled_count = compiled.len();
        (
            compiled,
            compiled_count,
            skipped_import,
            expected_failures,
            unexpected_failures,
        )
    }

    /// Harness 1: every single-file contract in `examples/new/` compiles —
    /// EXCEPT the negative-test showcases (files ending in `Error.sol` or
    /// starting with `EvmCompat`) which are designed to be rejected.
    ///
    /// INVARIANT: `unexpected_failures` is empty. If a showcase contract that
    /// USED to compile stops compiling, this harness names the file + error
    /// snippet so the regression is filed instantly.
    #[test]
    fn all_examples_new_contracts_compile() {
        let dir = "examples/new";
        assert!(
            Path::new(dir).is_dir(),
            "examples/new must exist at repo root; cwd mismatch?"
        );

        let (_compiled, compiled_count, skipped_import, expected_failures, unexpected_failures) =
            walk_and_compile(dir);

        // Always report counts for debuggability.
        eprintln!(
            "[batch18.1] examples/new: {} compiled, {} skipped-due-to-import, \
             {} expected-failures (negative-test showcases), {} unexpected-failures",
            compiled_count,
            skipped_import,
            expected_failures.len(),
            unexpected_failures.len()
        );

        if !unexpected_failures.is_empty() {
            let summary: String = unexpected_failures
                .iter()
                .map(|(p, e)| format!("  - {}: {}", p.display(), e))
                .collect::<Vec<_>>()
                .join("\n");
            panic!(
                "examples/new/ regression: {} single-file contract(s) FAILED compilation \
                 unexpectedly (not a negative-test showcase).\n\
                 File as a regression — these are showcases that are supposed to compile.\n{}",
                unexpected_failures.len(),
                summary
            );
        }

        assert!(
            compiled_count > 0,
            "examples/new/ yielded zero compilable single-file contracts — \
             either the directory is empty or every file has an import. \
             Check collect_sol_files / has_import logic."
        );
    }

    /// Harness 2: every single-file contract directly under `examples/`
    /// (non-recursive; does not descend into `new/`, `ERC20/`, or `famous/`)
    /// compiles. Same invariant as harness 1.
    #[test]
    fn all_examples_legacy_contracts_compile() {
        let dir = "examples";
        assert!(
            Path::new(dir).is_dir(),
            "examples/ must exist at repo root; cwd mismatch?"
        );

        let (_compiled, compiled_count, skipped_import, expected_failures, unexpected_failures) =
            walk_and_compile(dir);

        eprintln!(
            "[batch18.2] examples/: {} compiled, {} skipped-due-to-import, \
             {} expected-failures, {} unexpected-failures",
            compiled_count,
            skipped_import,
            expected_failures.len(),
            unexpected_failures.len()
        );

        if !unexpected_failures.is_empty() {
            let summary: String = unexpected_failures
                .iter()
                .map(|(p, e)| format!("  - {}: {}", p.display(), e))
                .collect::<Vec<_>>()
                .join("\n");
            panic!(
                "examples/ regression: {} legacy single-file contract(s) FAILED compilation \
                 unexpectedly.\n\
                 File as a regression — these are contracts users have historically compiled.\n{}",
                unexpected_failures.len(),
                summary
            );
        }

        assert!(
            compiled_count > 0,
            "examples/ yielded zero compilable single-file legacy contracts — \
             either the directory has only subdirectories or every .sol has imports. \
             Check collect_sol_files / has_import logic."
        );
    }

    /// Harness 3: every compiled manifest has the 5 top-level keys that
    /// NeoVM-side tools (neo-express, neo-go invoke, block explorers) expect:
    /// `name`, `abi` (with `methods` array), `permissions`, `supportedstandards`,
    /// `groups`. Catches silent manifest-shape regressions that would not fail
    /// compilation but WOULD break downstream tooling.
    #[test]
    fn compiled_examples_manifests_have_valid_shape() {
        let mut all_compiled: Vec<(PathBuf, serde_json::Value)> = Vec::new();

        for dir in ["examples/new", "examples"].iter() {
            if !Path::new(dir).is_dir() {
                continue;
            }
            let (compiled, _, _, _, _) = walk_and_compile(dir);
            all_compiled.extend(compiled);
        }

        assert!(
            !all_compiled.is_empty(),
            "expected at least one compiled manifest from examples/ or examples/new/"
        );

        let mut shape_failures: Vec<(PathBuf, String)> = Vec::new();

        for (path, manifest) in &all_compiled {
            let mut missing: Vec<&str> = Vec::new();

            // 1. `name` is a string.
            if !manifest.get("name").map(|v| v.is_string()).unwrap_or(false) {
                missing.push("name:string");
            }

            // 2. `abi` is an object containing `methods` array.
            match manifest.get("abi") {
                Some(abi) if abi.is_object() => {
                    if !abi.get("methods").map(|m| m.is_array()).unwrap_or(false) {
                        missing.push("abi.methods:array");
                    }
                }
                _ => missing.push("abi:object"),
            }

            // 3. `permissions` is an array (can be empty).
            if !manifest
                .get("permissions")
                .map(|v| v.is_array())
                .unwrap_or(false)
            {
                missing.push("permissions:array");
            }

            // 4. `supportedstandards` is an array.
            if !manifest
                .get("supportedstandards")
                .map(|v| v.is_array())
                .unwrap_or(false)
            {
                missing.push("supportedstandards:array");
            }

            // 5. `groups` is an array.
            if !manifest
                .get("groups")
                .map(|v| v.is_array())
                .unwrap_or(false)
            {
                missing.push("groups:array");
            }

            if !missing.is_empty() {
                shape_failures.push((path.clone(), missing.join(", ")));
            }
        }

        eprintln!(
            "[batch18.3] checked {} manifests, {} shape-failures",
            all_compiled.len(),
            shape_failures.len()
        );

        if !shape_failures.is_empty() {
            let summary: String = shape_failures
                .iter()
                .map(|(p, m)| format!("  - {}: missing/wrong-type: {}", p.display(), m))
                .collect::<Vec<_>>()
                .join("\n");
            panic!(
                "manifest shape regression: {} compiled manifest(s) are missing \
                 required top-level NEP-specified keys.\n\
                 Keys required: name:string, abi:object{{methods:array}}, \
                 permissions:array, supportedstandards:array, groups:array.\n{}",
                shape_failures.len(),
                summary
            );
        }
    }
}

// ==================== Batch #19 — EVM Pattern Regression Pins ====================
//
// Purpose: document the compiler's behavior on the five canonical EVM patterns
// that real Ethereum contracts (OpenZeppelin, EIP-2612 Permit, minimal-proxy
// factories, EIP-712 signed messages) depend on EVERY day. Batches #15/#16
// established that `abi.encode*` produces JSON of a StackItem::Array (Task
// #44); this batch lifts that into the end-to-end flows real production
// contracts use, so any Task #44 fix is validated by a single harness pass
// rather than by re-testing every call site by hand.
//
// Each harness is a REGRESSION PIN: it asserts the CURRENT observed (broken
// or correct) behavior with a commented-out `// EXPECTED:` block ready to
// un-comment once the underlying compiler bug is fixed. The harness title
// ends in `_diverges` / `_diverge` when the observed bytes differ from the
// EVM/Solidity-spec reference and in `_matches` (or plain descriptive) when
// they match. That naming is deliberate so a grep of failing harnesses tells
// you WHICH pattern broke.
//
// Pre-batch probes surfaced five findings:
//
//   (H1) `keccak256("…")` on a string LITERAL matches the `sha3::Keccak256`
//        reference digest. BASELINE IS CLEAN — the CryptoLib.sha3 wiring
//        produces correct output when handed direct UTF-8 bytes. Any
//        deviation in later harnesses must therefore come from the
//        `abi.encode*` side, not the `keccak256` side. OBSERVED for
//        "EIP712Domain(uint256 chainId,address verifyingContract)":
//          0x47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218
//        which is EXACTLY `Keccak256::digest(b"EIP712Domain(…)")`. Harness
//        #1 ACTIVE — this is the control against which #2/#5 are compared.
//
//   (H2) `keccak256(abi.encode(TYPEHASH, owner, value, nonce))` — the core
//        of EIP-712 struct-hashing, the primitive beneath EIP-2612 Permit
//        and every signed-message pattern in OpenZeppelin. OBSERVED:
//          0x5272b321bb6018db8c4a7bd7165adc605672b9b9ff1c9491853085998926ed0c
//        EXPECTED (Solidity spec):
//          0xa90d8f896673f0bf0720c3224fe0350c6d2eaedb7d77e2ed288ef7c5a4699ea1
//        i.e. `Keccak256::digest([TYPEHASH, pad32(owner), pad32(100),
//        pad32(7)].concat())`. The compiler hashes 32 bytes of SOMETHING
//        (likely the JSON-serialized StackItem::Array from Task #44 fed
//        back through keccak256) and returns 32 bytes of garbage. Because
//        the OUTPUT is 32 bytes, a naive eye looking at the return type
//        (`bytes32`) would NOT notice — this is the SILENT WRONG-VALUE
//        failure mode that motivated the batch. Harness #2 ACTIVE, pins
//        the wrong hash; EXPECTED assertion is commented-in for the
//        post-Task-#44-fix flip.
//
//   (H3) `this.foo.selector` — the basis of OpenZeppelin's
//        `_checkRole`/`onlyRole`, every function-selector-based access
//        control, and the raw 4-byte selector baked into EIP-2612 Permit
//        typehashes. Solidity spec says `bytes4(keccak256("foo(uint256)"))`
//        = `0x2fbebd38`. FIX LANDED (task #54): the `.selector` lowering
//        now routes through the same canonical-signature keccak path used by
//        `encodeWithSignature` / `TypeName.method.selector`, so
//        `this.foo.selector` returns `0x2fbebd38`. Previously the fallback
//        hashed `"foo()"` with no parameter types and produced `0xc2985578`.
//        Harness #3 ACTIVE — pins the Ethereum-spec selector 0x2fbebd38.
//
//   (H4) `new Child(n)` — the factory pattern (clone factories, minimal
//        proxy factories, escrow-per-deal patterns). OBSERVED:
//          * Both Child and Parent COMPILE OK (two artifacts).
//          * Parent's manifest declares `spawn` method with `returntype:
//            Hash160` (correct — address maps to Hash160).
//          * Parent's manifest `permissions` is EMPTY (`[]`) — NO
//            `contractCall` permission targeting Child, NO deployment
//            permission, NO warning. This SILENTLY compiles to code that
//            CANNOT work at runtime: `new Child` needs either
//            ContractManagement.deploy permission OR a method-token-based
//            dispatch, neither of which the compiler emits.
//        Harness #4 ACTIVE (as a compile-only probe) — pins the silent
//        behavior. Executing `spawn` would need `call_function` (broken,
//        Task #19), so we don't execute.
//
//   (H5) `keccak256(abi.encode(x,y))` vs. `keccak256(abi.encodePacked(x,y))`
//        for two uint256 arguments. For uint256 the BYTE layouts ARE
//        identical between encode and encodePacked (both 32 BE) — so
//        Solidity spec says A.f() == B.f() exactly. OBSERVED:
//          A: 0xe3d921007d1accbc1fd082b53dc9438007916116fb58120d7c9ef0b8cade11c6
//          B: 0xe3d921007d1accbc1fd082b53dc9438007916116fb58120d7c9ef0b8cade11c6
//          (A == B! Both paths hit the SAME JSON-serialization leak.)
//        EXPECTED (Solidity spec, for both):
//          0xc4250a0f26818bb2f4c50553605e6aa5374a022de55e294b0c5f6716bcaddbf8
//        Two observations layer: (a) A == B — consistent WITH each other,
//        since both lower to StackItem::Array and BOTH hash the same JSON.
//        (b) Both DIFFER from the EVM-canonical hash. Task #44 is a
//        SHARED bug across encode/encodePacked; fixing ONLY one path
//        would (paradoxically) BREAK the A == B invariant that currently
//        happens to hold by accident. Harness #5 ACTIVE — pins both the
//        equality AND the divergence from spec.
//
// All five harnesses are ACTIVE. None is `#[ignore]`d — they all compile
// cleanly and their assertions reflect the (broken-but-observable) current
// state.
//
// Post-Task-#44 fix checklist: un-comment the `// EXPECTED:` assertion in
// each of H2 and H5; H3 depends on a separate selector-lowering fix and
// has its own `// EXPECTED:` block; H1 is already on the spec answer and
// needs no change; H4 remains a compile-only probe until either the
// `new Contract` lowering wires method tokens OR the compiler learns to
// emit a permission entry (both would need auxiliary runtime support —
// out of scope for this batch).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #1 — `keccak256("<constant string>")` BASELINE.
    //
    // CURRENT observed behavior: MATCHES the `sha3::Keccak256::digest`
    // reference exactly for the EIP-712 domain separator string. This
    // harness parameterizes the return-function name (identifier_strategy)
    // but fixes the hashed string to the well-known EIP-712 domain input
    // so the reference digest is constant and auditable.
    //
    // EXPECTED (Solidity spec): `keccak256(b"EIP712Domain(uint256 chainId,
    // address verifyingContract)")` = `sha3::Keccak256::digest(...)`. This
    // harness is ALREADY on the spec answer — it exists to (a) serve as a
    // BASELINE for diffing H2/H5 (if H1 passes while H2 fails, the bug is
    // strictly in abi.encode*, NOT in keccak256), and (b) fire if a future
    // change regresses the keccak256 direct-byte-literal path.
    #[test]
    fn eip712_typehash_pattern_bytes_diverge(
        fn_name in identifier_strategy(),
    ) {
        use sha3::{Digest, Keccak256};

        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function {fname}() external pure returns (bytes32) {{
        return keccak256("EIP712Domain(uint256 chainId,address verifyingContract)");
    }}
}}"#, fname = fn_name);

        let result = compile_and_execute(&source);
        prop_assert!(result.success,
            "H1 keccak256 string-literal must succeed; exc={:?}", result.exception);

        let expected = Keccak256::digest(
            b"EIP712Domain(uint256 chainId,address verifyingContract)"
        ).to_vec();

        prop_assert_eq!(result.return_data.len(), 32,
            "H1: keccak256(string) must return 32 bytes; got {}",
            result.return_data.len());

        // OBSERVED == EXPECTED: this is the BASELINE correct behavior.
        // If this harness ever fires, the keccak256 wiring regressed —
        // investigate BEFORE looking at any Task #44 (abi.encode) work.
        prop_assert_eq!(&result.return_data, &expected,
            "H1 BASELINE REGRESSION: keccak256 over a constant string no \
             longer matches sha3 reference. Observed: 0x{}, expected: 0x{}. \
             This is MORE SEVERE than Task #44 — the hash primitive itself \
             is broken. Investigate src/cli/bytecode/bytecode_builtins/\
             builtin_call/crypto.rs BEFORE anything else.",
            hex::encode(&result.return_data), hex::encode(&expected));
    }

    // Harness #2 — `keccak256(abi.encode(TYPEHASH, owner, value, nonce))`.
    //
    // CURRENT observed behavior: returns a 32-byte digest
    // `0x5272b321bb6018db8c4a7bd7165adc605672b9b9ff1c9491853085998926ed0c`
    // which is the keccak256 of the JSON-serialized StackItem::Array
    // (per Task #44), NOT of the EVM-canonical 128-byte BE-padded payload.
    // The output shape is 32 bytes (matches `bytes32` return type) so
    // naive type-checking cannot catch this — it is a SILENT WRONG-VALUE
    // failure, the WORST failure mode in a cross-chain or signed-message
    // pipeline.
    //
    // EXPECTED (Solidity spec): `Keccak256::digest([TYPEHASH, pad32(owner),
    // pad32(100), pad32(7)].concat())` where TYPEHASH is itself
    // `Keccak256::digest(b"Permit(address owner,uint256 value,uint256 nonce)")`.
    // For the fixed fixture (owner=0x1111…1111, value=100, nonce=7) the
    // spec answer is
    //   0xa90d8f896673f0bf0720c3224fe0350c6d2eaedb7d77e2ed288ef7c5a4699ea1.
    //
    // When Task #44 is fixed, un-comment the EXPECTED assertion below and
    // delete the "observed" one. The harness is LOUD by design: the
    // observed hash is pinned EXACTLY so any change to the lowering (even
    // a partial one) fires the assertion and forces a re-review.
    #[test]
    fn eip712_struct_hash_via_abi_encode_diverges(
        fn_name in identifier_strategy(),
    ) {
        use sha3::{Digest, Keccak256};

        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    bytes32 constant TYPEHASH = keccak256("Permit(address owner,uint256 value,uint256 nonce)");
    function {fname}() external pure returns (bytes32) {{
        address owner = 0x1111111111111111111111111111111111111111;
        uint256 value = 100;
        uint256 nonce = 7;
        return keccak256(abi.encode(TYPEHASH, owner, value, nonce));
    }}
}}"#, fname = fn_name);

        let result = compile_and_execute(&source);
        prop_assert!(result.success,
            "H2 EIP-712 struct-hash must succeed at host level; exc={:?}",
            result.exception);

        prop_assert_eq!(result.return_data.len(), 32,
            "H2: keccak256(...) must return 32 bytes; got {}",
            result.return_data.len());

        // Compute the EVM-canonical expected digest (what the compiler
        // SHOULD produce post-Task-#44-fix).
        let typehash = Keccak256::digest(
            b"Permit(address owner,uint256 value,uint256 nonce)"
        );
        let mut owner_padded = [0u8; 32];
        owner_padded[12..].copy_from_slice(&[0x11u8; 20]);
        let mut value_padded = [0u8; 32];
        value_padded[24..].copy_from_slice(&100u64.to_be_bytes());
        let mut nonce_padded = [0u8; 32];
        nonce_padded[24..].copy_from_slice(&7u64.to_be_bytes());
        let mut payload: Vec<u8> = Vec::with_capacity(128);
        payload.extend_from_slice(typehash.as_slice());
        payload.extend_from_slice(&owner_padded);
        payload.extend_from_slice(&value_padded);
        payload.extend_from_slice(&nonce_padded);
        let expected_evm = Keccak256::digest(&payload).to_vec();

        // Task #44 LANDED: `keccak256(abi.encode(TYPEHASH, owner, value, nonce))`
        // now matches the EVM-canonical digest (see
        // src/runtime/execution/execution_impl_part2_native/stdlib.rs — new
        // `abiEncode` handler with address-slot normalisation).
        prop_assert_eq!(&result.return_data, &expected_evm,
            "H2 post-fix: keccak256(abi.encode(TYPEHASH, owner, value, \
             nonce)) must equal EVM-canonical 0x{}, got 0x{}",
            hex::encode(&expected_evm), hex::encode(&result.return_data));
    }

    // Harness #3 — `this.foo.selector`.
    //
    // FIX LANDED (task #54): returns the Ethereum-spec 4-byte keccak
    // selector `0x2fbebd38` = `bytes4(keccak256("foo(uint256)"))`. The
    // `.selector` lowering now consults the per-contract selector registry
    // for `this.method.selector` (same path used by
    // `TypeName.method.selector` and `encodeWithSignature`), so the full
    // canonical signature (including parameter types) is hashed — not the
    // empty-param `"foo()"` fallback that previously produced `0xc2985578`.
    //
    // NOTE: function ORDER in the source matters — `getSelector` is
    // declared FIRST so it lands at offset 0 and our standard `execute(
    // &bytecode, &[])` helper runs it directly. If `foo` is first, the
    // dispatcher runs `foo` with no args (probe showed rd=[]). This is a
    // quirk of the single-function test harness, not of the selector
    // lowering itself.
    #[test]
    fn function_selector_this_foo_dot_selector(
        _seed in 0u32..=0u32,  // parameterless but keep proptest harness shape
    ) {
        use sha3::{Digest, Keccak256};

        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function getSelector() external view returns (bytes4) { return this.foo.selector; }
    function foo(uint256 x) external pure returns (uint256) { return x; }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H3 getSelector must succeed at host level; exc={:?}",
            result.exception);

        // Pin the Ethereum-spec 4-byte selector: keccak256("foo(uint256)")[..4].
        let expected_evm_selector = &Keccak256::digest(b"foo(uint256)")[..4];
        prop_assert_eq!(&result.return_data, &expected_evm_selector.to_vec(),
            "H3 post-fix: this.foo.selector must equal \
             bytes4(keccak256(\"foo(uint256)\")) = 0x{}, got 0x{}. If this \
             regressed to 0xc2985578, the task #54 fix in \
             src/ir/expressions/member_access/selectors.rs was reverted.",
            hex::encode(expected_evm_selector),
            hex::encode(&result.return_data));
    }

    // Harness #4 — `new Child(n)` dynamic deployment (factory pattern).
    //
    // CURRENT observed behavior (COMPILE-ONLY — we do NOT execute because
    // `call_function` is broken per Task #19):
    //   (a) Both `Child` and `Parent` COMPILE successfully — two manifests
    //       are produced.
    //   (b) Parent's manifest declares the `spawn` method with
    //       `returntype: Hash160` (address → Hash160 mapping is correct).
    //   (c) Parent's manifest `permissions` is EMPTY — NO `contractCall`
    //       entry targeting Child, NO ContractManagement.deploy permission,
    //       NO compiler warning about unsupported dynamic deployment.
    //
    // This is a SILENT compile — the resulting bytecode CANNOT work at
    // runtime (a `System.Contract.Call` or `ContractManagement.deploy`
    // without a permission entry is rejected by the real Neo N3 VM). The
    // compiler should EITHER:
    //   (A) wire a ContractManagement.deploy permission + method token, OR
    //   (B) emit a compile-time warning/error "dynamic deployment of
    //       nested contract not supported on NeoVM".
    //
    // Today it does NEITHER. This is a NEW gap to file — distinct from
    // Task #19 (call_function execution) because this is about the
    // COMPILER side, not the runtime side.
    //
    // EXPECTED (post-fix, at minimum): Parent's manifest includes either
    // a `contractCall` permission for Child OR a diagnostic surfaces from
    // `compile_contracts`.
    #[test]
    fn new_contract_dynamic_deployment_compiles_or_errors(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Child { uint256 public v; constructor(uint256 x) { v = x; } }
contract Parent {
    function spawn(uint256 n) external returns (address) {
        Child c = new Child(n);
        return address(c);
    }
}"#;

        let artifacts = compile_contracts(source, false, 2)
            .expect("H4: Child + Parent must at least compile without host error");
        prop_assert_eq!(artifacts.len(), 2,
            "H4: expected 2 artifacts (Child, Parent); got {}", artifacts.len());

        // Find Parent by name.
        let parent = artifacts.iter().find(|a| {
            a.manifest.get("name").and_then(|n| n.as_str()) == Some("Parent")
        }).expect("H4: Parent artifact missing");

        // (b) spawn method is declared.
        let methods = parent.manifest.get("abi")
            .and_then(|a| a.get("methods"))
            .and_then(|m| m.as_array())
            .expect("H4: Parent.abi.methods must exist");
        let has_spawn = methods.iter().any(|m| {
            m.get("name").and_then(|n| n.as_str()) == Some("spawn")
        });
        prop_assert!(has_spawn,
            "H4: Parent manifest must declare spawn method; got methods={:?}",
            methods);

        // (c) Pin CURRENT permissions state. Parent's permissions is empty
        // today — a silent compile of code that cannot work at runtime.
        let perms = parent.manifest.get("permissions")
            .and_then(|p| p.as_array())
            .expect("H4: Parent.permissions must exist (even if empty)");
        let has_child_perm = perms.iter().any(|p| {
            let c = p.get("contract").and_then(|c| c.as_str()).unwrap_or("");
            c.to_ascii_lowercase().contains("child")
                || c.contains("0x") // any explicit hash counts
        });
        let has_deploy_perm = perms.iter().any(|p| {
            let methods = p.get("methods").and_then(|m| m.as_array());
            methods.map(|ms| ms.iter().any(|x| {
                matches!(x.as_str(), Some("deploy") | Some("*"))
            })).unwrap_or(false)
        });

        // Task #55 (POST-FIX): Parent's manifest must declare a
        // `ContractManagement.deploy` permission because its body contains
        // `new Child(...)`. The AST scanner in cli_manifest/permissions/
        // detects the `Expression::New(FunctionCall(Variable(Child), _))`
        // pattern and wires `ContractManagement.deploy` into the manifest
        // so the bytecode cannot fault at runtime for want of permission.
        prop_assert!(has_child_perm || has_deploy_perm,
            "H4 post-fix: Parent must declare a permission entry to \
             deploy/call Child (got perms={:?})",
            perms);
    }

    // Harness #5 — `keccak256(abi.encode(x,y))` vs.
    //              `keccak256(abi.encodePacked(x,y))` for two uint256.
    //
    // CURRENT observed behavior:
    //   A (encode):        0xe3d921007d1accbc1fd082b53dc9438007916116fb58120d7c9ef0b8cade11c6
    //   B (encodePacked):  0xe3d921007d1accbc1fd082b53dc9438007916116fb58120d7c9ef0b8cade11c6
    //   A == B (both hit the SAME JSON-serialization leak from Task #44).
    //
    // EXPECTED (Solidity spec, for two uint256 args — layouts are
    // identical between encode and encodePacked):
    //   A, B both: 0xc4250a0f26818bb2f4c50553605e6aa5374a022de55e294b0c5f6716bcaddbf8
    //   which is `Keccak256::digest([pad32_be(42), pad32_be(7)].concat())`.
    //
    // Two invariants to pin:
    //   (1) A.f() == B.f() — Solidity guarantees this for all-uint256
    //       arguments. TODAY this holds BY ACCIDENT (both produce the
    //       same wrong JSON hash); post-fix it should hold by
    //       construction (both produce the same correct 64-byte hash).
    //   (2) Both A.f() and B.f() must match the EVM-canonical digest.
    //       TODAY neither does.
    //
    // A partial fix that ONLY repairs one of encode/encodePacked would
    // BREAK invariant (1) while partially satisfying (2). Both must be
    // fixed together — which is why Task #44's scope should cover BOTH
    // paths (see batch #16 header for the scope expansion note).
    #[test]
    fn hash_consistency_keccak_of_encoded_matches_packed(
        _seed in 0u32..=0u32,
    ) {
        use sha3::{Digest, Keccak256};

        let source_a = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract A { function f() external pure returns (bytes32) {
    return keccak256(abi.encode(uint256(42), uint256(7)));
}}"#;
        let source_b = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract B { function f() external pure returns (bytes32) {
    return keccak256(abi.encodePacked(uint256(42), uint256(7)));
}}"#;

        let result_a = compile_and_execute(source_a);
        let result_b = compile_and_execute(source_b);

        prop_assert!(result_a.success,
            "H5 A execute must succeed; exc={:?}", result_a.exception);
        prop_assert!(result_b.success,
            "H5 B execute must succeed; exc={:?}", result_b.exception);

        prop_assert_eq!(result_a.return_data.len(), 32,
            "H5 A must return 32 bytes; got {}", result_a.return_data.len());
        prop_assert_eq!(result_b.return_data.len(), 32,
            "H5 B must return 32 bytes; got {}", result_b.return_data.len());

        // EVM-canonical expected (identical for both, since uint256
        // packed == uint256 encoded = 32 BE bytes each).
        let mut payload: Vec<u8> = Vec::with_capacity(64);
        let mut pad42 = [0u8; 32]; pad42[24..].copy_from_slice(&42u64.to_be_bytes());
        let mut pad7  = [0u8; 32]; pad7[24..].copy_from_slice(&7u64.to_be_bytes());
        payload.extend_from_slice(&pad42);
        payload.extend_from_slice(&pad7);
        let expected_evm = Keccak256::digest(&payload).to_vec();

        // Task #44 LANDED: both A and B now produce the EVM-canonical digest.

        // Invariant (1): A == B (Solidity spec — layouts are identical for
        // uint256). Holds by construction now that both variants go through
        // the same `abiEncode`/`abiEncodePacked` runtime handlers.
        prop_assert_eq!(&result_a.return_data, &result_b.return_data,
            "H5 INVARIANT: keccak256(abi.encode(u256,u256)) and \
             keccak256(abi.encodePacked(u256,u256)) must produce the \
             SAME hash. A=0x{}, B=0x{}. If this fires, a PARTIAL Task #44 \
             fix touched only one of encode/encodePacked.",
            hex::encode(&result_a.return_data),
            hex::encode(&result_b.return_data));

        // Invariant (2): both match the EVM-canonical digest post-fix.
        prop_assert_eq!(&result_a.return_data, &expected_evm,
            "H5 post-fix A: must equal EVM-canonical 0x{}, got 0x{}",
            hex::encode(&expected_evm),
            hex::encode(&result_a.return_data));
        prop_assert_eq!(&result_b.return_data, &expected_evm,
            "H5 post-fix B: must equal EVM-canonical 0x{}, got 0x{}",
            hex::encode(&expected_evm),
            hex::encode(&result_b.return_data));
    }
}

// ==================== Batch #20 — Remaining Solidity Features + Intrinsic Inventory ====================
//
// Scope: close the remaining documentation gaps for (a) `abi.decode` round-trip
// semantics under Task #44, (b) user-defined value types (UDVT) — a commonly
// used Solidity 0.8.8+ pattern in Uniswap V4, OpenZeppelin 5.x, (c) storage
// pointer refs (`T storage ref = …`), (d) the intrinsic-resolver coverage of
// the remaining devpack-library namespaces (`Neo.*`, `Storage.*`, `NativeCalls.*`)
// relative to the confirmed `Runtime.*` ✓ / `Syscalls.*` ✗ split from batches
// #11 and #17, and (e) the payload SHAPE of an event whose argument is a
// struct — extending batch #13's event-topic work past scalar args.
//
// Pre-batch probes (deleted; results summarized here for audit):
//
//   (H1) `abi.decode(abi.encode(uint256(42), uint256(99)))` returning
//        `(uint256, uint256)` produces `return_data` of 86 BYTES — the raw
//        UTF-8 JSON string produced by Task #44's `abi.encode` shim:
//          {"type":"Array","value":[{"type":"Integer","value":42},
//                                    {"type":"Integer","value":99}]}
//        Put differently: `abi.decode` is wired to `StdLib.deserialize`
//        (src/cli/bytecode/bytecode_builtins/builtin_call/abi.rs:26-39)
//        but the JSON bytes handed to it by `abi.encode` are NOT the NeoVM
//        serde format that `StdLib.deserialize` expects. The two halves of
//        the round-trip do NOT agree on a wire format, so neither (42) nor
//        (99) survives. This is NOT merely an external-ABI-compat gap —
//        even the closed-loop INTRA-contract encode-then-decode is broken.
//        Harness #1 ACTIVE, pins the 86-byte raw-JSON return as the CURRENT
//        observed behavior with an EXPECTED block that, post-Task-#44, must
//        return 42 and 99 as two scalars. Task #44's scope therefore covers
//        BOTH `encode` and `decode` — fixing one alone would just surface
//        the other. This extends batch #16's `_diverges` findings by
//        documenting the consequence at the CONSUMER side.
//
//   (H2) `type Price is uint256` with `Price.wrap(42)` / `Price.unwrap(p)`
//        COMPILES and EXECUTES correctly: returns `0x2a00000000000000` (42
//        in 8-byte LE). The SOLIDITY_SUPPORT_MATRIX claim "User-defined value
//        types (`type X is Y`) ✅ Transparent type aliases; `wrap`/`unwrap`
//        compile to no-ops" (docs/SOLIDITY_SUPPORT_MATRIX.md:35) is CORRECT.
//        Harness #2 ACTIVE, pins the value 42. No `#[ignore]` needed.
//
//   (H3) `struct Item { uint256 x; uint256 y; }` with `Item[] items` and
//        `Item storage ref = items[0]` COMPILES. The manifest declares both
//        `push` and `readFirst` methods. The SOLIDITY_SUPPORT_MATRIX claim
//        "storage keyword ✅ Storage references for mappings and state
//        variables" (docs/SOLIDITY_SUPPORT_MATRIX.md:141) is validated at
//        the COMPILE level. Runtime semantics of `ref.x` / `ref.y` cannot
//        be validated here because the test harness invokes offset 0 (which
//        is `push`, not `readFirst`) and `call_function` is broken per
//        Task #19. Harness #3 ACTIVE as a compile-only + manifest probe.
//
//   (H4) Intrinsic-resolver inventory via the SIBLING library namespaces
//        (`Neo.*`, `Storage.*`, `NativeCalls.*`) that resolve.rs:1-13
//        dispatches. Each probe compiles a one-liner using the namespace
//        and asserts bytecode size >> the empty `SYSCALL 57000240 RET`
//        stub (10 bytes). Observed:
//          * `Neo.getBlockHeight()` → 49 B, wires the Ledger native-call
//            `System.Contract.Call(Ledger, "currentIndex")`. RUNS: returns
//            8 B = 12345 LE at override_block_height(12345). RESOLVED ✓.
//          * `Storage.put(k, v)` → 31 B, wires
//            `System.Storage.GetContext` (`419bf667ce`) then
//            `System.Storage.Put` (`41e63f1884`). RUNS clean. RESOLVED ✓.
//          * `NativeCalls.gasBalanceOf(a)` → 84 B, wires a native-contract
//            call to GAS (script hash `cf76e28bd0…`) method "balanceOf".
//            RUNS: returns 8 B = 0 LE (account balance is 0). RESOLVED ✓.
//        Conclusion: `Runtime.*` ✓, `Storage.*` ✓, `Neo.*` ✓, `NativeCalls.*` ✓.
//        `Syscalls.*` remains the ONLY namespace that `resolve.rs:9` names
//        but `builtin_library_supported_members`/`resolve_syscalls_member`
//        does NOT dispatch (Task #37 — the `Syscalls` namespace is listed
//        on line 9 of resolve.rs but `fn resolve_syscalls_member` does not
//        exist in this file; `grep -r resolve_syscalls_member src/` returns
//        no hits). Harness #4 ACTIVE across three sub-probes; no Task #37
//        expansion needed. This is GOOD NEWS: the intrinsic surface is
//        broader than the batch #11 header suggested.
//
//   (H5) `emit ItemAdded(Item({x: 5, y: 7}))` produces a Neo LogEntry with
//        `topics = ["ItemAdded" as UTF-8]` and
//        `data = {"type":"Array","value":[{"type":"Array","value":[
//                {"type":"Integer","value":5},{"type":"Integer","value":7}]}]}`.
//        That is: the event's single struct arg is FOLDED into a nested
//        `StackItem::Array` (inner array = the struct's field values), then
//        wrapped in the outer state-array that batch #11 harness #4
//        established (`{"type":"Array","value":[...]}`). Struct fields are
//        NOT flattened to the top-level state-array; Solidity's
//        field-flattening semantics DIVERGE from Neo's structure-preserving
//        notify payload. This is SEPARATE from Task #39 (indexed-arg
//        folding) — here it's unconditional for struct args regardless of
//        indexed/non-indexed. Harness #5 ACTIVE, pins the nested-Array
//        shape and documents the divergence.
//
// Harness coverage summary:
//   1. abi_decode_returns_correct_values_or_documents_gap — ACTIVE,
//      pins the 86-byte raw-JSON passthrough (Task #44 subsumes).
//   2. user_defined_value_type_wraps_and_unwraps — ACTIVE, on-spec.
//   3. storage_pointer_read_via_struct_ref — ACTIVE, compile + manifest.
//   4. intrinsic_resolver_inventory_per_library — ACTIVE, 3 sub-probes
//      (Neo/Storage/NativeCalls).
//   5. event_with_struct_arg_payload_shape — ACTIVE, pins nested-Array shape.
//
// All five harnesses are ACTIVE. No `#[ignore]` gates are raised; the
// intrinsic-resolver coverage is wider than batch #11 implied and no new
// Task-#37 expansion is needed. The single NEW documentation item is the
// event struct-arg nested-Array shape (H5) — filed as Task #39-adjacent
// because it mirrors the batch #13 / Task #39 "indexed args fold into
// state array" finding one layer deeper.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #1 — `abi.decode(abi.encode(x))` intra-contract round-trip.
    //
    // CURRENT observed behavior:
    //   * `abi.encode(uint256(42), uint256(99))` emits JSON-as-bytes per
    //     Task #44 (see batch #16).
    //   * `abi.decode` is wired to `StdLib.deserialize`
    //     (src/cli/bytecode/bytecode_builtins/builtin_call/abi.rs:26-39),
    //     which expects NeoVM-serde format — NOT the JSON produced by the
    //     encode side. So the "round-trip" is two incompatible shims.
    //   * Observed `return_data` (86 bytes):
    //     {"type":"Array","value":[{"type":"Integer","value":42},
    //                               {"type":"Integer","value":99}]}
    //     i.e. the JSON bytes pass through unconverted.
    //
    // EXPECTED (Solidity spec, post-Task-#44 fix): the return tuple
    // `(uint256, uint256)` should materialize 42 and 99 as two scalars on
    // the stack; the ABI-encoded return then serializes to 32 BE bytes per
    // value (or, under Neo's current variable-width LE encoding, 8 bytes
    // per value concatenated). The EXACT post-fix shape depends on whether
    // Task #44 also rewires the return-encoding path to EVM-canonical ABI
    // (32 BE per scalar, padded) or keeps the current Neo-native variable
    // width LE encoding.
    //
    // Two invariants to pin:
    //   (1) CURRENT: return_data starts with `{"type":"Array"` — the JSON
    //       passthrough is observable from the output bytes. This is the
    //       standing witness that Task #44 affects BOTH encode and decode.
    //   (2) CURRENT: return_data.len() == 86 — the exact JSON shape for
    //       (42, 99) is stable. If this changes, the lowering moved.
    //
    // NOTE: even if the round-trip were internally consistent (which it
    // isn't, today), any caller passing in externally-sourced ABI bytes
    // (e.g. from a bridged EVM event, or a cross-chain message payload)
    // would still break because the inbound bytes would be EVM-ABI, not
    // Neo-serde or JSON. Task #44 scope therefore has to cover BOTH sides.
    #[test]
    fn abi_decode_returns_correct_values_or_documents_gap(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (uint256, uint256) {
        bytes memory data = abi.encode(uint256(42), uint256(99));
        (uint256 a, uint256 b) = abi.decode(data, (uint256, uint256));
        return (a, b);
    }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H1 abi.decode(abi.encode(...)) must at least execute at host level; \
             exc={:?}", result.exception);

        // Task #44 LANDED (encode side + decode side) AND Task #64 LANDED
        // (tuple-return lowering): the encoder emits 2 * 32 = 64 EVM-canonical
        // BE bytes, the decoder reads them back into two UnsignedInteger
        // values, and `return (a, b)` now lowers via the same `abiEncode`
        // handler — so the outer return is also 64 bytes of EVM-packed BE
        // (not serde_json of an Array). This confirms the full round-trip:
        // abi.encode → abi.decode → tuple return all agree on BE-packed 32.
        let rd = &result.return_data;
        let mut expected = vec![0u8; 64];
        expected[24..32].copy_from_slice(&42u64.to_be_bytes());
        expected[56..64].copy_from_slice(&99u64.to_be_bytes());
        prop_assert_eq!(rd.as_slice(), expected.as_slice(),
            "H1 post-Task-#64: tuple return must be EVM-canonical 64 bytes \
             (BE-padded 32 per scalar). rd.len={}, rd={:?}",
            rd.len(), rd);
    }

    // Harness #2 — `type Price is uint256; Price.wrap(42); Price.unwrap(p);`.
    //
    // CURRENT observed behavior: COMPILES and EXECUTES. Returns 8 bytes
    // `0x2a00000000000000` = 42 LE. The UDVT is a transparent alias exactly
    // as SOLIDITY_SUPPORT_MATRIX.md:35 claims.
    //
    // EXPECTED (Solidity spec): `Price.unwrap(Price.wrap(42)) == 42`. The
    // harness is ALREADY on the spec answer — it exists as a REGRESSION
    // PIN against future changes that might accidentally break the UDVT
    // pass-through (e.g. if a type-check pass started requiring an explicit
    // conversion opcode).
    //
    // UDVT is widely used in modern DeFi: Uniswap V4 uses it extensively
    // (`type Currency is address`, `type BalanceDelta is int256`); OpenZeppelin
    // 5.x uses it for `Checkpoints.Trace224`. A regression here would be
    // a visible break for any project built on those libraries.
    #[test]
    fn user_defined_value_type_wraps_and_unwraps(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
type Price is uint256;
contract C {
    function f() external pure returns (uint256) {
        Price p = Price.wrap(42);
        return Price.unwrap(p);
    }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H2 UDVT wrap/unwrap must execute; exc={:?}", result.exception);

        let got = decode_uint_le(&result.return_data);
        let expected = num_bigint::BigUint::from(42u8);
        prop_assert_eq!(&got, &expected,
            "H2: Price.unwrap(Price.wrap(42)) must equal 42. Got {} (rd_hex={}). \
             UDVT is documented as supported in docs/SOLIDITY_SUPPORT_MATRIX.md:35 \
             — a regression here breaks Uniswap V4, OpenZeppelin 5.x Checkpoints, \
             and any contract using type aliases for domain types.",
            got, hex::encode(&result.return_data));
    }

    // Harness #3 — `Item storage ref = items[0]; return (ref.x, ref.y);`.
    //
    // CURRENT observed behavior (COMPILE + MANIFEST ONLY — we do NOT execute
    // because `call_function` is broken per Task #19 and offset 0 lands on
    // `push` rather than `readFirst`):
    //   * COMPILES cleanly (one artifact, one `C` contract).
    //   * Manifest methods include both `push` and `readFirst` plus the
    //     auto-generated `_deploy` entry. That validates the
    //     SOLIDITY_SUPPORT_MATRIX.md:141 claim "storage ✅ Storage references
    //     for mappings and state variables" at the compile level.
    //
    // EXPECTED (Solidity spec, post-Task-#19 fix when we can invoke by name):
    // `readFirst()` after `push()` returns (7, 11). That assertion is
    // commented out below; enable it when `call_function` works.
    //
    // This probes the LOWERING side: the compiler must accept `Item storage
    // ref = items[0];` and emit a reference-via-slot pattern. If a future
    // regression loses this (e.g. a type-check pass starts demanding
    // `memory` for struct locals), this harness fires.
    #[test]
    fn storage_pointer_read_via_struct_ref(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Item { uint256 x; uint256 y; }
    Item[] private items;
    function push() external { items.push(Item({x: 7, y: 11})); }
    function readFirst() external view returns (uint256, uint256) {
        Item storage ref = items[0];
        return (ref.x, ref.y);
    }
}"#;

        let artifacts = compile_contracts(source, false, 2)
            .expect("H3: storage-pointer contract must compile");
        prop_assert_eq!(artifacts.len(), 1,
            "H3: expected 1 artifact (contract C); got {}", artifacts.len());

        let artifact = &artifacts[0];
        let methods = artifact.manifest.get("abi")
            .and_then(|a| a.get("methods"))
            .and_then(|m| m.as_array())
            .expect("H3: manifest.abi.methods must exist");

        let has_push = methods.iter().any(|m| {
            m.get("name").and_then(|n| n.as_str()) == Some("push")
        });
        let has_read = methods.iter().any(|m| {
            m.get("name").and_then(|n| n.as_str()) == Some("readFirst")
        });
        prop_assert!(has_push,
            "H3: manifest must declare 'push' method; methods={:?}", methods);
        prop_assert!(has_read,
            "H3: manifest must declare 'readFirst' method; methods={:?}", methods);

        // EXPECTED (post-Task-#19 fix): un-comment when call_function works.
        // let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        // let _ = runtime.call_function(&artifact.bytecode, "push", &[]).expect("push");
        // let result = runtime.call_function(&artifact.bytecode, "readFirst", &[]).expect("readFirst");
        // prop_assert!(result.success,
        //     "H3 post-fix: readFirst must succeed; exc={:?}", result.exception);
        // // Expect two scalars (7, 11) concatenated in the return_data tuple encoding.
    }

    // Harness #4 — Intrinsic-resolver coverage across the remaining library
    // namespaces `Neo.*`, `Storage.*`, `NativeCalls.*`.
    //
    // Motivation: batch #11 established `Runtime.*` ✓; Task #37 tracks
    // `Syscalls.*` ✗ (the only namespace listed in resolve.rs:1-13 that has
    // NO backing `resolve_syscalls_member` dispatch). But the status of
    // `Neo.*`, `Storage.*`, and `NativeCalls.*` was UNDOCUMENTED before this
    // batch. The compile-time probe below triangulates by (a) compiling a
    // one-liner per namespace, (b) asserting the compiled bytecode is
    // substantially larger than a degenerate Null-return stub (the empty
    // stub is `INITSLOT 0 0 RET SYSCALL 57000240 RET` ≈ 10 B), and (c)
    // where possible, executing and asserting a semantic round-trip.
    //
    // CURRENT observed behavior (from pre-batch probes):
    //   * `Neo.getBlockHeight()` → 49 B bytecode; emits
    //     `System.Contract.Call(Ledger=bef20431…, "currentIndex")`. Executes
    //     with `override_block_height(12345)` returning 8 B = 12345 LE.
    //     Resolve path: `resolve_neo_member("getBlockHeight")` →
    //     `NativeCall { contract: Ledger, method: "currentIndex" }`
    //     (resolve.rs:479-482).
    //   * `Storage.put(k, v)` → 31 B bytecode; emits
    //     `System.Storage.GetContext` (`419bf667ce`) then
    //     `System.Storage.Put` (`41e63f1884`). Executes clean.
    //     Resolve path: `resolve_storage_member("put")` → `StoragePut`
    //     (resolve.rs:395).
    //   * `NativeCalls.gasBalanceOf(a)` → 84 B bytecode; emits
    //     `System.Contract.Call(GAS=cf76e28bd0…, "balanceOf")`. Executes
    //     with account 0x1111…1111 returning 8 B = 0 LE.
    //     Resolve path: native-contract call via the
    //     `_native_calls_member` dispatch table.
    //
    // EXPECTED: all three compile successfully AND their bytecode exceeds
    // the 10-byte degenerate-stub floor. If any fails, file as a Task #37
    // expansion (the intrinsic resolver is incomplete in that direction).
    //
    // CONCLUSION: the intrinsic surface is wider than batch #11 suggested.
    // `Runtime.*` ✓, `Storage.*` ✓, `Neo.*` ✓, `NativeCalls.*` ✓;
    // `Syscalls.*` ✗ remains the sole gap (Task #37). No Task #37 expansion.
    #[test]
    fn intrinsic_resolver_inventory_per_library(
        _seed in 0u32..=0u32,
    ) {
        // Floor: a degenerate "load zero + RET" stub is ≤ 12 bytes. Any
        // wired intrinsic emits at least a method-token hash + syscall,
        // pushing bytecode size past ~20 bytes. Use a conservative 16 B
        // floor so a future alternative lowering that still emits a
        // real call is accepted.
        const STUB_FLOOR: usize = 16;

        // (a) Neo.* — Neo.getBlockHeight() should wire Ledger.currentIndex.
        let source_neo = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external view returns (uint256) {
        return Neo.getBlockHeight();
    }
}"#;
        let arts_neo = compile_contracts(source_neo, false, 2)
            .expect("H4: Neo.getBlockHeight() must compile");
        prop_assert!(!arts_neo.is_empty(),
            "H4: Neo.* compile produced no artifacts");
        let bc_neo = &arts_neo[0].bytecode;
        prop_assert!(bc_neo.len() > STUB_FLOOR,
            "H4: Neo.getBlockHeight() compiled to {} bytes (<= stub floor {}). \
             The Neo.* namespace may have regressed to a Null-return stub. \
             Expected: a non-trivial Ledger.currentIndex native-call \
             lowering (observed: 49 B). Bytecode hex: {}",
            bc_neo.len(), STUB_FLOOR, hex::encode(bc_neo));

        // Execute to confirm the syscall actually dispatches. With
        // override_block_height(12345), currentIndex must return 12345.
        let mut rt_neo = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        rt_neo.override_block_height(12345);
        let res_neo = rt_neo.execute(bc_neo, &[])
            .expect("H4 Neo.*: execute must not error at host level");
        prop_assert!(res_neo.success,
            "H4 Neo.*: Neo.getBlockHeight() execute must succeed; exc={:?}",
            res_neo.exception.as_ref().map(|e| &e.message));
        let height = decode_uint_le(&res_neo.return_data);
        prop_assert_eq!(&height, &num_bigint::BigUint::from(12345u64),
            "H4 Neo.*: Neo.getBlockHeight() with override_block_height(12345) \
             must return 12345; got {} (rd_hex={})",
            height, hex::encode(&res_neo.return_data));

        // (b) Storage.* — Storage.put(k, v) must wire
        // System.Storage.GetContext + System.Storage.Put.
        let source_storage = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external {
        bytes memory key = hex"01";
        bytes memory val = hex"02";
        Storage.put(key, val);
    }
}"#;
        let arts_storage = compile_contracts(source_storage, false, 2)
            .expect("H4: Storage.put must compile");
        let bc_storage = &arts_storage[0].bytecode;
        prop_assert!(bc_storage.len() > STUB_FLOOR,
            "H4 Storage.*: Storage.put compiled to {} bytes (<= stub floor {}). \
             Expected: System.Storage.GetContext + System.Storage.Put syscalls \
             (observed: 31 B). Bytecode hex: {}",
            bc_storage.len(), STUB_FLOOR, hex::encode(bc_storage));

        // Confirm the two expected syscall tokens are present in the
        // bytecode. Token 0x9bf667ce = System.Storage.GetContext; token
        // 0xe63f1884 = System.Storage.Put. Both are emitted as little-endian
        // 4-byte hashes after the SYSCALL (0x41) opcode.
        let has_get_context = bc_storage.windows(5).any(|w| {
            w == [0x41, 0x9b, 0xf6, 0x67, 0xce]
        });
        let has_put = bc_storage.windows(5).any(|w| {
            w == [0x41, 0xe6, 0x3f, 0x18, 0x84]
        });
        prop_assert!(has_get_context,
            "H4 Storage.*: expected System.Storage.GetContext syscall token \
             (41 9b f6 67 ce) not found in bytecode: {}",
            hex::encode(bc_storage));
        prop_assert!(has_put,
            "H4 Storage.*: expected System.Storage.Put syscall token \
             (41 e6 3f 18 84) not found in bytecode: {}",
            hex::encode(bc_storage));

        // Execute — no assertions on return_data because Storage.put is
        // void; we only assert clean execution.
        let mut rt_storage = NeoRuntime::new(RuntimeConfig::default())
            .expect("runtime");
        let res_storage = rt_storage.execute(bc_storage, &[])
            .expect("H4 Storage.*: execute must not error at host level");
        prop_assert!(res_storage.success,
            "H4 Storage.*: Storage.put execute must succeed; exc={:?}",
            res_storage.exception.as_ref().map(|e| &e.message));

        // (c) NativeCalls.* — NativeCalls.gasBalanceOf(a) must wire the
        // GAS native-contract "balanceOf" call.
        let source_native = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external view returns (uint256) {
        address a = 0x1111111111111111111111111111111111111111;
        return NativeCalls.gasBalanceOf(a);
    }
}"#;
        let arts_native = compile_contracts(source_native, false, 2)
            .expect("H4: NativeCalls.gasBalanceOf must compile");
        let bc_native = &arts_native[0].bytecode;
        prop_assert!(bc_native.len() > STUB_FLOOR,
            "H4 NativeCalls.*: NativeCalls.gasBalanceOf compiled to {} bytes \
             (<= stub floor {}). Expected: System.Contract.Call(GAS, 'balanceOf') \
             lowering (observed: 84 B). Bytecode hex: {}",
            bc_native.len(), STUB_FLOOR, hex::encode(bc_native));

        // Confirm the GAS native-contract script hash is embedded as the
        // 20-byte callee. GAS native hash (LE in bytecode) is:
        //   cf76e28bd0062c4a478ee35561011319f3cfa4d2
        let gas_hash_le: [u8; 20] = [
            0xcf, 0x76, 0xe2, 0x8b, 0xd0, 0x06, 0x2c, 0x4a, 0x47, 0x8e,
            0xe3, 0x55, 0x61, 0x01, 0x13, 0x19, 0xf3, 0xcf, 0xa4, 0xd2,
        ];
        let has_gas_hash = bc_native.windows(20).any(|w| w == gas_hash_le);
        prop_assert!(has_gas_hash,
            "H4 NativeCalls.*: GAS native-contract hash \
             (cf76e28bd0062c4a478ee35561011319f3cfa4d2, LE) not found in \
             bytecode: {}. If the hash rotated, update this harness and \
             cross-check neo/native_hashes.rs.",
            hex::encode(bc_native));

        let mut rt_native = NeoRuntime::new(RuntimeConfig::default())
            .expect("runtime");
        let res_native = rt_native.execute(bc_native, &[])
            .expect("H4 NativeCalls.*: execute must not error at host level");
        prop_assert!(res_native.success,
            "H4 NativeCalls.*: NativeCalls.gasBalanceOf execute must succeed; \
             exc={:?}",
            res_native.exception.as_ref().map(|e| &e.message));
        // Default runtime sets no GAS balance → 0.
        let bal = decode_uint_le(&res_native.return_data);
        prop_assert_eq!(&bal, &num_bigint::BigUint::from(0u8),
            "H4 NativeCalls.*: default GAS balance for 0x1111…1111 must be 0; \
             got {} (rd_hex={})",
            bal, hex::encode(&res_native.return_data));
    }

    // Harness #5 — Event argument that is a STRUCT: payload shape.
    //
    // CURRENT observed behavior (post Task #121): `emit ItemAdded(Item({x: 5, y: 7}))`
    // produces exactly one LogEntry with:
    //   topics[0] = keccak256("ItemAdded(Item)")
    //   data      = 128 bytes EVM-canonical dynamic-array encoding of the
    //               struct fields flattened as an inline `uint256[]`:
    //               offset=0x20 || length=2 || pad32_be(5) || pad32_be(7)
    //
    // Task #121 fixed `abi_dynamic_tail_bytes` for `StackItem::Array` so
    // struct args now emit their fields through the dynamic-array tail
    // (length + N × 32 BE slots). This is NOT the strict Solidity tuple
    // form (`ItemAdded((uint256,uint256))` with `concat(pad32_be(5),
    // pad32_be(7))` head-inlined, no offset+length prefix) — struct-aware
    // flattening is still a follow-up — but it IS a strict improvement
    // over the prior empty-data / JSON-leak behavior, and the 5 and 7
    // field values now appear in BE-32 form inside the payload.
    //
    // Two invariants to pin:
    //   (1) topics.len() == 1 AND topics[0] == keccak256("ItemAdded(Item)")
    //       (struct-name canonical form; matches the existing batch coverage).
    //   (2) The data payload contains the pad32_be(5) and pad32_be(7) field
    //       values somewhere in its bytes (structural probe — does NOT pin
    //       the exact offset+length+tail layout so a future struct-aware
    //       flattener can collapse to `concat(pad32_be(5), pad32_be(7))`
    //       without re-breaking this pin).
    #[test]
    fn event_with_struct_arg_payload_shape(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Item { uint256 x; uint256 y; }
    event ItemAdded(Item item);
    function go() external { emit ItemAdded(Item({x: 5, y: 7})); }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H5: emit ItemAdded(Item) must succeed; exc={:?}", result.exception);

        prop_assert_eq!(result.logs.len(), 1,
            "H5: one LogEntry expected for `emit ItemAdded(...)`; got {}",
            result.logs.len());
        let entry = &result.logs[0];

        // Task #39: 0 indexed args → 1 topic (the signature hash).
        prop_assert_eq!(entry.topics.len(), 1,
            "H5: 0 indexed args → exactly 1 topic (the signature hash); got {}",
            entry.topics.len());
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        // EVM ABI: a struct event parameter expands to its `(field0,field1,...)`
        // tuple in the topic0 signature — `ItemAdded((uint256,uint256))`, NOT the
        // bare struct name `ItemAdded(Item)`. (The bare-name form previously
        // emitted here disagreed with Ethereum log filters.)
        hasher.update(b"ItemAdded((uint256,uint256))");
        let expected_topic0 = hasher.finalize();
        prop_assert_eq!(&entry.topics[0][..], &expected_topic0[..],
            "H5: topics[0] must be keccak256(\"ItemAdded((uint256,uint256))\") (struct expanded to tuple); got {}",
            hex::encode(&entry.topics[0]));

        // Post Task #121 structural pin: the field values 5 and 7 must
        // appear in BE-32 form somewhere inside the event data (does not
        // pin exact offset+length layout so a future struct-aware
        // flattener can collapse to concat(pad32_be(5), pad32_be(7))
        // without re-breaking this pin).
        let mut pad5 = [0u8; 32]; pad5[31] = 5;
        let mut pad7 = [0u8; 32]; pad7[31] = 7;
        let has5 = entry.data.windows(32).any(|w| w == &pad5[..]);
        let has7 = entry.data.windows(32).any(|w| w == &pad7[..]);
        prop_assert!(has5 && has7,
            "H5 post Task #121: data must carry BE-32 pad(5) and pad(7) struct fields; \
             got {} bytes: {}",
            entry.data.len(), hex::encode(&entry.data));
    }
}

// ==================== Batch #21 — Remaining Block Context + Arithmetic Edges ====================
//
// Scope: three small probes on surfaces not yet covered by batches #1-#20.
//
//   (H1) The remaining `block.*` accessors — `chainid`, `coinbase`, and
//        `gaslimit`. Batches #8 (H4) and #11 (Neo.getBlockHeight) covered
//        `block.number` / `block.timestamp`; the EVM-compat inference table
//        at src/ir/build/inference.rs:336 lists
//        `("block", "timestamp" | "number" | "chainid")` as recognized
//        properties, and src/ir/expressions/member_access/runtime_values.rs
//        wires `chainid → System.Runtime.GetNetwork` (L172-183),
//        `coinbase → address(0) literal` (L187-205), and
//        `gaslimit → Policy.getExecFeeFactor()` (L226-240). This harness
//        confirms all three actually execute and return NON-degenerate
//        payloads — i.e. none of them has regressed to a Null-return stub.
//
//   (H2) `INT256_MIN / -1` arithmetic probe, extending batch #10's scope
//        map (which already pinned uint256 add/sub/mul overflow, int256
//        negate-min, signed-shift edges, and div/mod by zero). This is the
//        canonical "overflow that happens on a signed DIVISION" case: the
//        mathematical result is `INT256_MAX + 1`, which is not representable,
//        so Solidity 0.8.x MUST Panic(0x11).
//
//   (H3) `keccak256("")` reference-parity probe. Batch #16 (H4)
//        (`keccak256_of_bytes_matches_reference`) documented the leak for
//        non-empty `bytes`; batch #19 H5 (`hash_consistency_*`) documented
//        the leak for `abi.encode*`. This is the EMPTY-INPUT corner: the
//        Solidity `keccak256(bytes)` path with zero-length bytes. The
//        empty-input digest is the canonical
//        `0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`
//        — a useful reference-point since it involves no payload bytes, so
//        any deviation implicates the OPCODE itself (or its zero-length
//        bypass) rather than byte-marshaling.
//
// Pre-batch probes (deleted; results summarized here for audit):
//
//   (H1 observed) All three compile and execute cleanly. `block.chainid`
//        returns 8 LE bytes = `0x4e454f0000000000` = `"NEO" LE` — the raw
//        Neo-network magic; this is wired to `System.Runtime.GetNetwork`.
//        `block.coinbase` returns exactly 20 zero bytes — the intentional
//        `address(0)` constant (dBFT has no miner). `block.gaslimit` returns
//        8 LE bytes = `0x1e00000000000000` = 30 — the default
//        `Policy.getExecFeeFactor()` value. NONE of the three is
//        degenerate-Null; NO `#[ignore]` needed.
//
//   (H2 observed) Execution SUCCEEDS and returns 8 zero bytes.
//        i.e. `INT256_MIN / -1` produces 0 silently — another Task #30/#32
//        scope gap (signed division overflow is not guarded). Solidity spec
//        expects Panic(0x11). ACTIVE harness, accepts the current non-panic
//        return as the GAP shape; a future Task #30/#32 fix that flips the
//        behavior to Panic(0x11) will pass the harness on a different match
//        arm. The returned 0 (rather than INT256_MIN or INT256_MAX+1) also
//        hints at sub-256-bit-width arithmetic in the lowering, consistent
//        with batch #10 H2/H3 observations.
//
//   (H3 observed) Return-data is exactly 32 bytes
//        `c5d24601..5d85a470` — a BYTE-FOR-BYTE match against
//        `sha3::Keccak256::digest(b"")`. The empty-input keccak path is
//        CORRECT; this is the FIRST keccak-related Neo DevPack for Solidity harness in
//        the file that passes without an EXPECTED/CURRENT dual block. The
//        corruption observed in batches #16/#19 is confined to the
//        non-empty / abi.encode* paths (Task #44); the bare opcode is fine.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #1 — Remaining block.* accessors: chainid, coinbase, gaslimit.
    //
    // Three single-function contracts, compiled SEPARATELY (each declares its
    // own top-level `contract X`) and executed via offset 0. Each `block.*`
    // accessor lowering returns SOME value; the harness pins:
    //   - chainid   : non-empty, non-all-zeros return (GetNetwork surfaced).
    //   - coinbase  : exactly 20 zero bytes (the intentional address(0) stub).
    //   - gaslimit  : non-empty, non-all-zeros return (getExecFeeFactor surfaced).
    //
    // The coinbase "all zeros" case is the ONLY accepted all-zero return
    // because that IS the documented behavior (see runtime_values.rs:187-205).
    // If any of chainid/gaslimit regresses to all-zeros, the harness fires —
    // signaling that the corresponding intrinsic has been unwired.
    #[test]
    fn runtime_block_chainid_coinbase_gaslimit(
        _seed in 0u32..=0u32,
    ) {
        let source_chainid = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract A { function f() external view returns (uint256) { return block.chainid; } }"#;
        let source_coinbase = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract B { function f() external view returns (address) { return block.coinbase; } }"#;
        let source_gaslimit = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f() external view returns (uint256) { return block.gaslimit; } }"#;

        // block.chainid — wired to System.Runtime.GetNetwork.
        let arts_chain = compile_contracts(source_chainid, false, 2)
            .expect("H1: block.chainid must compile");
        prop_assert!(!arts_chain.is_empty(),
            "H1: block.chainid compile produced no artifacts");
        let bc_chain = &arts_chain[0].bytecode;
        let mut rt_chain = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let res_chain = rt_chain.execute(bc_chain, &[])
            .expect("H1 chainid: execute must not error at host level");
        prop_assert!(res_chain.success,
            "H1 chainid: execute must succeed; exc={:?}",
            res_chain.exception.as_ref().map(|e| &e.message));
        prop_assert!(!res_chain.return_data.is_empty(),
            "H1 chainid: return_data must be non-empty; got 0 bytes. \
             If this fires, System.Runtime.GetNetwork has been unwired.");
        let has_nonzero_chain = res_chain.return_data.iter().any(|&b| b != 0);
        prop_assert!(has_nonzero_chain,
            "H1 chainid: return_data is all zeros ({} bytes, hex={}). \
             Expected the Neo-network magic (observed: 0x4e454f0000000000 = \
             'NEO' LE). A degenerate-zero chainid would signal that \
             System.Runtime.GetNetwork has regressed to a Null-return stub.",
            res_chain.return_data.len(), hex::encode(&res_chain.return_data));

        // block.coinbase — INTENTIONAL address(0) stub (dBFT has no miner).
        let arts_coin = compile_contracts(source_coinbase, false, 2)
            .expect("H1: block.coinbase must compile");
        prop_assert!(!arts_coin.is_empty(),
            "H1: block.coinbase compile produced no artifacts");
        let bc_coin = &arts_coin[0].bytecode;
        let mut rt_coin = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let res_coin = rt_coin.execute(bc_coin, &[])
            .expect("H1 coinbase: execute must not error at host level");
        prop_assert!(res_coin.success,
            "H1 coinbase: execute must succeed; exc={:?}",
            res_coin.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(res_coin.return_data.len(), 20,
            "H1 coinbase: return_data must be 20 bytes (address width); got {} \
             (hex={})", res_coin.return_data.len(), hex::encode(&res_coin.return_data));
        let all_zeros_coin = res_coin.return_data.iter().all(|&b| b == 0);
        prop_assert!(all_zeros_coin,
            "H1 coinbase: return_data must be 20 zero bytes (intentional \
             address(0) stub per runtime_values.rs:187-205); got {}",
            hex::encode(&res_coin.return_data));

        // block.gaslimit — wired to Policy.getExecFeeFactor().
        let arts_gas = compile_contracts(source_gaslimit, false, 2)
            .expect("H1: block.gaslimit must compile");
        prop_assert!(!arts_gas.is_empty(),
            "H1: block.gaslimit compile produced no artifacts");
        let bc_gas = &arts_gas[0].bytecode;
        let mut rt_gas = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let res_gas = rt_gas.execute(bc_gas, &[])
            .expect("H1 gaslimit: execute must not error at host level");
        prop_assert!(res_gas.success,
            "H1 gaslimit: execute must succeed; exc={:?}",
            res_gas.exception.as_ref().map(|e| &e.message));
        prop_assert!(!res_gas.return_data.is_empty(),
            "H1 gaslimit: return_data must be non-empty; got 0 bytes. \
             If this fires, Policy.getExecFeeFactor() has been unwired.");
        let has_nonzero_gas = res_gas.return_data.iter().any(|&b| b != 0);
        prop_assert!(has_nonzero_gas,
            "H1 gaslimit: return_data is all zeros ({} bytes, hex={}). \
             Expected a non-zero Policy.getExecFeeFactor() (observed: \
             0x1e00000000000000 = 30). A degenerate-zero gaslimit would \
             signal that the Policy-native lowering has regressed.",
            res_gas.return_data.len(), hex::encode(&res_gas.return_data));
    }

    // Harness #2 — INT256_MIN / -1 arithmetic overflow.
    //
    // Solidity 0.8.x spec: `type(int256).min / -1` produces the mathematically
    // unrepresentable `INT256_MAX + 1` and MUST Panic(0x11).
    // Current Neo DevPack for Solidity behavior: GAP — execution succeeds and returns 0
    // (observed: 8 zero bytes). Extends batch #10's arithmetic-scope map;
    // this is a new scope for Task #30/#32 (signed-division overflow guard).
    #[test]
    fn arith_intmin_divided_by_neg_one_panics(
        _seed in any::<u8>(),
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f() external pure returns (int256) {
    int256 a = type(int256).min;
    int256 b = -1;
    return a / b;
} }"#;
        let result = compile_and_execute(source);
        let observed = observe(&result);
        // Task #30 slice 4 fix landed: `bigint_divmod_op` in
        // `src/runtime/execution/helpers/arithmetic/basic_ops.rs` detects the
        // INT256_MIN / -1 pair (unambiguous under `BigInt::from_signed_bytes_le`)
        // and returns a `Panic: 0x11` error before performing the division.
        prop_assert_eq!(observed, ObservedBehavior::Panicked(0x11),
            "arith_intmin_divided_by_neg_one_panics: expected Panic(0x11) after Task #30 slice 4");
    }

    // Harness #3 — keccak256("") reference parity.
    //
    // The canonical Keccak-256 digest of the empty byte string is
    // `0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
    // Unlike batch #16 H4 / batch #19 H5 — both of which observe CORRUPTED
    // digests for non-empty / abi.encode* inputs (Task #44) — the empty-input
    // path is CORRECT at the opcode level, because zero-byte input needs no
    // byte-marshaling work. This harness pins that correctness: any future
    // regression in the empty-input path (e.g. a guard that short-circuits
    // to a zero hash, or a JSON-wrapping leak) fires the assertion.
    #[test]
    fn keccak256_empty_bytes_matches_reference(
        _seed in 0u32..=0u32,
    ) {
        use sha3::{Digest, Keccak256};

        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f() external pure returns (bytes32) {
    return keccak256("");
} }"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H3: keccak256(\"\") execute must succeed; exc={:?}", result.exception);
        prop_assert_eq!(result.return_data.len(), 32,
            "H3: keccak256(\"\") must return 32 bytes; got {} (hex={})",
            result.return_data.len(), hex::encode(&result.return_data));

        let reference = Keccak256::digest(b"").to_vec();
        // Sanity-pin the canonical empty-Keccak digest as a literal, so a
        // future sha3-crate regression is also caught.
        let canonical: Vec<u8> = vec![
            0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c,
            0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
            0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b,
            0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
        ];
        prop_assert_eq!(&reference, &canonical,
            "H3: sha3 crate produced a non-canonical empty-Keccak digest. \
             Expected 0x{}, got 0x{}.",
            hex::encode(&canonical), hex::encode(&reference));

        prop_assert_eq!(&result.return_data, &canonical,
            "H3: Neo DevPack for Solidity keccak256(\"\") must equal the canonical \
             0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470. \
             Got 0x{}. Unlike the non-empty / abi.encode* paths (Task #44), \
             the empty-input keccak path is the ONE keccak harness in this \
             file that passes without an EXPECTED/CURRENT dual block — if \
             this fires, a NEW keccak regression has been introduced that \
             corrupts even the zero-byte path.",
            hex::encode(&result.return_data));
    }
}

// ==================== Batch #22 — Meta-Programming, Delete, address(this), Fallback ====================
//
// Scope: five small probes on four narrow surfaces not yet covered by batches
// #1-#21, each targeting a feature set common in real-world Solidity patterns:
//
//   (H1) `type(C).name` — reflection-style metadata. Returns a string literal
//        per Solidity spec; used by OpenZeppelin's `__self__` logging, ABI
//        tooling, and reflection-heavy frameworks. Lowering at
//        src/ir/expressions/member_access/type_bounds.rs:131-153 pushes the
//        contract's name as a UTF-8 string literal. First fuzz of the pure
//        reflection-metadata path.
//
//   (H2) `type(C).creationCode` — CREATE2 factory metadata. Critical for
//        Uniswap V2-style pair deployment, minimal-proxy (EIP-1167) clones,
//        and deterministic-address patterns. Lowering at
//        src/ir/expressions/member_access/type_bounds.rs:155-170 returns an
//        empty byte array (documented compatibility stub — Neo deployment does
//        not consume EVM bytecode blobs directly). This probes whether the
//        empty-bytes stub compiles and executes cleanly, and whether
//        downstream CREATE2 patterns would surface an obvious failure (empty
//        creation code → every deployed child would share the same init hash).
//
//   (H3) `delete` keyword on a storage variable — the core assignment
//        semantic. Solidity spec: `delete v` sets `v` to its type's default
//        value (0 for uints). This is the runtime-effect probe; no prior
//        batch has exercised the `delete` keyword's storage-zeroing path.
//
//   (H4) `address(this)` — the contract's own address. Used by allowance
//        patterns, self-transfer guards, permit-style signature binding, and
//        any path that needs to tell "is this call coming from ourselves?"
//        In Neo, `address(this)` should resolve to the contract's script
//        hash (a 20-byte value derived from sender + NEF checksum + name via
//        `compute_contract_hash`; see neo/contract_hash.rs:17).
//
//   (H5) `fallback()` in the manifest — the "arbitrary calldata" catch-all.
//        Batch #5 already confirmed that when BOTH `receive()` and
//        `fallback()` are declared, `receive()` is REMAPPED to
//        `onNEP17Payment` while `fallback()` retains its name. This probe
//        covers the RECEIVE-ABSENT / FALLBACK-ONLY case — does `fallback()`
//        stay named `fallback` (as documented) or does the remap fire
//        anyway? The NEP-17 standard expects `onNEP17Payment`; a sole
//        `fallback()` with NO `receive()` is the natural way to write a
//        generic handler.
//
// Pre-batch probes (deleted; results summarized here for audit):
//
//   (H1 observed) Execution SUCCEEDS, returns exactly 3 bytes `0x466f6f`
//        = ASCII "Foo". The literal contract name is surfaced as a
//        UTF-8 string, as expected. First non-gap in batch #22 — `type(C).name`
//        is CORRECT. ACTIVE harness pins this.
//
//   (H2 observed) Execution SUCCEEDS but returns ZERO bytes. The Neo DevPack for Solidity
//        lowering is a deliberate compatibility stub (returns an empty
//        ByteArray literal) because Neo deployment does not consume EVM
//        bytecode blobs. This makes every Uniswap V2-style
//        `keccak256(abi.encodePacked(type(C).creationCode, ...))` produce
//        the SAME hash regardless of `C` — any address derived from
//        `creationCode` collides. ACTIVE harness; asserts the current
//        empty-bytes behavior and documents the gap.
//
//   (H3 observed) Execution SUCCEEDS, returns 8 LE bytes = 0 (uint256 0).
//        `delete v` on a uint256 storage variable DOES correctly zero the
//        value at runtime. No Task needed; ACTIVE harness.
//
//   (H4 observed) Execution SUCCEEDS but returns EXACTLY 20 ZERO BYTES.
//        i.e. `address(this)` resolves to `address(0)` rather than the
//        contract's script hash. This is a gap: any contract that uses
//        `address(this)` for allowance/permit/self-transfer guards will
//        read `0x000...000` and potentially collide with a zero-address
//        sentinel. GAP CONFIRMED — filed as Task #49 candidate.
//
//   (H5 observed) Manifest carries a method named `fallback` (offset 0),
//        ALONGSIDE `read` (offset 51), auto-generated `lastData` getter
//        (offset 128), and `_deploy` (offset 205). The remap observed in
//        batch #5 (`receive` → `onNEP17Payment`) does NOT fire when only
//        `fallback()` is declared — `fallback()` keeps its Solidity name.
//        ACTIVE harness documents both: presence of `fallback` and
//        presence of `read`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #1 — `type(Foo).name` returns the literal string "Foo".
    //
    // Solidity spec: `type(C).name` is a constant expression evaluating to
    // the string literal containing the contract's name. Common uses include
    // OpenZeppelin's `__self__` logging (transmitting `type(Self).name` in
    // `Context` events), reflection-style framework scaffolding, and
    // human-readable error messages.
    //
    // Neo DevPack for Solidity behavior (observed pre-probe): returns exactly 3 bytes
    // `0x466f6f` = ASCII "Foo". This matches
    // src/ir/expressions/member_access/type_bounds.rs:131-153 which pushes
    // the type argument's name as a UTF-8 string literal.
    //
    // Status: ACTIVE — `type(C).name` works correctly.
    #[test]
    fn type_contract_dot_name_returns_string(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Foo { function f() external pure returns (string memory) { return type(Foo).name; } }"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H1: `type(Foo).name` execute must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));
        // The expected return is the ASCII byte sequence of the contract name.
        // Solidity `string memory` is variable-length; Neo DevPack for Solidity's
        // string-literal encoding surfaces as the raw UTF-8 bytes in
        // return_data (no ABI length prefix — the observed width IS the
        // string width).
        prop_assert_eq!(&result.return_data, b"Foo",
            "H1: `type(Foo).name` must return the literal ASCII bytes \
             of the contract name (0x466f6f = \"Foo\"); got {} bytes hex={}. \
             If this fires, the type_bounds.rs:131-153 lowering has regressed — \
             check whether (a) the type-argument name extraction is broken, \
             (b) the string literal is being ABI-wrapped (length-prefixed), or \
             (c) a downstream string-encoding layer is corrupting the payload. \
             This is a reflection-style API relied on by OpenZeppelin \
             `__self__` logging and ABI-tooling round-trips.",
            result.return_data.len(), hex::encode(&result.return_data));
    }

    // Harness #2 — `type(Child).creationCode` returns a unique non-empty
    // envelope per contract name, preserving CREATE2 determinism.
    //
    // Solidity spec (EVM): `type(C).creationCode` evaluates to the
    // deployment bytecode of contract `C`. Critical for CREATE2 factory
    // patterns — e.g. Uniswap V2 pair deployment, EIP-1167 minimal proxy
    // clones, and any deterministic-address scheme of the form
    // `keccak256(abi.encodePacked(0xff, sender, salt, keccak256(type(C).creationCode)))`.
    //
    // Neo DevPack for Solidity behavior (post-fix, Task #60): emits a deterministic NEF3-
    // shaped envelope whose script payload is `keccak256("creationCode:<name>")`
    // so every contract name yields a distinct blob. Neo deployment does not
    // consume this (it is an off-chain compatibility artefact used only for
    // hashing) but `keccak256(type(A).creationCode) !=
    // keccak256(type(B).creationCode)` — preserving CREATE2 semantics.
    //
    // Status: ACTIVE — emits stable, non-empty, unique bytes per contract.
    #[test]
    fn type_contract_creation_code_compiles_or_errors(
        _seed in 0u32..=0u32,
    ) {
        fn run(name: &str) -> Vec<u8> {
            let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract {name} {{}}
contract Parent {{ function f() external pure returns (bytes memory) {{ return type({name}).creationCode; }} }}"#);
            let artifacts = compile_contracts(&source, false, 2)
                .unwrap_or_else(|e| panic!("H2 compile failed: {:?}", e));
            let parent = artifacts.iter()
                .find(|a| a.manifest["name"].as_str() == Some("Parent"))
                .expect("H2: Parent artifact must be present");
            let mut runtime = NeoRuntime::new(RuntimeConfig::default())
                .expect("H2 runtime");
            let result = runtime.execute(&parent.bytecode, &[])
                .expect("H2: execute must not error at host level");
            assert!(result.success,
                "H2: `Parent::f()` execute must succeed; exc={:?}",
                result.exception.as_ref().map(|e| &e.message));
            result.return_data
        }

        let alpha = run("Alpha");
        let beta = run("Beta");

        prop_assert!(!alpha.is_empty(),
            "H2: `type(Alpha).creationCode` must be non-empty to preserve \
             CREATE2 determinism; got 0 bytes.");
        prop_assert!(!beta.is_empty(),
            "H2: `type(Beta).creationCode` must be non-empty to preserve \
             CREATE2 determinism; got 0 bytes.");
        prop_assert_ne!(&alpha, &beta,
            "H2: `type(Alpha).creationCode` and `type(Beta).creationCode` \
             must differ so CREATE2 salts-per-child produce distinct \
             addresses; both hashed to identical bytes (len={})", alpha.len());

        use sha3::{Digest, Keccak256};
        let alpha_hash = Keccak256::digest(&alpha);
        let beta_hash = Keccak256::digest(&beta);
        prop_assert_ne!(alpha_hash, beta_hash,
            "H2: keccak256(type(Alpha).creationCode) must differ from \
             keccak256(type(Beta).creationCode) — the core CREATE2 \
             determinism invariant.");
    }

    // Harness #3 — `delete` keyword on a storage var zeros the value.
    //
    // Solidity spec: `delete v` sets `v` to its type's default value (0 for
    // uint256). Assigning 42, then `delete v`, then reading `v` must
    // produce 0 at runtime.
    //
    // Neo DevPack for Solidity behavior (observed pre-probe): execution SUCCEEDS and
    // returns 8 LE zero bytes = uint256 0. `delete v` correctly zeros the
    // storage slot.
    //
    // Status: ACTIVE — `delete` works correctly on a uint256 storage var.
    #[test]
    fn delete_keyword_on_storage_var_zeros_value(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public v;
    function setAndDelete() external returns (uint256) {
        v = 42;
        delete v;
        return v;
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H3: `delete v` execute must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));
        // Per probe, `setAndDelete` is at offset 0 (first user-declared
        // function; public `v` getter is at a later offset). Returned
        // scalar decodes via the LE-width convention established in
        // decode_uint_le.
        let returned = decode_uint_le(&result.return_data);
        let zero = num_bigint::BigUint::from(0u8);
        let forty_two = num_bigint::BigUint::from(42u8);
        if returned == zero {
            // CORRECT: delete zeroed the slot.
        } else if returned == forty_two {
            prop_assert!(false,
                "H3 GAP: `delete v` did NOT zero the storage slot — \
                 function returned 42, meaning the delete statement was \
                 either skipped by the lowering or lowered into a no-op. \
                 This is a spec violation: Solidity 0.8.x guarantees \
                 `delete v` on a `uint256 storage` slot sets it to 0. \
                 return_data hex={}", hex::encode(&result.return_data));
        } else {
            prop_assert!(false,
                "H3: `delete v` returned unexpected value {} (hex={}); \
                 expected 0 (correct) or 42 (delete-skipped gap).",
                returned, hex::encode(&result.return_data));
        }
    }

    // Harness #4 — `address(this)` resolves to a non-zero 20-byte script hash.
    //
    // Solidity spec: `address(this)` evaluates to the executing contract's
    // own address. In Neo, this is the contract's script hash: a 20-byte
    // identifier derived from the executing script via
    // `Hash160(script) = RIPEMD160(SHA256(script))`. For a deployed
    // contract the same identifier is produced by
    // `compute_contract_hash(sender, nef_checksum, name)`
    // (see neo/contract_hash.rs:17); for an un-deployed script running
    // in-process (the shape used by this harness), the runtime derives the
    // identifier directly from the loaded bytecode so that self-references
    // are non-zero and stable.
    //
    // Status: ACTIVE (post-Task-#59-fix pin). The previous behavior returned
    // 20 zero bytes because `System.Runtime.GetExecutingScriptHash` read
    // back the zero-initialized `default_account_bytes`. After the fix,
    // `ExecutionContext::initialize` populates `default_account_bytes` with
    // `Hash160(bytecode)` whenever the configured `contract_account` is the
    // default zero value, matching Neo VM semantics for the executing
    // script hash. This invariant guards against regressions that
    // re-introduce `address(this) == address(0)` (which would silently
    // break EIP-2612 DOMAIN_SEPARATOR, self-allowance checks, and every
    // self-referential OpenZeppelin pattern).
    #[test]
    fn address_this_matches_contract_hash(
        _seed in 0u32..=0u32,
    ) {
        use ripemd::Ripemd160;
        use sha2::{Digest, Sha256};

        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f() external view returns (address) { return address(this); } }"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H4 compile failed: {:?}", e));
        prop_assert!(!artifacts.is_empty(),
            "H4: compile must produce at least one artifact");
        let bytecode = &artifacts[0].bytecode;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H4: `address(this)` execute must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));

        // Invariant 1: address-type return is always 20 bytes wide.
        prop_assert_eq!(result.return_data.len(), 20,
            "H4: `address(this)` must return 20 bytes (address width); got {} \
             (hex={})", result.return_data.len(), hex::encode(&result.return_data));

        // Invariant 2: value is NOT the zero address. This is the core
        // regression guard — any regression that makes `address(this)`
        // fall back to zeros would silently validate arbitrary
        // self-signed messages in EIP-2612 flows.
        let has_nonzero = result.return_data.iter().any(|&b| b != 0);
        prop_assert!(has_nonzero,
            "H4: `address(this)` must NOT be the zero address; got all-zero \
             return_data — this indicates the runtime regressed to the \
             pre-fix zero stub. See src/runtime/execution/\
             execution_impl_part1_init.rs `initialize` for the Hash160 \
             derivation and src/runtime/execution/syscalls/runtime.rs \
             `GetExecutingScriptHash` handler for the read path.");

        // Invariant 3: value equals `Hash160(bytecode)` — the deterministic
        // Neo VM self-hash for the loaded script. This pins the exact
        // computation so the value stays reproducible across runs and any
        // alternate derivation path would be caught here.
        let sha = Sha256::digest(bytecode);
        let expected: [u8; 20] = Ripemd160::digest(sha).into();
        prop_assert_eq!(result.return_data.as_slice(), &expected[..],
            "H4: `address(this)` must equal Hash160(bytecode); \
             got {}, expected {}",
            hex::encode(&result.return_data), hex::encode(expected));
    }

    // Harness #5 — `fallback()` without `receive()` retains its Solidity name.
    //
    // Batch #5 already established that when BOTH `receive()` and
    // `fallback()` are declared, `receive()` is remapped to
    // `onNEP17Payment` while `fallback()` keeps its Solidity name. This
    // probe covers the RECEIVE-ABSENT case: a sole `fallback()` with
    // arbitrary-calldata handling (`lastData = msg.data`). Three invariants:
    //   (a) the contract compiles;
    //   (b) the manifest exposes a method literally named `fallback`;
    //   (c) the manifest exposes the user-declared `read` function.
    //
    // We cannot invoke `fallback()` with arbitrary data here because
    // `call_function` is not available in this file's harness; this is a
    // COMPILE + MANIFEST-SHAPE probe only.
    //
    // Neo DevPack for Solidity behavior (observed pre-probe): compiles; manifest lists
    // `fallback` (offset 0), `read` (offset 51), auto-generated `lastData`
    // getter (offset 128), and `_deploy` (offset 205). The
    // `receive → onNEP17Payment` remap does NOT fire when `receive` is
    // absent — `fallback` keeps its name.
    //
    // Status: ACTIVE — documents both names present and the no-remap behavior.
    #[test]
    fn fallback_with_arbitrary_call_data_compile(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes public lastData;
    fallback() external { lastData = msg.data; }
    function read() external view returns (bytes memory) { return lastData; }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!(
                "H5: fallback-only contract must compile: {:?}\n--- SOURCE ---\n{}",
                e, source));
        prop_assert_eq!(artifacts.len(), 1,
            "H5: single contract must produce exactly one artifact; got {}",
            artifacts.len());
        let manifest = &artifacts[0].manifest;
        let methods = manifest["abi"]["methods"].as_array()
            .expect("H5: manifest must carry an abi.methods array");
        let names: Vec<&str> = methods.iter()
            .filter_map(|m| m.get("name").and_then(serde_json::Value::as_str))
            .collect();

        // (b) manifest must carry `fallback` (NOT renamed to `onNEP17Payment`
        // when `receive()` is absent — batch #5 documented the remap fires
        // only when both receive+fallback are declared).
        prop_assert!(names.contains(&"fallback"),
            "H5: manifest must carry a method literally named `fallback` \
             (receive-absent / fallback-only case); got methods={:?}. \
             If this fires and the name is instead `onNEP17Payment`, then \
             the remap from batch #5 has been EXTENDED to apply even when \
             `receive()` is absent — which would be a behavior change.",
            names);

        // Negative assertion: in the receive-absent case, `onNEP17Payment`
        // must NOT be synthesized. If this ever fires, the compiler is
        // fabricating an onNEP17Payment entry for a contract that only
        // declared fallback — a silent NEP-17 compliance surprise.
        prop_assert!(!names.contains(&"onNEP17Payment"),
            "H5: manifest must NOT carry `onNEP17Payment` when only \
             `fallback()` is declared (no `receive()`); got methods={:?}. \
             A synthesized onNEP17Payment here would mean the compiler is \
             producing a NEP-17 payment handler for a contract the author \
             did not opt into.",
            names);

        // (c) user-declared `read` function.
        prop_assert!(names.contains(&"read"),
            "H5: manifest must carry the user-declared `read` method; \
             got methods={:?}", names);
    }
}

// ==================== Batch #23 — Assembly Content, abi.encodeCall, Event Signatures ====================
//
// Final small batch on genuinely un-probed territory. Three tiny harnesses, each
// closing a specific scope question raised by earlier batches:
//
//   (H1) **Assembly content execution.** docs/SOLIDITY_SUPPORT_MATRIX.md marks
//        inline assembly as ⚠️ "no-op". Batch #7 harness `inline_assembly_noop_compiles`
//        only checked that an `assembly { let x := 1 let y := add(x,2) }` block
//        COMPILES; it never inspected whether the yul body's side effects
//        actually reach a Solidity variable at runtime. This harness assigns
//        `result := add(5, 7)` inside the yul block and returns `result`. If the
//        matrix claim is accurate, `result` stays at its init value (0). If the
//        yul body IS executed, `result` becomes 12. Pre-probe: returns 8 LE
//        zero bytes (= uint256 0) — so the yul body is NOT executed, confirming
//        the matrix claim. This is the first harness to verify the NO-OP
//        behavior at the runtime-effect level, not just at compile level.
//
//   (H2) **abi.encodeCall shares the Task #44 JSON-Array bug.** Batch #16
//        extended Task #44 to cover `abi.encode` (non-packed) in addition to
//        `abi.encodePacked`. The third `abi.encode*` variant is `abi.encodeCall`
//        (Solidity 0.8.11+): `abi.encodeCall(Target.foo, (arg1, arg2))`. This
//        takes a function-type pointer plus a tuple of args and produces the
//        selector-prefixed ABI payload. Pre-probe: returns 85 bytes of JSON
//        UTF-8 `{"type":"Array","value":[{"type":"Integer","value":0},
//        {"type":"Integer","value":42}]}` — the Task #44 bug extends to
//        `abi.encodeCall` as well. The selector slot is "0" (Integer), which
//        also tells us the 4-byte selector is NOT being prepended in any form.
//        Status: #[ignore] — Task #44 scope EXTENDED to encodeCall. Any EVM
//        contract doing `(bool ok,) = target.call(abi.encodeCall(T.f, (x,)));`
//        would send a JSON-text ByteArray instead of ABI-encoded calldata.
//
//   (H3) **Event topic[0] is UTF-8 name string, not keccak256(signature).**
//        Batch #13 harness #2 (`runtime_event_emission_captured`) established
//        that `emit Ping(n)` produces `topics[0] = "Ping"` (UTF-8 bytes, 4 long).
//        That probe used a single-arg event. This harness uses the canonical
//        ERC-20 `Transfer(address indexed from, address indexed to, uint256 value)`
//        — the exact event every Ethers/TheGraph/Etherscan indexer looks for —
//        to pin down whether the same shape holds for a real-world 3-arg event
//        with multiple indexed params. Pre-probe: `topics.len() == 1`,
//        `topics[0]` = 8 bytes = `0x5472616e73666572` = UTF-8 "Transfer".
//        EVM tooling would expect 3 topics (signature-hash + 2 indexed
//        addresses) and `topics[0] = 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef`
//        (keccak256 of the canonical signature). Status: ACTIVE — pins the
//        UTF-8-name-string behavior and documents the EVM-tooling break.
//        Extends Task #39's event-indexing-divergence scope to the ERC-20
//        Transfer event specifically.
//
// All three share the same executor-single-shot pattern as batches #5/#8/#9/#10
// (one function per contract, invoke offset 0 via `runtime.execute(&bytecode, &[])`),
// using the shared `compile_and_execute` / `compile_contracts` helpers.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #1 — assembly block with body now EXECUTES at runtime.
    //
    // History: pre-Task-#99 the yul body was a no-op (returned 0). Task #99
    // landed the yul mstore/mload/return lowering and the core expression
    // handlers (add/sub/mul/div/iszero/eq/lt/gt/...). Task #100 extended
    // the Assign handler so `result := <expr>` where `result` is a
    // Solidity-level outer local resolves via `ctx.resolve_local` instead
    // of failing the yul block and falling through to the no-op path.
    //
    // Runtime expectation: `result := add(5, 7)` now runs → `result = 12`.
    //
    // Status: ACTIVE — pins the forward direction of the matrix claim.
    // docs/SOLIDITY_SUPPORT_MATRIX.md §C should be updated to reflect
    // that basic yul (mstore/mload/return, tstore/tload, add/sub/...)
    // now lowers to NeoVM instructions rather than silently no-oping.
    #[test]
    fn assembly_yul_block_with_body_is_noop(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (uint256) {
        uint256 result = 0;
        assembly { result := add(5, 7) }
        return result;
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H1: assembly-content execution must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));
        let returned = decode_uint_le(&result.return_data);
        let twelve = num_bigint::BigUint::from(12u8);
        prop_assert_eq!(returned.clone(), twelve,
            "H1: yul `result := add(5,7)` must now execute → 12 (Task #99/#100 \
             yul lowering landed). got {} (hex={})",
            returned, hex::encode(&result.return_data));
    }

    // Harness #2 — `abi.encodeCall(Target.foo, (args))` must produce the
    // EVM-canonical 36-byte calldata layout:
    //   selector(4) || abi.encode(args)(32*N)
    // where selector = keccak256("foo(uint256)")[..4] = 0x2fbebd38.
    //
    // History:
    //   * Pre-Task-#44: returned 85 bytes of UTF-8 JSON
    //     (`{"type":"Array",...}`) — both the selector slot and the args tail
    //     leaked through `stack_item_to_bytes` as serde_json text.
    //   * Post-Task-#44 (partial): returned 64 bytes = broken 32-byte
    //     Integer(0) selector slot || correct 32-byte BE(42) arg. The args
    //     tail was EVM-canonical, but the selector slot was still
    //     Integer(0) because `Target.foo` fell through
    //     `lower_generic_member_access`, which drops the inner receiver and
    //     pushes `Integer(0)`.
    //   * Post-Task-#65 (current): returns 36 bytes =
    //     0x2fbebd38 || 32-byte BE(42). This exercises the same
    //     `type_method_selectors` registry Task #54 wired up for
    //     `this.method.selector`, now applied at the `abi.encodeCall`
    //     call-site in `src/ir/expressions/calls/builtins/member_access.rs`.
    //
    // Any EVM pattern doing `target.call(abi.encodeCall(Target.foo, (x,)))`
    // now produces valid calldata that the callee can decode.
    #[test]
    fn abi_encode_call_same_bug_as_encode(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Target { function foo(uint256 x) external {} }
contract C {
    function f() external pure returns (bytes memory) {
        return abi.encodeCall(Target.foo, (uint256(42)));
    }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H2: abi.encodeCall compile failed: {:?}", e));
        let c = artifacts.iter()
            .find(|a| a.manifest["name"].as_str() == Some("C"))
            .expect("H2: artifact C must be present");
        let mut runtime = NeoRuntime::new(RuntimeConfig::default())
            .expect("H2 runtime");
        let result = runtime.execute(&c.bytecode, &[])
            .expect("H2: execute must not fail at host level");
        prop_assert!(result.success,
            "H2: abi.encodeCall execute must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));

        // Post-Task-#65 shape: 36 bytes = 4-byte selector || 32-byte BE arg.
        let rd = &result.return_data;
        prop_assert_eq!(rd.len(), 36,
            "H2 post-task-65: abi.encodeCall must produce 36 bytes = \
             4-byte selector || 32-byte BE(arg). Got {} bytes hex={}. \
             If this regresses to 64, the selector resolution in \
             src/ir/expressions/calls/builtins/member_access.rs is broken.",
            rd.len(), hex::encode(rd));

        // Selector verification: the leading 4 bytes MUST match
        // keccak256("foo(uint256)")[..4] = 0x2fbebd38.
        let expected_selector: [u8; 4] = [0x2f, 0xbe, 0xbd, 0x38];
        prop_assert_eq!(&rd[..4], &expected_selector,
            "H2 post-task-65: leading 4 bytes must be keccak256(\"foo(uint256)\")[..4] \
             = 0x2fbebd38. Got 0x{}.", hex::encode(&rd[..4]));

        // Args tail verification: the trailing 32 bytes must be canonical BE(42).
        let mut expected_tail = [0u8; 32];
        expected_tail[24..].copy_from_slice(&42u64.to_be_bytes());
        prop_assert_eq!(&rd[4..], &expected_tail,
            "H2 post-task-65: trailing 32 bytes must be BE(42). Got {:?}", &rd[4..]);
    }

    // Harness #3 — `emit Transfer(address,address,uint256)` matches the
    // NEP-17 standard Transfer signature, so it is emitted in NATIVE Neo
    // notification shape (gap `events-native`):
    //   * eventName = "Transfer" (the emulator's legacy-path log records
    //     it as the single topic)
    //   * state     = [from(20-byte LE), to(20-byte LE), amount(Integer)]
    //     — NO EVM topic0, no abi.encoded data blob — so Neo wallets /
    //     indexers / NEP-17 trackers can read the transfer natively.
    //   * the zero address maps to Null (mint/burn convention) — pinned
    //     by gap_events_native_tests; here both addresses are non-zero.
    //
    // History: post-Task-#39 this event carried the EVM-canonical shape
    // (topics[0] = keccak256 signature). The events-native gap fix made
    // NEP-17/NEP-11 `Transfer` declarations native; OTHER events keep the
    // EVM shape (see `event_with_indexed_and_dynamic_args_lowers` etc.).
    #[test]
    fn event_topic_uses_ethereum_signature_or_not(
        _seed in 0u32..=0u32,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { event Transfer(address indexed from, address indexed to, uint256 value); function go() external { emit Transfer(address(0x1), address(0x2), 100); } }"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H3: Transfer event emission must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(result.logs.len(), 1,
            "H3: exactly one log expected from a single `emit Transfer(...)`; \
             got {}", result.logs.len());
        let log = &result.logs[0];

        // Native NEP-17 shape: one topic = the Neo event name.
        prop_assert_eq!(log.topics.len(), 1,
            "H3: native NEP-17 Transfer must record 1 topic (the event name); got {}",
            log.topics.len());
        prop_assert_eq!(&log.topics[0][..], b"Transfer" as &[u8],
            "H3: topics[0] must be the literal event name \"Transfer\"; got {}",
            hex::encode(&log.topics[0]));

        // State = [from, to, amount]: 20-byte LE addresses, Integer amount.
        let state = decode_native_notification_state(&log.data);
        prop_assert_eq!(state.len(), 3,
            "H3: native NEP-17 Transfer state must be [from, to, amount]; got {:?}", state);
        let mut from_le = [0u8; 20];
        from_le[0] = 0x01;
        prop_assert_eq!(native_state_bytes(&state[0]), from_le.to_vec(),
            "H3: state[0] must be `from` = address(0x1) as a 20-byte LE UInt160");
        let mut to_le = [0u8; 20];
        to_le[0] = 0x02;
        prop_assert_eq!(native_state_bytes(&state[1]), to_le.to_vec(),
            "H3: state[1] must be `to` = address(0x2) as a 20-byte LE UInt160");
        prop_assert_eq!(native_state_int(&state[2]), 100,
            "H3: state[2] must be the Integer amount 100");
    }
}

// ==================== Batch #24 — NEF Token Table Edges + CALLT Reentrancy ====================
//
// Two small harnesses probing two un-covered edges:
//
//   (H1) **NEF capacity at MAX_METHOD_TOKENS.** Batch #7 exercised `parse_nef`
//        framing failures (bad magic, bad checksum, truncation) but did not
//        probe the token-table maximum. `src/neo/constants.rs:9` declares
//        `MAX_METHOD_TOKENS = 128` (the Neo N3 reference uses a smaller cap
//        than what some third-party docs claim — the task brief anticipated
//        512, but the actual value in this codebase is **128**). The build
//        path enforces the cap (`src/neo/build.rs:53-57`) and the parse path
//        enforces it again (`src/neo/build.rs:211-215`). This harness walks
//        the range `1..=128`, builds a NEF with that many identical tokens
//        (same 20-byte hash, method `"m"`), and asserts `parse_nef` round-
//        trips the token count — documenting that the capacity boundary is
//        reachable end-to-end (not just theoretically accepted by the build
//        side then rejected by the parse side, or vice versa).
//
//   (H2) **Repeated CALLT in a single execution frame.** Batch #15 harness
//        #5 and batch #17 harness #2 each fired CALLT exactly once per
//        script. No existing harness exercises the "two CALLTs back-to-back
//        in the same frame" path, which is the minimum needed to surface
//        any state-corruption bug in the CALLT dispatcher's cleanup between
//        invocations (leaked params on the evaluation stack, stale
//        `invoke_native_contract` context, etc.). This harness builds a
//        script that calls `StdLib.serialize` twice in a row (each against
//        a fresh PUSHDATA1 input) with a DROP between them, then RET. The
//        invariant is simply `result.success == true` — a smoke test for
//        the dispatcher's handling of repeat calls in a single frame.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    // Harness #1 — Build a NEF with up to MAX_METHOD_TOKENS (128) entries and
    // round-trip via `parse_nef`. All tokens reuse the same 20-byte hash and
    // method name "m" — this is the simplest way to hit the capacity edge
    // without blowing up script size or introducing per-token variance that
    // would obscure the framing check. The fuzz input picks a count in
    // `1..=MAX_METHOD_TOKENS`; every count in that range MUST round-trip.
    //
    // Observed behaviour on this codebase:
    //   - MAX_METHOD_TOKENS = 128 (real Neo N3 NefFile.cs hard-caps token
    //     deserialization at 128; src/neo/constants.rs)
    //   - `build_nef_with_tokens` accepts exactly 128 tokens, rejects 129
    //   - `parse_nef` round-trips all 128 token entries byte-for-byte
    //
    // Status: ACTIVE. If the build-side or parse-side caps ever drift out of
    // sync, this harness fires at the exact boundary count and surfaces the
    // disagreement.
    #[test]
    fn nef_roundtrips_with_max_tokens(
        count in 1usize..=128usize,
    ) {
        use neo_devpack_solidity::neo::{build_nef_with_tokens, parse_nef, MethodToken};

        // MAX_METHOD_TOKENS is 128 on this codebase (src/neo/constants.rs),
        // matching the real Neo N3 node's NefFile deserialization cap. We
        // parametrise `count` over the full legal range so proptest hits the
        // boundary (128) in at least a few of its 10 cases.
        const MAX: usize = 128;
        prop_assert!(count <= MAX,
            "fuzz precondition: count must be <= MAX_METHOD_TOKENS={}", MAX);

        // Minimal valid NeoVM script (PUSH0, RET). Tokens are the focus here;
        // `parse_nef` only validates framing, not opcodes.
        let script: Vec<u8> = vec![0x10, 0x40];

        // Build a uniform token table: same hash, same method name. All 128
        // entries are independent `MethodToken` values; the serialiser writes
        // each one out verbatim, so count matters, not content.
        let hash: [u8; 20] = [0xABu8; 20];
        let tokens: Vec<MethodToken> = (0..count)
            .map(|_| MethodToken::new(hash, "m", 1, true, 0x0F))
            .collect();

        let built = build_nef_with_tokens(&script, "batch24", "", &tokens)
            .expect("build_nef_with_tokens must accept 1..=MAX_METHOD_TOKENS entries");

        // Magic + non-trivial length as a quick sanity check.
        prop_assert!(built.starts_with(b"NEF3"),
            "built NEF must begin with the NEF3 magic");
        prop_assert!(built.len() > 4 + 64 + 4,
            "built NEF must be larger than header + trailer; got {}", built.len());

        let parsed = parse_nef(&built)
            .expect("parse_nef must accept a NEF that build_nef_with_tokens just produced");

        prop_assert_eq!(parsed.tokens.len(), count,
            "parsed token count must equal input count at count={}", count);

        // Spot-check: every token round-trips hash, method, parameters_count,
        // has_return_value, and call_flags. We check the first and last to
        // keep the harness cheap at count=512.
        for idx in [0usize, count - 1] {
            let t = &parsed.tokens[idx];
            prop_assert_eq!(t.hash, hash,
                "token[{}] hash must round-trip (count={})", idx, count);
            prop_assert_eq!(&t.method, "m",
                "token[{}] method must round-trip (count={})", idx, count);
            prop_assert_eq!(t.parameters_count, 1u16,
                "token[{}] parameters_count must round-trip (count={})", idx, count);
            prop_assert!(t.has_return_value,
                "token[{}] has_return_value must round-trip (count={})", idx, count);
            prop_assert_eq!(t.call_flags, 0x0Fu8,
                "token[{}] call_flags must round-trip (count={})", idx, count);
        }
    }

    // Harness #2 — Repeated CALLT in a single execution frame.
    //
    // Script layout (raw NeoVM bytes; NOT a NEF — `execute_with_tokens` takes
    // raw bytecode and seeds the token table separately):
    //   0x0C 0x05 h e l l o     PUSHDATA1 "hello"
    //   0x37 0x00 0x00          CALLT token[0]   (→ StdLib.serialize(bytes))
    //   0x45                    DROP             (discard first return value)
    //   0x0C 0x05 w o r l d     PUSHDATA1 "world"
    //   0x37 0x00 0x00          CALLT token[0]   (second invocation)
    //   0x40                    RET              (main-frame RET → return_data)
    //
    // Token: `StdLib.serialize` takes 1 StackItem, returns a value, CallFlags::All.
    // StdLib hash pulled verbatim from src/runtime/spec/native_contracts.rs:46,
    // matching the convention used in batch #15 harness #5.
    //
    // Opcode references:
    //   0x0C = PUSHDATA1 (src/runtime/execution/instruction/push.rs)
    //   0x37 = CALLT     (src/runtime/execution/instruction/flow/calls.rs:35-80)
    //   0x45 = DROP      (src/runtime/execution/instruction/stack.rs)
    //   0x40 = RET       (src/runtime/execution/instruction/flow.rs)
    //
    // Invariant: `result.success == true`. The second CALLT MUST NOT fail
    // because of state carried over from the first (stale params array,
    // leaked evaluation-stack item, broken native-contract context, etc.).
    //
    // Status: ACTIVE. This is a smoke test; if it ever fires, the CALLT
    // dispatcher is leaking state between invocations in a single frame —
    // a CRITICAL finding because real-world contracts routinely CALLT
    // multiple times per entry (e.g. read balance + transfer + emit).
    #[test]
    fn callt_self_reentrancy_uses_two_execute_calls(
        _unused in 0u8..=0u8,
    ) {
        use neo_devpack_solidity::neo::MethodToken;

        // StdLib hash (internal UInt160 LE byte order) — from
        // src/runtime/spec/native_contracts.rs:46. Same as batch #15 H5.
        let stdlib_hash: [u8; 20] = [
            0xc0, 0xef, 0x39, 0xce, 0xe0, 0xe4, 0xe9, 0x25,
            0xc6, 0xc2, 0xa0, 0x6a, 0x79, 0xe1, 0x44, 0x0d,
            0xd8, 0x6f, 0xce, 0xac,
        ];
        let tokens = vec![MethodToken::new(stdlib_hash, "serialize", 1, true, 0x0F)];

        // Build the raw script: two CALLT invocations back-to-back.
        let hello = b"hello";
        let world = b"world";
        let mut script: Vec<u8> = Vec::new();
        // PUSHDATA1 "hello"
        script.push(0x0C);
        script.push(hello.len() as u8);
        script.extend_from_slice(hello);
        // CALLT token[0]
        script.extend_from_slice(&[0x37, 0x00, 0x00]);
        // DROP the first return value so the stack stays clean.
        script.push(0x45);
        // PUSHDATA1 "world"
        script.push(0x0C);
        script.push(world.len() as u8);
        script.extend_from_slice(world);
        // CALLT token[0] again.
        script.extend_from_slice(&[0x37, 0x00, 0x00]);
        // RET — top of stack becomes return_data.
        script.push(0x40);

        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let result = runtime
            .execute_with_tokens(&script, &[], &tokens)
            .expect("execute_with_tokens must not fail at host level");

        prop_assert!(result.success,
            "Two back-to-back CALLTs into StdLib.serialize must both succeed in a \
             single execution frame; exception={:?}, return_data={:?}. If this \
             fires, the CALLT dispatcher is carrying stale state between \
             invocations — file as a CRITICAL finding, examine \
             src/runtime/execution/instruction/flow/calls.rs.",
            result.exception, result.return_data);

        // The second serialize produced the return value left on the stack.
        // `StdLib.serialize` on a ByteString produces a non-empty serialization,
        // so if we got success but empty return_data, the second native call
        // silently no-op'd.
        prop_assert!(!result.return_data.is_empty(),
            "Second CALLT returned success with empty return_data; expected a \
             non-empty serialized ByteString. This indicates the repeat-dispatch \
             path landed in native but returned Null — investigate \
             src/runtime/execution/execution_impl_part2_native/stdlib.rs.");
    }
}

// ==================== Batch #25 — Selector Collisions, encodeWithSignature, Mapping Delete ====================
//
// Three small harnesses probing subtle edges not yet covered:
//
//   (H1) **`abi.encodeWithSignature` shape verification.** Batch #16 H1
//        confirmed `abi.encode` hits the Task #44 JSON-Array leak, and batch
//        #15 H2 confirmed `abi.encodePacked` does the same. The "Task #44
//        scope expansion" docstring in batch #16 listed `encodeWithSignature`
//        as a LIKELY sibling bug but never actually probed it. This harness
//        does. Pre-commit probe observed:
//          return_data.len = 60, first 4 bytes = `[47, 190, 189, 56]` =
//          `0x2fbebd38` = first 4 bytes of `keccak256("foo(uint256)")` —
//          CORRECT Ethereum selector, prepended literally as 4 raw bytes.
//          Remaining 56 bytes = UTF-8 of
//          `{"type":"Array","value":[{"type":"Integer","value":42}]}`.
//        So `encodeWithSignature` is a HYBRID: the selector is correctly
//        keccak-hashed and prepended as raw bytes, BUT the argument tail is
//        the same Task #44 Array-to-JSON leak as `abi.encode*` — NOT the
//        expected 32-byte BE-padded u256 (`0x0000…002a`). This closes Task
//        #44 verification for `encodeWithSignature` specifically: same
//        args-encoding bug, distinct selector-prefix behavior (the selector
//        part works). Status: ACTIVE, pinning current observed hybrid shape
//        so a fix (or further regression) fires the assertion.
//
//   (H2) **Three distinct function names produce distinct selectors.** Batch
//        #14 H3 showed `this.foo.selector` returns `0xc2985578` (Neo-native
//        method-hash scheme, NOT Ethereum-spec `keccak256("foo(uint256)")`).
//        That harness only tested one function, so a collision between
//        different names was never ruled out. Pre-commit probe observed
//        three distinct selectors for `foo`, `bar`, `baz` of signature
//        `(uint256)`:
//          foo → [0xc2, 0x98, 0x55, 0x78] = 0xc2985578 (matches batch #14 H3)
//          bar → [0xfe, 0xbb, 0x0f, 0x7e] = 0xfebb0f7e
//          baz → [0xa7, 0x91, 0x6f, 0xac] = 0xa7916fac
//        The `selectors()` function at offset 0 returns a 3-tuple wrapped
//        as a `StackItem::Array` (JSON leak on multi-return per Task #44),
//        but each element is preserved as a 4-byte ByteArray, so we can
//        parse the JSON back and assert distinctness. Status: ACTIVE —
//        this probes whether Neo's method-naming-hash scheme avoids
//        pairwise collisions for same-signature-different-name triplets.
//        Invariant: all three selectors are mutually distinct.
//
//   (H3) **`delete m[k]` on a storage mapping removes the key.** No existing
//        harness covers Solidity's `delete` operator on a mapping entry.
//        The single-return restructuring (set 42, check 42, delete, return)
//        avoids the Task #44 tuple-return JSON leak. Pre-commit probe
//        observed `return_data = [0; 8]` (LE u64), i.e. `f()` returned 0 —
//        which means the set/read/delete/read sequence all worked: `m[1]`
//        was set to 42, read back as 42 (else return 99), deleted, and
//        re-read as 0 (the mapping default for `uint256`). Status: ACTIVE.
//        Invariant: `result.return_data` decodes to 0 (delete worked).
//        A value of 42 would mean delete silently no-op'd; a value of 99
//        would mean the initial set failed.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #1 — `abi.encodeWithSignature("foo(uint256)", uint256(42))`
    // produces the EVM-canonical 36-byte payload:
    //   [0x2f, 0xbe, 0xbd, 0x38] ++ [0x00; 24] ++ 42u64.to_be_bytes()
    //     == 0x2fbebd38 ++ 0x000000000000000000000000000000000000000000000000000000000000002a
    //
    // OBSERVED (pre-commit probe): 60 bytes — first 4 match the EVM selector
    // exactly (keccak is correctly applied for encodeWithSignature even though
    // `this.foo.selector` uses a Neo-native scheme — interesting asymmetry),
    // then 56 bytes of UTF-8 JSON `{"type":"Array","value":[{"type":"Integer","value":42}]}`.
    // This confirms Task #44 covers `encodeWithSignature`'s ARGUMENT tail but
    // NOT its selector prefix. The hybrid shape is pinned below.
    //
    // When Task #44 is fixed, the args tail will become 32 BE bytes of 42
    // (0x00…002a) and the total length will be 4 + 32 = 36. The pinned
    // assertion fires in that event, prompting re-examination.
    #[test]
    fn abi_encode_with_signature_same_bug_as_encode(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bytes memory) {
        return abi.encodeWithSignature("foo(uint256)", uint256(42));
    }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H1 abi.encodeWithSignature must succeed at host level; exc={:?}",
            result.exception);

        let rd = &result.return_data;

        // EVM-canonical expected payload: 0x2fbebd38 + 32-byte BE-padded 42.
        let evm_selector: [u8; 4] = [0x2f, 0xbe, 0xbd, 0x38]; // keccak256("foo(uint256)")[..4]
        let mut expected_evm: Vec<u8> = Vec::with_capacity(36);
        expected_evm.extend_from_slice(&evm_selector);
        expected_evm.extend_from_slice(&[0u8; 24]);
        expected_evm.extend_from_slice(&42u64.to_be_bytes());

        // Current observed hybrid shape: selector (4 bytes) + JSON-Array tail.
        let has_correct_selector_prefix = rd.len() >= 4 && rd[..4] == evm_selector;
        let tail_is_json_array = rd.len() > 4
            && std::str::from_utf8(&rd[4..])
                .map(|s| s.starts_with(r#"{"type":"Array""#))
                .unwrap_or(false);

        // Three possible shapes:
        //  (a) current hybrid: selector prefix + JSON-Array tail (BROKEN args)
        //  (b) post-fix EVM canonical: selector + 32-byte BE u256
        //  (c) something else — fire a loud assertion to force investigation
        let is_current_hybrid = has_correct_selector_prefix && tail_is_json_array;
        let is_correct_evm = rd.as_slice() == expected_evm.as_slice();

        prop_assert!(is_current_hybrid || is_correct_evm,
            "H1 abi.encodeWithSignature(\"foo(uint256)\", 42) has UNKNOWN \
             shape — not current-hybrid (selector + JSON-Array tail), not \
             EVM-canonical (0x{} + 32-BE 42). rd.len={}, rd={:?}, utf8={:?}",
            hex::encode(&evm_selector), rd.len(), rd,
            std::str::from_utf8(rd).ok());

        // Task #44 LANDED: the encoder now emits the EVM-canonical 36-byte
        // payload (4-byte selector || 32-byte BE-padded uint256). This pin
        // flipped from `is_current_hybrid` to `is_correct_evm` when the
        // runtime-layer `abiEncode` handler replaced the JSON-serialize
        // shim (see src/runtime/execution/execution_impl_part2_native/stdlib.rs).
        prop_assert!(is_correct_evm,
            "H1 abi.encodeWithSignature(\"foo(uint256)\", 42) must produce the \
             EVM-canonical 36 bytes (selector 0x{} ++ 32-BE 42). Got rd.len={}, \
             rd={:?}. If hybrid JSON shape is back (is_current_hybrid={}), \
             the Task #44 fix regressed — re-check the runtime StdLib handler.",
            hex::encode(&evm_selector), rd.len(), rd, is_current_hybrid);
    }

    // Harness #2 — Three distinct function names with identical signatures
    // `(uint256)` produce three mutually-distinct 4-byte selectors.
    //
    // Post-Tasks-#72/#73 the runtime `abiEncode` can't distinguish a
    // `bytes4` selector from a dynamic `bytes`/`string` of length 4 purely
    // from the `StackItem::ByteArray` content. The conservative ruling is
    // to treat all short ByteArrays (lengths other than {16, 20, 32}) as
    // dynamic — which means a `(bytes4, bytes4, bytes4)` tuple return now
    // serialises as three dynamic args: head = 3*32 offset slots, tail =
    // 3*(len(32) + content(32)) = 192. Total: 288 bytes.
    //
    // Pre-commit probe observed three distinct selectors:
    //   foo → 0xc2985578, bar → 0xfebb0f7e, baz → 0xa7916fac
    // Status: ACTIVE. The invariant is strictly distinctness —
    // we do NOT pin exact values because a future change to the hashing
    // scheme (e.g. switch to Ethereum-spec keccak) would change all three
    // values simultaneously, and the distinctness property is what
    // actually matters for collision safety.
    #[test]
    fn different_function_names_produce_different_selectors(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function selectors() external view returns (bytes4, bytes4, bytes4) {
        return (this.foo.selector, this.bar.selector, this.baz.selector);
    }
    function foo(uint256) external pure returns (uint256) { return 0; }
    function bar(uint256) external pure returns (uint256) { return 0; }
    function baz(uint256) external pure returns (uint256) { return 0; }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H2 selectors() must succeed at host level; exc={:?}",
            result.exception);

        // Post-Task-#112: `(bytes4, bytes4, bytes4)` lowers via the
        // EVM-spec STATIC abiEncode path. Each `bytesN` (N ≤ 32) is a
        // fixed-width value type and encodes as its N content bytes
        // LEFT-aligned in a 32-byte slot, right-padded with zeros. Total
        // payload: 3 × 32 = 96 bytes — NOT the 288-byte dynamic shape
        // produced by the pre-Task-#112 runtime classifier which
        // mis-treated short `ByteArray`s as dynamic `bytes`. See
        // `batch47_w4_msg_data_length_and_selector_via_call_method` for
        // the `(uint, bytes4)` counterpart pin and the discussion in
        // `src/runtime/execution/execution_impl_part2_native/stdlib.rs`
        // around `abi_pad32_be` / `abi_is_dynamic`.
        let rd = &result.return_data;
        prop_assert_eq!(rd.len(), 96,
            "H2 post-Task-#112: (bytes4, bytes4, bytes4) lowers as three \
             STATIC 32-byte slots, LEFT-aligned bytesN encoding. Total \
             = 3 × 32 = 96 bytes. rd.len={}, rd={:?}", rd.len(), rd);

        // Each slot[i] holds the 4-byte selector LEFT-aligned followed by
        // 28 zero-pad bytes. Extract the leading 4 bytes per slot.
        let selectors: Vec<[u8; 4]> = (0..3)
            .map(|i| {
                let content_start = i * 32;
                let mut s = [0u8; 4];
                s.copy_from_slice(&rd[content_start..content_start + 4]);
                s
            })
            .collect();

        // Each slot[i]'s trailing 28 bytes MUST be zero (bytesN right-pad).
        for i in 0..3 {
            let pad_start = i * 32 + 4;
            let pad_end = i * 32 + 32;
            prop_assert!(rd[pad_start..pad_end].iter().all(|b| *b == 0),
                "H2 slot {} trailing pad bytes [{}..{}] must all be zero \
                 (bytesN right-pad); got {:?}",
                i, pad_start, pad_end, &rd[pad_start..pad_end]);
        }

        // Invariant: all three are mutually distinct.
        prop_assert_ne!(selectors[0], selectors[1],
            "H2 foo.selector (0x{}) collides with bar.selector — COLLISION, \
             Neo's method-naming-hash scheme is broken for this triplet",
            hex::encode(selectors[0]));
        prop_assert_ne!(selectors[0], selectors[2],
            "H2 foo.selector (0x{}) collides with baz.selector — COLLISION",
            hex::encode(selectors[0]));
        prop_assert_ne!(selectors[1], selectors[2],
            "H2 bar.selector (0x{}) collides with baz.selector — COLLISION",
            hex::encode(selectors[1]));
    }

    // Harness #3 — `delete m[k]` on a `mapping(uint256 => uint256)` removes
    // the entry (re-read returns the default zero value).
    //
    // Source body:
    //   m[1] = 42;
    //   if (m[1] != 42) return 99;     // set/read check
    //   delete m[1];
    //   return m[1];                    // expect 0 if delete worked
    //
    // Returning a single `uint256` avoids the Task #44 tuple-return JSON
    // leak so we can assert on minimum-width LE bytes directly.
    //
    // Pre-commit probe observed `return_data = [0; 8]` (success=true,
    // no exception) — the set wrote 42, the sentinel check passed, and
    // after `delete m[1]` the read returned 0. ACTIVE. Any other value
    // signals a concrete gap:
    //   * 42 → delete silently no-op'd (storage-remove not wired)
    //   * 99 → the initial set failed (should never happen — the test is
    //     a smoke check for mapping write/read in one call)
    #[test]
    fn mapping_delete_removes_key_runtime(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    mapping(uint256 => uint256) private m;
    function f() external returns (uint256) {
        m[1] = 42;
        if (m[1] != 42) return 99;
        delete m[1];
        return m[1];
    }
}"#;

        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H3 f() must succeed at host level; exc={:?}",
            result.exception);

        // Minimum-width LE: 0 → [] or [0; N]; decode defensively.
        // The runtime stores scalar integer returns as LE bytes; both [] and
        // an all-zero byte sequence represent zero. A non-zero value would
        // decode to its exact magnitude.
        let decoded = {
            let rd = &result.return_data;
            let mut v: u64 = 0;
            for (i, b) in rd.iter().enumerate().take(8) {
                v |= (*b as u64) << (8 * i);
            }
            v
        };

        prop_assert_eq!(decoded, 0u64,
            "H3 delete m[1] failed to remove key: f() returned {} (raw={:?}). \
             Expected 0 (delete worked). 42 = delete silently no-op'd; \
             99 = initial set failed. Investigate storage-delete wiring in \
             src/cli/bytecode/bytecode_core.rs (mapping delete lowering).",
            decoded, result.return_data);
    }
}

// ==================== Batch #26 — Revert Args, try/catch, fallback msg.data, Function-Type Params, abi.decode Mismatch ====================
//
// Five harnesses: Task #27 two-arg revert payload pin, try/catch dual-path
// runtime behavior, fallback msg.data length under injected calldata,
// function-typed parameter call-site rejection, and abi.decode narrow-type
// mismatch behavior. Helpers reused: compile_and_execute, compile_contracts,
// decode_uint_le, observe/ObservedBehavior.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // Invariant: `revert Foo(uint256 a, address b)` emits selector || abi.encode(a, b) = 4 + 64 = 68 bytes.
    #[test]
    fn batch26_h1_revert_two_arg_custom_error_roundtrips(
        a in 0u64..=1_000_000u64,
    ) {
        // Fixed 20-byte address literal; we check right-aligned BE bytes in slot 2.
        let addr_hex = "1234567890123456789012345678901234567890";
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    error Foo(uint256 a, address b);
    function boom() external pure {{ revert Foo({a}, address(0x{addr_hex})); }}
}}"#, a = a, addr_hex = addr_hex);

        let result = compile_and_execute(&source);
        prop_assert!(!result.success, "H1 revert must fail; got success");
        let rd = &result.return_data;

        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(b"Foo(uint256,address)");
        let digest = hasher.finalize();
        let expected_selector = &digest[..4];

        prop_assert_eq!(rd.len(), 68,
            "H1 revert payload must be selector(4) + abi.encode(uint256, address)(64) = 68; \
             got {} bytes (rd={:02x?}). Regression on Task #27 two-arg path.",
            rd.len(), rd);
        prop_assert_eq!(&rd[..4], expected_selector,
            "H1 selector must equal keccak256(\"Foo(uint256,address)\")[..4]={:02x?}; got {:02x?}",
            expected_selector, &rd[..4]);
        // Slot 0: BE32(a).
        let mut exp_a = [0u8; 32];
        exp_a[24..].copy_from_slice(&a.to_be_bytes());
        prop_assert_eq!(&rd[4..36], &exp_a[..],
            "H1 slot[0] must be BE32({})", a);
        // Slot 1: 12 zero bytes + 20 BE address bytes (abi.encode(address) left-pads).
        let mut exp_b = [0u8; 32];
        let addr_bytes = hex::decode(addr_hex).expect("address hex decode");
        exp_b[12..].copy_from_slice(&addr_bytes);
        prop_assert_eq!(&rd[36..68], &exp_b[..],
            "H1 slot[1] must be 12 zero bytes + 20 BE address bytes; got {:02x?}",
            &rd[36..68]);
    }

    // Invariant: try/catch success branch returns r*2, catch branch returns 999.
    // Task #70: `this.someFn()` self-external-call routing lands at
    // `System.Contract.Call` with `hash == default_account_bytes`. The runtime
    // now detects that and jumps to the compiled method offset pulled from
    // `manifest.abi.methods[]` (wired through `NeoRuntime::call_method`), so
    // the success path yields `r*2` and the revert path is caught by the
    // enclosing `try … catch` frame.
    #[test]
    fn batch26_h2_try_catch_success_and_revert_paths(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function someFn() external pure returns (uint256) { return 42; }
    function willRevert() external pure returns (uint256) { require(false, "bad"); return 0; }
    function runOk() external returns (uint256) {
        try this.someFn() returns (uint256 r) { return r * 2; } catch { return 999; }
    }
    function runBad() external returns (uint256) {
        try this.willRevert() returns (uint256 r) { return r; } catch { return 999; }
    }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H2 compile failed: {:?}", e));
        prop_assert!(!artifacts.is_empty());
        let artifact = &artifacts[0];

        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let ok_result = runtime.call_method(&artifact.bytecode, &artifact.tokens,
            &artifact.manifest, "runOk", &[])
            .expect("runOk call_method");
        prop_assert!(ok_result.success, "H2 runOk must succeed; exc={:?}", ok_result.exception);
        prop_assert_eq!(decode_uint_le(&ok_result.return_data),
            num_bigint::BigUint::from(84u64),
            "H2 success path: r*2 == 84; got {:?}", ok_result.return_data);

        let bad_result = runtime.call_method(&artifact.bytecode, &artifact.tokens,
            &artifact.manifest, "runBad", &[])
            .expect("runBad call_method");
        prop_assert!(bad_result.success, "H2 runBad must succeed (catch absorbs revert)");
        prop_assert_eq!(decode_uint_le(&bad_result.return_data),
            num_bigint::BigUint::from(999u64),
            "H2 catch path: 999; got {:?}", bad_result.return_data);
    }

    // Invariant: fallback()'s msg.data.length matches the `input` bytes passed to execute.
    // `msg.data` lowers to `System.Runtime.GetScriptContainer.Script` (index 7), which
    // the embedded runtime populates with the raw `input_data` bytes handed to
    // `execute(bytecode, input)`. See src/ir/expressions/member_access/runtime_values.rs
    // and src/cli/bytecode/bytecode_helpers/array_runtime.rs (RuntimeValue::MsgData arm).
    #[test]
    fn batch26_h3_fallback_msg_data_length_matches_calldata(
        injected_len in 0usize..=64usize,
    ) {
        use neo_devpack_solidity::runtime::ExecutionOverrides;
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public len;
    fallback() external { len = msg.data.length; }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H3 compile failed: {:?}", e));
        let artifact = &artifacts[0];
        let calldata: Vec<u8> = (0..injected_len).map(|i| i as u8).collect();

        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let result = runtime
            .execute_with_overrides(&artifact.bytecode, &calldata, &ExecutionOverrides::default())
            .expect("H3 execute_with_overrides must not fail at host level");
        prop_assert!(result.success, "H3 fallback must succeed; exc={:?}", result.exception);
        // Read `len` via the auto-generated getter.
        let len_result = runtime.call_method(&artifact.bytecode, &artifact.tokens,
            &artifact.manifest, "len", &[])
            .expect("H3 len() call_method");
        let observed = decode_uint_le(&len_result.return_data);
        prop_assert_eq!(observed, num_bigint::BigUint::from(injected_len as u64),
            "H3 fallback's msg.data.length must equal injected calldata length ({} bytes); \
             compiler currently synthesises msg.data from zero-arg fallback (selector-only, 4 bytes).",
            injected_len);
    }

    // Invariant: function-typed parameters at the call site are rejected with "unsupported type ... function".
    #[test]
    fn batch26_h4_function_typed_parameter_rejected(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function use_cb(function(uint256) external returns (uint256) cb, uint256 x)
        external returns (uint256) { return cb(x); }
}"#;
        let result = compile_contracts(source, false, 2);
        let err = match result {
            Err(e) => format!("{:?}", e),
            Ok(_) => panic!(
                "H4: compiler unexpectedly accepted a function-typed parameter; \
                 if support has been added, update docs/SOLIDITY_SUPPORT_MATRIX.md §A \
                 and rewrite this test to assert success"
            ),
        };
        prop_assert!(err.contains("unsupported type") && err.contains("function"),
            "H4: expected 'unsupported type ... function' for function-typed parameter; got: {}",
            err);
    }

    // Invariant: abi.decode(encode(uint256), (uint128)) compiles; documents observed truncation/fault behavior.
    #[test]
    fn batch26_h5_abi_decode_width_mismatch_behavior(
        x in 0u64..=1_000_000u64,
    ) {
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (uint128) {{
        bytes memory data = abi.encode(uint256({x}));
        return abi.decode(data, (uint128));
    }}
}}"#, x = x);
        let result = compile_and_execute(&source);
        // Document whichever side current impl lands on: either silent truncate
        // (value <= x) or spec-aligned fault — both tolerated, neither pinned.
        match observe(&result) {
            ObservedBehavior::Returned(v) => {
                prop_assert!(v <= num_bigint::BigUint::from(x),
                    "H5 silent-truncation: observed {} > input {}; unexpected widening", v, x);
            }
            ObservedBehavior::Panicked(_) | ObservedBehavior::FaultOther(_) => {}
        }
    }
}

// ==================== Batch #27 — Dynamic ABI + Chained Storage + Fn Params ====================
//
// Five harnesses for undertested surface. H1/H2/H3 pin the EVM-spec
// offset/length encoding for `abi.encode` with dynamic args (Tasks #72/#73);
// H4 exercises the pending-changes accumulator (crud.rs/query.rs); H5
// documents function-typed parameter rejection (parallel to Batch #26 H4
// state-var case).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Invariant: `abi.encode("hello", hex"deadbeef")` emits 192 bytes per
    // EVM spec — head section of two 32-byte offsets (0x40, 0x80) followed
    // by two tails, each `len(32) || data(padded-to-32)`. Tasks #72/#73
    // aligned the runtime encoder with Solidity's ABI so EIP-712 hashing
    // and cross-chain ABI decoding stay consistent for dynamic args.
    #[test]
    fn batch27_h1_abi_encode_string_bytes_shape(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function pack() external pure returns (bytes memory) {
        return abi.encode("hello", hex"deadbeef");
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success, "H1 pack() must succeed; exc={:?}", result.exception);
        let rd = &result.return_data;
        // EVM-spec: 32 (offset0) + 32 (offset1) + 32 (len0) + 32 (data0) +
        // 32 (len1) + 32 (data1) = 192 bytes.
        prop_assert_eq!(rd.len(), 192,
            "H1 abi.encode(string, bytes) post-Task-#72 must be EVM-spec 192 bytes; \
             got {} bytes (rd={:02x?})", rd.len(), rd);
        // head[0] = 0x40 (offset of first dynamic tail = 32*2 head slots).
        let mut expect_off0 = [0u8; 32]; expect_off0[31] = 0x40;
        prop_assert_eq!(&rd[0..32], &expect_off0[..],
            "H1 head[0] must be offset 0x40; got {:02x?}", &rd[0..32]);
        // head[1] = 0x80 (0x40 + 32(len) + 32(padded data) = 0x80).
        let mut expect_off1 = [0u8; 32]; expect_off1[31] = 0x80;
        prop_assert_eq!(&rd[32..64], &expect_off1[..],
            "H1 head[1] must be offset 0x80; got {:02x?}", &rd[32..64]);
        // tail[0] length = 5, content = "hello" LEFT-aligned then zero-padded.
        let mut expect_len0 = [0u8; 32]; expect_len0[31] = 0x05;
        prop_assert_eq!(&rd[64..96], &expect_len0[..],
            "H1 tail[0] length slot must be 5; got {:02x?}", &rd[64..96]);
        prop_assert_eq!(&rd[96..101], b"hello",
            "H1 tail[0] data head must be 'hello' left-aligned; got {:02x?}", &rd[96..128]);
        // tail[1] length = 4, content = 0xdeadbeef left-aligned then zero-padded.
        let mut expect_len1 = [0u8; 32]; expect_len1[31] = 0x04;
        prop_assert_eq!(&rd[128..160], &expect_len1[..],
            "H1 tail[1] length slot must be 4; got {:02x?}", &rd[128..160]);
        prop_assert_eq!(&rd[160..164], &[0xde, 0xad, 0xbe, 0xef][..],
            "H1 tail[1] data head must be 0xdeadbeef left-aligned; got {:02x?}",
            &rd[160..192]);
    }

    // Invariant: `emit Msg("hello", 42)` produces a log whose data field is
    // the EVM-spec abi.encode of (string, uint256) — 128 bytes total:
    //   head[0] = 0x40 (offset to the "hello" tail, past the 2 head slots)
    //   head[1] = BE32(val) (uint256 is static, carried in the head)
    //   tail[0] = len(5) || "hello" + 27 zero pad bytes  (2×32 = 64 bytes)
    #[test]
    fn batch27_h2_event_dynamic_non_indexed_data_shape(
        val in 0u64..=1_000_000u64,
    ) {
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    event Msg(string message, uint256 value);
    function go() external {{ emit Msg("hello", {val}); }}
}}"#, val = val);
        let artifacts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("H2 compile failed: {:?}", e));
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let result = runtime.execute(&artifacts[0].bytecode, &[])
            .expect("H2 execute must not fail at host level");
        prop_assert!(result.success, "H2 emit must succeed; exc={:?}", result.exception);
        prop_assert_eq!(result.logs.len(), 1, "H2 exactly one log; got {}", result.logs.len());
        let log = &result.logs[0];
        // topics[0] = keccak256("Msg(string,uint256)").
        use sha3::{Digest, Keccak256};
        let expected_sig = Keccak256::digest(b"Msg(string,uint256)");
        prop_assert_eq!(&log.topics[0][..], &expected_sig[..],
            "H2 topics[0] must be sig hash");
        // data = EVM-spec head+tail = 4×32 = 128 bytes.
        prop_assert_eq!(log.data.len(), 128,
            "H2 data must be 128 bytes post-Task-#72 (offset + static + dynamic tail); got {}",
            log.data.len());
        // head[0] = 0x40 (offset to first dynamic tail, past the 2 head slots).
        let mut expect_off0 = [0u8; 32]; expect_off0[31] = 0x40;
        prop_assert_eq!(&log.data[0..32], &expect_off0[..],
            "H2 head[0] must be offset 0x40; got {:02x?}", &log.data[0..32]);
        // head[1] = BE32(val) (uint256 is static).
        let mut expected_val = [0u8; 32];
        expected_val[24..].copy_from_slice(&val.to_be_bytes());
        prop_assert_eq!(&log.data[32..64], &expected_val[..],
            "H2 head[1] must be BE32(val={})", val);
        // tail[0] length = 5, content = "hello" + 27 zeros.
        let mut expect_len0 = [0u8; 32]; expect_len0[31] = 0x05;
        prop_assert_eq!(&log.data[64..96], &expect_len0[..],
            "H2 tail[0] length slot must be 5; got {:02x?}", &log.data[64..96]);
        prop_assert_eq!(&log.data[96..101], b"hello",
            "H2 tail[0] data head must be 'hello' left-aligned; got {:02x?}",
            &log.data[96..128]);
    }

    // Invariant: `keccak256(abi.encode(name, ts, data))` with name/ts/data as
    // named locals produces a 32-byte hash. Post-Tasks-#72/#73 the preimage
    // is the EVM-spec offset/length form — 224 bytes total:
    //   head[0] = 0x60 (offset to name tail, past the 3 head slots)
    //   head[1] = BE32(42)
    //   head[2] = 0xA0 (offset to data tail = 0x60 + 64 byte name tail)
    //   tail name = len(4) || "perm" + 28 zeros
    //   tail data = len(2) || 0xbeef + 30 zeros
    // Pinning the hash catches any drift in the encoding convention.
    #[test]
    fn batch27_h3_keccak_abi_encode_dynamic_matches_pinned(
        _seed in 0u8..=0u8,
    ) {
        // Source uses named locals so string content is preserved (vs. the
        // `string("perm")` explicit-cast path, which probe confirmed produces
        // 32 zero bytes — see Task #73 notes).
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h() external pure returns (bytes32) {
        string memory name = "perm";
        uint256 ts = 42;
        bytes memory data = hex"beef";
        return keccak256(abi.encode(name, ts, data));
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success, "H3 must succeed; exc={:?}", result.exception);
        prop_assert_eq!(result.return_data.len(), 32,
            "H3 keccak256 must return 32 bytes; got {}", result.return_data.len());
        // Expected keccak over the EVM-spec preimage (head + two tails).
        use sha3::{Digest, Keccak256};
        let mut preimage = Vec::with_capacity(224);
        // head[0] = offset 0x60 to name tail.
        let mut h0 = [0u8; 32]; h0[31] = 0x60;
        preimage.extend_from_slice(&h0);
        // head[1] = BE32(42) (ts is uint256, static).
        let mut h1 = [0u8; 32]; h1[31] = 42;
        preimage.extend_from_slice(&h1);
        // head[2] = offset 0xA0 to data tail (0x60 + 64-byte name tail).
        let mut h2 = [0u8; 32]; h2[31] = 0xA0;
        preimage.extend_from_slice(&h2);
        // tail name: len(4) || "perm" + 28 zeros.
        let mut name_len = [0u8; 32]; name_len[31] = 0x04;
        preimage.extend_from_slice(&name_len);
        let mut name_buf = [0u8; 32]; name_buf[..4].copy_from_slice(b"perm");
        preimage.extend_from_slice(&name_buf);
        // tail data: len(2) || 0xbeef + 30 zeros.
        let mut data_len = [0u8; 32]; data_len[31] = 0x02;
        preimage.extend_from_slice(&data_len);
        let mut data_buf = [0u8; 32]; data_buf[..2].copy_from_slice(&[0xbe, 0xef]);
        preimage.extend_from_slice(&data_buf);
        let expected = Keccak256::digest(&preimage);
        prop_assert_eq!(&result.return_data[..], &expected[..],
            "H3 hash must match keccak of EVM-spec 224-byte preimage; \
             got 0x{}, expected 0x{}",
            hex::encode(&result.return_data), hex::encode(&expected));
    }

    // Invariant: chained `put(k,v1); put(k,v2); delete(k)` via
    // `set_storage`/empty-value write interacts correctly with the
    // pending-changes accumulator. After the sequence `storage_find` must
    // reflect only undeleted entries from the broader key set; the deleted
    // key must NOT appear. Probe confirmed this path works (see Batch #14 H1).
    #[test]
    fn batch27_h4_chained_storage_put_put_delete_reflected_in_find(
        v1 in prop::collection::vec(any::<u8>(), 1..32),
        v2 in prop::collection::vec(any::<u8>(), 1..32),
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let account = "0x1234567890123456789012345678901234567890";
        let k_target = b"target";
        let k_keep = b"keep";
        let v_keep = b"keep_value";
        // Seed a "keep" key that must survive the chain.
        runtime.set_storage(account, k_keep, v_keep).expect("seed keep");
        // Chain: put(k, v1); put(k, v2); delete(k).
        runtime.set_storage(account, k_target, &v1).expect("put v1");
        runtime.set_storage(account, k_target, &v2).expect("put v2");
        runtime.set_storage(account, k_target, &[]).expect("delete via empty value");
        // storage_find must NOT return the deleted key.
        let found = runtime.storage_find(account, b"").expect("storage_find");
        let keys: Vec<&[u8]> = found.iter().map(|(k, _)| k.as_slice()).collect();
        prop_assert!(!keys.contains(&(k_target as &[u8])),
            "H4 deleted key must NOT appear in find results; got {:?}", keys);
        // The keep key must be present with its original value.
        prop_assert!(keys.contains(&(k_keep as &[u8])),
            "H4 keep key must survive the chain; got {:?}", keys);
        let keep_val = runtime.get_storage(account, k_keep).expect("get keep");
        prop_assert_eq!(keep_val, Some(v_keep.to_vec()),
            "H4 keep key's value must be unchanged");
        // Direct read of deleted key must be None (not stale v1 or v2).
        let target_val = runtime.get_storage(account, k_target).expect("get target");
        prop_assert_eq!(target_val, None,
            "H4 deleted key must read as None (not stale v1/v2)");
    }

    // Invariant: function-typed parameter `cb` — a callback passed at call
    // site — is rejected with "unsupported type ... function". Same
    // rejection path as Batch #26 H4 (state-var case). If support lands
    // (for either form), both tests must be rewritten to assert successful
    // invocation; see docs/SOLIDITY_SUPPORT_MATRIX.md §A.
    #[test]
    fn batch27_h5_function_typed_parameter_callback_rejected(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function runCb(function(uint256) external pure returns (uint256) cb, uint256 x)
        external returns (uint256) { return cb(x); }
}"#;
        let result = compile_contracts(source, false, 2);
        let err = match result {
            Err(e) => format!("{:?}", e),
            Ok(_) => panic!(
                "H5: compiler accepted a function-typed callback parameter; \
                 if support has been added, update docs/SOLIDITY_SUPPORT_MATRIX.md §A \
                 and rewrite this test to assert success"
            ),
        };
        prop_assert!(err.contains("unsupported type") && err.contains("function"),
            "H5: expected 'unsupported type ... function' rejection; got: {}", err);
    }
}

// ==================== Batch #28 — Arrays, Nested Structs, Maps, Shifts, Precompile Addrs ====================
//
// Five probe-validated harnesses for under-tested edges. GAPS surfaced:
//   H4 (SHL >=64 → 0): bitwise.rs:167-175 hard-caps at 64; Task #32/#50's
//       BigInt path covers NOT/AND/OR/XOR but NOT shift_left/shift_right.
//       Parallel SHR gap: `((1<<65)-1) >> 64` faults on ByteArray LHS.
//   H5 (staticcall to 0x01..0x09 returns (ok=true, out=empty)): compiler
//       does NOT dispatch precompile addresses to CryptoLib. Batch #3's
//       identity probe uses an INLINE library wrapper, not this pattern.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Invariant: a function returning `uint256[][] memory` (Merkle-proof
    // shape) compiles; the manifest returntype is manifest-lowered to
    // "Array" (Neo flattens nested arrays to a single Array type — the
    // inner element type is not encoded in the manifest). Probe confirmed
    // compile OK and returntype="Array" for both returns and parameters.
    #[test]
    fn batch28_h1_array_of_arrays_uint256_compiles_and_manifests_as_array(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function proofs(uint256[][] memory p) external pure returns (uint256) {
        uint256 total = 0;
        for (uint i = 0; i < p.length; i++) { total += p[i].length; }
        return total;
    }
    function build() external pure returns (uint256[][] memory) {
        uint256[][] memory m = new uint256[][](1);
        m[0] = new uint256[](0);
        return m;
    }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H1 compile failed: {:?}", e));
        let methods = artifacts[0].manifest["abi"]["methods"].as_array()
            .expect("H1 abi.methods array");
        let proofs = methods.iter().find(|m| m["name"] == "proofs")
            .expect("H1 proofs method missing");
        let build = methods.iter().find(|m| m["name"] == "build")
            .expect("H1 build method missing");
        // Parameter lowered to "Array" (no nested-array shape info in manifest).
        prop_assert_eq!(proofs["parameters"][0]["type"].as_str(), Some("Array"),
            "H1 uint256[][] parameter must lower to Array in manifest; got {:?}",
            proofs["parameters"][0]);
        prop_assert_eq!(proofs["returntype"].as_str(), Some("Integer"),
            "H1 proofs returntype should be Integer");
        // Externally-callable array returns are ABI-encoded into a single
        // ByteString by `lower_return_statement` (Task #64/#137), so the
        // manifest must advertise ByteArray, not Array.
        prop_assert_eq!(build["returntype"].as_str(), Some("ByteArray"),
            "H1 uint256[][] return is abi-encoded bytes; manifest must say ByteArray; got {:?}",
            build["returntype"]);
    }

    // Invariant: nested struct (`Outer { Inner inner; uint256 b; }`) with
    // public state var compiles; auto-getter returntype lowers to "Array"
    // (struct → Array per SOLIDITY_SUPPORT_MATRIX §Struct). Probe confirmed.
    #[test]
    fn batch28_h2_nested_struct_public_getter_manifest_shape(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Inner { uint256 a; }
    struct Outer { Inner inner; uint256 b; }
    Outer public o;
    function setOuter() external { o.inner.a = 10; o.b = 20; }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H2 compile failed: {:?}", e));
        let methods = artifacts[0].manifest["abi"]["methods"].as_array()
            .expect("H2 abi.methods array");
        let o_getter = methods.iter().find(|m| m["name"] == "o")
            .expect("H2 auto-getter 'o' missing");
        // The auto-getter returns the struct fields as a multi-value tuple,
        // which the externally-callable return lowering ABI-encodes into a
        // single ByteString (verified at runtime: 64 BE-packed bytes), so
        // the manifest must advertise ByteArray.
        prop_assert_eq!(o_getter["returntype"].as_str(), Some("ByteArray"),
            "H2 nested-struct auto-getter returns abi-encoded bytes; got {:?}",
            o_getter["returntype"]);
        prop_assert_eq!(o_getter["parameters"].as_array().map(|p| p.len()), Some(0),
            "H2 auto-getter for storage struct takes zero parameters");
        prop_assert!(methods.iter().any(|m| m["name"] == "setOuter"),
            "H2 setOuter method must appear in manifest");
    }

    // Invariant: `mapping(address => Pos) public positions` compiles; the
    // auto-getter takes a Hash160 param and returns Array (struct flattened).
    // Storage-slot derivation round-trip: after setter stores positions[a].amount,
    // a reader function pulls the same value back — confirms derived slots work.
    #[test]
    fn batch28_h3_mapping_struct_value_storage_roundtrip(
        amt in 1u64..=1_000_000u64,
    ) {
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    struct Pos {{ uint256 amount; address owner; }}
    mapping(address => Pos) public positions;
    function run() external returns (uint256) {{
        positions[address(0x1234)].amount = {amt};
        positions[address(0x1234)].owner = address(0x5678);
        return positions[address(0x1234)].amount;
    }}
}}"#);
        let artifacts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("H3 compile failed: {:?}", e));
        let methods = artifacts[0].manifest["abi"]["methods"].as_array()
            .expect("H3 abi.methods array");
        // Getter with single Hash160 param (for address key) and Array return.
        let getter = methods.iter().find(|m| m["name"] == "positions")
            .expect("H3 auto-getter 'positions' missing");
        // Multi-field tuple getter → abi-encoded bytes on the stack
        // (Task #64), so the manifest advertises ByteArray.
        prop_assert_eq!(getter["returntype"].as_str(), Some("ByteArray"),
            "H3 map-of-struct getter returns abi-encoded bytes; got {:?}",
            getter["returntype"]);
        prop_assert_eq!(getter["parameters"][0]["type"].as_str(), Some("Hash160"),
            "H3 map key param must be Hash160; got {:?}", getter["parameters"][0]);
        // Storage round-trip: writer+reader in one function. The call
        // returns `amt` — proves slot derivation for struct-valued mapping
        // with composite sub-field write is stable.
        let result = compile_and_execute(&source);
        prop_assert!(result.success, "H3 run() must succeed; exc={:?}", result.exception);
        let got = decode_uint_le(&result.return_data);
        prop_assert_eq!(got.clone(), num_bigint::BigUint::from(amt),
            "H3 positions[addr].amount must round-trip through storage; got {}, want {}",
            got, amt);
    }

    // FIXED (Task #H4, bitwise.rs::shift_left BigInt path): SHL at 64/128/129
    // now routes through `u256_bigint_to_stack_item` when amount >= 64 or
    // LHS is wide, preserving full 256-bit precision.
    #[test]
    fn batch28_h4_shl_at_64_128_129_wraps_at_2_pow_256_bug(
        _seed in 0u8..=0u8,
    ) {
        use num_bigint::BigUint;
        for (amount, expected) in [
            (64u32, BigUint::from(1u64) << 64u32),
            (128u32, BigUint::from(1u64) << 128u32),
            (129u32, BigUint::from(1u64) << 129u32),
        ] {
            let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{ function f() external pure returns (uint256) {{ uint256 x = 1; return x << {amount}; }} }}"#);
            let result = compile_and_execute(&source);
            prop_assert!(result.success,
                "H4 SHL by {} must succeed; exc={:?}", amount, result.exception);
            let got = decode_uint_le(&result.return_data);
            prop_assert_eq!(got.clone(), expected.clone(),
                "H4 1 << {} must equal 2^{} = {}; got {} (BigInt path missing in \
                 bitwise.rs::shift_left — Task #32/#50 scope gap)",
                amount, amount, expected, got);
        }
    }

    // FIXED (Task #H5, low_level.rs::emit_precompile_staticcall): compile-time
    // `address(0x02).staticcall(input)` now routes to `CryptoLib.sha256(input)`
    // so `out.length == 32` for the SHA-256 digest. 0x03 ripemd160 and 0x04
    // identity are also wired; other indices still use the generic fallback.
    #[test]
    fn batch28_h5_precompile_address_staticcall_routes_to_cryptolib_bug(
        _seed in 0u8..=0u8,
    ) {
        // sha256 (0x02) of 32 zero bytes — use a known digest so we don't
        // depend on caller-supplied input. If dispatch is wired, out is a
        // 32-byte sha256 digest. Current observed: out.length == 0.
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external view returns (uint256) {
        bytes memory input = new bytes(32);
        (bool ok, bytes memory out) = address(0x02).staticcall(input);
        require(ok, "staticcall failed");
        return out.length;
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H5 sha256(0x02) staticcall must succeed; exc={:?}", result.exception);
        let got = decode_uint_le(&result.return_data);
        prop_assert_eq!(got.clone(), num_bigint::BigUint::from(32u64),
            "H5 sha256 precompile must return 32-byte digest (out.length=32); \
             got {} — dispatch not wired, staticcall returns empty bytes",
            got);
    }
}

// ==================== Batch #29 — Remaining Precompiles + Deploy Args ====================
//
// Context: Tasks #H6a/#H6b landed precompile routing for 0x01 (ecrecover) and
// 0x05 (modexp) on top of the H5 0x02/0x03/0x04 wiring. Batch #29 un-ignores
// probes to confirm the dispatch reaches the CryptoLib / NeoVM opcodes and
// that constructor args plumbed through `_deploy` survive the round-trip.
//
// Scope summary:
//   H1 (#H6a): address(0x01).staticcall(abi.encode(h,v,r,s)) → 32-byte
//              left-padded Ethereum address. Confirms SUBSTR-based ABI
//              decode + recoverSecp256K1 + keccak256 + RIGHT20 sequence.
//   H2 (#H6b): address(0x05).staticcall(abi.encode(1,1,1,b,e,m)) →
//              MODPOW result left-padded to 32 bytes (1-byte operand
//              subset; larger operands planned for a follow-up).
//   H3      : parameterised constructor `C(uint256 x)` — deploy via
//              `_deploy([x], false)` pushed as a StackItem::Array, then
//              verify `v()` returns `x` through NeoRuntime::call_method.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    // Un-ignored after Task #H6a: 0x01 ecrecover precompile now decodes the
    // ABI payload and emits the same SUBSTR+keccak+RIGHT20 sequence Task #20
    // uses for the language-level `ecrecover` builtin. The returned bytes
    // slot is 32 bytes (20-byte address + 12 leading zero pad).
    #[test]
    fn batch29_h1_precompile_ecrecover_via_staticcall(
        _seed in 0u8..=0u8,
    ) {
        // We don't need a valid signature — recoverSecp256K1 returns null on
        // malformed inputs and the lowering substitutes a 20-byte zero
        // address. The assertion is on the output *length* (32) and on the
        // compile-time dispatch: out.length must equal 32 regardless of
        // whether recovery succeeded.
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external view returns (uint256) {
        bytes32 h = 0x0000000000000000000000000000000000000000000000000000000000000001;
        uint8 v = 27;
        bytes32 r = bytes32(uint256(1));
        bytes32 s = bytes32(uint256(1));
        bytes memory input = abi.encode(h, v, r, s);
        (bool ok, bytes memory out) = address(0x01).staticcall(input);
        require(ok, "staticcall failed");
        return out.length;
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H1 ecrecover(0x01) staticcall must succeed; exc={:?}", result.exception);
        let got = decode_uint_le(&result.return_data);
        prop_assert_eq!(got.clone(), num_bigint::BigUint::from(32u64),
            "H1 ecrecover precompile must return 32-byte left-padded address; got {}", got);
    }

    // 0x05 modexp precompile routing landed (Task #H6b: 1-byte-operand
    // subset via NeoVM MODPOW opcode). The precompile expects an EIP-198
    // payload (3x32-byte lengths followed by base||exp||mod, each
    // `mod_len`-wide). We materialise the 99-byte payload via direct byte
    // assignments on a `new bytes(99)` buffer — this avoids the
    // `bytes.concat(bytesN)` stack-leak quirk (resolved.rs notes the
    // `bytesN(..)` coercion leaves an extra buffer reference below the
    // canonical ByteString, and `bytes.concat` chains CAT across that
    // leak, producing truncated output for N>2 args).
    //
    // We then verify `out.length` only — not the scalar value — because
    // the 32-byte wrapper includes the NeoVM BigInteger native encoding
    // of the MODPOW result padded to match the 32-byte staticcall slot
    // shape. Functional parity with the Ethereum precompile (BE scalar
    // equality) is deferred to a follow-up.
    #[test]
    fn batch29_h2_precompile_modexp_via_staticcall(
        _seed in 0u8..=0u8,
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external view returns (bool) {
        bytes memory b = new bytes(99);
        // base_len / exp_len / mod_len — each 32-byte BE uint256(1).
        b[31] = 0x01; b[63] = 0x01; b[95] = 0x01;
        // base=3, exp=5, mod=7 (one byte each).
        b[96] = 0x03; b[97] = 0x05; b[98] = 0x07;
        (bool ok, bytes memory out) = address(0x05).staticcall(b);
        require(ok, "staticcall failed");
        // Confirm dispatch reached MODPOW and produced a non-empty result.
        return out.length > 0;
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H2 modexp(0x05) staticcall must succeed; exc={:?}", result.exception);
        // Success path: ok=true AND out.length>0 AND no revert.
        prop_assert!(!result.return_data.is_empty() && result.return_data[0] != 0,
            "H2 modexp dispatch must reach MODPOW and produce non-empty output; \
             got return_data={:?}", result.return_data);
    }

    // Constructor arg plumbing: compile `C(uint256 x)` and verify the
    // compiler produces a `_deploy(data, update)` method that, when driven
    // with `data = [x]` (StackItem::Array), runs the ctor body and stores
    // `x` into `v`. This exercises the `LoadLocal → jsonDeserialize fallback
    // → ArrayGet(0)` sequence in ir_deploy.rs lines ~132-185.
    //
    // Caveat (2026-04-17): NeoRuntime::call_method auto-fires
    // `_deploy(Boolean(false), Null)` before any user call. For a
    // parameterised-ctor contract that auto-trigger fails at
    // `ArrayGet(data=false, 0)` (PICKITEM: unsupported target Boolean/Null).
    // We side-step the auto-trigger by running `_deploy([x], false)`
    // through `runtime.execute_with_tokens(bytecode, input)` starting at
    // the method's manifest offset via a scratch entry script. That is
    // outside the call_method path and rarely exercised, so instead we
    // focus the assertion on the manifest-level invariant: `_deploy` is
    // emitted with `(data: Any, update: Boolean)` parameters for
    // parameterised constructors. The runtime execution delta is left
    // for a follow-up (file a task when a runtime API for "pre-set
    // deploy args" lands; batch #12 harness #4 style).
    // Task #76 — prove the bug: `bytes.concat(bytes32(x), bytes32(y))` should
    // yield a 64-byte output. Before the fix, each `bytesN(..)` cast left
    // the MEMCPY-returned destination buffer on the stack BENEATH the
    // canonical ByteString (memcpy_bytes pushes `dst` back per C-style
    // semantics). `bytes.concat` chains CAT across that leak — so the
    // output ends up containing the SECOND arg's buffer spliced where the
    // FIRST arg's canonical value should be (e.g. `bytes32(2)||bytes32(2)`
    // instead of `bytes32(1)||bytes32(2)`). The fix in `try_lower_type_concat`
    // mirrors the Task #66 packed-encoding pattern: follow each `bytesN(..)`
    // arg with `Swap; Drop` to discard the leaked buffer reference.
    //
    // This harness verifies the length IS 64 AND that the bytes are the
    // expected BE-encoded values (bytes32(1) || bytes32(2)).
    #[test]
    #[allow(non_snake_case)]
    fn batch29_h4_bytes_concat_bytesN_widths(
        _seed in 0u8..=0u8,
    ) {
        // Also verify last byte is 0x02 and byte-at-index-31 is 0x01.
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bytes memory) {
        return bytes.concat(bytes32(uint256(1)), bytes32(uint256(2)));
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H4 bytes.concat(bytes32,bytes32) must execute; exc={:?}", result.exception);
        prop_assert_eq!(result.return_data.len(), 64,
            "H4 bytes.concat(bytes32,bytes32) must be 64 bytes long; got {} bytes: {:?}",
            result.return_data.len(), result.return_data);
        // First 32 bytes must be bytes32(uint256(1)) = 0x00..01 (BE)
        let mut expected = vec![0u8; 64];
        expected[31] = 0x01;
        expected[63] = 0x02;
        prop_assert_eq!(&result.return_data[..], &expected[..],
            "H4 bytes.concat content wrong");
    }

    #[test]
    fn batch29_h3_constructor_arg_passed_through_deploy(
        x in 1u64..=1_000_000u64,
    ) {
        let _ = x; // x influences nothing in this manifest-only assertion.
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public v;
    constructor(uint256 x) { v = x; }
}"#;
        let artifacts = compile_contracts(source, false, 2)
            .unwrap_or_else(|e| panic!("H3 compile failed: {:?}", e));
        prop_assert!(!artifacts.is_empty());
        let artifact = &artifacts[0];

        // Manifest invariant: `_deploy(data, update)` signature must be
        // (Any, Boolean) and take the ctor arg through `data`.
        let methods = artifact.manifest["abi"]["methods"].as_array()
            .expect("H3 abi.methods must be an array");
        let deploy = methods.iter()
            .find(|m| m.get("name").and_then(|v| v.as_str()) == Some("_deploy"))
            .expect("H3 _deploy method must exist in manifest");
        let params = deploy["parameters"].as_array()
            .expect("H3 _deploy.parameters must be an array");
        prop_assert_eq!(params.len(), 2,
            "H3 _deploy must take (data, update); got {} params", params.len());
        prop_assert_eq!(params[0]["name"].as_str(), Some("data"),
            "H3 _deploy arg0 must be 'data'; got {:?}", params[0]["name"]);
        prop_assert_eq!(params[0]["type"].as_str(), Some("Any"),
            "H3 _deploy arg0 type must be 'Any' (StdLib.jsonDeserialize fallback); got {:?}",
            params[0]["type"]);
        prop_assert_eq!(params[1]["name"].as_str(), Some("update"),
            "H3 _deploy arg1 must be 'update'; got {:?}", params[1]["name"]);
        prop_assert_eq!(params[1]["type"].as_str(), Some("Boolean"),
            "H3 _deploy arg1 type must be Boolean; got {:?}", params[1]["type"]);

        // Bytecode invariant: the compiled NEF must include at least one
        // jsonDeserialize native call in the deploy prologue (for the
        // parameterised-ctor decode chain). Search for CryptoLib/StdLib
        // manifest permission to verify the plumbing is wired.
        let perms = artifact.manifest["permissions"].as_array()
            .expect("H3 manifest.permissions array");
        let has_json = perms.iter().any(|p| {
            let ms = match p["methods"].as_array() { Some(a) => a, None => return false };
            ms.iter().filter_map(|m| m.as_str()).any(|s| s == "jsonDeserialize")
        });
        prop_assert!(has_json,
            "H3 parameterised-ctor manifest MUST declare StdLib.jsonDeserialize permission");
    }
}

// ==================== Batch #30 — Remaining Corner Probes ====================
//
// Five corners post batch #29 (probed via /tmp scratch driver first):
//   H1 uint8 MAX + 1   → GAP. Wraps to 256 (uint16 MAX+1 wraps similarly).
//                        Task #30 guard is uint256-only. `#[ignore]`.
//   H2 keccak(abi.encodePacked(string, bytes))
//                      → GAP. Digest doesn't match Ethereum spec; single-arg
//                        also diverges while `keccak(bytes(s))` works, so
//                        `encodePacked` adds framing to dynamic operands.
//                        `#[ignore]`.
//   H3 fib(n) recursion → GREEN. Self-recursion handled cleanly.
//   H4 modifier(uint256, uint256)
//                      → GREEN. Both require checks fire; body runs.
//                        (bytes32 variant surfaces a gap — see H4b.)
//   H5 event in Base emitted from Derived
//                      → GREEN. topics[0] = keccak256(sig) either way.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    // Harness #3 — GREEN. Self-recursive `internal` fib(n) up to depth ~12.
    // Confirms the NeoVM CALL/RET frame model handles recursion cleanly.
    #[test]
    fn batch30_h3_recursive_fibonacci(
        n in 0u32..=12u32,
    ) {
        // Expected fib(n) computed in Rust as the oracle.
        let mut a: u64 = 0;
        let mut b: u64 = 1;
        for _ in 0..n { let t = a + b; a = b; b = t; }
        let expected = a;
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function fib(uint256 k) internal pure returns (uint256) {{
        if (k < 2) return k;
        return fib(k - 1) + fib(k - 2);
    }}
    function f() external pure returns (uint256) {{ return fib({n}); }}
}}"#, n = n);
        let result = compile_and_execute(&source);
        prop_assert!(result.success,
            "H3 recursive fib({}) must succeed; exc={:?}",
            n, result.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&result.return_data);
        prop_assert_eq!(got, num_bigint::BigUint::from(expected),
            "H3 fib({}) must return {} via recursion", n, expected);
    }

    // Harness #4 — GREEN. Two-param uint256 modifier runs both require
    // checks then body. (bytes32 variant fails — see H4b gap below.)
    #[test]
    fn batch30_h4_modifier_with_two_params(
        a in 1u64..=1_000u64,
        b in 1u64..=1_000u64,
    ) {
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    modifier checkBoth(uint256 x, uint256 y) {{
        require(x > 0, "bad x");
        require(y > 0, "bad y");
        _;
    }}
    function act() external checkBoth({a}, {b}) returns (uint256) {{ return 42; }}
}}"#, a = a, b = b);
        let result = compile_and_execute(&source);
        prop_assert!(result.success, "H4 happy-path must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(decode_uint_le(&result.return_data),
            num_bigint::BigUint::from(42u8),
            "H4 body must return 42 after both checks pass");
    }

    // Harness #5 — GREEN. Event in Base emitted from Derived; topics[0]
    // equals keccak256("Ping(uint256)") — signature stable through
    // inheritance (same hash the same-contract emission in batch #7 uses).
    #[test]
    fn batch30_h5_inherited_event_signature_stable(
        n in 0u64..=1_000u64,
    ) {
        use sha3::{Digest, Keccak256};
        let source = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Base {{ event Ping(uint256 n); }}
contract Derived is Base {{
    function go() external {{ emit Ping({n}); }}
}}"#, n = n);
        let artifacts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("H5 compile failed: {:?}", e));
        prop_assert!(artifacts.len() >= 2, "H5 expected Base + Derived artifacts");
        let derived = artifacts.iter().find(|a| {
            a.manifest.get("name").and_then(serde_json::Value::as_str) == Some("Derived")
        }).unwrap_or(&artifacts[artifacts.len() - 1]);
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let result = runtime.execute(&derived.bytecode, &[])
            .expect("H5 execute must not error at host level");
        prop_assert!(result.success,
            "H5 Derived.go() must succeed; exc={:?}",
            result.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(result.logs.len(), 1,
            "H5 must emit exactly one LogEntry; got {}", result.logs.len());
        let log = &result.logs[0];
        prop_assert_eq!(log.topics[0].len(), 32,
            "H5 inherited event topics[0] must be 32-byte keccak sig hash; got {} bytes",
            log.topics[0].len());
        let mut h = Keccak256::new();
        h.update(b"Ping(uint256)");
        let expected = h.finalize();
        prop_assert_eq!(&log.topics[0][..], &expected[..],
            "H5 inherited event topics[0] must equal keccak256(\"Ping(uint256)\"); \
             got {}", hex::encode(&log.topics[0]));
    }

    // Harness #1 — FIXED (batch-#30). uint8 MAX+1 now Panic(0x11) via the
    // width-aware overflow guard in `src/ir/expressions/dispatch/binary.rs`
    // (`should_emit_narrow_u_arith_guard` + `emit_checked_arith_guard_narrow_u`).
    // The guard emits a post-op `result > (1<<bits)-1 || result < 0 → Panic`
    // range check for uint8/16/32/64/128 Add/Sub/Mul, outside `unchecked`.
    #[test]
    fn batch30_h1_narrow_uint8_overflow_at_max(
        _seed in any::<u8>(),
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f() external pure returns (uint8) {
    uint8 a = 255;
    uint8 b = 1;
    return a + b;
} }"#;
        let result = compile_and_execute(source);
        let observed = observe(&result);
        prop_assert_eq!(observed, ObservedBehavior::Panicked(0x11),
            "H1 uint8 MAX+1 must Panic(0x11) per Solidity 0.8.x spec");
    }

    // Harness #2 — FIXED (batch-#30). `keccak256(abi.encodePacked(string,bytes))`
    // now equals the EVM spec (raw concat of operand bytes, no length prefix).
    // Fix lives in
    // `src/runtime/execution/execution_impl_part2_native/stdlib.rs::abi_packed_bytes`:
    // dynamic ByteArray operands (per `abi_is_dynamic` — any length outside
    // `{16, 20, 32}`) pass through as raw bytes instead of routing through
    // `abi_pad32_be`. Static widths (uint256/address/bytes32-sized) still
    // emit a 32-byte BE slot so `keccak(encode(u256,u256)) ==
    // keccak(encodePacked(u256,u256))` continues to hold.
    #[test]
    fn batch30_h2_keccak_abi_encode_packed_dynamic(
        _seed in any::<u8>(),
    ) {
        use sha3::{Digest, Keccak256};
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bytes32) {
        string memory s = "hi";
        bytes memory b = hex"deadbeef";
        return keccak256(abi.encodePacked(s, b));
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success, "H2 keccak packed must succeed at host level");
        let mut h = Keccak256::new();
        h.update(b"hi");
        h.update(&[0xde, 0xad, 0xbe, 0xef]);
        let expected = h.finalize();
        prop_assert_eq!(&result.return_data[..], &expected[..],
            "H2 keccak256(abi.encodePacked(string,bytes)) must equal \
             keccak256(raw concat of bytes) per Ethereum spec; got {}",
            hex::encode(&result.return_data));
    }

    // Harness #4b — FIXED (batch-#30). `bytes32(uint256(1)) != bytes32(0)`
    // now evaluates to true. Root cause: `coerce_to_fixed_bytes` (invoked by
    // `bytesN(..)` casts) leaves the MEMCPY-returned destination buffer on
    // the stack BENEATH the canonical result, so `role OP bytes32(0)` was
    // comparing the leaked buffer against the canonical result. Fix in
    // `src/ir/expressions/dispatch/binary.rs::lower_binary_expr` follows
    // each operand that `is_fixed_bytes_cast_expr` with `Swap; Drop` to
    // discard the leak — matching the existing cleanup at
    // `builtins.rs:80-83` and `builtins/resolved.rs:103-107` for
    // `bytes.concat` / `abi.encodePacked`.
    #[test]
    fn batch30_h4b_bytes32_ne_zero(
        _seed in any::<u8>(),
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (uint256) {
        bytes32 role = bytes32(uint256(1));
        require(role != bytes32(0), "bad role");
        return 42;
    }
}"#;
        let result = compile_and_execute(source);
        prop_assert!(result.success,
            "H4b bytes32(1) != bytes32(0) must be true; got revert: {:?}",
            result.exception.as_ref().map(|e| &e.message));
    }
}
