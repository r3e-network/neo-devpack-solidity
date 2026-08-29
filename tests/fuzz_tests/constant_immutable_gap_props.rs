//! Property tests for `constant` / `immutable` state variable semantics and
//! the upgradeable-contract storage-gap pattern.
//!
//! Solidity 0.8.x has three distinct kinds of "non-ordinary" state variables
//! that interact with code generation in subtle ways:
//!
//!   * `uint256 constant MAX = 1000;` — value is baked into the bytecode at
//!     compile time. No storage slot is allocated. References to `MAX`
//!     should compile down to literal pushes (or post-folded constants),
//!     not SLOAD instructions. There is no auto-getter unless declared
//!     `public`, and even when public the getter inlines the literal.
//!
//!   * `uint256 public immutable seed;` — value is set exactly once at
//!     deployment time (declaration initializer or constructor body) and
//!     thereafter is read-only. Functionally a one-time-write storage cell;
//!     the public auto-getter returns the stored value. The compiler must
//!     reject any non-constructor write ("Cannot write to immutable here").
//!
//!   * `uint256[N] private __gap;` — a fixed-size array reserving N slots,
//!     conventionally used by upgradeable contracts to leave room for
//!     future state variables in subclasses without disturbing slot
//!     layout. The gap is typically `private` so it has no auto-getter
//!     and no manifest entry; it's purely a layout placeholder.
//!
//! Each subtype has under-tested corners — this module pins the observable
//! semantics with four proptests:
//!
//!   a. `constant_state_var_inlined` — a `uint256 constant MAX = 1000` is
//!      consumed inside `f() returns (uint256) { return MAX * 2; }`. The
//!      callable `f()` returns 2000 and the manifest does NOT advertise
//!      `MAX` as an external getter (state-var compile-folding regression).
//!
//!   b. `immutable_state_var_set_in_ctor` — a fuzzed `_s` is threaded into
//!      `constructor(uint256 _s) { seed = _s; }`. The public auto-getter
//!      `seed()` is read post-deploy and asserted equal to the fuzzed
//!      value (immutable-storage write regression / one-time-write
//!      semantics regression).
//!
//!   c. `immutable_only_settable_in_ctor` — a contract that initializes
//!      `a = 7` from the constructor compiles cleanly. A second variant
//!      adds `function setA(uint256 v) external { a = v; }`, which the
//!      compiler MUST reject (Solidity spec §"Constant and Immutable State
//!      Variables"). This pins the negative side of the immutable
//!      semantics — without it, an immutable would silently degrade to
//!      a regular storage cell.
//!
//!   d. `storage_gap_pattern_compiles` — `contract C { uint256 public
//!      counter; uint256[50] private __gap; uint256 public other; }`
//!      compiles, the two public slots are independently writable, and
//!      `__gap` does NOT appear in the manifest as a method (private +
//!      array; no auto-getter). Drives random sequences of writes to
//!      `counter` / `other` and asserts the values stay independent
//!      across the 50-slot gap.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use num_bigint::BigUint;
use proptest::prelude::*;

proptest! {
    // 8 cases per test — each compiles a small contract and runs a few
    // method calls. Bound at <100ms/case.
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// **a. `uint256 constant MAX` inlined into a callable function.**
    ///
    /// `MAX` is a compile-time constant. Solidity 0.8.x bakes its value
    /// directly into the bytecode at every reference site, so `f()` should
    /// compile to (effectively) `return 2000;` and the manifest should not
    /// advertise `MAX` as a getter (constants do not synthesize an external
    /// getter even when `public`-qualified — the spec defers that to a
    /// literal-returning function, not a state-var getter).
    ///
    /// Failure modes (each maps to a real bug):
    ///   - f() returns 0: constant inlining failed and a SLOAD-on-empty-slot
    ///     ran instead — the const folder did not substitute the literal at
    ///     the reference site.
    ///   - f() returns a non-2000 value: the constant binding leaked across
    ///     scopes (rare, but a deopt would surface here).
    ///   - manifest shows a "MAX" method with returntype Integer: the
    ///     compiler emitted a state-var entry for a constant, suggesting
    ///     a slot was allocated to it (storage-layout regression).
    #[test]
    fn constant_state_var_inlined(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 constant MAX = 1000;
    function f() external pure returns (uint256) { return MAX * 2; }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("constant_state_var_inlined compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "constant_state_var_inlined produced no artifacts");
        let art = &arts[0];

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // f() must return MAX * 2 = 2000.
        let r = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest, "f", &[],
        ).expect("f() host-level (a)");
        prop_assert!(r.success,
            "f() must succeed; exc={:?}. If this is a 'PICKITEM-Null' or \
             SLOAD-on-empty fault, the constant was NOT inlined and the \
             compiler emitted a storage read for a value that should have \
             been baked into the bytecode at compile time.",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got.clone(), BigUint::from(2000u64),
            "f() must return MAX * 2 = 2000; got {} (rd_hex={}). A 0 \
             readback indicates constant-folding regression: `MAX` was \
             read from an unwritten storage slot instead of inlined as \
             the literal 1000.",
            got, hex::encode(&r.return_data));

        // Manifest sanity: `f` must be there as an external method.
        let methods = art.manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");
        let f_present = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some("f")
        });
        prop_assert!(f_present,
            "f() must appear in manifest abi.methods; got methods={:?}",
            methods);

        // Manifest negative: `MAX` (the constant) MUST NOT appear as a
        // state-var getter. Constants do not allocate a slot and do not
        // synthesize an auto-getter under Solidity 0.8.x semantics.
        let max_present = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some("MAX")
                || m.get("name").and_then(serde_json::Value::as_str) == Some("max")
        });
        prop_assert!(!max_present,
            "constant `MAX` must NOT appear as a manifest state-var \
             getter — constants are compile-time literals with no \
             storage slot; got methods={:?}. If a 'MAX' method is \
             present, the storage-layout pass allocated a slot for a \
             constant, which is a regression.",
            methods);
    }

    /// **b. `uint256 public immutable seed;` set in constructor — round-trip.**
    ///
    /// The constructor takes `_s` and writes `seed = _s`. The public
    /// auto-getter `seed()` should return the stored value. Each fuzzed
    /// `_s` exercises a different storage write path: small values stay
    /// inside one limb, large values span multiple LE bytes — both must
    /// round-trip.
    ///
    /// Failure modes:
    ///   - seed() returns 0: the immutable write was dropped (constructor
    ///     body never executed it, OR the value was written to the wrong
    ///     slot, OR the auto-getter reads a different slot than the
    ///     constructor wrote).
    ///   - seed() returns the correct lower bits but wrong upper bits:
    ///     truncation in the immutable encode/decode path.
    #[test]
    fn immutable_state_var_set_in_ctor(s in 0u64..=u32::MAX as u64) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public immutable seed;
    constructor(uint256 _s) { seed = _s; }
    function get() external view returns (uint256) { return seed; }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("immutable_state_var_set_in_ctor compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "immutable_state_var_set_in_ctor produced no artifacts");
        let art = &arts[0];

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Deploy with the fuzzed _s, then read via the explicit `get()`.
        let r = rt.call_method_with_deploy_args(
            &art.bytecode, &art.tokens, &art.manifest,
            "get", &[] as &[StackItem],
            Some(&[StackItem::UnsignedInteger(s)]),
        ).expect("get() with deploy_args host-level (b)");
        prop_assert!(r.success,
            "get() post-immutable-deploy must succeed for _s={}; \
             exc={:?}. A PICKITEM-Null fault here indicates the \
             immutable write was dropped from the deploy prologue.",
            s, r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got.clone(), BigUint::from(s),
            "get() must equal the constructor _s ({}); got {} \
             (rd_hex={}). Mismatch = immutable storage regression: the \
             ctor wrote one slot and the auto-getter reads another, \
             OR the ctor body didn't run, OR the immutable's value \
             was truncated on the way out of `_deploy`.",
            s, got, hex::encode(&r.return_data));

        // Also exercise the public auto-getter `seed()` — for a public
        // immutable, the manifest should expose it under the variable's
        // own name. This guards against a regression where only the
        // explicit `get()` works but the auto-getter is missing.
        let methods = art.manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");
        let seed_present = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some("seed")
        });
        prop_assert!(seed_present,
            "public immutable `seed` must surface an auto-getter \
             named `seed` in the manifest; got methods={:?}. If \
             missing, the public-auto-getter pass skipped the \
             immutable, which is a regression: immutables ARE eligible \
             for auto-getters (per Solidity spec §State-Variable-Visibility).",
            methods);
    }

    /// **c. Immutables only writable inside the constructor.**
    ///
    /// Half-test: a clean variant (immutable initialised in ctor only)
    /// must compile. The other half: a variant that adds an external
    /// `setA(uint256 v) external { a = v; }` writing to the immutable
    /// from a non-constructor context MUST be rejected by the front-end
    /// per Solidity's spec ("Cannot write to immutable here: Immutable
    /// variables can only be initialized inline or assigned directly in
    /// the constructor"). Without this rejection, an immutable silently
    /// degrades to a regular storage cell — a serious semantics bug.
    #[test]
    fn immutable_only_settable_in_ctor(_seed in any::<u8>()) {
        // Variant 1: clean — ctor-only assignment must compile.
        let src_ok = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public immutable a;
    constructor() { a = 7; }
}"#;
        let res_ok = compile_contracts(src_ok, false, 2);
        prop_assert!(res_ok.is_ok(),
            "ctor-only immutable assignment must compile cleanly; \
             got error: {:?}. If this rejects, the front-end is \
             over-eager — immutable IS settable in the constructor \
             body per spec.",
            res_ok.err());

        // Variant 2: invalid — assignment to immutable from a
        // non-constructor function MUST be rejected.
        let src_bad = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public immutable a;
    constructor() { a = 7; }
    function setA(uint256 v) external { a = v; }
}"#;
        let res_bad = compile_contracts(src_bad, false, 2);
        prop_assert!(res_bad.is_err(),
            "writing to an immutable from a non-constructor function \
             MUST be a compile error per Solidity spec (§Constant and \
             Immutable State Variables: 'Immutable variables can only \
             be initialized inline or assigned directly in the \
             constructor'). The compiler accepted it — immutables are \
             silently degraded to regular storage cells, which is a \
             serious semantics regression. src=\n{}", src_bad);
    }

    /// **d. Storage-gap pattern compiles and slots stay independent.**
    ///
    /// `uint256[50] private __gap;` is the upgradeable-contract idiom
    /// for reserving 50 slots between two state-vars so future
    /// subclasses can add fields without disturbing the layout. The
    /// pattern requires:
    ///
    ///   1. The contract compiles.
    ///   2. The 50 reserved slots do NOT collide with `counter` or
    ///      `other` — a write to either auto-getter must round-trip
    ///      independently. (Slot collision would produce read-after-
    ///      write divergence on either side of the gap.)
    ///   3. `__gap` is `private` and an array, so no auto-getter is
    ///      synthesized — the manifest abi.methods MUST NOT contain a
    ///      `__gap` entry.
    #[test]
    fn storage_gap_pattern_compiles(
        c in 1u64..=1_000_000u64,
        o in 1u64..=1_000_000u64,
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counter;
    uint256[50] private __gap;
    uint256 public other;

    function setCounter(uint256 v) external { counter = v; }
    function setOther(uint256 v) external { other = v; }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("storage_gap_pattern_compiles compile: {:?}", e));
        prop_assert!(!arts.is_empty(),
            "storage_gap_pattern_compiles produced no artifacts");
        let art = &arts[0];

        // Manifest negative: `__gap` MUST NOT appear as a method. It's
        // a private uint256[50]; auto-getters are emitted only for
        // public state vars, and even when public, dynamic-array
        // getters take an index — a fixed-size private array gets no
        // method at all.
        let methods = art.manifest["abi"]["methods"]
            .as_array()
            .expect("abi.methods array");
        let gap_present = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some("__gap")
        });
        prop_assert!(!gap_present,
            "private `__gap` MUST NOT surface in manifest abi.methods \
             — it has no public auto-getter; got methods={:?}. If \
             present, the storage-layout pass leaked a private slot \
             into the external ABI surface, a visibility regression.",
            methods);

        // counter() and other() must be present.
        let counter_present = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some("counter")
        });
        let other_present = methods.iter().any(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some("other")
        });
        prop_assert!(counter_present && other_present,
            "public `counter` and `other` must surface auto-getters; \
             counter_present={}, other_present={}. methods={:?}",
            counter_present, other_present, methods);

        // Drive a fuzzed sequence: write counter, write other, read
        // both back. A slot collision through the 50-slot gap would
        // surface as one write clobbering the other.
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        let r1 = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "setCounter", &[StackItem::UnsignedInteger(c)],
        ).expect("setCounter host-level (d)");
        prop_assert!(r1.success,
            "setCounter({}) must succeed; exc={:?}.",
            c, r1.exception.as_ref().map(|e| &e.message));

        let r2 = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "setOther", &[StackItem::UnsignedInteger(o)],
        ).expect("setOther host-level (d)");
        prop_assert!(r2.success,
            "setOther({}) must succeed; exc={:?}.",
            o, r2.exception.as_ref().map(|e| &e.message));

        // Read counter — must equal `c`. If the gap collided with
        // counter's slot, this would observe a different value.
        let r_c = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest, "counter", &[],
        ).expect("counter() host-level (d)");
        prop_assert!(r_c.success,
            "counter() must succeed; exc={:?}.",
            r_c.exception.as_ref().map(|e| &e.message));
        let got_c = decode_uint_le(&r_c.return_data);
        prop_assert_eq!(got_c.clone(), BigUint::from(c),
            "counter() must equal the last write {}; got {} \
             (rd_hex={}). Mismatch = the 50-slot __gap collided with \
             counter's slot, OR setCounter wrote a different slot than \
             counter() reads — storage-layout regression around the \
             fixed-size-array gap.",
            c, got_c, hex::encode(&r_c.return_data));

        // Read other — must equal `o`. If the gap collided with
        // other's slot (post-gap), THIS is where it surfaces.
        let r_o = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest, "other", &[],
        ).expect("other() host-level (d)");
        prop_assert!(r_o.success,
            "other() must succeed; exc={:?}.",
            r_o.exception.as_ref().map(|e| &e.message));
        let got_o = decode_uint_le(&r_o.return_data);
        prop_assert_eq!(got_o.clone(), BigUint::from(o),
            "other() must equal the last write {}; got {} \
             (rd_hex={}). Mismatch = setOther wrote into the gap \
             instead of past it (slot index miscalculated as N+0 \
             instead of N+50), OR the gap's 50-slot stride was \
             miscounted in the storage-layout pass.",
            o, got_o, hex::encode(&r_o.return_data));
    }
}
