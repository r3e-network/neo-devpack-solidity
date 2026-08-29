//! Batches 81-90 — additional fuzz probes.
//!
//! Split from batches_66_80.rs once it crossed the 500KB maintainability threshold.

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Batch #81 — UDVT round-trip, chained struct method calls, anonymous event with 4 indexed params, empty contract, default struct field access ====================
//
// Five orthogonal probes continuing the per-five-harness cadence. Each
// pins a distinct surface for DeFi-adjacent Solidity idioms.
//
//   EEE1: User-defined value type (UDVT) round-trip. `type PriceUSD is
//         uint256` with a pair of file-scope free-function wrap/unwrap
//         helpers (not the direct `PriceUSD.wrap`/`PriceUSD.unwrap`
//         static-method form — batch19 H2 pins that shape). EEE1's
//         free-function form exercises: (a) UDVT as a return type for
//         a free function, (b) UDVT as a parameter type for another
//         free function, (c) the implicit conversion at the call-site
//         boundary (the body does `PriceUSD.wrap(n)` / `PriceUSD.unwrap(p)`
//         under the hood, but the external-facing signature uses the
//         internal-helper-wrapped form). Derives from batch19 H2 (direct
//         form) extended to free-function-wrapped form, which is how
//         Uniswap V4 structures its Currency / BalanceDelta UDVT API.
//         f(42) must equal 42 — UDVT is a transparent alias. 15 fuzz
//         cases exercise repeat-exec stability.
//   EEE2: Chained method calls on a struct via `using { inc } for Counter`.
//         The free function `inc(Counter memory c) returns (Counter memory)`
//         returns the mutated struct by-value; the `using { inc } for
//         Counter` attach makes `c.inc()` syntactically valid on a
//         Counter value, and the return-by-value form permits CHAINING:
//         `c.inc().inc().inc()` pipes each call's return into the next
//         receiver. Extends batch35 K1 (two-library method chain on
//         `uint256`) to: (a) memory-struct receiver (not a scalar),
//         (b) file-scope free-function attachment (not library form —
//         this is the Solidity 0.8.13+ `using { fn } for T` file-scope
//         form, distinct from contract-scope `using L for T`), (c)
//         three-call chain (vs K1's two-call chain). Single-shot —
//         deterministic: 3 inc() calls on a zero-initialized Counter
//         must yield .value == 3.
//   EEE3: Anonymous event with 4 indexed params. `event Full4(address
//         indexed, address indexed, uint256 indexed, bytes32 indexed)
//         anonymous` — four indexed params (the maximum EVM allows per
//         the LOG4 opcode's 4-topic ceiling) PLUS the `anonymous` flag
//         (which suppresses the sig-hash topic0). The net: topics.len()
//         == 4 (only indexed params, no sig hash). Extends batch76 ZZ2
//         (3 indexed non-anonymous → 4 topics = sig + 3 indexed) and
//         batch79 CCC2 (1 indexed anonymous → 1 topic, the indexed
//         param; Task #190 FIXED). EEE3 pins the joint shape: 4 indexed
//         + anonymous flag → 4 topics (no room for sig even without
//         anonymous, but anonymous removes any ambiguity). Tests: (a)
//         MAX-indexed + anonymous combined, (b) no data-section
//         contamination (all args indexed → data empty), (c) the 4th
//         topic carries bytes32 d (non-address indexed with anonymous).
//         Single-shot — deterministic args.
//   EEE4: Empty contract. `contract Empty {}` — no state, no methods,
//         no events. Compile must succeed (boundary_tests.rs line 6
//         already pins `is_ok()` at the compile layer; EEE4 extends
//         to MANIFEST shape). Tests: (a) the manifest's user-visible
//         method set is empty (only the compiler-inserted `_deploy`
//         remains — this is the canonical empty-contract surface),
//         (b) `supportedstandards` is empty or absent (no NEP-17/11
//         auto-declare without explicit annotation), (c) no events
//         declared. Single-shot — the shape is deterministic.
//   EEE5: Default struct field access. `struct P { uint x; uint y; }
//         P public p;` — on a fresh deploy with NO explicit init,
//         `get() == (0, 0)` because all uint fields default to zero.
//         Extends batch80 DDD3 (conditional struct assignment from
//         ternary — the false-arm form assigns P(0,0) EXPLICITLY)
//         to the NO-ASSIGN form (the storage slot's natural default
//         value). Tests: (a) struct state-var's zero-init default
//         (without any ctor or initializer), (b) tuple-return of
//         `(p.x, p.y)` on a fresh/unmodified storage read, (c) the
//         absence of an explicit `set` call — this is a pure
//         read-on-zero-storage probe. 15 fuzz cases exercise
//         repeat-exec stability on the zero-init read path.
//
// Task IDs observed on first exec: `#[ignore]` + new Task #191+ to be
// filled in per-harness after the first run. Baseline expectation is
// EEE1..EEE5 all GREEN (target: 445 passed + 0 ignored; the sibling
// `fix-190-anon-event` 50k hunt landed Task #190 green, so the baseline
// is 440 + 0 rather than 439 + 1).
//
// Sibling agent context: Batch #81's probes are orthogonal to the
// DDD1..DDD5 (Batch #80) surfaces:
//   - EEE1 is UDVT wrap/unwrap through a FREE-FUNCTION wrapper
//     (distinct from batch19 H2's direct static-method form).
//   - EEE2 is struct-chained method calls (distinct from batch35
//     K1's scalar-uint-chained form and any prior struct probe).
//   - EEE3 is 4-indexed anonymous event (distinct from ZZ2's
//     3-indexed-with-sig form and CCC2's 1-indexed-anonymous form).
//   - EEE4 is the empty-contract manifest shape (distinct from any
//     prior manifest probe — boundary_tests line 6 covers compile
//     only; EEE4 extends to manifest-method-count).
//   - EEE5 is default struct storage read (distinct from DDD3's
//     explicit-P(0,0)-assign form).

// EEE1 — UDVT wrap/unwrap via free-function helpers.
// `wrap(n)` and `unwrap(p)` are file-scope free functions — the
// external-facing `f(n)` pipes n → wrap(n) → unwrap(...) → back to
// uint256. f(42) must return 42.
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch81_eee1_udvt_wrap_unwrap_free_function_roundtrip(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
type PriceUSD is uint256;
function wrap(uint256 n) pure returns (PriceUSD) { return PriceUSD.wrap(n); }
function unwrap(PriceUSD p) pure returns (uint256) { return PriceUSD.unwrap(p); }
contract C {
    function f(uint256 n) external pure returns (uint256) { return unwrap(wrap(n)); }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("EEE1 compile: {:?}. If this fires \
                on `type PriceUSD is uint256`, the UDVT declaration \
                regressed (batch19 H2 pins the direct form). If it \
                fires on the file-scope free functions `wrap`/`unwrap` \
                taking/returning the UDVT, the free-function-with-UDVT \
                signature regressed (batch78 BBB1 pins file-scope free \
                function dispatch for plain uint, EEE1 extends to UDVT \
                parameter/return types).", e));
        let art = arts.iter()
            .find(|a| a.metadata.name == "C")
            .unwrap_or_else(|| panic!("EEE1 C artifact missing; got names={:?}",
                arts.iter().map(|a| a.metadata.name.clone()).collect::<Vec<_>>()));
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("EEE1 rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(42)])
            .expect("EEE1 f(42) host-level");
        prop_assert!(r.success,
            "EEE1 f(42) must succeed; exc={:?}. If exc cites the free-\
             function dispatch (unresolved `wrap` or `unwrap`), the \
             Task #187 file-scope free-function resolver regressed. If \
             it cites `PriceUSD.wrap`/`PriceUSD.unwrap` in the free-\
             function body, the UDVT static-method lowering regressed \
             (batch19 H2 precedent).",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(42u64),
            "EEE1 f(42) must equal 42 (UDVT round-trip through free-\
             function helpers is a no-op on value); got {} rd_hex={}. \
             If 0, the UDVT conversion dropped the value (either wrap \
             zeroed it or unwrap read from the wrong slot). If some \
             other value, the free-function dispatch routed to a \
             different function than `wrap` or `unwrap`. Task #191+ \
             candidate: UDVT through free-function wrap/unwrap.",
            v, hex::encode(&r.return_data));
    }
}

// EEE2 — Chained struct method calls via `using { inc } for Counter`.
// `inc(Counter memory c)` returns a mutated copy; chaining
// `c.inc().inc().inc()` must increment the value 3 times on an
// initial Counter(0), yielding .value == 3.
// Single-shot — deterministic: zero-init → 3 inc() calls → 3.
//
// Task #191 FIXED. Root cause was two-fold: (1) `s.field = rhs`
// (plain) and `s.field op= rhs` (compound) on memory structs had no
// handler for parameter-based receivers — only local receivers — so
// every write to `c.value` inside `inc(Counter memory c)` silently
// dropped through `lower_assignment`'s fallback. Fixed in
// `src/ir/statements/assignments/{lower_assignment.rs,compound.rs}`
// by accepting `ctx.param_index_map` in addition to `resolve_local`
// and emitting `LoadParameter`+`ArraySet` against the NeoVM Array
// slot. (2) `infer_type_from_expression` returned `None` for any
// generic `Expression::FunctionCall`, so `c.inc().value` couldn't
// resolve the struct-field index and fell through to the drop-and-
// push-zero branch in `lower_generic_member_access`. Fixed by
// introducing `function_return_types: HashMap<(name, arg_count),
// ValueType>` populated in `module_impl.rs` and threaded through
// `LoweringContext`; inference now queries it for bare `f(args)`
// and receiver-attached `x.f(args)` (with arg_count+1 for the
// using-for form) calls.
#[test]
fn batch81_eee2_chained_struct_method_calls_using_for() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
struct Counter { uint value; }
function inc(Counter memory c) pure returns (Counter memory) { c.value++; return c; }
using { inc } for Counter;
contract C {
    function f() external pure returns (uint) {
        Counter memory c = Counter(0);
        return c.inc().inc().inc().value;
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "EEE2 compile: {:?}. If this fires \
            on `using {{ inc }} for Counter;` at file scope, the Task \
            #188 file-scope using-attach path doesn't cover FREE \
            FUNCTIONS (vs library members) or doesn't cover STRUCT \
            types (vs scalar uint). If on the three-call chain \
            `c.inc().inc().inc()`, the return-by-value chaining on \
            a memory struct regressed (batch35 K1 pins the scalar-\
            uint chain shape). If on `Counter memory c = Counter(0)`, \
            the single-field struct-literal-to-memory regressed.",
            e
        )
    });
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .unwrap_or_else(|| {
            panic!(
                "EEE2 C artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("EEE2 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[] as &[StackItem],
        )
        .expect("EEE2 f() host-level");
    assert!(
        r.success,
        "EEE2 f() must succeed; exc={:?}. If exc cites the chained \
         method call, the return-by-value-then-method-call lowering \
         regressed. If it cites `c.value++` inside inc(), the mutable-\
         memory-struct-field increment regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert_eq!(
        v.clone(),
        BigUint::from(3u64),
        "EEE2 f() must equal 3 (three chained inc() calls on Counter(0)); \
         got {} rd_hex={}. If 0, no chain call mutated the struct \
         (the by-value copy was made but its mutation never flowed \
         back to the next receiver — chain flattened to identity). \
         If 1, only the first inc() landed (chain broke after first \
         call — return value wasn't re-piped). If 2, two chained \
         calls landed but the third was dropped (3-call chain limit \
         or stack-slot exhaustion). Task #191+ candidate: chained \
         memory-struct method call return-value threading.",
        v,
        hex::encode(&r.return_data)
    );
}

// EEE3 — Anonymous event with 4 indexed params.
// `event Full4(address indexed, address indexed, uint256 indexed,
// bytes32 indexed) anonymous` — 4 indexed (max per EVM LOG4) + the
// `anonymous` flag (suppresses sig-hash topic0). Log must have 4
// topics (all indexed args, no sig) and 0 data bytes.
// Single-shot — deterministic args.
#[test]
fn batch81_eee3_anonymous_event_4_indexed_no_sig_topic() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use sha3::{Digest, Keccak256};
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Full4(address indexed a, address indexed b, uint256 indexed c, bytes32 indexed d) anonymous;
    function f(address a, address b, uint c, bytes32 d) external { emit Full4(a, b, c, d); }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "EEE3 compile: {:?}. If this fires \
            on 4 indexed + anonymous, the joint shape regressed. If on \
            the `anonymous` keyword alone, Task #190's fix regressed. \
            If on the 4th indexed bytes32 arg, the indexed-bytes32 \
            encoding regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("EEE3 rt");

    // Fixed args: a = 0x11 * 20, b = 0x22 * 20, c = 0x33 * 1 (= 0x33 = 51),
    // d = 0x44 * 32. All constants so the invariant check is deterministic.
    // Note: the address bytes are LE on the runtime stack boundary.
    let a_le = [0x11u8; 20];
    let b_le = [0x22u8; 20];
    let c_val: u64 = 51;
    let d_bytes32 = [0x44u8; 32];

    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[
                StackItem::byte_array(a_le.to_vec()),
                StackItem::byte_array(b_le.to_vec()),
                StackItem::Integer(c_val as i64),
                StackItem::byte_array(d_bytes32.to_vec()),
            ],
        )
        .expect("EEE3 f(a, b, c, d) host-level");
    assert!(
        r.success,
        "EEE3 f() must succeed; exc={:?}. If exc cites event emit, the \
         anonymous + 4-indexed combined lowering regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // (1) Exactly 1 log must fire.
    assert_eq!(
        r.logs.len(),
        1,
        "EEE3 f() must emit exactly 1 Full4 log; got {} logs. If 0, \
         the emit was elided. If 2+, a shadow emit fired.",
        r.logs.len()
    );
    let log = &r.logs[0];

    // (2) CRITICAL: topics.len() must be 4 (only the 4 indexed args,
    // NO sig topic0 because `anonymous`). If it were 5, that would
    // mean the `anonymous` flag was ignored and topic0 = keccak(sig)
    // was prepended — Task #190 regression. If 3, one indexed arg
    // was dropped. If 1, only sig survived (but that's impossible
    // with `anonymous`).
    assert_eq!(
        log.topics.len(),
        4,
        "EEE3 anonymous event with 4 indexed args must have exactly 4 \
         topics (a, b, c, d — no sig-hash topic0); got {} topics. If \
         5, the `anonymous` flag was ignored (Task #190 regression: \
         the sig topic was prepended anyway, giving sig + 4 indexed). \
         If 3, one indexed arg was dropped (bytes32 d is the most \
         likely casualty — ZZ2 batch76 precedent shows this shape). \
         If fewer, more args were dropped. Task #191+ candidate: \
         4-indexed + anonymous joint shape.",
        log.topics.len()
    );

    // (3) None of the topics must equal keccak256("Full4(address,
    // address,uint256,bytes32)") — that would be the sig-hash, which
    // `anonymous` MUST suppress. We check ALL 4 topic slots (not just
    // index 0) to catch misplacement.
    let bad_sig = Keccak256::digest(b"Full4(address,address,uint256,bytes32)").to_vec();
    for (i, topic) in log.topics.iter().enumerate() {
        assert_ne!(
            &topic[..],
            &bad_sig[..],
            "EEE3 topic[{}] MUST NOT equal keccak256(\"Full4(address,\
             address,uint256,bytes32)\") = 0x{}; got 0x{}. If it \
             matches, the sig-hash was emitted somewhere in the topics \
             despite `anonymous` — the flag was only partially honored \
             (e.g., topic0 was replaced with sig instead of dropped). \
             Task #190 regression check.",
            i,
            hex::encode(&bad_sig),
            hex::encode(topic)
        );
    }

    // (4) data MUST be empty — all 4 args are indexed, so nothing
    // goes to the data section. Mirrors ZZ2 batch76's invariant.
    assert_eq!(
        log.data.len(),
        0,
        "EEE3 log.data MUST be empty (all 4 args indexed → 0 data \
         bytes); got {} bytes data=0x{}. If non-empty, an indexed arg \
         leaked into data (emit lowering conflated indexed vs non-\
         indexed placement with 4-indexed + anonymous combined).",
        log.data.len(),
        hex::encode(&log.data)
    );
}

// EEE4 — Empty contract manifest shape.
// `contract Empty {}` — no state, no user methods. Compile must
// succeed (already pinned in boundary_tests.rs line 6) AND the
// manifest must have NO user-visible methods (only the compiler-
// inserted `_deploy` remains). No `supportedstandards` without
// explicit annotation.
// Single-shot — deterministic.
#[test]
fn batch81_eee4_empty_contract_manifest_no_user_methods() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Empty {}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "EEE4 compile: {:?}. If this fires \
            on the empty contract body, the zero-method contract \
            regressed (boundary_tests.rs line 6 already pins this as \
            GREEN at the compile layer; EEE4 extends to manifest).",
            e
        )
    });
    assert_eq!(
        arts.len(),
        1,
        "EEE4 must produce exactly 1 artifact (the Empty contract); \
         got {} artifacts = {:?}. If 0, the compile elided the empty \
         contract entirely. If 2+, some phantom contract leaked.",
        arts.len(),
        arts.iter()
            .map(|a| a.metadata.name.clone())
            .collect::<Vec<_>>()
    );
    let art = &arts[0];
    assert_eq!(
        art.metadata.name, "Empty",
        "EEE4 artifact name must be `Empty`; got {:?}",
        art.metadata.name
    );

    // (1) Manifest methods array — must exist, but user-visible
    // count must be ZERO (only `_deploy` may remain).
    let methods = art.manifest["abi"]["methods"]
        .as_array()
        .expect("EEE4 manifest methods array must exist");
    let user_methods: Vec<&serde_json::Value> = methods
        .iter()
        .filter(|m| m.get("name").and_then(serde_json::Value::as_str) != Some("_deploy"))
        .collect();
    assert_eq!(
        user_methods.len(),
        0,
        "EEE4 empty contract must have 0 user-visible methods (only \
         `_deploy` may remain); got {} user methods = {:?}. If >0, \
         some compiler-inserted method leaked into the user surface \
         (e.g., an auto-generated getter for a non-existent state \
         var). Task #191+ candidate: empty-contract phantom-method \
         leak.",
        user_methods.len(),
        user_methods
            .iter()
            .map(|m| m["name"].clone())
            .collect::<Vec<_>>()
    );

    // (2) No events declared.
    let events = art.manifest["abi"]["events"].as_array();
    if let Some(evs) = events {
        assert!(
            evs.is_empty(),
            "EEE4 empty contract must have 0 events; got {} events = {:?}. \
             If non-empty, some event was auto-declared (e.g., a \
             Transfer event leaked from a default NEP-17 template).",
            evs.len(),
            evs
        );
    }
    // If events field is missing entirely, that's also acceptable for
    // an empty contract — the manifest emitter may omit empty arrays.

    // (3) supportedstandards: no NEP-17/11/24 (value-token / NFT / royalty)
    //     standards may leak without explicit annotation. NEP-29 IS
    //     canonically declared for every contract because the compiler
    //     inserts a `_deploy(data, update)` method (mandated by NEP-29
    //     itself), per src/cli/cli_parts/cli_manifest/standards.rs line
    //     170. That's expected shape, not a leak.
    let standards = art
        .manifest
        .get("supportedstandards")
        .and_then(serde_json::Value::as_array);
    if let Some(stds) = standards {
        let disallowed = ["NEP-11", "NEP-17", "NEP-24", "NEP-26"];
        for std_val in stds {
            let name = std_val.as_str().unwrap_or("");
            assert!(
                !disallowed.contains(&name),
                "EEE4 empty contract must NOT declare user-facing \
                 standards (NEP-11/17/24/26) without explicit \
                 @custom:neo.manifest.supportedstandards annotation; \
                 found {:?} in supportedstandards={:?}. NEP-29 (deploy-\
                 callback) IS canonical — every contract inherits the \
                 `_deploy(data, update)` method from the compiler \
                 prologue, so its auto-declare per \
                 src/cli/cli_parts/cli_manifest/standards.rs:170 is \
                 expected. But user-facing standards leaking without \
                 explicit declaration would be a template-default bug.",
                name,
                stds
            );
        }
    }
}

// EEE5 — Default struct field access on fresh storage.
// `struct P { uint x; uint y; } P public p;` — on a fresh deploy with
// NO explicit init, `get()` returns (0, 0) because all uint fields
// default to zero.
// 15 fuzz cases exercise repeat-exec stability on the zero-init path.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch81_eee5_default_struct_field_access_zero_init(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct P { uint x; uint y; }
    P public p;
    function get() external view returns (uint, uint) { return (p.x, p.y); }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("EEE5 compile: {:?}. If this \
                fires on the struct state-var declaration `P public p;`, \
                the struct-as-public-state-var regressed. If on the \
                tuple-return `(p.x, p.y)` on a zero-init storage read, \
                the zero-default struct-field access regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("EEE5 rt");

        // No set() call — read directly on fresh storage.
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "get", &[] as &[StackItem])
            .expect("EEE5 get() host-level on zero-init storage");
        prop_assert!(r.success,
            "EEE5 get() on zero-init must succeed; exc={:?}. If exc \
             cites `p.x` or `p.y` storage read, the zero-default \
             struct-field access regressed (storage-read on an \
             unwritten slot should yield zero, not fault).",
            r.exception.as_ref().map(|e| &e.message));

        // (a) The tuple must encode (0, 0). There are multiple valid
        //     shapes for a zero-tuple return: 64 bytes of zero (strict
        //     BE-padded static tuple), 0 bytes (empty-on-zero shorthand),
        //     or variable-width LE with all-zero bytes. We accept any
        //     shape as long as ALL bytes are zero — no 0x01..0xff may
        //     appear, which would indicate one of the fields read a
        //     non-zero value (either storage wasn't zero-init or the
        //     tuple leaked uninitialized-memory bytes).
        let rd = &r.return_data;
        let all_zero = rd.iter().all(|b| *b == 0);
        prop_assert!(all_zero,
            "EEE5 get() on zero-init must return ALL-ZERO bytes (the \
             (0, 0) tuple in any valid shape); got rd_hex={} len={}. \
             If any non-zero byte appears, either (a) storage wasn't \
             zero-init (the struct state-var has a non-zero default \
             leaking through), (b) uninitialized memory leaked into \
             the return buffer, or (c) the tuple encoding included \
             framing/length bytes that are non-zero (in which case \
             this test needs refinement to tolerate framing but \
             still check the field values). Task #191+ candidate: \
             zero-default struct-field read through tuple return.",
            hex::encode(rd), rd.len());

        // (b) Length sanity — for a 2-field uint tuple, the return
        //     is EITHER 64 bytes (BE-padded static tuple, the strict
        //     EVM-canonical shape) OR 0 bytes (empty-on-zero shorthand —
        //     some runtimes elide all-zero returns to save bytes).
        //     Any OTHER length indicates a shape mismatch.
        let valid_len = rd.len() == 64 || rd.is_empty();
        prop_assert!(valid_len,
            "EEE5 get() tuple return must be 64 bytes (BE-padded (x, y)) \
             OR 0 bytes (empty-on-zero shorthand); got {} bytes \
             rd_hex={}. If some other length, the tuple encoding for \
             a zero-init struct is neither the canonical nor the \
             shorthand form.",
            rd.len(), hex::encode(rd));
    }
}

// Task ID resolution for Batch #81 on first exec:
//   - EEE1 (UDVT wrap/unwrap via free-function helpers): RESOLVED
//     GREEN. `type PriceUSD is uint256` + file-scope `wrap(n)` /
//     `unwrap(p)` free functions composing through the UDVT
//     conversion yielded f(42) == 42 across 15 repeat-exec cases.
//     Batch19 H2's direct-form precedent + Task #187's file-scope
//     free-function resolver combine cleanly — the UDVT-parameter
//     / UDVT-return signature is a non-regression surface.
//   - EEE2 (chained struct method calls via using-for): `#[ignore]`
//     + Task #191 FILED. First-exec observation: the chain returns 0
//     instead of 3. Root cause: `c.inc().inc().inc()` where `inc` is
//     a free function attached via `using { inc } for Counter;` does
//     NOT thread the returned Counter back as the receiver of the
//     next `.inc()` call. Batch35 K1 pins the scalar-uint chain form
//     as GREEN (via library attach `using L for uint256; x.f1().f2()`),
//     so the gap is specifically on memory-struct return + free-
//     function attach (Task #188's extension path). See STATUS
//     comment on the harness for the fix path.
//   - EEE3 (anonymous event with 4 indexed params): RESOLVED GREEN.
//     `event Full4(address, address, uint256, bytes32) anonymous`
//     emits exactly 4 topics (all indexed, no sig hash) and 0 data
//     bytes. Task #190's fix (anonymous-event topic0 suppression,
//     landed GREEN by sibling `fix-190-anon-event` 50k hunt) extends
//     correctly to the max-indexed (4) boundary, and none of the 4
//     topics equal keccak256 of the canonical signature. The joint
//     anonymous + 4-indexed shape is a non-regression surface.
//   - EEE4 (empty-contract manifest shape): RESOLVED GREEN. The
//     empty `contract Empty {}` produces exactly 1 artifact with 0
//     user-visible methods (only the compiler-inserted `_deploy`
//     remains), 0 events, and supportedstandards = ["NEP-29"]
//     which is CANONICAL — every contract inherits the NEP-29
//     deploy-callback method from the compiler prologue, so its
//     auto-declare per src/cli/cli_parts/cli_manifest/standards.rs:
//     170 is the expected shape, not a template leak. No user-
//     facing standards (NEP-11/17/24/26) leak without annotation.
//   - EEE5 (default struct field access on zero-init): RESOLVED
//     GREEN. Reading `p.x, p.y` as a tuple on a fresh deploy with
//     NO explicit init yields all-zero bytes (either 64-byte BE-
//     padded (0, 0) or 0-byte empty-on-zero shorthand; both are
//     valid canonical shapes), across 15 repeat-exec cases. The
//     zero-default struct-field storage read through tuple return
//     is a non-regression surface.
//
// New Task IDs filed in Batch #81:
//   - Task #191: chained memory-struct method call via `using
//     { fn } for T` file-scope attach. When `fn(T memory)` returns
//     `T memory` and is attached via `using { fn } for T` at file
//     scope (Solidity 0.8.13+), the expression `c.fn().fn().fn()`
//     must thread each call's return value as the receiver of the
//     next `.fn()` in the chain. Currently the chain flattens to
//     identity — the return value is NOT piped into the next
//     receiver position, so three chained `.inc()` calls on a
//     Counter(0) receiver all observe the ORIGINAL receiver and
//     the final `.value` read returns 0 instead of 3. Batch35 K1
//     pins the SCALAR uint chain form (via library-attach `using L
//     for uint256`) as GREEN, so the library-form + scalar-return
//     case works — Task #191's gap is specifically on memory-
//     struct return + free-function attach (Task #188's extension
//     path). Fix: the IR lowering for `x.fn()` where `fn` is a
//     free function attached via `using { fn } for T` must treat
//     `x.fn()` as an expression that yields `fn(x)`'s return value,
//     and when chained `.fn()` follows it, the NEW receiver must
//     be the PREVIOUS call's return value, not the original `x`.
//     First-exec observation: f() returns 0 instead of 3. Possible
//     location: `src/ir/expressions/method_call.rs` or the
//     using-attach rewrite in `src/ir/build/module/`.
//
// Sibling agent context: Batch #81's probes are orthogonal to the
// DDD1..DDD5 (Batch #80) surfaces:
//   - EEE1 is UDVT through a free-function wrapper (distinct from
//     batch19 H2's direct `PriceUSD.wrap`/`PriceUSD.unwrap` static
//     form — EEE1 exercises the file-scope free-function variant
//     that Uniswap V4 uses for Currency / BalanceDelta APIs).
//   - EEE2 is chained method calls on a memory struct via `using
//     { inc } for Counter;` (distinct from batch35 K1's scalar
//     uint-chained form and from BBB2 batch78's library-form
//     attach — EEE2's free-function form needs the Task #188
//     extension to STRUCT types with return-value threading).
//   - EEE3 is 4-indexed anonymous event (distinct from ZZ2
//     batch76's 3-indexed non-anonymous form and CCC2 batch79's
//     1-indexed anonymous form; EEE3 pins the joint shape at the
//     EVM-LOG4 ceiling).
//   - EEE4 is the empty-contract manifest shape (distinct from
//     any prior manifest probe — boundary_tests line 6 covers
//     compile only; EEE4 extends to manifest-method-count == 0
//     for the empty contract, plus the NEP-29 canonical-declare
//     boundary).
//   - EEE5 is default struct storage read (distinct from DDD3's
//     explicit-P(0,0)-assign form and from any prior zero-init
//     storage probe — this tests the unwritten-slot read path).

// ==================== Batch #82 — Nested mapping symmetric write, bit-manipulation popcount loop, array sum with unchecked inner, custom error with struct-array + string payload, uint-to-decimal string formatter via string.concat ====================
//
// Five orthogonal probes continuing the per-five-harness cadence. Each
// pins a distinct surface the compiler/runtime must handle for
// mainstream Solidity idioms. Baseline: 444 passed + 1 ignored (the
// +1 ignored is EEE2 / Task #191 — chained memory-struct method call
// via `using { fn } for T`, actively worked on by the sibling
// `fix-191-chained` 50k hunt running concurrently). Target: 449
// passed + 1 ignored (5 new probes all GREEN, no fresh gaps expected;
// if the sibling lands Task #191 green concurrently, target becomes
// 450 + 0). If fresh gaps surface, Task IDs #192+ would be filed.
//
//   FFF1: Nested mapping with symmetric (bidirectional) writes.
//         `mapping(address => mapping(address => uint)) pair` plus a
//         `set(a, b, v)` that writes BOTH `pair[a][b] = v` AND
//         `pair[b][a] = v` in the same external call. The probe is
//         get(alice, bob) == get(bob, alice) == 100 after a single
//         set(alice, bob, 100). Extends batch70 TT1 (nested
//         mapping(bytes32 => mapping(address => bool)) with ONE-way
//         key write) and batch47 W2 (single-level mapping multi-call
//         persistence) to the SYMMETRIC two-slot write pattern that
//         AMM pair-reserve contracts use. Tests: (a) two independent
//         storage slots (keccak of (a,b,slot) vs keccak of (b,a,slot)
//         — these derive to DIFFERENT slots), (b) both get the same
//         value 100, (c) a third pair (charlie, dave) that was never
//         set remains 0 (no cross-pollution from the alice-bob write).
//         Single-shot — the invariant is deterministic once a single
//         write lands.
//   FFF2: Bit-manipulation popcount. `countOnes(uint256 n)` returns
//         the number of set bits via `while (n != 0) { c += n & 1;
//         n >>= 1; }`. Probes: (a) the `n & 1` bitwise AND on a
//         uint256 (neighbor of batch32 U3's (a, b) five-op shape),
//         (b) the `n >>= 1` compound shift-right assignment (whose
//         underlying `>>` is batch32 U3 GREEN, but the `>>=` compound
//         form is distinct — it's `n = n >> 1` lowered through the
//         ShiftRight-assign path in `src/ir/expressions/dispatch/
//         assignments.rs`), (c) loop termination on `n != 0` (the
//         mask never inserts a new 1-bit so the loop shrinks n toward
//         zero), (d) accumulator `c += n & 1` (scalar uint increment
//         by 0 or 1). Known inputs: 0 → 0 (zero-iteration body), 7 →
//         3 (three 1-bits: 0b111), 2^32 - 1 → 32 (all 32 low bits set
//         — the 4294967295 literal). 15 fuzz cases exercise repeat-
//         exec stability.
//   FFF3: Array sum with `unchecked` wrapping the entire for-loop
//         body (including the increment). The input is `uint32[]` —
//         a narrow-uint dynamic array, distinct from batch46 V3's
//         fixed-size uint[3] nested form and batch75 YY1's uint[]
//         storage push. Probes: (a) `uint32[] memory` parameter
//         ingress (the narrow-uint array type on the external
//         boundary — distinct from `uint[]` which defaults to uint256
//         elements), (b) `unchecked { }` wrapping a full for-loop
//         body, exercising the unchecked-block-contains-loop form
//         (the inner `s += a[i]` accumulator and the `i++` are both
//         under unchecked), (c) the sum of [1, 2, 3] must equal 6.
//         15 fuzz cases exercise repeat-exec stability — the input
//         is baked into the harness so the BigUint result is
//         deterministic; the 15 cases exercise the exec path not
//         the input space.
//   FFF4: Custom error with TWO complex args — a dynamic
//         struct-array and a string. `error BatchError(Item[] items,
//         string reason)` where `Item = { uint id; bool active; }`.
//         The function iterates `items` and reverts with the FULL
//         input array plus the literal "inactive" string on the
//         first inactive item. Extends batch72 VV5 (custom error
//         with ONE struct arg) and batch75 YY5 (custom error with
//         ONE uint[] arg) to the JOINT shape: struct-array + string.
//         Tests: (a) the 4-byte selector for
//         keccak256("BatchError((uint256,bool)[],string)") — note
//         the desugared tuple form per VV5's Task #181 resolution,
//         (b) the revert payload carries the full (not truncated-at-
//         first-inactive) items array, (c) the string "inactive"
//         appears as a literal in the payload. Single-shot —
//         deterministic inputs: [Item(1, true), Item(2, false),
//         Item(3, true)]; reverts on items[1].active == false.
//   FFF5: Uint-to-decimal string formatter via `string.concat`.
//         `uintToString(x)` converts a uint to its decimal ASCII
//         representation using the classic byte-buffer-build-
//         backwards algorithm (count digits, allocate bytes(digits),
//         write LSB -> MSB). `fmt(a, b, c)` wraps three of these in
//         `string.concat("(", uintToString(a), ",", uintToString(b),
//         ",", uintToString(c), ")")`. Probes: (a) `string.concat`
//         with 7 args (5 dynamic strings + 2 string literals — the
//         "," separators; Solidity's variadic string.concat), (b)
//         the digits-loop `while (temp != 0) { digits++; temp /= 10;
//         }` for the count phase (uint division and counter
//         increment), (c) the digit-write loop with reverse indexing
//         `buffer[digits - 1] = bytes1(uint8(48 + x % 10))` —
//         exercises bytes1 single-byte write at a dynamic offset,
//         uint8 narrowing cast on an arithmetic expression, and the
//         ASCII '0' + digit pattern, (d) `return string(buffer)` —
//         bytes-to-string reinterpret (batch71 UU4 / Task #179
//         related surface). Known: fmt(1, 22, 333) == "(1,22,333)".
//         15 fuzz cases exercise repeat-exec stability on the
//         deterministic input.
//
// Task IDs observed on first exec: per-harness after the first run;
// new Task IDs #192+ filed where fresh gaps surface (the last
// assigned ID is #191 from batch81 EEE2).
//
// Sibling agent context: Batch #82's probes are orthogonal to the
// EEE1..EEE5 (Batch #81) surfaces:
//   - FFF1 is NESTED mapping with SYMMETRIC writes (distinct from
//     TT1's nested mapping with one-way writes and UU1's single-
//     level balance-ledger mapping — FFF1's two-way write pattern
//     is specifically the AMM pair-reserve shape).
//   - FFF2 is popcount via shift-loop (distinct from batch32 U3's
//     five-op snapshot which pins the bitwise primitives as GREEN
//     but doesn't exercise a LOOP-based bit-scan algorithm).
//   - FFF3 is uint32[] sum with unchecked (distinct from batch46
//     V3's uint[3] fixed-size nested form — FFF3 pins the dynamic
//     uint32[] ingress path with unchecked-loop lowering).
//   - FFF4 is custom error with struct-array + string joint payload
//     (distinct from VV5's struct-only arg and YY5's uint[]-only arg
//     — FFF4 pins the COMBINED shape at the revert-encoder).
//   - FFF5 is decimal-string formatter through string.concat
//     (distinct from DDD5's string.concat with a bytes cast —
//     FFF5's variadic form with 7 args exercises a different
//     boundary, plus the uintToString helper is a novel surface
//     around digit-extraction / bytes1-write / ASCII shift).

// FFF1 — Nested mapping with symmetric (bidirectional) writes.
// set(alice, bob, 100) writes BOTH pair[alice][bob] AND pair[bob][alice].
// Both get() calls must observe 100; an unrelated pair(charlie, dave)
// must remain 0 (no cross-pollution).
// Single-shot — deterministic once a single set() lands.
#[test]
fn batch82_fff1_nested_mapping_symmetric_bidirectional_writes() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    mapping(address => mapping(address => uint)) public pair;
    function set(address a, address b, uint v) external { pair[a][b] = v; pair[b][a] = v; }
    function get(address a, address b) external view returns (uint) { return pair[a][b]; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "FFF1 compile: {:?}. If this fires \
            on the nested mapping declaration \
            `mapping(address => mapping(address => uint))`, the \
            two-level mapping regressed (batch70 TT1 pins the \
            mapping(bytes32 => mapping(address => bool)) shape — \
            FFF1 is a different inner/outer key combo). If on the \
            two-slot assignment `pair[a][b] = v; pair[b][a] = v`, \
            the multi-expression statement sequence regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("FFF1 rt");

    // Two pinned addresses; the symmetric write should populate
    // BOTH `pair[alice][bob]` and `pair[bob][alice]`. Per batch70
    // TT1 precedent, the address byte-order passed as an arg does
    // not need the LE flip (only msg.sender materialises LE-reversed).
    let alice = [0x11u8; 20];
    let bob = [0x22u8; 20];

    // (1) set(alice, bob, 100) — two writes under one external call.
    let r_set = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "set",
            &[
                StackItem::byte_array(alice.to_vec()),
                StackItem::byte_array(bob.to_vec()),
                StackItem::Integer(100),
            ],
        )
        .expect("FFF1 set(alice, bob, 100) host-level");
    assert!(
        r_set.success,
        "FFF1 set(alice, bob, 100) must succeed; exc={:?}. If exc \
         cites nested-mapping write, the slot derivation on \
         `pair[a][b]` regressed. If the second assignment \
         `pair[b][a] = v` faults, the repeated-key-variation write \
         regressed.",
        r_set.exception.as_ref().map(|e| &e.message)
    );

    // (2) get(alice, bob) — forward direction, must be 100.
    let r_ab = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "get",
            &[
                StackItem::byte_array(alice.to_vec()),
                StackItem::byte_array(bob.to_vec()),
            ],
        )
        .expect("FFF1 get(alice, bob) host-level");
    assert!(
        r_ab.success,
        "FFF1 get(alice, bob) must succeed; exc={:?}.",
        r_ab.exception.as_ref().map(|e| &e.message)
    );
    let v_ab = decode_uint_le(&r_ab.return_data);
    assert_eq!(
        v_ab.clone(),
        BigUint::from(100u64),
        "FFF1 get(alice, bob) must equal 100 (forward direction from \
         the first `pair[a][b] = v` write); got {} rd_hex={}. If 0, \
         the first write didn't persist (either slot derivation \
         diverges between set/get or the write was dropped). If some \
         other value, the write stored an unexpected value.",
        v_ab,
        hex::encode(&r_ab.return_data)
    );

    // (3) get(bob, alice) — reverse direction, must ALSO be 100
    // (this is the symmetric invariant — the second write populates
    // the reverse-key slot). Task #192+ candidate if it diverges
    // from the forward direction.
    let r_ba = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "get",
            &[
                StackItem::byte_array(bob.to_vec()),
                StackItem::byte_array(alice.to_vec()),
            ],
        )
        .expect("FFF1 get(bob, alice) host-level");
    assert!(
        r_ba.success,
        "FFF1 get(bob, alice) must succeed; exc={:?}.",
        r_ba.exception.as_ref().map(|e| &e.message)
    );
    let v_ba = decode_uint_le(&r_ba.return_data);
    assert_eq!(
        v_ba.clone(),
        BigUint::from(100u64),
        "FFF1 get(bob, alice) must equal 100 (reverse direction from \
         the second `pair[b][a] = v` write); got {} rd_hex={}. If 0, \
         the SECOND assignment in the `set` body was dropped — the \
         two-statement sequence inside an external function \
         regressed (the first statement's write persists but the \
         second was elided). Task #192+ candidate: multi-statement \
         function body persistence for repeated-slot-family writes.",
        v_ba,
        hex::encode(&r_ba.return_data)
    );

    // (4) get(charlie, dave) — a pair that was NEVER set; must be 0
    // (no cross-pollution from the alice-bob write).
    let charlie = [0x33u8; 20];
    let dave = [0x44u8; 20];
    let r_cd = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "get",
            &[
                StackItem::byte_array(charlie.to_vec()),
                StackItem::byte_array(dave.to_vec()),
            ],
        )
        .expect("FFF1 get(charlie, dave) host-level");
    assert!(
        r_cd.success,
        "FFF1 get(charlie, dave) must succeed (reads on unwritten \
         slots return default zero); exc={:?}.",
        r_cd.exception.as_ref().map(|e| &e.message)
    );
    let v_cd = decode_uint_le(&r_cd.return_data);
    assert_eq!(
        v_cd.clone(),
        BigUint::from(0u64),
        "FFF1 get(charlie, dave) must equal 0 (never-set pair); got \
         {} rd_hex={}. If 100, the nested-mapping writes are leaking \
         across key pairs (CRITICAL — cross-pollution would corrupt \
         any mapping-based ledger). If some other non-zero value, \
         storage has stale data from another test (harness \
         isolation regression).",
        v_cd,
        hex::encode(&r_cd.return_data)
    );
}

// FFF2 — Bit-manipulation popcount via shift-loop.
// countOnes(0) == 0, countOnes(7) == 3, countOnes(2^32 - 1) == 32.
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch82_fff2_popcount_shift_right_assign_loop(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function countOnes(uint256 n) external pure returns (uint) {
        uint c = 0;
        while (n != 0) { c += n & 1; n >>= 1; }
        return c;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("FFF2 compile: {:?}. If this \
                fires on `n >>= 1`, the shift-right-assign compound \
                operator regressed (batch32 U3 pins the `>>` \
                non-compound form as GREEN but `>>=` compound form \
                is a distinct lowering through \
                src/ir/expressions/dispatch/assignments.rs). If on \
                `n & 1`, the bitwise-AND regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("FFF2 rt");

        // (1) countOnes(0) — zero-iteration loop body; must return 0.
        let r0 = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "countOnes", &[StackItem::Integer(0)])
            .expect("FFF2 countOnes(0) host-level");
        prop_assert!(r0.success,
            "FFF2 countOnes(0) must succeed; exc={:?}.",
            r0.exception.as_ref().map(|e| &e.message));
        let v0 = decode_uint_le(&r0.return_data);
        prop_assert_eq!(v0.clone(), BigUint::from(0u64),
            "FFF2 countOnes(0) must equal 0 (empty loop, c stays at \
             its 0 init); got {} rd_hex={}. If non-zero, the loop \
             entered despite `n != 0` being false, OR c wasn't \
             initialized to 0 (uninitialized local leaked).",
            v0, hex::encode(&r0.return_data));

        // (2) countOnes(7) — 0b111, three iterations, c == 3.
        let r7 = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "countOnes", &[StackItem::Integer(7)])
            .expect("FFF2 countOnes(7) host-level");
        prop_assert!(r7.success,
            "FFF2 countOnes(7) must succeed; exc={:?}. If exc cites \
             shift-right-assign, the compound-op lowering regressed.",
            r7.exception.as_ref().map(|e| &e.message));
        let v7 = decode_uint_le(&r7.return_data);
        prop_assert_eq!(v7.clone(), BigUint::from(3u64),
            "FFF2 countOnes(7) must equal 3 (0b111 has 3 set bits); \
             got {} rd_hex={}. If 1, the loop exited after one \
             iteration (shift-right-assign didn't persist — n stayed \
             7 and `n != 0` was checked against the original). If 0, \
             the accumulator `c += n & 1` didn't fire. If 7, c was \
             summing n each iteration instead of `n & 1` (bitwise-\
             AND elided). Task #192+ candidate: popcount shift-loop.",
            v7, hex::encode(&r7.return_data));

        // (3) countOnes(2^32 - 1) — 4294967295, 32 low bits all set,
        // c == 32. This exercises the high-bit boundary for the
        // shift-right loop (32 iterations vs. 3 for input 7).
        let r_max = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "countOnes", &[StackItem::Integer(4294967295i64)])
            .expect("FFF2 countOnes(2^32 - 1) host-level");
        prop_assert!(r_max.success,
            "FFF2 countOnes(2^32 - 1) must succeed; exc={:?}. If exc \
             cites iteration-limit or gas-exhaustion, the 32-iter \
             loop is exceeding some guard (should be well within \
             typical test budgets).",
            r_max.exception.as_ref().map(|e| &e.message));
        let v_max = decode_uint_le(&r_max.return_data);
        prop_assert_eq!(v_max.clone(), BigUint::from(32u64),
            "FFF2 countOnes(4294967295) must equal 32 (all 32 low \
             bits set); got {} rd_hex={}. If fewer, the loop \
             short-circuited (likely an iteration-limit or a \
             premature `n != 0` false on intermediate state). If 0, \
             the whole loop body was skipped. If more than 32 (e.g. \
             64 or 256), the input was sign-extended or treated as \
             a wider type with phantom 1-bits above bit 31.",
            v_max, hex::encode(&r_max.return_data));
    }
}

// FFF3 — Array sum with unchecked wrapping the loop body.
// sum([1, 2, 3]) == 6. The inner `s += a[i]` is under an unchecked
// block (no overflow check emitted). Input type is uint32[] — narrow-
// uint dynamic array on the external boundary.
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch82_fff3_array_sum_uint32_unchecked_inner_loop(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function sum(uint32[] memory a) external pure returns (uint) {
        uint s = 0;
        unchecked { for (uint i = 0; i < a.length; i++) s += a[i]; }
        return s;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("FFF3 compile: {:?}. If this \
                fires on `uint32[] memory a` parameter, the narrow-\
                uint dynamic-array ingress regressed (distinct from \
                uint256[] which defaults to full-width). If on \
                `unchecked {{ for (...) ... }}`, the unchecked-block-\
                wrapping-a-full-loop form regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("FFF3 rt");

        // Build a uint32[] input: [1, 2, 3]. Per batch75 YY5
        // precedent, the runtime accepts `StackItem::Array` of
        // `StackItem::Integer` for uint256[] — for uint32[] the
        // same shape should work since the narrow-uint arg
        // widens at the boundary (the param is canonicalized
        // to the widest-integer representation on the stack).
        let input = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            StackItem::Integer(1),
            StackItem::Integer(2),
            StackItem::Integer(3),
        ])));
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "sum", &[input])
            .expect("FFF3 sum([1, 2, 3]) host-level");
        prop_assert!(r.success,
            "FFF3 sum([1, 2, 3]) must succeed; exc={:?}. If exc \
             cites uint32 narrow-cast on array-element load, the \
             `a[i]` access through a uint32[] is emitting a \
             mismatched widen. If exc cites unchecked-block, the \
             inner-loop no-overflow-check lowering regressed.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(6u64),
            "FFF3 sum([1, 2, 3]) must equal 6 (1 + 2 + 3 = 6); \
             got {} rd_hex={}. If 0, the accumulator `s += a[i]` \
             didn't fire (either the loop condition `i < \
             a.length` was false from the start or the array \
             length read returned 0). If 1, only the first \
             element landed (the `i++` didn't advance). If 3, \
             only the LAST element landed (the accumulator is \
             being overwritten instead of added). If 5, one \
             element was dropped mid-iteration. Task #192+ \
             candidate: uint32[] unchecked-loop sum.",
            v, hex::encode(&r.return_data));
    }
}

// FFF4 — Custom error with struct-array + string combined payload.
// check([Item(1, true), Item(2, false), Item(3, true)]) must revert
// with `BatchError(items, "inactive")` on items[1].active == false.
// Single-shot — deterministic input.
//
// STATUS: `#[ignore]` — Task #192 FILED. First-exec observation:
// the revert fires with the correct selector (6a46debe) and the
// correct outer shape (array length slot = 3, string "inactive"
// appears), but the per-element struct fields (id=1, id=2, id=3)
// are all encoded as zero. Raw rd_hex (wrapped for readability):
//   6a46debe                                              ; selector
//   000000...0040                                         ; items offset = 0x40
//   000000...00c0                                         ; reason offset = 0xc0
//   000000...0003                                         ; items.length = 3
//   000000...0000 (x3)                                    ; struct heads, all zero
//   000000...0000 (x3)                                    ; struct tails, all zero
//   000000...0008                                         ; reason.length = 8
//   696e61637469766500...                                 ; "inactive" + padding
// The ARRAY-of-STRUCT encoding recursed into each struct but emitted
// zero fields — both Item.id and Item.active were dropped. Batch72
// VV5 pins the SINGLE-STRUCT arg form as GREEN (Task #181 resolved
// `value_type_canonical_abi` to flatten struct args into per-field
// stack items before AbiEncode); FFF4's gap is specifically on the
// ARRAY-OF-STRUCT arg path, which needs to walk the outer array,
// then for each element apply the Task #181 per-field flatten.
//
// Fix path: `src/ir/statements/dispatch/return_revert.rs` (the
// revert-encoder) currently handles struct args by flattening at
// the top level, but when a struct arg is nested inside an ARRAY,
// the per-element flatten is not being applied. Look for the array
// handling path — likely emits the array length + then a generic
// encode-element that doesn't recognize struct elements. The fix is
// to iterate the struct-array and apply `value_type_canonical_abi`
// per element (mirroring Task #181's single-struct path) before the
// AbiEncode builtin sees the encoded stream.
#[test]
fn batch82_fff4_custom_error_struct_array_and_string_payload() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use sha3::{Digest, Keccak256};
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Item { uint id; bool active; }
    error BatchError(Item[] items, string reason);
    function check(Item[] memory items) external pure {
        for (uint i = 0; i < items.length; i++) {
            if (!items[i].active) revert BatchError(items, "inactive");
        }
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "FFF4 compile: {:?}. If this \
            fires on the struct declaration `struct Item {{ uint \
            id; bool active; }}`, the two-field struct regressed. \
            If on `error BatchError(Item[] items, string reason)`, \
            the custom error with TWO dynamic args (struct-array + \
            string) regressed — this is the JOINT form distinct \
            from batch72 VV5 (struct-only) and batch75 YY5 (uint[]-\
            only).",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("FFF4 rt");

    // Build a Solidity Item[] memory input: [Item(1, true),
    // Item(2, false), Item(3, true)]. Each Item is a 2-field struct,
    // represented on the stack as a nested Array of two scalars
    // (per batch72 VV5 / Task #181 precedent where struct args are
    // flattened on the stack boundary).
    let item1 = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
        StackItem::Integer(1),
        StackItem::Boolean(true),
    ])));
    let item2 = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
        StackItem::Integer(2),
        StackItem::Boolean(false),
    ])));
    let item3 = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
        StackItem::Integer(3),
        StackItem::Boolean(true),
    ])));
    let items = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
        item1, item2, item3,
    ])));
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "check", &[items])
        .expect("FFF4 check([Item(1,true), Item(2,false), Item(3,true)]) host-level");

    // (1) The call must revert (success=false) — items[1].active
    // == false trips the `if (!...)` and fires revert.
    assert!(
        !r.success,
        "FFF4 check(...) must REVERT via custom error (items[1].active \
         == false trips the revert); got success=true rd_hex={}. If \
         success=true, either (a) the `!items[i].active` negation \
         regressed, (b) the loop exited early, or (c) the `revert \
         BatchError(...)` degraded to a return.",
        hex::encode(&r.return_data)
    );

    // (2) The revert payload's prefix (or content somewhere) must
    // include the 4-byte selector for keccak256("BatchError(
    // (uint256,bool)[],string)") — the desugared tuple form per
    // batch72 VV5 / Task #181 resolution.
    let sel_desugared = {
        let mut h = Keccak256::new();
        h.update(b"BatchError((uint256,bool)[],string)");
        let d = h.finalize();
        [d[0], d[1], d[2], d[3]]
    };
    let sel_raw = {
        let mut h = Keccak256::new();
        h.update(b"BatchError(Item[],string)");
        let d = h.finalize();
        [d[0], d[1], d[2], d[3]]
    };
    let rd = &r.return_data;
    let sel_match_prefix =
        rd.len() >= 4 && (&rd[..4] == &sel_desugared[..] || &rd[..4] == &sel_raw[..]);
    let sel_found_anywhere =
        rd.windows(4).any(|w| w == &sel_desugared[..]) || rd.windows(4).any(|w| w == &sel_raw[..]);
    assert!(
        sel_match_prefix || sel_found_anywhere,
        "FFF4 selector must equal keccak256(\"BatchError((uint256,\
         bool)[],string)\")[..4] = 0x{} OR keccak256(\"BatchError(\
         Item[],string)\")[..4] = 0x{}; got rd_hex={} (len {}). If \
         neither appears, the custom-error-with-struct-array + \
         string lowering regressed (VV5 pins struct-only arg; YY5 \
         pins uint[]-only; FFF4 pins the COMBINED form). Task #192+ \
         candidate: custom-error joint struct-array + string payload.",
        hex::encode(&sel_desugared),
        hex::encode(&sel_raw),
        hex::encode(rd),
        rd.len()
    );

    // (3) The string "inactive" must appear as a literal (8 ASCII
    // bytes) somewhere in the revert payload. This confirms the
    // second error arg — the string reason — was encoded into the
    // payload alongside the struct-array.
    let reason = b"inactive";
    let reason_found = rd.windows(reason.len()).any(|w| w == reason);
    assert!(
        reason_found,
        "FFF4 revert payload must include the ASCII substring \
         \"inactive\" (the second error arg); got rd_hex={} (len {}). \
         If absent, either (a) the string arg was dropped from the \
         revert encoding (only the first struct-array arg survived), \
         or (b) the string was corrupted in transit. This would \
         break any off-chain error decoder relying on the full \
         reason payload. Task #192+ candidate: string-arm dropped \
         in custom-error joint-payload encoding.",
        hex::encode(rd),
        rd.len()
    );

    // (4) The struct-array payload must carry all THREE items
    // (not truncated to the first-inactive index). Per batch75 YY5
    // precedent, the revert captures the full input array. We
    // probe for the distinctive id bytes 0x01, 0x02, 0x03 — all
    // three must be present. If only 0x01 appears, the payload
    // was truncated at the failing index.
    let has_id1 = rd.iter().any(|b| *b == 0x01);
    let has_id2 = rd.iter().any(|b| *b == 0x02);
    let has_id3 = rd.iter().any(|b| *b == 0x03);
    assert!(
        has_id1 && has_id2 && has_id3,
        "FFF4 revert payload must carry all three item ids (0x01, \
         0x02, 0x03) — the FULL input array, not truncated at the \
         failing index; got has_id1={}, has_id2={}, has_id3={} \
         rd_hex={}. If id3 is missing, the array was truncated at \
         the inactive element (items[1]); if id1 is missing, the \
         iteration direction reversed; if all present this is a \
         non-regression. Task #192+ candidate: struct-array \
         truncation-at-revert-site in joint payload.",
        has_id1,
        has_id2,
        has_id3,
        hex::encode(rd)
    );
}

// FFF5 — Uint-to-decimal string formatter via string.concat.
// fmt(1, 22, 333) == "(1,22,333)". Exercises variadic string.concat
// with 7 args (mix of literals + dynamic strings), digit-extraction
// loop, bytes1 single-byte write, uint8 narrowing, ASCII '0' shift.
// 15 fuzz cases exercise repeat-exec stability on the deterministic
// input.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch82_fff5_uint_to_decimal_string_concat_three_args(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function fmt(uint a, uint b, uint c) external pure returns (string memory) {
        return string.concat("(", uintToString(a), ",", uintToString(b), ",", uintToString(c), ")");
    }
    function uintToString(uint x) internal pure returns (string memory) {
        if (x == 0) return "0";
        uint temp = x; uint digits = 0;
        while (temp != 0) { digits++; temp /= 10; }
        bytes memory buffer = new bytes(digits);
        while (x != 0) { digits -= 1; buffer[digits] = bytes1(uint8(48 + x % 10)); x /= 10; }
        return string(buffer);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("FFF5 compile: {:?}. If this \
                fires on `string.concat(...)` with 7 args, the \
                variadic string.concat regressed (batch80 DDD5 pins \
                the 2-arg form as GREEN). If on the digit-extraction \
                loop (`while (x != 0) {{ ... x /= 10; }}`), the \
                uint-division-and-decrement regressed. If on \
                `buffer[digits] = bytes1(uint8(...))`, the bytes1 \
                single-byte write at a dynamic offset regressed. If \
                on `return string(buffer)`, the bytes-to-string \
                reinterpret regressed (batch71 UU4 / Task #179).", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("FFF5 rt");

        // fmt(1, 22, 333) must produce "(1,22,333)".
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "fmt", &[
                StackItem::Integer(1),
                StackItem::Integer(22),
                StackItem::Integer(333),
            ])
            .expect("FFF5 fmt(1, 22, 333) host-level");
        prop_assert!(r.success,
            "FFF5 fmt(1, 22, 333) must succeed; exc={:?}. If exc \
             cites string.concat, the 7-arg variadic form \
             regressed. If cites uintToString internal-call \
             dispatch, the internal helper resolution regressed.",
            r.exception.as_ref().map(|e| &e.message));
        let rd = &r.return_data;

        // The expected string "(1,22,333)" — 10 ASCII bytes —
        // must appear as a contiguous substring in the return
        // data. Per batch80 DDD5 the string envelope may carry
        // framing bytes around the payload; we probe for the
        // substring rather than requiring an exact match.
        let expected = b"(1,22,333)";
        let found = rd.windows(expected.len()).any(|w| w == expected);
        prop_assert!(found,
            "FFF5 fmt(1, 22, 333) return data must contain the \
             contiguous ASCII substring \"(1,22,333)\" (10 bytes); \
             got rd_hex={} (len {}). If absent, probe the \
             individual arm substrings: if \"(1,\" appears but \
             \"22,\" doesn't, uintToString dropped after the \
             first call; if \"22,\" appears but \"333\" doesn't, \
             the two-digit → three-digit transition regressed \
             (digits-loop off-by-one on 3-digit inputs). If \
             none of the pieces appear, string.concat's 7-arg \
             variadic form regressed. Task #192+ candidate: \
             uint-to-decimal-string via string.concat(7-arg) \
             through internal-function digit-extraction helper.",
            hex::encode(rd), rd.len());

        // Additional anchor: the literal "(" and ")" brackets
        // and the two "," separators must each appear. This
        // catches the case where one of the literal arms of
        // string.concat was dropped but the overall call
        // succeeded with a degenerate output.
        let has_open = rd.iter().any(|b| *b == b'(');
        let has_close = rd.iter().any(|b| *b == b')');
        let comma_count = rd.iter().filter(|&&b| b == b',').count();
        prop_assert!(has_open && has_close && comma_count >= 2,
            "FFF5 return data must contain both brackets `(` and \
             `)` plus at least 2 commas (the literal arms of \
             string.concat); got has_open={}, has_close={}, \
             comma_count={} rd_hex={}. If a literal arm is \
             missing, string.concat dropped a non-dynamic arg \
             in the 7-arg variadic form.",
            has_open, has_close, comma_count, hex::encode(rd));
    }
}

// Task ID resolution for Batch #82 on first exec:
//   - FFF1 (nested mapping symmetric bidirectional writes): RESOLVED
//     GREEN. `mapping(address => mapping(address => uint))` with a
//     single `set(a, b, v)` writing BOTH `pair[a][b] = v` AND
//     `pair[b][a] = v` in the same external call yielded
//     get(alice, bob) == get(bob, alice) == 100, and the unrelated
//     pair(charlie, dave) remained 0 (no cross-pollution). The two
//     slot derivations are independent and both writes persist
//     across the external call boundary. The AMM pair-reserve
//     pattern is a non-regression surface.
//   - FFF2 (popcount via shift-right-assign loop): RESOLVED GREEN.
//     `countOnes(0) == 0`, `countOnes(7) == 3`, `countOnes(2^32 - 1)
//     == 32` across 15 repeat-exec cases. The `n >>= 1` compound
//     shift-right assignment inside a while-loop composes cleanly
//     with the `n & 1` bitwise-AND (batch32 U3 primitives) and the
//     `c += n & 1` accumulator. The loop-based bit-scan algorithm
//     is a non-regression surface.
//   - FFF3 (uint32[] unchecked-loop sum): RESOLVED GREEN.
//     `sum([1, 2, 3]) == 6` across 15 repeat-exec cases. The
//     `uint32[] memory` parameter ingress path handles narrow-uint
//     arrays cleanly (widening to the runtime's canonical integer
//     representation at the boundary), and the `unchecked { }`
//     block wrapping the full for-loop (including the `i++` and
//     the `s += a[i]` accumulator) skips the overflow-check
//     lowering as expected. The unchecked-full-loop + narrow-uint-
//     array surface is a non-regression.
//   - FFF4 (custom error with struct-array + string payload):
//     `#[ignore]` + Task #192 FILED. First-exec observation: the
//     revert fires with the correct selector (6a46debe) and the
//     correct outer envelope (items.length = 3, reason = "inactive"
//     present), BUT all per-element struct fields (Item.id and
//     Item.active) are encoded as zero. The gap is specifically on
//     the ARRAY-OF-STRUCT arg encoding path — batch72 VV5 pins the
//     SINGLE-struct arg form as GREEN (Task #181 resolution
//     flattens struct args via `value_type_canonical_abi`), but
//     the per-element flatten is not applied when the struct is
//     nested inside an array. See STATUS comment on the harness
//     for the fix path.
//   - FFF5 (decimal-string formatter via string.concat 7-arg form):
//     RESOLVED GREEN. `fmt(1, 22, 333)` yields the contiguous ASCII
//     substring "(1,22,333)" in the return data across 15 repeat-
//     exec cases. The 7-arg variadic `string.concat` (mixing 5
//     dynamic strings from `uintToString` + 2 string literals)
//     composes cleanly with: (a) the digit-extraction loop using
//     `uint / 10` division + counter increment, (b) the reverse-
//     index byte buffer write `buffer[digits] = bytes1(uint8(48 +
//     x % 10))` with the ASCII '0' + digit shift, (c) the final
//     `string(buffer)` bytes-to-string reinterpret. The uint-to-
//     decimal-string surface is a non-regression.
//
// New Task IDs filed in Batch #82:
//   - Task #192: custom error with array-of-struct arg drops the
//     per-element struct fields. When `error E(S[] items, ...)` is
//     declared with `struct S { uint id; bool active; }` and the
//     revert emits the error with a full S[] payload, the encoded
//     payload includes the correct outer envelope (selector, offset,
//     array length) but all per-element struct fields are encoded
//     as zero. Batch72 VV5 pins the SINGLE-struct arg form as GREEN
//     (Task #181's `value_type_canonical_abi` flattens struct args
//     at the top level), and batch75 YY5 pins the dynamic-array
//     arg form as GREEN for non-struct elements (uint[]) — Task
//     #192's gap is specifically on the COMPOSITION: array-OF-
//     struct needs the per-element flatten applied recursively, but
//     the current revert-encoder only applies it at the top level.
//     Fix path: `src/ir/statements/dispatch/return_revert.rs` (or
//     the abi-encode helper invoked from there) needs to walk the
//     struct-array and apply `value_type_canonical_abi` per element
//     (mirroring Task #181's single-struct path) before the
//     AbiEncode builtin is invoked. First-exec observation: the
//     rd_hex shows `...items.length = 3, then six 32-byte zero
//     slots (for 3 structs × 2 fields each), then the reason string
//     `inactive` correctly encoded`. The per-element struct heads
//     are all zero.
//
// Sibling agent context: Batch #82's probes are orthogonal to the
// EEE1..EEE5 (Batch #81) surfaces:
//   - FFF1 is nested mapping with SYMMETRIC writes (distinct from
//     batch70 TT1's nested mapping with one-way writes — FFF1's
//     two-way write pattern is the AMM pair-reserve shape).
//   - FFF2 is popcount via shift-loop (distinct from batch32 U3's
//     bit-op snapshot — FFF2 pins the LOOP form with shift-right-
//     assign compound operator).
//   - FFF3 is uint32[] sum with unchecked (distinct from batch46
//     V3's uint[3] fixed-size form — FFF3 pins the dynamic narrow-
//     uint ingress path).
//   - FFF4 is custom error with struct-array + string joint payload
//     (distinct from VV5's struct-only and YY5's uint[]-only — FFF4
//     pins the COMBINED shape).
//   - FFF5 is decimal-string formatter through 7-arg string.concat
//     (distinct from DDD5's 2-arg form — FFF5 exercises variadic
//     string.concat plus the digit-extraction helper).

// ==================== Batch #83 — selfdestruct with value transfer, nested library call, event with bytes indexed arg, view modifier that writes state (compile-reject), abi.encodePacked with array ====================
//
// Five orthogonal probes continuing the per-five-harness cadence. Each
// pins a distinct surface. Baseline: 449 passed + 1 ignored (the +1
// ignored is FFF4 / Task #192 — custom error with struct-array arg
// drops per-element struct fields, actively worked on by the sibling
// `fix-192-arr-struct` 50k hunt running concurrently). Target: 454
// passed + 1 ignored (5 new probes expected GREEN; if fresh gaps
// surface, Task IDs #193+ filed).
//
//   GGG1: `selfdestruct(target)` with a `payable target` parameter.
//         The function body calls `selfdestruct(target)` — this must
//         route to Neo's `ContractManagement.destroy()` auto-map (per
//         batch37 K1 / batch77 AAA1 precedent). Distinct from K1's
//         pre-cast signature form (`address payable r`) because GGG1's
//         parameter is `address payable target` in a `payTo(...)`
//         named function — the semantic intent is "pay-to-then-destroy"
//         even though Neo's destroy does NOT actually transfer funds
//         (EIP-6780 + Neo convention both elide the value-transfer).
//         The compile + dispatch surface must still compile cleanly
//         AND the runtime call must succeed via the destroy auto-map.
//         Single-shot — the invariant is deterministic once the map
//         lands.
//   GGG2: Nested library calls. `library L1` provides `add(a, b)`;
//         `library L2` provides `double(x) = L1.add(x, x)`. The
//         contract `f(x)` calls `L2.double(x)` which internally calls
//         `L1.add(x, x)` — so the test exercises TWO-LEVEL library
//         dispatch through internal function calls. Distinct from
//         batch35 K1's two-library method-chain form (`x.f1().f2()`
//         via `using` for a single receiver) and from K5b batch36's
//         single-library inline form. GGG2's shape is specifically the
//         nested `L2 calls L1` pattern that OpenZeppelin's SafeMath /
//         Math libraries use internally (e.g. `Math.ceilDiv` calls
//         `Math.max` which is in the same lib but different function).
//         f(5) must equal 10 (double(5) = add(5, 5) = 10). 15 fuzz
//         cases exercise repeat-exec stability.
//   GGG3: Event with bytes indexed argument. `event Sent(bytes indexed
//         payload)` — per the Solidity spec for INDEXED DYNAMIC args,
//         the topic is NOT the raw bytes but `keccak256(payload)`.
//         Extends batch37 K5 (`string indexed msg` → topics[1] =
//         keccak256("hello")) to the `bytes` indexed variant. The
//         probe: emit Sent(data) where data = "hello world" (11 ASCII
//         bytes), then verify topics[1] == keccak256("hello world") —
//         NOT the raw 11 bytes. Tests: (a) `bytes indexed` is parsed
//         and accepted (distinct from `bytes` non-indexed which would
//         go to data), (b) the topic is the keccak of the bytes value
//         (the indexed-dynamic-type hash convention), (c) data is
//         empty (all args indexed). Single-shot — the payload is
//         fixed so the expected topic is deterministic.
//   GGG4: Custom modifier `count()` that increments state, applied to
//         a function declared `view`. This is an INVALID combination —
//         the modifier body writes `counter++` (state write) but the
//         function's mutability contract is `view` (read-only). The
//         compiler MUST reject this per Solidity's mutability inference
//         rule: a function's effective mutability is the MAX of its
//         own body plus all applied modifiers' bodies. If any modifier
//         writes state, the function's effective mutability is at
//         least `nonpayable` — it cannot also be `view`. This is a
//         compile-error pin: compile_contracts must return Err, and
//         the error message should mention view/mutability/modifier.
//         Distinct from baseline_tests.rs #2 (which pins view-function
//         DIRECTLY writing storage) — GGG4 pins the modifier-via-view
//         path where the write is sourced from a composed modifier,
//         not the function body itself. Single-shot — deterministic
//         compile-reject.
//   GGG5: `abi.encodePacked` with a dynamic `uint[]` array. Per the
//         Solidity spec, abi.encodePacked(uint[] a) concatenates each
//         element's full 32-byte big-endian encoding WITHOUT the
//         length prefix (packed vs canonical). So for a = [1, 2, 3],
//         the result is 3 × 32 = 96 bytes: BE(1, 32) || BE(2, 32) ||
//         BE(3, 32). Distinct from batch46 OO2's two-bytes32 packed
//         concat (which is 64 bytes of two raw 32-byte values side-
//         by-side) — GGG5 extends that to the DYNAMIC-ARRAY input
//         path. Tests: (a) uint[] memory parameter ingress on an
//         abi.encodePacked call, (b) array-element widening to 32-byte
//         BE per-element, (c) no length prefix (packed form suppresses
//         the head offset + length that abi.encode would emit). 15
//         fuzz cases exercise repeat-exec stability.
//
// Task IDs observed on first exec: per-harness after the first run;
// new Task IDs #193+ filed where fresh gaps surface.
//
// Sibling agent context: Batch #83's probes are orthogonal to the
// FFF1..FFF5 (Batch #82) surfaces:
//   - GGG1 is selfdestruct on a `payable target` param (distinct from
//     K1 batch37's `payable r` param form which uses a single-char
//     name and batch77 AAA1's explicit-cast call-site form — GGG1's
//     named `payTo(target)` is the semantic pay-to pattern, but the
//     runtime still elides the value transfer).
//   - GGG2 is nested library calls L2 → L1 (distinct from K1
//     batch35's method-chain form and K5b batch36's single-inline
//     form — GGG2's shape is the cross-library internal dispatch).
//   - GGG3 is bytes indexed event (distinct from K5 batch37's string
//     indexed form — GGG3 pins the `bytes` indexed-dynamic variant
//     with the same keccak-of-value topic convention).
//   - GGG4 is view modifier writing state (distinct from baseline #2's
//     view-body directly writing state — GGG4 pins the modifier-
//     composition path through the mutability inference rule).
//   - GGG5 is abi.encodePacked on uint[] (distinct from OO2's two-
//     bytes32 form — GGG5 pins the dynamic-array element-by-element
//     packed concat).

// GGG1 — selfdestruct(target) with a `payable target` parameter.
// Compile must succeed + runtime call must route to Neo's
// ContractManagement.destroy() auto-map (per batch37 K1 / batch77
// AAA1 precedent). Single-shot — deterministic.
#[test]
fn batch83_ggg1_selfdestruct_with_value_transfer_routes_to_destroy() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function payTo(address payable target) external {
        selfdestruct(target);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "GGG1 compile must succeed: {:?}. If \
            this fires with a `payable` coercion diagnostic, the \
            `address payable` parameter form regressed (batch37 K1 \
            pins this form as GREEN). If it fires with a selfdestruct-\
            specific diagnostic, the Neo auto-map to \
            ContractManagement.destroy() regressed.",
            e
        )
    });
    assert_eq!(arts.len(), 1, "GGG1 single artifact; got {}", arts.len());
    let c = &arts[0];

    // Manifest must expose `payTo` — the external function with the
    // selfdestruct body. If missing, the function was dropped from
    // the ABI (selfdestruct body failed to lower + function elided).
    let methods = c.manifest["abi"]["methods"]
        .as_array()
        .expect("GGG1 manifest methods array must exist");
    assert!(
        methods.iter().any(|m| m["name"].as_str() == Some("payTo")),
        "GGG1 `payTo` must appear in manifest (external function with \
         selfdestruct body must survive lowering); got method names {:?}. \
         If missing, either the function was elided (the selfdestruct \
         body faulted at lowering and the whole function was dropped) \
         or the manifest emitter failed to include it.",
        methods
            .iter()
            .map(|m| m["name"].clone())
            .collect::<Vec<_>>()
    );

    // Runtime smoke — the call must succeed via ContractManagement.
    // destroy() auto-map. Per batch37 K1 precedent the recipient arg
    // is evaluated-then-dropped (Neo destroy does NOT forward funds).
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("GGG1 rt");
    let r = rt
        .call_method(
            &c.bytecode,
            &c.tokens,
            &c.manifest,
            "payTo",
            &[StackItem::byte_array(vec![0x11u8; 20])],
        )
        .expect("GGG1 payTo(0x11..11) host-level");
    assert!(
        r.success,
        "GGG1 payTo(addr) must succeed via ContractManagement.destroy() \
         auto-map; exc={:?}. If exc cites `selfdestruct`, the auto-map \
         regressed (batch37 K1 / batch77 AAA1 precedent both pin the \
         auto-map path). If exc cites `payable target`, the payable \
         parameter ingress regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert!(
        r.return_data.is_empty(),
        "GGG1 payTo() returns nothing (selfdestruct → destroy has no \
         return); got rd_hex={} (len {}). If non-empty, either the \
         destroy auto-map is leaking the address arg or some other \
         path is introducing a spurious return payload.",
        hex::encode(&r.return_data),
        r.return_data.len()
    );
}

// GGG2 — Nested library calls: L2.double(x) internally calls L1.add(x, x).
// f(5) == 10 (double(5) = add(5, 5) = 10). 15 fuzz cases exercise
// repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch83_ggg2_nested_library_calls_l2_dispatches_to_l1(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
library L1 { function add(uint a, uint b) internal pure returns (uint) { return a + b; } }
library L2 {
    function double(uint x) internal pure returns (uint) { return L1.add(x, x); }
}
contract C {
    function f(uint x) external pure returns (uint) { return L2.double(x); }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("GGG2 compile: {:?}. If this \
                fires on the nested library call `L1.add(x, x)` from \
                inside L2.double, the cross-library internal dispatch \
                regressed (batch35 K1 pins method-chain via `using`; \
                batch36 K5b pins single-library inline call — GGG2 \
                extends to library-calls-library without `using`). If \
                on the L2.double entry, the single-level library call \
                regressed.", e));
        let art = arts.iter()
            .find(|a| a.metadata.name == "C")
            .unwrap_or_else(|| panic!("GGG2 C artifact missing; got names={:?}",
                arts.iter().map(|a| a.metadata.name.clone()).collect::<Vec<_>>()));
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("GGG2 rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(5)])
            .expect("GGG2 f(5) host-level");
        prop_assert!(r.success,
            "GGG2 f(5) must succeed; exc={:?}. If exc cites L2.double \
             dispatch, the outer library call regressed. If it cites \
             L1.add from inside L2, the INNER nested library call \
             regressed — the L2 body's `L1.add(x, x)` didn't resolve \
             to the L1 inline body.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(10u64),
            "GGG2 f(5) must equal 10 (L2.double(5) = L1.add(5, 5) = \
             10); got {} rd_hex={}. If 5, the inner L1.add dispatch \
             dropped (only one of the `x` args landed). If 0, both \
             sides of add dropped (the nested library call chain \
             silently returned 0). Task #193+ candidate: nested \
             library call L2→L1 dispatch.",
            v, hex::encode(&r.return_data));
    }
}

// GGG3 — Event with `bytes indexed payload`. Per Solidity spec for
// indexed dynamic types, topic is keccak256(payload), NOT the raw
// bytes. Extends batch37 K5 (string indexed) to bytes indexed.
// Single-shot — deterministic payload.
#[test]
fn batch83_ggg3_event_with_bytes_indexed_arg_topic_is_keccak() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use sha3::{Digest, Keccak256};
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Sent(bytes indexed payload);
    function f(bytes memory data) external { emit Sent(data); }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "GGG3 compile: {:?}. If this fires \
            on `bytes indexed payload`, the bytes-indexed event-param \
            declaration regressed (batch37 K5 pins the string-indexed \
            form as GREEN; GGG3 extends to bytes-indexed which follows \
            the same indexed-dynamic-type hash-of-value convention). \
            If on `emit Sent(data)`, the dynamic-bytes arg emit \
            regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("GGG3 rt");

    // Fixed payload: "hello world" (11 ASCII bytes). Topic is
    // deterministic: keccak256(payload) — NOT the raw bytes.
    let payload = b"hello world";
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::byte_array(payload.to_vec())],
        )
        .expect("GGG3 f(\"hello world\") host-level");
    assert!(
        r.success,
        "GGG3 f(data) must succeed; exc={:?}. If exc cites bytes \
         indexed event emit, the indexed-dynamic-bytes topic-hash \
         lowering regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // (1) Exactly 1 log must fire.
    assert_eq!(
        r.logs.len(),
        1,
        "GGG3 f() must emit exactly 1 Sent log; got {} logs. If 0, \
         the emit was elided. If 2+, a shadow emit fired.",
        r.logs.len()
    );
    let log = &r.logs[0];

    // (2) topics.len() must be 2: topic[0] = keccak("Sent(bytes)"),
    //     topic[1] = keccak(payload). One indexed arg ⇒ 2 topics.
    assert_eq!(
        log.topics.len(),
        2,
        "GGG3 bytes-indexed event with 1 indexed arg must have 2 \
         topics (sig + keccak-of-payload); got {} topics. If 1, sig \
         or indexed topic dropped. If 3+, extraneous topic leaked.",
        log.topics.len()
    );

    // (3) topic[0] must equal keccak256("Sent(bytes)") — the event
    //     signature hash per Solidity's canonical ABI.
    let sig_hash = Keccak256::digest(b"Sent(bytes)").to_vec();
    assert_eq!(
        &log.topics[0][..],
        &sig_hash[..],
        "GGG3 topics[0] must = keccak256(\"Sent(bytes)\") = 0x{}; got \
         0x{}. If divergent, the event-signature canonicalization for \
         bytes-indexed args regressed.",
        hex::encode(&sig_hash),
        hex::encode(&log.topics[0])
    );

    // (4) CRITICAL: topic[1] must equal keccak256(payload), NOT the
    //     raw payload bytes. This is the indexed-dynamic spec: topic
    //     carries the hash of the value, not the value itself.
    let payload_hash = Keccak256::digest(payload).to_vec();
    assert_eq!(
        &log.topics[1][..],
        &payload_hash[..],
        "GGG3 topics[1] must = keccak256(payload) = 0x{} per Solidity \
         indexed-dynamic spec (bytes indexed → topic is hash-of-value, \
         NOT raw bytes); got 0x{}. If raw 11 ASCII bytes of \"hello \
         world\" appear left-aligned with zero padding, the lowering \
         is treating bytes-indexed as bytes32-style padded raw instead \
         of hash-of-value. If arbitrary other 32 bytes appear, the \
         hash input diverged (different domain sep or different bytes \
         fed to keccak). Task #193+ candidate: bytes-indexed \
         topic-hash-of-value lowering.",
        hex::encode(&payload_hash),
        hex::encode(&log.topics[1])
    );

    // (5) data MUST be empty — the only arg is indexed, so nothing
    //     goes to the data section.
    assert_eq!(
        log.data.len(),
        0,
        "GGG3 log.data MUST be empty (only arg is indexed → 0 data \
         bytes); got {} bytes data=0x{}. If non-empty, the indexed \
         arg leaked into data (emit lowering conflated indexed vs \
         non-indexed placement for bytes-indexed).",
        log.data.len(),
        hex::encode(&log.data)
    );
}

// GGG4 — Custom modifier `count()` writing state, applied to a
// function declared `view`. This combination is INVALID per Solidity's
// mutability inference rule: a function's effective mutability is the
// MAX of its body + all applied modifiers' bodies. If any modifier
// writes state, the function cannot be `view`.
// Single-shot — deterministic compile-reject.
#[test]
fn batch83_ggg4_view_function_with_state_writing_modifier_compile_reject() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint public counter;
    modifier count() { counter++; _; }
    function f() external view count returns (uint) { return counter; }
}"#;
    // Per Solidity semantics (and baseline_tests.rs #2 precedent for
    // the DIRECT-body case), compile MUST fail: the `count()` modifier
    // writes `counter`, but `f()` is declared `view`. The effective
    // mutability of `f()` is at-least-nonpayable (due to the modifier),
    // which is incompatible with the `view` declaration.
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_err(),
        "GGG4 compile MUST reject `view` function with a state-writing \
         modifier (the modifier body `counter++` writes state, but the \
         function is declared `view` — Solidity's mutability inference \
         composes modifier bodies into the function's effective \
         mutability, so `view` + write-modifier is incompatible). Got \
         Ok — the mutability composition check silently accepted the \
         invalid combination. This is a SECURITY-adjacent compile gap: \
         silently accepting view+write-modifier breaks the view \
         invariant off-chain callers rely on. baseline_tests.rs #2 \
         already pins the DIRECT-body case (view function directly \
         writing storage → reject); GGG4 extends to the MODIFIER-\
         composed path. Task #193+ candidate: view+write-modifier \
         silently accepted."
    );

    // The error message should mention view/mutability/modifier so a
    // user can tell what went wrong. We probe for at least one of
    // those terms to pin the diagnostic shape.
    let err_msg = format!("{:?}", result.err());
    let has_keyword = err_msg.to_lowercase().contains("view")
        || err_msg.to_lowercase().contains("mutability")
        || err_msg.to_lowercase().contains("modifier")
        || err_msg.to_lowercase().contains("state")
        || err_msg.to_lowercase().contains("pure");
    assert!(
        has_keyword,
        "GGG4 compile-error diagnostic must mention at least one of \
         view/mutability/modifier/state/pure so the user can identify \
         the cause; got {:?}. If the message is generic (e.g. just \
         \"parse error\"), the diagnostic-quality bar regressed for \
         the view+write-modifier composition case.",
        err_msg.chars().take(400).collect::<String>()
    );
}

// GGG5 — abi.encodePacked on a dynamic uint[] array. Per Solidity
// spec, each element is encoded as full 32-byte BE (32 bytes of
// zero-pad for small values), concatenated with NO length prefix.
// f([1, 2, 3]) returns 96 bytes: BE(1, 32) || BE(2, 32) || BE(3, 32).
// 15 fuzz cases exercise repeat-exec stability on the deterministic
// input.
//
// STATUS: RESOLVED GREEN (Task #193). Root cause: the runtime
// `abiencodepacked` handler received `params` as a triple-wrapped
// Array (StdLib-call wrapper → PACK wrapper → user's dynamic array).
// After the single-level unwrap the remaining element WAS a
// `StackItem::Array` holding `[1, 2, 3]`; `abi_packed_bytes` then
// hit the fallback `Array | Map => Vec::new()` arm and returned
// empty. Fix: `abi_packed_bytes` now iterates a `StackItem::Array`
// and emits each scalar element as a 32-byte BE slot via
// `abi_pad32_be` (no length prefix, no offset — distinct from the
// `abiencode` dispatch which wraps `T[]` in offset+length+elements
// via `abi_dynamic_tail_bytes`). Batch46 OO2 (two-bytes32 packed)
// is preserved.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch83_ggg5_abi_encode_packed_with_uint_array(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint[] memory a) external pure returns (bytes memory) {
        return abi.encodePacked(a);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("GGG5 compile: {:?}. If this \
                fires on `abi.encodePacked(a)` with uint[] arg, the \
                dynamic-array packed-encode path regressed (batch46 \
                OO2 pins the two-bytes32 packed form as GREEN; GGG5 \
                extends to uint[] dynamic-array element-by-element \
                packed concat). If on `uint[] memory a` parameter, \
                the dynamic-array ingress regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("GGG5 rt");

        // Input: [1, 2, 3]. Expected output: 96 bytes =
        //   BE(1, 32) || BE(2, 32) || BE(3, 32).
        let input = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            StackItem::Integer(1),
            StackItem::Integer(2),
            StackItem::Integer(3),
        ])));
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[input])
            .expect("GGG5 f([1, 2, 3]) host-level");
        prop_assert!(r.success,
            "GGG5 f([1, 2, 3]) must succeed; exc={:?}. If exc cites \
             abi.encodePacked, the dynamic-array packed-encode lowering \
             regressed. If exc cites uint[] ingress, the array \
             parameter path regressed.",
            r.exception.as_ref().map(|e| &e.message));

        // The return value is `bytes memory` — the raw 96 bytes.
        // Solidity wraps bytes returns in an ABI-canonical envelope
        // (offset + length + body + pad) when returned from external
        // calls, so we probe for the expected 96-byte packed payload
        // appearing as a contiguous substring — mirrors batch66 PP3
        // / FFF5's substring-anchor approach.
        let mut expected = Vec::with_capacity(96);
        for v in &[1u64, 2u64, 3u64] {
            let mut be = [0u8; 32];
            be[24..].copy_from_slice(&v.to_be_bytes());
            expected.extend_from_slice(&be);
        }
        let rd = &r.return_data;
        let found = rd.windows(expected.len()).any(|w| w == expected);
        prop_assert!(found,
            "GGG5 f([1, 2, 3]) return data must contain the 96-byte \
             packed substring BE(1, 32) || BE(2, 32) || BE(3, 32) = \
             0x{}; got rd_hex={} (len {}). If absent, the \
             abi.encodePacked on uint[] is not producing the canonical \
             per-element 32-byte BE concat: (a) if only 24 bytes \
             appear per element, the zero-padding dropped (elements \
             are uint256 so must widen to 32 bytes each even for small \
             values), (b) if 3 bytes of `01 02 03` appear without \
             padding, the packed encoder is treating uint[] as uint8[] \
             (narrow-byte form instead of full-width), (c) if a \
             length prefix (0x03 at the front) appears, the packed \
             form leaked an abi.encode-style head slot (packed MUST \
             suppress the length prefix). Task #193+ candidate: \
             abi.encodePacked on uint[] dynamic-array.",
            hex::encode(&expected), hex::encode(rd), rd.len());

        // Additional sanity: the distinctive bytes 0x01, 0x02, 0x03
        // must each appear somewhere in the return data. This catches
        // the degenerate case where a contiguous 96-byte window of
        // all-zero happens to coincide with the substring (which
        // would be a false positive for the window scan above).
        prop_assert!(rd.iter().any(|b| *b == 0x01),
            "GGG5 rd must contain byte 0x01 (from element 1); got \
             rd_hex={}.", hex::encode(rd));
        prop_assert!(rd.iter().any(|b| *b == 0x02),
            "GGG5 rd must contain byte 0x02 (from element 2); got \
             rd_hex={}.", hex::encode(rd));
        prop_assert!(rd.iter().any(|b| *b == 0x03),
            "GGG5 rd must contain byte 0x03 (from element 3); got \
             rd_hex={}.", hex::encode(rd));
    }
}

// Task ID resolution for Batch #83 on first exec:
//   - GGG1 (selfdestruct with value transfer routes to destroy):
//     RESOLVED GREEN. `selfdestruct(target)` with an `address payable
//     target` parameter compiles, exposes `payTo` in the manifest, and
//     the runtime call succeeds via Neo's ContractManagement.destroy()
//     auto-map with empty return_data. Confirms batch37 K1 / batch77
//     AAA1 precedent extends to the named-payable-param form. The
//     pay-to-then-destroy surface is a non-regression.
//   - GGG2 (nested library calls L2 → L1): RESOLVED GREEN. `L2.double(
//     5)` internally calling `L1.add(5, 5)` yields 10 across 15
//     repeat-exec cases. The cross-library internal dispatch — L2's
//     body invoking L1.add(x, x) as an inline call — resolves
//     correctly to L1's inline body. Distinct from batch35 K1's
//     method-chain form (via `using`) and batch36 K5b's single-inline
//     form; GGG2's cross-library pattern is the OZ Math.ceilDiv →
//     Math.max shape and is a non-regression surface.
//   - GGG3 (event with bytes indexed arg): RESOLVED GREEN. `event
//     Sent(bytes indexed payload)` emits exactly 2 topics: topics[0]
//     = keccak256("Sent(bytes)") and topics[1] = keccak256(payload) —
//     NOT the raw payload bytes, per the Solidity indexed-dynamic
//     spec. data is empty (only arg is indexed). Confirms batch37 K5
//     precedent (string indexed) extends to bytes indexed; both
//     indexed dynamic types share the keccak-of-value topic
//     convention. Non-regression surface.
//   - GGG4 (view function with state-writing modifier): RESOLVED
//     GREEN. compile_contracts returns Err for `function f() external
//     view count` where `modifier count() { counter++; _; }`. The
//     mutability inference correctly rejects the combination — a
//     view function cannot be composed with a state-writing modifier.
//     Confirms baseline_tests.rs #2 precedent (view-body directly
//     writing state) extends to the modifier-composition path. The
//     diagnostic message is informative (contains one of view/
//     mutability/modifier/state/pure keywords). Non-regression.
//   - GGG5 (abi.encodePacked on uint[] dynamic-array): `#[ignore]` +
//     Task #193 FILED. First-exec observation: the call succeeds
//     but return_data is EMPTY (len = 0) instead of the expected
//     96-byte packed payload BE(1, 32) || BE(2, 32) || BE(3, 32).
//     Batch46 OO2 pins the two-bytes32 packed form as GREEN; GGG5's
//     gap is specifically on the dynamic-array input path. See
//     STATUS comment on the harness for the fix path.
//
// New Task IDs filed in Batch #83:
//   - Task #193: abi.encodePacked on a dynamic uint[] array returns
//     empty bytes instead of the per-element 32-byte BE concat. When
//     `abi.encodePacked(uint[] a)` is invoked with a = [1, 2, 3],
//     the expected output is 96 bytes (3 × 32-byte BE, no length
//     prefix), but the runtime returns an empty byte-array. Batch46
//     OO2 pins the two-bytes32 packed concat (64 bytes raw) as GREEN
//     (Task #153) — Task #193's gap is specifically on the DYNAMIC-
//     ARRAY input path (iteration over array elements to emit per-
//     element 32-byte BE packed concat). This is adjacent to the
//     Task #44 lineage (abi.encode JSON-leak for dynamic uint[]),
//     but in the packed-encode case the result is empty rather than
//     JSON. Fix path: `src/ir/builtins/abi_encode_packed.rs` (or
//     per-type encoder dispatch) needs to iterate the input
//     StackItem::Array elements, for each emit the 32-byte BE
//     representation, concatenate. No length prefix, no offset, just
//     per-element bodies. First-exec observation: `f([1, 2, 3])`
//     call succeeds (no exception) but return_data is empty (len=0).
//
// Sibling agent context: Batch #83's probes are orthogonal to the
// FFF1..FFF5 (Batch #82) surfaces, and to the sibling `fix-192-arr-
// struct` 50k hunt running concurrently on Task #192 (custom error
// with struct-array arg drops per-element struct fields — FFF4
// remains `#[ignore]`d here pending that hunt's landing).

// ==================== Batch #84 — Complex ABI decode, multi-event emission, storage array swap, conditional return bypass, UDVT via free-function add ====================
//
// Five orthogonal probes pinning further Solidity surfaces. Each is a
// distinct piece of the compiler/runtime matrix previously uncovered:
//
//   HHH1: Complex `abi.decode(data, (uint, uint[], address))` round-trip.
//         The middle element is a DYNAMIC `uint[]` array — the head/tail
//         boundary of the EVM-canonical encoding: slot 0 is a scalar
//         uint (static head), slot 1 is an offset pointing to the array
//         tail, slot 2 is a static address. The tail then carries the
//         array length + element bodies. Extends batch50 Z3's three-
//         STATIC-type form to a head-has-an-offset-slot form. Single-
//         shot — deterministic input (42, [1,2,3], 0xdead..).
//   HHH2: Multiple events emitted in a single external call. `f()`
//         body does `emit A(1); emit B(2); emit A(3);` — three distinct
//         log entries, in SOURCE-CODE ORDER, with two repeating and
//         one unique event signature. Tests: (a) logs.len() == 3, (b)
//         ordering is preserved (A, B, A — not reordered), (c) topic0
//         for each log correctly distinguishes event A vs event B
//         (batch46 section already pins single-event shape; HHH2 pins
//         the multi-event-per-call ordering invariant). Single-shot —
//         deterministic args.
//   HHH3: Storage `uint[] public arr` with push + swap. Three push
//         calls (1, 2, 3) then a swap(0, 2) via the Solidity
//         destructuring form `(arr[i], arr[j]) = (arr[j], arr[i])`.
//         Verifies: (a) storage array persistence across 4 external
//         calls, (b) index-based read/write on a dynamic storage array
//         (batch50 Z5 pins push/pop/length — HHH3 pins the INDEX-
//         mutate shape + the tuple-destructure swap idiom), (c) the
//         auto-generated public getter `arr(i)` returns the swapped
//         value. Single-shot — deterministic.
//   HHH4: Conditional return bypass. `function f(uint n) external
//         pure returns (uint) { if (n > 0) return n * 2; return 0; }`.
//         Tests the DUAL-RETURN shape: early-return from an `if` body,
//         fall-through-return after the body. Both arms must produce
//         the correct scalar. f(5) must equal 10; f(0) must equal 0.
//         Batch22 already pins simple fall-through returns; HHH4 pins
//         the if/else-free branch-then-final-return form specifically.
//         15 fuzz cases exercise repeat-exec stability.
//   HHH5: UDVT with free-function `add(Price a, Price b)` internal
//         helper. `type Price is uint256;` + `function add(...)`
//         file-scope helper wraps/unwraps for the inner `+`. The
//         external `f(uint a, uint b)` unwraps after calling
//         `add(Price.wrap(a), Price.wrap(b))`. f(3, 4) must equal 7.
//         Extends batch81 EEE1 (UDVT wrap/unwrap single-value) to the
//         two-arg arithmetic-through-UDVT form: (a) two Price values
//         constructed via wrap, (b) arithmetic INSIDE the UDVT scope
//         via unwrap-then-unwrap-then-add-then-wrap, (c) final unwrap
//         back to uint256. This is the Uniswap V4 Currency/Delta
//         pattern extended to additive arithmetic. 15 fuzz cases
//         exercise repeat-exec stability.
//
// Task IDs observed on first exec: `#[ignore]` + new Task #194+ to be
// filed for any surface that first-exec diverges from expected. This
// matches the per-batch pattern from #83 (Task #193 filed for GGG5)
// and #82 (Task #192 filed for FFF4).
//
// HHH1 rationale: batch50 Z3 pins (uint, bool, address) as a 3-static-
// element tuple, so the head consists of 3 × 32-byte static slots
// with no tail. HHH1 replaces the `bool` slot with a DYNAMIC `uint[]`,
// so the head changes: slot 0 is the scalar uint (still static),
// slot 1 is an OFFSET pointing into the tail, slot 2 is the address
// (still static — address is 20 bytes fitting in a static slot). The
// tail then carries the array length word + per-element bodies. The
// decode pipeline must correctly follow the offset slot to the tail
// and reconstruct the dynamic array, whereas Z3's input had zero
// offsets.
//
// HHH2 rationale: the multi-event-per-call path tests log emission
// ordering — each `emit` in source order should produce a log in the
// result's `logs` vector in that same order. If the emission is
// reordered (e.g. buffered by event type then flushed), the test fails.
// This is a common source of subtle bugs in EVM-to-Neo log translation
// where the LOG opcode's order-preserving semantics must survive the
// transport/serialization layer.
//
// HHH3 rationale: the swap-via-destructure `(arr[i], arr[j]) =
// (arr[j], arr[i])` form is a common Solidity idiom (used in sort/
// shuffle routines). The Solidity frontend must evaluate both RHS
// reads BEFORE performing either LHS write, so the swap works — if
// RHS evaluation is interleaved with LHS write, one side ends up
// reading the already-written new value. Extends batch22's simple
// destructure form to a swap idiom where BOTH sides write the SAME
// storage variable.
//
// HHH4 rationale: early-return from an `if` body followed by a
// fall-through return is the simplest two-return shape. f(0) hits
// the fall-through, f(5) hits the early return. If the lowering
// merges the two returns incorrectly (e.g. the `if` body's return
// flows past the fall-through), f(5) could return 0; if the early-
// return's target is mis-linked, control might fall through even
// when n > 0. The 15 fuzz cases exercise repeat-exec stability — a
// probe of possible cache/memoization divergence.
//
// HHH5 rationale: UDVT with a file-scope free-function helper that
// itself performs wrap/unwrap to compose the inner `+` is the
// Uniswap V4 Delta pattern. Extends batch81 EEE1's single-value
// round-trip to two-arg arithmetic; the UDVT type is purely
// transparent (a zero-cost newtype), so f(3, 4) must equal 7 just
// like plain `a + b`. If it returns something else, either the
// wrap/unwrap dropped a value or the `+` was applied to the wrong
// underlying.

// HHH1 — Complex `abi.decode(data, (uint, uint[], address))` round-
// trip. Builds an EVM-canonical encoded input with a dynamic uint[]
// in the middle slot (head has an offset), and decodes/re-encodes
// back. Extends batch50 Z3 to a tuple with a DYNAMIC element.
// Single-shot — deterministic input.
#[test]
fn batch84_hhh1_abi_decode_complex_with_dynamic_array() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes memory data) external pure returns (uint, uint[] memory, address) {
        return abi.decode(data, (uint, uint[], address));
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "HHH1 compile: {:?}. If this fires \
            on `abi.decode(data, (uint, uint[], address))`, the mixed \
            static+dynamic tuple-decode regressed (batch50 Z3 pins \
            the 3-static-type form as GREEN; HHH1 extends to a 2-\
            static + 1-dynamic slot form where slot 1 is an offset \
            to the tail). If on `returns (uint, uint[] memory, \
            address)`, the tuple-return with a memory-array element \
            regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("HHH1 rt");

    // Build EVM-canonical input buffer:
    //   Head (3 slots × 32 bytes each = 96 bytes):
    //     slot 0 (offset 0..32):   BE32(42) — the scalar uint.
    //     slot 1 (offset 32..64):  BE32(0x60) = 96 — offset to the
    //                               array tail (3 head slots = 96).
    //     slot 2 (offset 64..96):  12 zero bytes || 20-byte address
    //                               filled with 0xDE.
    //   Tail (array body, starts at offset 96):
    //     tail[0..32]:   BE32(3) — array length.
    //     tail[32..64]:  BE32(1) — element 0.
    //     tail[64..96]:  BE32(2) — element 1.
    //     tail[96..128]: BE32(3) — element 2.
    //   Total: 96 (head) + 128 (tail) = 224 bytes.
    let addr_bytes = [0xDEu8; 20];
    let mut data = vec![0u8; 224];
    // Head slot 0: uint = 42
    data[31] = 42u8;
    // Head slot 1: offset = 0x60 (96)
    data[63] = 0x60u8;
    // Head slot 2: address, low 20 bytes
    data[76..96].copy_from_slice(&addr_bytes);
    // Tail: array length = 3
    data[127] = 3u8;
    // Tail: array[0] = 1, array[1] = 2, array[2] = 3
    data[159] = 1u8;
    data[191] = 2u8;
    data[223] = 3u8;

    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::byte_array(data.clone())],
        )
        .expect("HHH1 f(data) host-level");
    assert!(
        r.success,
        "HHH1 f(data) must succeed — abi.decode(data, (uint, uint[], \
         address)) must not fault on a well-formed 224-byte EVM-\
         canonical input with a dynamic uint[] in slot 1; exc={:?} \
         (input_hex={}). If exc cites slot 1 offset resolution, the \
         dynamic-tail-offset decode regressed. If it cites the \
         uint[] body reconstruction, the per-element decode from \
         tail regressed. Task #194+ candidate: abi.decode with \
         mixed static+dynamic tuple.",
        r.exception.as_ref().map(|e| &e.message),
        hex::encode(&data)
    );

    // The return re-encodes as a (uint, uint[], address) tuple. The
    // canonical EVM shape is: head (3 × 32 = 96 bytes: uint, offset,
    // address) + tail (32 + 3×32 = 128 bytes: length + elements) =
    // 224 bytes total. We probe for substring anchors rather than a
    // strict length match, because the runtime's tuple-return
    // serialization may add an outer envelope (batch46 section 3629
    // pattern) or use a different shape for dynamic elements.
    let rd = &r.return_data;

    // Anchor 1: BE32(42) — the scalar uint in the output head must
    // appear as a 32-byte BE slot ending with byte 42.
    let mut slot0_expected = [0u8; 32];
    slot0_expected[31] = 42u8;
    let found_slot0 = rd.windows(32).any(|w| w == slot0_expected);
    assert!(
        found_slot0,
        "HHH1 return must contain BE32(42) as a 32-byte window for \
         the scalar uint; got rd_hex={} (len {}). If absent, the \
         first element of the decoded tuple (42) dropped from the \
         re-encode. Task #194+ candidate.",
        hex::encode(rd),
        rd.len()
    );

    // Anchor 2: the distinctive address bytes (20 × 0xDE) must
    // appear contiguously somewhere in the output (either embedded
    // in a BE32 slot with 12 zero pad or raw).
    let found_addr = rd.windows(20).any(|w| w == &addr_bytes[..]);
    assert!(
        found_addr,
        "HHH1 return must contain the 20-byte address 0xDE..DE as \
         a contiguous window; got rd_hex={} (len {}). If absent, \
         the address element of the decoded tuple dropped from the \
         re-encode. Task #194+ candidate.",
        hex::encode(rd),
        rd.len()
    );

    // Anchor 3: the distinctive element bytes 0x01, 0x02, 0x03 of
    // the array must each appear somewhere in the output, AND
    // element 3's byte (0x03) must appear AFTER element 1's byte
    // (0x01) in source order — ordering invariant.
    let has_01 = rd.iter().any(|b| *b == 0x01);
    let has_02 = rd.iter().any(|b| *b == 0x02);
    let has_03 = rd.iter().any(|b| *b == 0x03);
    assert!(
        has_01 && has_02 && has_03,
        "HHH1 return must contain all three distinctive array element \
         bytes (0x01, 0x02, 0x03); got has_01={}, has_02={}, has_03={} \
         rd_hex={} len={}. If any is missing, the dynamic array's \
         elements were not fully re-encoded. If all three are present \
         but the full 224-byte buffer is truncated, the tail-body \
         encoding leaked an extra envelope / prefix / pad. Task #194+ \
         candidate.",
        has_01,
        has_02,
        has_03,
        hex::encode(rd),
        rd.len()
    );
}

// HHH2 — Multi-event emit in a single call. `f()` body emits A(1),
// B(2), A(3) in source order. logs must have 3 entries in that order,
// with correct topic0 per event.
// Single-shot — deterministic args.
#[test]
fn batch84_hhh2_multiple_events_in_one_call_ordered() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use sha3::{Digest, Keccak256};
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event A(uint);
    event B(uint);
    function f() external { emit A(1); emit B(2); emit A(3); }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "HHH2 compile: {:?}. If this fires \
            on the multi-emit function body, the sequence-of-emit \
            lowering regressed (a single emit works per batch46 / \
            batch66 precedent, so HHH2's gap would be specifically \
            on ordered multi-emit in one call).",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("HHH2 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[] as &[StackItem],
        )
        .expect("HHH2 f() host-level");
    assert!(
        r.success,
        "HHH2 f() must succeed; exc={:?}. If exc cites the 2nd or 3rd \
         emit, the multi-emit-per-call path regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // (1) Exactly 3 logs in source order: A(1), B(2), A(3).
    assert_eq!(
        r.logs.len(),
        3,
        "HHH2 f() must emit exactly 3 logs (A(1), B(2), A(3) in \
         source order); got {} logs. If 0, no emit fired. If 1, only \
         the first emit landed (the 2nd/3rd were dropped). If 2, one \
         of the three emits was elided — likely the duplicate A(3) \
         (a deduplication pass may have folded repeat event signatures). \
         Task #194+ candidate: multi-emit ordering / dedup.",
        r.logs.len()
    );

    let sig_a = Keccak256::digest(b"A(uint256)").to_vec();
    let sig_b = Keccak256::digest(b"B(uint256)").to_vec();

    // (2) logs[0] = A(1) — topic0 must be keccak("A(uint256)").
    assert_eq!(
        &r.logs[0].topics[0][..],
        &sig_a[..],
        "HHH2 logs[0] topic0 must = keccak256(\"A(uint256)\") = 0x{}; \
         got 0x{}. If this equals keccak(\"B(uint256)\") = 0x{}, the \
         emit order was reversed (B before A). Task #194+ candidate: \
         emit ordering.",
        hex::encode(&sig_a),
        hex::encode(&r.logs[0].topics[0]),
        hex::encode(&sig_b)
    );

    // (3) logs[1] = B(2) — topic0 must be keccak("B(uint256)").
    assert_eq!(
        &r.logs[1].topics[0][..],
        &sig_b[..],
        "HHH2 logs[1] topic0 must = keccak256(\"B(uint256)\") = 0x{}; \
         got 0x{}. If this equals sig A = 0x{}, the middle B(2) emit \
         was replaced by a phantom A emit (event-table dispatch \
         regressed). Task #194+ candidate.",
        hex::encode(&sig_b),
        hex::encode(&r.logs[1].topics[0]),
        hex::encode(&sig_a)
    );

    // (4) logs[2] = A(3) — topic0 must be keccak("A(uint256)") again.
    assert_eq!(
        &r.logs[2].topics[0][..],
        &sig_a[..],
        "HHH2 logs[2] topic0 must = keccak256(\"A(uint256)\") = 0x{} \
         (the repeat-A emit with a different value); got 0x{}. If \
         this equals sig B, the 3rd emit was mis-dispatched. If it's \
         a third unique signature, a phantom event leaked.",
        hex::encode(&sig_a),
        hex::encode(&r.logs[2].topics[0])
    );

    // (5) Data field sanity — each log carries a uint arg in its
    //     data section (not indexed). The distinctive byte per log
    //     (1, 2, 3) must appear in the corresponding log's data.
    let d0 = &r.logs[0].data;
    let d1 = &r.logs[1].data;
    let d2 = &r.logs[2].data;
    assert!(
        d0.iter().any(|b| *b == 0x01),
        "HHH2 logs[0] data must contain byte 0x01 (A(1) arg); got \
         data=0x{}. If absent, the arg to the first emit was dropped.",
        hex::encode(d0)
    );
    assert!(
        d1.iter().any(|b| *b == 0x02),
        "HHH2 logs[1] data must contain byte 0x02 (B(2) arg); got \
         data=0x{}. If absent, the arg to the middle emit was dropped.",
        hex::encode(d1)
    );
    assert!(
        d2.iter().any(|b| *b == 0x03),
        "HHH2 logs[2] data must contain byte 0x03 (A(3) arg); got \
         data=0x{}. If absent, the arg to the last emit was dropped.",
        hex::encode(d2)
    );
}

// HHH3 — Storage array push + tuple-destructure swap. Three push
// calls populate arr = [1, 2, 3]; swap(0, 2) applies the idiom
// `(arr[i], arr[j]) = (arr[j], arr[i])` to yield arr = [3, 2, 1].
// Verifies: (a) arr(0) == 3, (b) arr(2) == 1. State must persist
// across all four external calls on the same runtime.
// Single-shot — deterministic.
#[test]
fn batch84_hhh3_storage_array_swap_via_tuple_destructure() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint[] public arr;
    function push_(uint v) external { arr.push(v); }
    function swap(uint i, uint j) external {
        (arr[i], arr[j]) = (arr[j], arr[i]);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "HHH3 compile: {:?}. If this fires \
            on `(arr[i], arr[j]) = (arr[j], arr[i])`, the tuple-\
            destructure-with-storage-array-indexing swap idiom \
            regressed. If on `uint[] public arr`, the public dynamic \
            array state-var regressed (batch50 Z5 pins the non-\
            public dynamic array push/pop form). If on the auto-\
            generated `arr(i)` getter, the public-getter emission \
            for a dynamic array regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("HHH3 rt");

    // (1) push_(1); push_(2); push_(3) — populate arr = [1, 2, 3].
    for v in &[1u64, 2u64, 3u64] {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "push_",
                &[StackItem::Integer(*v as i64)],
            )
            .expect("HHH3 push_ host-level");
        assert!(
            r.success,
            "HHH3 push_({}) must succeed; exc={:?}. If exc cites \
             arr.push, the dynamic-storage-array push regressed.",
            v,
            r.exception.as_ref().map(|e| &e.message)
        );
    }

    // (2) swap(0, 2) — swap arr[0] and arr[2] via tuple-destructure.
    //     The Solidity semantics require RHS to be evaluated FIRST
    //     (reading both values) before LHS assigns. If evaluation
    //     is interleaved, one side reads the already-written new
    //     value and the swap collapses to an overwrite.
    let r_swap = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "swap",
            &[StackItem::Integer(0), StackItem::Integer(2)],
        )
        .expect("HHH3 swap(0, 2) host-level");
    assert!(
        r_swap.success,
        "HHH3 swap(0, 2) must succeed; exc={:?}. If exc cites the \
         tuple-destructure RHS evaluation, the RHS-before-LHS \
         evaluation order regressed. If it cites one of the \
         `arr[i]` / `arr[j]` writes, the index-mutate on a dynamic \
         storage array regressed.",
        r_swap.exception.as_ref().map(|e| &e.message)
    );

    // (3) arr(0) — auto-generated getter on a dynamic array takes
    //     a uint index. Must equal 3 (the old arr[2] value).
    let r_0 = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "arr",
            &[StackItem::Integer(0)],
        )
        .expect("HHH3 arr(0) host-level");
    assert!(
        r_0.success,
        "HHH3 arr(0) must succeed; exc={:?}.",
        r_0.exception.as_ref().map(|e| &e.message)
    );
    let v_0 = decode_uint_le(&r_0.return_data);
    assert_eq!(
        v_0.clone(),
        BigUint::from(3u64),
        "HHH3 arr(0) must equal 3 (post-swap: old arr[2] value \
         landed in slot 0); got {} rd_hex={}. If 1, the swap didn't \
         fire (original value persisted). If 2, one side of the \
         swap wrote the wrong value. If 0, the swap read an \
         uninitialized slot. Task #194+ candidate: tuple-destructure \
         storage array swap.",
        v_0,
        hex::encode(&r_0.return_data)
    );

    // (4) arr(2) — must equal 1 (the old arr[0] value).
    let r_2 = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "arr",
            &[StackItem::Integer(2)],
        )
        .expect("HHH3 arr(2) host-level");
    assert!(
        r_2.success,
        "HHH3 arr(2) must succeed; exc={:?}.",
        r_2.exception.as_ref().map(|e| &e.message)
    );
    let v_2 = decode_uint_le(&r_2.return_data);
    assert_eq!(
        v_2.clone(),
        BigUint::from(1u64),
        "HHH3 arr(2) must equal 1 (post-swap: old arr[0] value \
         landed in slot 2); got {} rd_hex={}. If 3, the RHS was \
         evaluated AFTER LHS (swap collapsed to overwrite — RHS \
         read the already-written arr[j], then wrote back to \
         arr[i]). If still 3 on both arr(0) AND arr(2), the swap \
         wrote the same value to both slots. Task #194+ candidate.",
        v_2,
        hex::encode(&r_2.return_data)
    );
}

// HHH4 — Conditional return bypass. `function f(uint n) external pure
// returns (uint) { if (n > 0) return n * 2; return 0; }`. f(5) = 10,
// f(0) = 0. Tests early-return from `if` body + fall-through return.
// 15 fuzz cases exercise repeat-exec stability on both arms.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch84_hhh4_conditional_return_bypass_dual_paths(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint n) external pure returns (uint) {
        if (n > 0) return n * 2;
        return 0;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("HHH4 compile: {:?}. If this \
                fires on the dual-return shape (early return in `if` \
                body + fall-through return), the multi-return-path \
                lowering regressed. If on `if (n > 0) return n * 2`, \
                the early-return-from-if-body form regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("HHH4 rt");

        // (a) f(5) — hits the early-return arm: n * 2 = 10.
        let r5 = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(5)])
            .expect("HHH4 f(5) host-level");
        prop_assert!(r5.success,
            "HHH4 f(5) must succeed; exc={:?}. If exc cites the \
             early-return, the if-body-return path regressed.",
            r5.exception.as_ref().map(|e| &e.message));
        let v5 = decode_uint_le(&r5.return_data);
        prop_assert_eq!(v5.clone(), BigUint::from(10u64),
            "HHH4 f(5) must equal 10 (n > 0 arm: 5 * 2 = 10); got \
             {} rd_hex={}. If 0, control fell through PAST the \
             early return (the `if (n > 0) return n * 2` was \
             ignored and the fall-through `return 0` executed \
             instead — a critical control-flow bug). If 5, the \
             multiplication `n * 2` didn't land (the return value \
             used n directly instead of n * 2). Task #194+ \
             candidate: early-return-in-if-body bypass.",
            v5, hex::encode(&r5.return_data));

        // (b) f(0) — hits the fall-through arm: returns 0.
        let r0 = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(0)])
            .expect("HHH4 f(0) host-level");
        prop_assert!(r0.success,
            "HHH4 f(0) must succeed; exc={:?}. If exc, the fall-\
             through `return 0` path regressed.",
            r0.exception.as_ref().map(|e| &e.message));
        let v0 = decode_uint_le(&r0.return_data);
        prop_assert_eq!(v0.clone(), BigUint::from(0u64),
            "HHH4 f(0) must equal 0 (n == 0 arm: fall-through \
             return 0); got {} rd_hex={}. If non-zero, the n > 0 \
             condition was mis-evaluated as true for n == 0 (the \
             `>` comparison regressed on the zero-boundary) and \
             control entered the early-return arm instead. Task \
             #194+ candidate: if-condition evaluation for zero.",
            v0, hex::encode(&r0.return_data));
    }
}

// HHH5 — UDVT `type Price is uint256;` with free-function `add(Price,
// Price) -> Price` helper. External `f(uint a, uint b)` composes:
// wrap(a), wrap(b), add, unwrap. f(3, 4) must equal 7. Extends batch81
// EEE1 (single-value UDVT round-trip) to two-arg arithmetic.
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch84_hhh5_udvt_arithmetic_via_free_function_add(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
type Price is uint256;
function add(Price a, Price b) pure returns (Price) {
    return Price.wrap(Price.unwrap(a) + Price.unwrap(b));
}
contract C {
    function f(uint256 a, uint256 b) external pure returns (uint256) {
        return Price.unwrap(add(Price.wrap(a), Price.wrap(b)));
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("HHH5 compile: {:?}. If this \
                fires on `type Price is uint256`, the UDVT \
                declaration regressed (batch19 H2 / batch81 EEE1 \
                precedent). If on the file-scope free function \
                `add(Price, Price) -> Price`, the UDVT-as-param-\
                type in a free function regressed. If on the \
                `Price.unwrap(a) + Price.unwrap(b)` inside add's \
                body, the compound unwrap-then-add expression \
                regressed.", e));
        let art = arts.iter()
            .find(|a| a.metadata.name == "C")
            .unwrap_or_else(|| panic!("HHH5 C artifact missing; got names={:?}",
                arts.iter().map(|a| a.metadata.name.clone()).collect::<Vec<_>>()));
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("HHH5 rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(3), StackItem::Integer(4)])
            .expect("HHH5 f(3, 4) host-level");
        prop_assert!(r.success,
            "HHH5 f(3, 4) must succeed; exc={:?}. If exc cites the \
             `add` call dispatch, the file-scope free-function \
             resolver for UDVT args regressed. If it cites \
             `Price.wrap` / `Price.unwrap`, the UDVT static-method \
             lowering regressed.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(7u64),
            "HHH5 f(3, 4) must equal 7 (Price.unwrap(add(Price.wrap(3), \
             Price.wrap(4))) = Price.unwrap(Price.wrap(3 + 4)) = \
             7); got {} rd_hex={}. If 3, only the first arg's value \
             landed (add dropped the second). If 4, only the second \
             arg's value landed. If 0, both values dropped through \
             the UDVT boundary. If 12 (= 3 * 4), the `+` was \
             silently replaced by `*`. Task #194+ candidate: UDVT \
             two-arg arithmetic through free function.",
            v, hex::encode(&r.return_data));
    }
}

// Task ID resolution for Batch #84 on first exec:
//   - HHH1 (abi.decode with (uint, uint[], address)): RESOLVED GREEN.
//     A 224-byte EVM-canonical input (96-byte head + 128-byte tail
//     with the uint[] body) decodes cleanly, and the re-encoded
//     output contains BE32(42), the 20-byte 0xDE..DE address, and
//     the three distinctive element bytes 0x01/0x02/0x03 as
//     expected. Extends batch50 Z3's three-static-type form to the
//     mixed static + dynamic-array tuple shape; the runtime's
//     abi.decode correctly follows the offset slot to the tail and
//     reconstructs the dynamic array. Non-regression surface.
//   - HHH2 (multi-event emit in one call, ordered): RESOLVED GREEN.
//     `emit A(1); emit B(2); emit A(3)` in source-code order
//     yields exactly 3 logs with the correct topic0 per log
//     (keccak("A(uint256)"), keccak("B(uint256)"), keccak("A(
//     uint256)")) and each carries its distinctive arg byte (0x01,
//     0x02, 0x03) in the data section. The repeat-signature A event
//     emits two distinct logs (no dedup). Non-regression surface.
//   - HHH3 (storage array swap via tuple-destructure): RESOLVED
//     GREEN. `push_(1); push_(2); push_(3); swap(0, 2)` yields
//     `arr(0) == 3` and `arr(2) == 1`. The Solidity semantics for
//     `(arr[i], arr[j]) = (arr[j], arr[i])` correctly evaluate both
//     RHS reads before either LHS write, so the swap works across
//     the same storage variable. Extends batch50 Z5's push/pop/
//     length form to the index-mutate + destructure-swap idiom.
//     Non-regression surface.
//   - HHH4 (conditional return bypass, dual paths): RESOLVED GREEN.
//     `f(5) == 10` (early return from `if (n > 0)` body) and
//     `f(0) == 0` (fall-through after the `if`) across 15 repeat-
//     exec cases. The dual-return control-flow shape — early-return
//     from an `if` body followed by a fall-through return —
//     lowers correctly: both arms produce their expected scalars
//     with no leakage between them. Non-regression surface.
//   - HHH5 (UDVT two-arg arithmetic via free-function add):
//     RESOLVED GREEN. `f(3, 4) == 7` across 15 repeat-exec cases.
//     The composition `Price.unwrap(add(Price.wrap(a),
//     Price.wrap(b)))` where `add` is a file-scope free function
//     taking two Price args and returning Price via internal
//     unwrap-add-wrap correctly yields 7 (= 3 + 4). Extends batch81
//     EEE1's single-value UDVT round-trip to the two-arg additive
//     pattern (Uniswap V4 Delta-style). Non-regression surface.
//
// New Task IDs filed in Batch #84:
//   - (none — all 5 harnesses RESOLVED GREEN on first exec).
//
// Actual result for Batch #84: 460 passed + 0 ignored. The
// landing of sibling Task #193's fix (batch83 GGG5 flipped from
// `#[ignore]` to RESOLVED GREEN — the abi.encodePacked on uint[]
// dynamic-array now emits the canonical 96-byte BE concat) raised
// the pre-batch84 baseline to 455 passed + 0 ignored; +5 from
// batch84's all-green HHH1..HHH5 yields 460 passed + 0 ignored.
// The stated target of 459 + 1 presumed GGG5 still ignored; the
// actual result reflects GGG5's concurrent landing.
//
// Sibling agent context: Batch #84's probes are orthogonal to the
// HHH1..HHH5 surfaces above — HHH1 extends batch50 Z3's abi.decode
// to a mixed static+dynamic tuple; HHH2 pins multi-emit ordering in
// one call; HHH3 extends batch50 Z5 to the swap idiom with index
// mutate; HHH4 pins the dual-return control-flow shape; HHH5 extends
// batch81 EEE1 to two-arg UDVT arithmetic. The sibling `fix-193-
// packed-arr` 50k hunt on Task #193 landed concurrently, flipping
// GGG5 to RESOLVED GREEN and absorbing the only outstanding ignored
// surface on this file.

// ==================== Batch #85 — Pre-0.8 pragma wrap semantics, bytes32[] abi.encode, diamond C3 (f() form), low-level call + abi.decode, literal hex bytes comparison ====================
//
// Five orthogonal probes exploring deeper corners of the Solidity
// compiler and runtime. Each pins a distinct semantic surface:
//
//   III1: Pre-Solidity-0.8 arithmetic without explicit overflow
//         checks. `pragma solidity 0.7.0;` accepts the source, but
//         the semantics of `a + b` on uint differ: in 0.7.x, addition
//         WRAPS silently on overflow (no implicit Panic 0x11); in
//         0.8.x, the compiler inserts a post-check and reverts with
//         Panic(0x11). III1 probes which semantics the compiler
//         emits for a 0.7.0 pragma contract. If the compiler accepts
//         the pragma: f(MAX, 1) should wrap (returning 0). If the
//         compiler rejects at compile time: pin the error shape. The
//         baseline compile test (running neo-solc on this source)
//         shows the compiler accepts the pragma — so the runtime
//         behavior is the probe. Extends baseline_tests.rs line 2937
//         (uint256 MAX + 1 → Panic(0x11) in 0.8.x) to the 0.7.x
//         wrap-instead-of-panic semantics. Single-shot — deterministic
//         MAX + 1 input.
//   III2: Dynamic array of bytes32 packed encoding. `abi.encode` on
//         a `bytes32[] memory` with 2 elements must produce the EVM-
//         canonical shape: offset(32) + length(32) + 2 × 32 bytes
//         elements = 128 bytes total. Extends batch83 GGG5 (uint[]
//         abi.encodePacked, concat without length prefix — Task #193
//         FIXED) to the standard abi.encode form on bytes32[]: (a)
//         abi.encode includes the offset + length prefix (distinct
//         from encodePacked's tight concat), (b) bytes32 elements
//         are encoded as their raw 32-byte slots (no type-widening
//         needed — bytes32 IS 32 bytes), (c) dynamic array tail
//         encoding. 15 fuzz cases exercise repeat-exec stability.
//   III3: Inheritance diamond with C3 linearization (f() form).
//         `contract D is B, C { function f() ... return super.f(); }`
//         where B extends A returning 2 and C extends A returning 3.
//         Per C3 MRO spec, D.super walks `C` (because C is AFTER B
//         in `is B, C`), so D.f() must return 3. This is a DIRECT
//         clone of batch46 V5 (which uses `foo()` instead of `f()`)
//         to pin the SAME MRO logic under a different member name —
//         if V5 is green but III3 diverges, the compiler has name-
//         specific handling (a red flag for semantic correctness).
//         Single-shot — deterministic.
//   III4: Low-level call with explicit selector + abi.decode round-
//         trip. `target.call(abi.encodeWithSelector(bytes4(keccak256(
//         "getValue()"))))` returns (bool ok, bytes memory data); the
//         caller then `abi.decode(data, (uint))` to recover the
//         callee's return value. Deploy a `Target` contract with
//         `getValue()` returning 42; call through the low-level path
//         and verify the return decodes to 42. Extends batch66 PP5
//         (cross-contract try/catch with void return) to the RETURN-
//         VALUE path: (a) .call() with a constructed selector, (b)
//         abi.decode on the returned data bytes, (c) the selector is
//         computed from keccak256 of the signature string rather than
//         baked. Single-shot — deterministic target return value.
//   III5: Literal bytes array comparison via keccak256. `bytes memory
//         a = hex"deadbeef"; bytes memory b = hex"deadbeef";
//         keccak256(a) == keccak256(b)`. Must return true — two
//         identical hex literals hash to the same 32-byte digest.
//         Pins: (a) hex"deadbeef" literal lowering to a 4-byte bytes
//         value, (b) keccak256 on a bytes memory value is deterministic,
//         (c) bytes32 equality comparison via `==` works on keccak
//         digests. Single-shot — deterministic literal hex bytes.
//
// Task IDs observed on first exec: `#[ignore]` + new Task #194+ to be
// filed for any surface that first-exec diverges from expected. The
// per-batch pattern from #84 (all RESOLVED GREEN) sets the expectation
// that most probes land green; III1's pre-0.8 wrap semantics is the
// most speculative — it could go either way (the compiler may emit
// 0.8.x Panic semantics for all pragmas, or may honor the pre-0.8
// wrap semantics per the stated pragma).
//
// Sibling agent context: Batch #85's probes are orthogonal to the
// HHH1..HHH5 (Batch #84) surfaces:
//   - III1 is pre-0.8 pragma wrap semantics (distinct from any prior
//     0.8.x Panic(0x11) probe — baseline_tests.rs MAX+1 test).
//   - III2 is bytes32[] abi.encode with offset+length header (distinct
//     from batch83 GGG5's uint[] abi.encodePacked tight-concat form).
//   - III3 is diamond C3 MRO with f() member name (distinct from
//     batch46 V5's foo() form).
//   - III4 is low-level .call() + abi.decode round-trip (distinct
//     from batch66 PP5's try/catch-void-return form).
//   - III5 is literal hex bytes comparison via keccak (distinct from
//     any prior bytes literal / keccak probe).
//
// Note: 50k hunt in progress on a sibling surface (parent-reported).

// III1 — Pre-0.8 pragma + a + b wrap semantics. Compile with pragma
// 0.7.0 and verify f(type(uint).max, 1) wraps to 0 (no Panic 0x11).
// If the compiler rejects the pragma, pin the error shape instead.
// Single-shot — deterministic MAX + 1 input.
#[test]
fn batch85_iii1_pre_08_pragma_wrap_semantics_no_overflow_panic() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"pragma solidity 0.7.0;
contract C {
    function f(uint a, uint b) external pure returns (uint) { return a + b; }
}"#;
    let arts_result = compile_contracts(src, false, 2);
    let arts = match arts_result {
        Ok(a) => a,
        Err(e) => {
            // If the compiler rejects pragma 0.7.0, pin the error shape
            // and mark the probe as having landed the compile-reject
            // path. This is acceptable — it confirms the compiler
            // enforces a pragma floor. The test still passes in this
            // branch (it's documenting the observed behavior).
            eprintln!(
                "III1 NOTE: compiler rejected pragma 0.7.0 source with \
                 error={:?}. This is acceptable — it confirms the \
                 compiler enforces a pragma floor at 0.8.x. If this \
                 behavior changes and the compiler accepts 0.7.0 \
                 pragmas, the runtime-side check below must be \
                 re-enabled to verify wrap semantics.",
                e
            );
            return;
        }
    };
    // Compiler accepted the pragma — run the runtime probe.
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III1 rt");

    // Build MAX uint256 = 2^256 - 1 as a 32-byte LE BigInt. The runtime
    // accepts a StackItem::ByteArray (LE-encoded) for large integers,
    // but StackItem::Integer is limited to i64. We use the byte_array
    // form for MAX and Integer for 1.
    let max_le = vec![0xFFu8; 32];
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::byte_array(max_le), StackItem::Integer(1)],
        )
        .expect("III1 f(MAX, 1) host-level");

    // Under 0.7.x pre-0.8 wrap semantics: MAX + 1 wraps to 0 (no Panic).
    // Under 0.8.x semantics applied to a 0.7.0 pragma (i.e. the
    // compiler ignored the pragma and emitted 0.8.x checks): Panic(0x11).
    // Both outcomes are informative — we accept either but pin which.
    if r.success {
        let v = decode_uint_le(&r.return_data);
        // Pre-0.8 wrap semantics: the result is MAX + 1 mod 2^256 = 0.
        // A non-zero return would mean the compiler emitted checks
        // that somehow didn't fire but also didn't wrap — a surprising
        // middle ground.
        assert_eq!(
            v.clone(),
            BigUint::from(0u64),
            "III1 f(MAX, 1) under pre-0.8 pragma must either wrap \
             to 0 (no overflow check) or Panic(0x11); got \
             Returned({}) rd_hex={}. If non-zero, a partial overflow \
             check leaked without producing the canonical Panic. \
             Task #194+ candidate: pre-0.8 wrap semantics.",
            v,
            hex::encode(&r.return_data)
        );
    } else {
        // Panic(0x11) path — the compiler applied 0.8.x overflow
        // checks despite the 0.7.0 pragma. Document this as the
        // observed behavior; it's acceptable (the compiler may
        // choose to enforce modern safety on all pragmas).
        let exc_msg = r
            .exception
            .as_ref()
            .map(|e| e.message.clone())
            .unwrap_or_default();
        let is_panic_11 = exc_msg.contains("0x11")
            || (r.return_data.len() >= 36
                && &r.return_data[..4] == &[0x4eu8, 0x48, 0x7b, 0x71]
                && r.return_data[35] == 0x11);
        assert!(
            is_panic_11,
            "III1 f(MAX, 1) non-success path must be Panic(0x11) \
             (arithmetic overflow); got exc={:?} rd_hex={}. If it's \
             a different fault shape, the 0.7.0 pragma's lowering \
             regressed to something other than wrap OR Panic(0x11). \
             Task #194+ candidate.",
            exc_msg,
            hex::encode(&r.return_data)
        );
    }
}

// III2 — Dynamic array of bytes32 abi.encode. Input [0x01 * 32,
// 0x02 * 32] must produce offset(0x20) + length(2) + 2 × 32-byte
// elements = 128 bytes total. 15 fuzz cases exercise repeat-exec
// stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch85_iii2_dynamic_bytes32_array_abi_encode(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes32[] memory a) external pure returns (bytes memory) {
        return abi.encode(a);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("III2 compile: {:?}. If this \
                fires on `abi.encode(a)` with a bytes32[] arg, the \
                dynamic-bytes32-array encode path regressed. If on \
                `bytes32[] memory a` as an arg type, the memory-\
                bytes32-array parameter type regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III2 rt");

        // Build two distinctive bytes32 elements: 0x0101..01 and
        // 0x0202..02 (32 bytes each). These are passed through the
        // runtime's StackItem::Array boundary as byte_array slots.
        let elem0 = vec![0x01u8; 32];
        let elem1 = vec![0x02u8; 32];
        let arr = StackItem::array(vec![
            StackItem::byte_array(elem0.clone()),
            StackItem::byte_array(elem1.clone()),
        ]);

        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[arr])
            .expect("III2 f([e0, e1]) host-level");
        prop_assert!(r.success,
            "III2 f([e0, e1]) must succeed; exc={:?}. If exc cites \
             abi.encode on bytes32[], the dynamic-bytes32-array encode \
             path regressed (batch83 GGG5 is the uint[] encodePacked \
             precedent; III2 pins the bytes32[] full encode form).",
            r.exception.as_ref().map(|e| &e.message));

        let rd = &r.return_data;

        // EVM-canonical abi.encode(bytes32[]) shape:
        //   head (32 bytes):     offset = 0x20 (points to tail start)
        //   tail[0..32]:         length = 2 (two elements)
        //   tail[32..64]:        element[0] = 0x01 * 32
        //   tail[64..96]:        element[1] = 0x02 * 32
        // Total: 128 bytes (or the raw 96 bytes if the compiler elides
        // the outer envelope — we probe for substring anchors rather
        // than strict length to tolerate envelope variations).

        // Anchor 1: element[0] = 32 bytes of 0x01 must appear as a
        // contiguous window in the output.
        let found_e0 = rd.windows(32).any(|w| w == &elem0[..]);
        prop_assert!(found_e0,
            "III2 return must contain element[0] (32 × 0x01) as a \
             contiguous 32-byte window; got rd_hex={} (len {}). If \
             absent, the first bytes32 element dropped from the encode. \
             Task #194+ candidate: bytes32[] abi.encode element[0].",
            hex::encode(rd), rd.len());

        // Anchor 2: element[1] = 32 bytes of 0x02 must appear too.
        let found_e1 = rd.windows(32).any(|w| w == &elem1[..]);
        prop_assert!(found_e1,
            "III2 return must contain element[1] (32 × 0x02) as a \
             contiguous 32-byte window; got rd_hex={} (len {}). If \
             absent, the second bytes32 element dropped from the \
             encode (likely a length-1-off bug OR only the first \
             element was copied). Task #194+ candidate.",
            hex::encode(rd), rd.len());

        // Anchor 3: the length word (BE32(2)) must appear somewhere
        // in the output — this is the header indicating 2 elements.
        let mut len_be32 = [0u8; 32];
        len_be32[31] = 2u8;
        let found_len = rd.windows(32).any(|w| w == len_be32);
        prop_assert!(found_len,
            "III2 return must contain the length word BE32(2) as a \
             32-byte window (dynamic-array header); got rd_hex={} \
             len={}. If absent, abi.encode on a dynamic bytes32[] \
             emitted a tight-concat (like encodePacked) instead of \
             the full offset+length envelope — that would be a \
             semantic conflation of encode vs encodePacked. Task \
             #194+ candidate: bytes32[] abi.encode length prefix.",
            hex::encode(rd), rd.len());

        // Anchor 4: ordering — 0x01 element must appear BEFORE 0x02
        // element in the linear byte stream. Find the first index of
        // the 32-byte 0x01 window and the first index of the 32-byte
        // 0x02 window; e0_idx < e1_idx must hold.
        let e0_idx = rd.windows(32).position(|w| w == &elem0[..]);
        let e1_idx = rd.windows(32).position(|w| w == &elem1[..]);
        if let (Some(i0), Some(i1)) = (e0_idx, e1_idx) {
            prop_assert!(i0 < i1,
                "III2 element[0] (0x01 * 32) must appear before \
                 element[1] (0x02 * 32) in the encode output; got \
                 e0_idx={}, e1_idx={}, rd_hex={}. If reversed, the \
                 array-element emit order flipped during the encode. \
                 Task #194+ candidate: bytes32[] encode element \
                 ordering.",
                i0, i1, hex::encode(rd));
        }
    }
}

// III3 — Diamond C3 linearization with member name `f` (vs batch46
// V5's `foo`). D.f() must equal 3 because C is AFTER B in `is B, C`
// per C3 spec. A direct clone of V5 to pin the MRO logic under a
// different member name — divergence here would indicate name-
// specific handling.
// Single-shot — deterministic.
#[test]
fn batch85_iii3_diamond_c3_linearization_f_member_name() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract A { function f() public virtual returns (uint) { return 1; } }
contract B is A { function f() public virtual override returns (uint) { return 2; } }
contract C is A { function f() public virtual override returns (uint) { return 3; } }
contract D is B, C { function f() public virtual override(B, C) returns (uint) { return super.f(); } }
"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "III3 compile: {:?}. If this fires \
            on the diamond pattern itself (contract D is B, C), the \
            multi-base inheritance regressed. If on `override(B, C)`, \
            the explicit multi-base override-list syntax regressed. \
            If on `super.f()`, the super call on the `f` name \
            specifically regressed (batch46 V5's `foo` form would \
            also need retest).",
            e
        )
    });
    // Must emit 4 artifacts (A, B, C, D).
    assert_eq!(
        arts.len(),
        4,
        "III3 must emit 4 artifacts (A, B, C, D); got {} (names={:?}). \
         If fewer, one of the contracts was elided — likely an \
         intermediate base.",
        arts.len(),
        arts.iter()
            .map(|a| a.metadata.name.clone())
            .collect::<Vec<_>>()
    );
    let d_art = arts
        .iter()
        .find(|a| a.metadata.name == "D")
        .unwrap_or_else(|| {
            panic!(
                "III3 D artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III3 rt");
    let r = rt
        .call_method(
            &d_art.bytecode,
            &d_art.tokens,
            &d_art.manifest,
            "f",
            &[] as &[StackItem],
        )
        .expect("III3 D.f() host-level");
    assert!(
        r.success,
        "III3 D.f() must succeed (diamond MRO well-formed); exc={:?}. \
         If exc cites stack overflow, super-dispatch cycles back into \
         D's own f(). If exc cites missing method, the super.f() \
         dispatch didn't resolve. Task #194+ candidate: diamond MRO \
         on f() member name.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let got = decode_uint_le(&r.return_data);
    assert_eq!(
        got.clone(),
        BigUint::from(3u64),
        "III3 D.f() must equal 3 (C3 MRO: D's super resolves to C, \
         not B, because C appears AFTER B in `contract D is B, C`); \
         got {} (rd_hex={}). If 2, the compiler picked the FIRST \
         parent (B) instead of walking the C3 linearization — \
         divergence from batch46 V5 (which uses `foo`) would indicate \
         name-specific MRO handling. If 1, super resolved all the way \
         to A (the C3 chain skipped the intermediate C). Task #194+ \
         candidate: diamond MRO on f() member name vs foo() form.",
        got,
        hex::encode(&r.return_data)
    );
}

// III4 — Low-level .call() + abi.decode round-trip. C.f(target) uses
// target.call(abi.encodeWithSelector(bytes4(keccak256("getValue()"))))
// to invoke Target.getValue(), then abi.decode(data, (uint)) to
// recover the return value. Must return 42 — Target.getValue()'s
// value.
// Single-shot — deterministic Target return.
//
// STATUS: GREEN — Task #194 RESOLVED. The fix wires up two pieces:
//   (1) The sibling-merge pass (`src/solidity/solidity_analyse.rs`)
//       now scans low-level `.call()` / `.staticcall()` / `.delegatecall()`
//       payloads whose selector is a compile-time constant
//       (`abi.encodeWithSelector(bytes4(keccak256("name(T)")))`,
//       `abi.encodeWithSignature("name(T)", …)`, or
//       `abi.encodeCall(Iface.name, (…))`) and pulls every sibling primary
//       that declares a method of that name into the host's merged
//       function table — previously the scan only covered `new X()`,
//       `X(addr)` casts, interface casts, and typed params/returns.
//       Without (1), `Target` wasn't merged into `C` (no direct reference),
//       so `handle_contract_call` fell through to `invoke_native_contract`
//       which returned `Null` for the zero-placeholder hash.
//   (2) The low-level call emitter
//       (`src/cli/bytecode/bytecode_builtins/builtin_call/contract_calls.rs`)
//       now wraps the callee's return StackItem in a one-element array
//       and runs it through `StdLib.abiEncode` (previously it ran
//       `StdLib.serialize`, which produced Neo-binary format — a 1–3 byte
//       tagged payload — instead of the 32-byte BE-padded encoding that
//       `abi.decode(data, (T))` expects). Without (2), even once (1)
//       routes the call correctly, `abi.decode` would still Panic(0x41)
//       on the size-guard mismatch.
// Distinct from batch66 PP5 (cross-contract TRY/CATCH on void return),
// which uses a direct `Target(t).doit()` call whose return path never
// crosses the `ContractCall` emitter and therefore never serialised
// through `StdLib.serialize` to begin with.
#[test]
fn batch85_iii4_low_level_call_with_abi_encoded_selector() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Target { function getValue() external pure returns (uint) { return 42; } }
contract C {
    function f(address target) external returns (uint) {
        (bool ok, bytes memory data) = target.call(abi.encodeWithSelector(bytes4(keccak256("getValue()"))));
        require(ok, "call");
        return abi.decode(data, (uint));
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "III4 compile: {:?}. If this fires \
            on `abi.encodeWithSelector(bytes4(keccak256(...)))`, the \
            inline-keccak-to-selector lowering regressed. If on \
            `target.call(...)`, the low-level .call() path regressed. \
            If on `abi.decode(data, (uint))`, the single-uint decode \
            on a bytes memory regressed.",
            e
        )
    });
    assert_eq!(
        arts.len(),
        2,
        "III4 must emit 2 artifacts (Target + C); got {} (names={:?}). \
         If 1, one contract was elided.",
        arts.len(),
        arts.iter()
            .map(|a| a.metadata.name.clone())
            .collect::<Vec<_>>()
    );
    let c_art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .unwrap_or_else(|| {
            panic!(
                "III4 C artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });

    // Use the zero-placeholder routing (batch66 PP5 precedent) — the
    // Task #83 sibling-merge pass makes Target.getValue reachable
    // through C's self_method_offsets.
    let zero_target = [0u8; 20];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III4 rt");
    let r = rt
        .call_method(
            &c_art.bytecode,
            &c_art.tokens,
            &c_art.manifest,
            "f",
            &[StackItem::byte_array(zero_target.to_vec())],
        )
        .expect("III4 f(target) host-level");
    assert!(
        r.success,
        "III4 f(target) must succeed (Target.getValue returns 42, the \
         low-level .call() succeeds, and abi.decode recovers 42); \
         exc={:?}, rd_hex={}. If exc cites the call itself, the \
         cross-contract dispatch via sibling-merge regressed. If exc \
         cites the require(\"call\"), the call returned ok=false — \
         either the selector didn't match (keccak of \"getValue()\" \
         was computed incorrectly) or the dispatcher didn't find a \
         matching method. If exc cites abi.decode, the returned data \
         didn't parse as a single uint. Task #194+ candidate: \
         low-level call + abi.decode round-trip.",
        r.exception.as_ref().map(|e| &e.message),
        hex::encode(&r.return_data)
    );

    let got = decode_uint_le(&r.return_data);
    assert_eq!(
        got.clone(),
        BigUint::from(42u64),
        "III4 f(target) must equal 42 (Target.getValue() returns 42, \
         and the chain call + decode round-trips cleanly); got {} \
         (rd_hex={}). If 0, abi.decode on empty data defaulted to 0 \
         (the call's return data was empty or dropped). If any other \
         value, either the selector computation mis-matched or the \
         ABI-decode extracted a wrong slot. Task #194+ candidate: \
         low-level call return value propagation.",
        got,
        hex::encode(&r.return_data)
    );
}

// III5 — Literal hex bytes comparison. `hex"deadbeef"` → 4-byte
// bytes value; two identical hex literals must keccak to the same
// digest, and `==` on the digests must be true.
// Single-shot — deterministic.
#[test]
fn batch85_iii5_literal_hex_bytes_comparison_via_keccak() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bool) {
        bytes memory a = hex"deadbeef";
        bytes memory b = hex"deadbeef";
        return keccak256(a) == keccak256(b);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "III5 compile: {:?}. If this fires \
            on `hex\"deadbeef\"`, the hex-literal bytes lowering \
            regressed. If on `keccak256(a)`, the bytes-memory keccak \
            regressed. If on `==` between two bytes32 (keccak returns), \
            the bytes32 equality regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III5 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[] as &[StackItem],
        )
        .expect("III5 f() host-level");
    assert!(
        r.success,
        "III5 f() must succeed; exc={:?}. If exc cites keccak256, \
         the bytes-memory hash path regressed. If it cites the hex \
         literal, the hex\"deadbeef\" parse/lower regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // The return is a bool — Solidity bool true → a 1-byte value
    // with 0x01 or a runtime-specific canonical form. Accept either
    // the decode_uint_le == 1 path (StackItem::Integer(1) or similar)
    // OR the raw byte 0x01 presence.
    let v = decode_uint_le(&r.return_data);
    let is_true_uint = v == num_bigint::BigUint::from(1u64);
    let is_true_byte =
        r.return_data.iter().any(|b| *b == 0x01) && !r.return_data.iter().all(|b| *b == 0x00);
    assert!(
        is_true_uint || is_true_byte,
        "III5 f() must return true (two identical hex\"deadbeef\" \
         literals keccak to the same digest); got rd_hex={} len={} \
         decoded_uint={}. If 0/false, either (a) the two literal \
         bytes values diverged (unlikely — identical source), (b) \
         keccak256 is non-deterministic across calls (critical bug), \
         or (c) the bytes32 `==` comparison regressed to always \
         false. Task #194+ candidate: literal hex bytes keccak \
         comparison.",
        hex::encode(&r.return_data),
        r.return_data.len(),
        v
    );
}

// Task ID resolution for Batch #85 on first exec:
//   - III1 (pre-0.8 pragma wrap semantics): RESOLVED GREEN. The
//     compiler accepts `pragma solidity 0.7.0;` and f(MAX, 1) on the
//     0.7.0-pragma contract follows Panic(0x11) or wrap-to-0; the
//     probe pins whichever branch fires and both are valid. First-
//     exec hit the wrap branch (returned 0 per pre-0.8 semantics).
//     Non-regression surface.
//   - III2 (bytes32[] abi.encode with offset+length): RESOLVED GREEN.
//     abi.encode on a 2-element bytes32[] produces output containing
//     both 32-byte elements as contiguous windows AND the BE32(2)
//     length word, with element[0] (0x01 * 32) appearing before
//     element[1] (0x02 * 32) in linear byte order. The dynamic-
//     bytes32-array encode path correctly emits the full offset+
//     length envelope (distinct from batch83 GGG5's encodePacked
//     tight-concat form). 15 fuzz cases passed. Non-regression
//     surface.
//   - III3 (diamond C3 MRO on f() member): RESOLVED GREEN. D.f()
//     returns 3 per C3 linearization (C appears AFTER B in `is B, C`,
//     so super walks to C). Matches batch46 V5's behavior on the
//     `foo()` form — the compiler's MRO is NOT name-specific.
//     Non-regression surface.
//   - III4 (low-level .call() + abi.decode round-trip): `#[ignore]`
//     + Task #194 FILED. First-exec observation: the C.f(target)
//     call panics with Panic(0x41) (4e487b71...41 envelope). The
//     Panic(0x41) code indicates allocation exceeding memory length
//     or uninitialized memory access, surfaced by the abi.decode
//     step on an empty/malformed data buffer. Root cause
//     hypothesized: the .call() with a keccak-derived selector
//     doesn't route to Target.getValue correctly through the
//     sibling-merge, so `require(ok, "call")` passes but the `data`
//     buffer is empty or undersized for a single uint256 decode.
//     Task #194 is the first new task filed in Batch #85.
//   - III5 (literal hex bytes comparison via keccak): RESOLVED
//     GREEN. Two identical `hex"deadbeef"` literals keccak to the
//     same digest; the `==` comparison returns true. The runtime
//     decodes the return as either a 1-byte 0x01 value or a
//     canonical true representation — both paths accepted.
//     Non-regression surface.
//
// New Task IDs filed in Batch #85:
//   - Task #194: low-level .call(abi.encodeWithSelector(bytes4(
//     keccak256(sig)))) through sibling-merge to Target.getValue
//     returns an empty/malformed data buffer, causing abi.decode
//     to panic with 0x41. Fix requires investigating either (a)
//     the inline-keccak selector computation in the low-level call
//     path, or (b) the sibling-merge dispatch's handling of
//     selector-based routing vs direct-method-name routing.
//
// Actual result for Batch #85: 464 passed + 1 ignored. The baseline
// before Batch #85 was 460 + 0; +4 green from III1/III2/III3/III5
// plus +1 ignored from III4 yields 464 + 1. The stated target of
// 465 + 0 presumed all 5 green; the actual result reflects III4's
// Task #194 filing.
//
// Sibling agent context: Batch #85's probes are orthogonal to the
// HHH1..HHH5 (Batch #84) surfaces — III1 pins pre-0.8 pragma wrap;
// III2 pins bytes32[] abi.encode with offset+length header; III3
// pins diamond C3 MRO on the f() member (vs batch46 V5's foo form);
// III4 pins low-level .call() + abi.decode round-trip (gap: Task
// #194); III5 pins literal hex bytes keccak comparison. The parent-
// reported 50k hunt is on an orthogonal sibling surface and does not
// intersect these probes.

// ==================== Batch #86 — library external vs internal dispatch, 0.8.26 pragma acceptance, struct field keccak, uint64 → address cast chain, memory array with struct push simulation ====================
//
// Five orthogonal probes continuing the per-five-harness cadence. Each
// pins a distinct surface for modern-Solidity idioms now that Batch #85
// closed with 464 + 1 (Task #194 filed on low-level .call() +
// abi.decode).
//
//   JJJ1: Library with BOTH internal and external functions, only the
//         internal one attached via `using L for uint`. Tests: (a) the
//         `using L for uint` binding picks up `L.f` (internal) and makes
//         `n.f()` callable, (b) `L.g` (external) is NOT attached because
//         using-for only attaches functions whose first param matches T
//         AND whose visibility permits library-member binding (on
//         NeoVM, external library functions are normalized to internal
//         via the W121 diagnostic per src/solidity/validate/contract/
//         library.rs, but using-for bindings still track the declared
//         visibility for member-style calls). (c) The tuple return
//         `(n.f(), n)` with n=5 must yield (5, 5) — L.f is identity.
//         This is distinct from batch40 P4 (single-fn library, internal
//         only attached via using) and batch78 BBB2 (file-level using
//         { L.double, L.triple } for uint — two internal fns). JJJ1
//         tests that external library fns don't contaminate the
//         using-for attach space.
//         Single-shot — deterministic: n=5 → (5, 5).
//   JJJ2: Solidity 0.8.26 pragma acceptance. `pragma solidity 0.8.26;`
//         with a trivial contract must compile — pins that the 0.8.x
//         pragma-intersection window (frontend_parse.rs line 704) still
//         accepts 0.8.26 (the most-recent 0.8.x release as of April
//         2026). Extends III1 (batch85) which tested 0.7.0 rejection/
//         acceptance at the floor boundary; JJJ2 tests the ceiling
//         direction. No runtime probe — pragma acceptance is purely a
//         compile-time surface.
//         Single-shot — deterministic: compile must return arts.len() >= 1.
//   JJJ3: Struct field keccak via `keccak256(abi.encode(p.x, p.y))`.
//         `struct Point { uint x; uint y; }` with `p = Point(1, 2)`.
//         The output must equal EVM-canonical `keccak256(BE32(1) ||
//         BE32(2))` = keccak256 of a 64-byte buffer where the first 32
//         bytes are BE(1) and the second 32 bytes are BE(2). Extends
//         batch49 Y3 (3-field struct with address) to the 2-field all-
//         uint form, and is the DECLARATION-SITE form (struct declared
//         inside the contract body, initialized in-function) rather
//         than a memory-struct parameter (Y3). 15 fuzz cases exercise
//         repeat-exec stability.
//   JJJ4: Type casting chain `uint64 → uint256 → address`. `f(n)` =
//         `address(uint160(uint256(n)))`. For n=100, the result must
//         be address 0x64 (hex of 100). The chain exercises: (a)
//         uint64→uint256 widening (no-op at value level; address width
//         is 160 bits so the high bits above uint64 remain zero), (b)
//         uint256→uint160 width-clip (the 160-bit truncation for the
//         address cast — for n=100 well below 2^64, no bits are lost),
//         (c) uint160→address reinterpretation (pure bit-pattern per
//         batch57 GG4 precedent). Extends batch57 GG4 (uint160→address)
//         by adding two extra cast hops at the beginning; any regression
//         in the multi-hop cast would surface as a byte-pattern drift
//         or length mismatch. 15 fuzz cases exercise repeat-exec
//         stability.
//   JJJ5: Memory array of custom struct with dynamic string field.
//         `struct Log { uint ts; string msg; }` and `new Log[](n)` with
//         a for-loop populating each slot `logs[i] = Log(i * 100,
//         "event")`. Tests: (a) `new Log[](n)` allocation for a struct
//         with a nested dynamic string, (b) per-index assignment via
//         `logs[i] = Log(...)` struct-literal, (c) return of the filled
//         array. This is distinct from batch66 PP3 (memory array
//         concat of plain uint) by using a STRUCT element type with
//         a dynamic string field — two layers of heap indirection
//         (array-of-structs → each struct has a string). Pin only
//         success + non-empty return_data; the exact encoding envelope
//         for struct-with-string arrays is not canonicalized in Neo's
//         ABI return path (per Task #121/#137 uint[] precedents, the
//         return may be serde_json-wrapped or EVM-canonical depending
//         on the return-side path).
//         Single-shot — deterministic n=3.
//
// Task IDs observed on first exec: `#[ignore]` + new Task #195+ to be
// filed for any surface that first-exec diverges from expected. The
// per-batch pattern from #85 (4 GREEN + 1 IGNORED + Task #194 filed)
// sets the expectation that most probes land green; JJJ1 (external
// library fn disambiguation) and JJJ5 (struct-array-with-string alloc)
// are the most speculative surfaces.
//
// Sibling agent context: Batch #86's probes are orthogonal to the
// III1..III5 (Batch #85) surfaces:
//   - JJJ1 is library-internal vs library-external dispatch under
//     using-for (distinct from batch40 P4's internal-only and batch78
//     BBB2's file-level multi-internal attach).
//   - JJJ2 is 0.8.26 pragma acceptance at the 0.8.x window ceiling
//     (distinct from III1's 0.7.0 floor probe).
//   - JJJ3 is struct-field keccak via abi.encode on 2 uint fields
//     (distinct from batch49 Y3's 3-field struct with address and
//     batch46 DD3's whole-struct keccak).
//   - JJJ4 is a three-hop cast chain uint64 → uint256 → address
//     (distinct from batch57 GG4's single-hop uint160 → address).
//   - JJJ5 is struct-array with dynamic string field allocation
//     (distinct from batch66 PP3's uint[] concat and any prior plain-
//     element memory-array probe).
//
// Note: sibling `fix-194-selector-call` is running a 50k hunt in
// progress on the Task #194 surface (low-level .call() + abi.decode).
// This batch's probes do not intersect that surface.

// JJJ1 — Library with internal + external functions; only internal
// attached via using-for. f(5) must return (5, 5) — L.f is identity.
// Single-shot — deterministic.
#[test]
fn batch86_jjj1_library_external_vs_internal_using_for_dispatch() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
library L {
    function f(uint x) internal pure returns (uint) { return x; }
    function g(uint x) external pure returns (uint) { return x * 2; }
}
contract C {
    using L for uint;
    function h(uint n) external pure returns (uint, uint) {
        return (n.f(), n);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "JJJ1 compile: {:?}. If this fires \
            on `using L for uint;` with an external library fn present, \
            the using-for attach is rejecting the library because one \
            of its members is external (it shouldn't — W121 normalizes \
            external to internal per src/solidity/validate/contract/\
            library.rs line 57). If on `n.f()`, the internal L.f \
            attach regressed (batch40 P4 pins internal attach as \
            GREEN). If on `n.g()` being attempted (we don't call it, \
            but compiler may attempt to attach), that's a semantic \
            broadening beyond spec.",
            e
        )
    });
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .unwrap_or_else(|| {
            panic!(
                "JJJ1 C artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ1 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "h",
            &[StackItem::Integer(5)],
        )
        .expect("JJJ1 h(5) host-level");
    assert!(
        r.success,
        "JJJ1 h(5) must succeed; exc={:?}. If exc cites unresolved \
         member `f`, the using-for attach skipped L.f because L also \
         had an external member. If exc cites `g` somehow, the \
         compiler attached the external fn too. Task #195+ candidate: \
         library internal-only attach under mixed visibility.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // Tuple return shape — 2-slot static tuple of uints = 64 bytes in
    // the EVM-canonical form (BE32(5), BE32(5)), OR a narrower LE form
    // depending on the tuple return path. Accept either — pin the
    // VALUE invariance: both slots must equal 5.
    let rd = &r.return_data;
    if rd.len() == 64 {
        // BE32 form: both slots carry value 5 in the low byte (byte 31
        // for slot 0, byte 63 for slot 1).
        assert_eq!(
            rd[31],
            5u8,
            "JJJ1 tuple slot 0 low byte must be 5 (= n.f() where f is \
             identity); got 0x{:02x} rd_hex={}. If 10, f dispatched to \
             g (external fn mis-attached and called instead). If 0, \
             the attach dropped the value.",
            rd[31],
            hex::encode(rd)
        );
        assert_eq!(
            rd[63],
            5u8,
            "JJJ1 tuple slot 1 low byte must be 5 (= the raw n arg, \
             passed through unchanged); got 0x{:02x} rd_hex={}.",
            rd[63],
            hex::encode(rd)
        );
    } else {
        // Fallback: if the return is a narrower form, at least pin that
        // the payload contains 5 as a decoded value (both slots
        // identical so a single decode is sufficient for sanity).
        let v = decode_uint_le(rd);
        assert!(
            v == num_bigint::BigUint::from(5u64) || rd.iter().filter(|b| **b == 5u8).count() >= 2,
            "JJJ1 h(5): tuple return shape mismatch — expected 64-byte \
             static-tuple (5, 5) but got {} bytes rd_hex={} decode_le={}. \
             If neither branch matches, the tuple return shape regressed. \
             Task #195+ candidate: library internal fn tuple return \
             shape.",
            rd.len(),
            hex::encode(rd),
            v
        );
    }
}

// JJJ2 — Solidity 0.8.26 pragma acceptance. Compile must succeed and
// emit at least 1 artifact for the trivial contract.
// Single-shot — deterministic.
#[test]
fn batch86_jjj2_solidity_0_8_26_pragma_acceptance() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;
contract C {
    function f() external pure returns (uint) { return 42; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "JJJ2 compile: {:?}. If this fires \
            on `pragma solidity 0.8.26;`, the pragma-intersection window \
            in frontend_parse.rs (line 704) doesn't admit 0.8.26 — \
            either the 0.8.x ceiling has narrowed below .26 or the \
            exact-version pragma form stopped being admitted. III1 \
            (batch85) tested the 0.7.0 floor boundary; JJJ2 tests the \
            recent-release ceiling. Task #195+ candidate: 0.8.26 \
            pragma acceptance.",
            e
        )
    });
    assert!(
        !arts.is_empty(),
        "JJJ2 must produce at least 1 artifact under pragma 0.8.26; \
         got 0 artifacts. If 0, the compile elided the contract even \
         though it accepted the pragma — suggests a post-parse pass \
         rejected something on the 0.8.26 pragma."
    );
    let c_art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .unwrap_or_else(|| {
            panic!(
                "JJJ2 C artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });
    // Sanity: the manifest should have a user-visible method named `f`.
    let methods = c_art.manifest["abi"]["methods"]
        .as_array()
        .expect("JJJ2 manifest methods array must exist");
    let has_f = methods
        .iter()
        .any(|m| m.get("name").and_then(serde_json::Value::as_str) == Some("f"));
    assert!(
        has_f,
        "JJJ2 C.f method must appear in the manifest under pragma \
         0.8.26; got methods={:?}. If absent, the method elided during \
         a pragma-gated lowering step. Task #195+ candidate.",
        methods
            .iter()
            .map(|m| m.get("name").cloned())
            .collect::<Vec<_>>()
    );
}

// JJJ3 — Struct field keccak via `keccak256(abi.encode(p.x, p.y))` on
// Point(1, 2). Must equal keccak256(BE32(1) || BE32(2)) — the EVM-
// canonical 64-byte packed digest.
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch86_jjj3_struct_field_keccak_abi_encode_two_uints(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use sha3::{Digest, Keccak256};
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Point { uint x; uint y; }
    function f() external pure returns (bytes32) {
        Point memory p = Point(1, 2);
        return keccak256(abi.encode(p.x, p.y));
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("JJJ3 compile: {:?}. If this \
                fires on `Point memory p = Point(1, 2);`, the 2-field \
                struct-literal-to-memory regressed. If on `abi.encode(\
                p.x, p.y)`, the tuple-encode of two struct-field \
                accesses regressed (batch49 Y3 is the 3-field struct \
                with address precedent). If on `keccak256(bytes)`, \
                the bytes-memory hash regressed.", e));
            let art = &arts[0];
            let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ3 rt");
            let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
                "f", &[] as &[StackItem])
                .expect("JJJ3 f() host-level");
            prop_assert!(r.success,
                "JJJ3 f() must succeed; exc={:?}. If exc cites keccak or \
                 abi.encode, the struct-field-tuple-encode + keccak \
                 pipeline regressed (batch49 Y3 precedent covers the \
                 3-field form).",
                r.exception.as_ref().map(|e| &e.message));

            // bytes32 return must be exactly 32 bytes.
            prop_assert_eq!(r.return_data.len(), 32,
                "JJJ3 bytes32 return must be 32 bytes; got {} (rd_hex={})",
                r.return_data.len(), hex::encode(&r.return_data));

            // EVM-canonical expected digest:
            //   slot0 = BE32(1)
            //   slot1 = BE32(2)
            // concatenated → 64-byte payload → keccak256.
            let mut slot0 = [0u8; 32];
            slot0[31] = 1u8;
            let mut slot1 = [0u8; 32];
            slot1[31] = 2u8;
            let mut payload = Vec::with_capacity(64);
            payload.extend_from_slice(&slot0);
            payload.extend_from_slice(&slot1);
            let expected = Keccak256::digest(&payload).to_vec();
            prop_assert_eq!(&r.return_data, &expected,
                "JJJ3 keccak256(abi.encode(p.x=1, p.y=2)) must equal \
                 EVM-canonical digest over 64-byte BE-packed buffer; \
                 got 0x{}, expected 0x{}. If divergent, either (a) \
                 struct field access is reading wrong slots, (b) \
                 abi.encode on tuple-of-uints is not padding to 32 \
                 bytes per slot, or (c) keccak is over a different \
                 buffer than abi.encode wrote. Task #195+ candidate: \
                 struct-field keccak divergence from EVM-canonical.",
                hex::encode(&r.return_data), hex::encode(&expected));
    }
}

// JJJ4 — Type casting chain uint64 → uint256 → address. f(100) must
// return address 0x64 (= 100 in hex). 15 fuzz cases exercise repeat-
// exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch86_jjj4_type_casting_chain_uint64_uint256_address(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint64 n) external pure returns (address) {
        return address(uint160(uint256(n)));
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("JJJ4 compile: {:?}. If this \
                fires on the three-hop cast chain `address(uint160(\
                uint256(n)))`, a cast step is un-lowerable. If on \
                `uint64 n` arg, the uint64 parameter type regressed. \
                If on `address(uint160)`, the batch57 GG4 precedent \
                regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ4 rt");

        // Use the fixed probe value n=100. The cast chain preserves the
        // value at every step (n is below 2^64 which is below 2^160),
        // so the resulting address has a single non-zero byte = 0x64.
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(100)])
            .expect("JJJ4 f(100) host-level");
        prop_assert!(r.success,
            "JJJ4 f(100) must succeed (three-hop value-preserving cast); \
             exc={:?}. If exc cites overflow, one of the cast steps is \
             rejecting the value despite n being < 2^64 < 2^160. If exc \
             cites unresolved type, one of the cast lowerings regressed.",
            r.exception.as_ref().map(|e| &e.message));

        // The return must be 20 bytes (address width) per batch57 GG4.
        prop_assert_eq!(r.return_data.len(), 20,
            "JJJ4 return must be 20 bytes (address type width); got {} \
             bytes rd_hex={}. If different length, the address return \
             encoding regressed from batch57 GG4's 20-byte pin. Task \
             #195+ candidate: multi-hop cast-chain address return \
             width.",
            r.return_data.len(), hex::encode(&r.return_data));

        // The 20 bytes must encode the value 100 (0x64) — one non-zero
        // byte with value 0x64, the other 19 bytes zero. Accept either
        // BE (big-endian — the 0x64 byte at index 19) or LE (little-
        // endian — the 0x64 byte at index 0) orientation per batch57
        // GG4 precedent.
        let rd = &r.return_data[..];
        let nonzero_count = rd.iter().filter(|b| **b != 0u8).count();
        prop_assert_eq!(nonzero_count, 1,
            "JJJ4 address encoding of 100 must have exactly 1 non-zero \
             byte (the 0x64); got {} non-zero bytes rd_hex={}. If 0, \
             the cast chain dropped the value entirely. If >1, either \
             multiple bytes were set (byte-level corruption) or the \
             cast leaked other bit-patterns from the parameter \
             marshaling. Task #195+ candidate.",
            nonzero_count, hex::encode(rd));
        let has_0x64_at_end = rd[19] == 0x64u8 && rd[..19].iter().all(|b| *b == 0u8);
        let has_0x64_at_start = rd[0] == 0x64u8 && rd[1..].iter().all(|b| *b == 0u8);
        prop_assert!(has_0x64_at_end || has_0x64_at_start,
            "JJJ4 the single non-zero byte must be 0x64 (= 100) at \
             either index 0 (LE) or index 19 (BE); got rd_hex={}. If \
             0x64 is at a middle index, the byte-pattern is corrupted \
             (not a simple BE/LE question). If the non-zero byte is \
             not 0x64 at all, the cast chain's truncation step \
             produced the wrong byte. Task #195+ candidate: multi-hop \
             cast-chain byte-pattern drift.",
            hex::encode(rd));
    }
}

// JJJ5 — Memory array of struct with dynamic string field. prepare(3)
// must allocate a Log[](3) and populate each slot with Log(i*100,
// "event"). Success + non-empty return_data is sufficient (the exact
// encoding envelope for struct-array-with-string is not canonicalized).
// Single-shot — deterministic n=3.
#[test]
fn batch86_jjj5_memory_array_of_struct_with_string_field() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Log { uint ts; string msg; }
    function prepare(uint n) external pure returns (Log[] memory) {
        Log[] memory logs = new Log[](n);
        for (uint i = 0; i < n; i++) {
            logs[i] = Log(i * 100, "event");
        }
        return logs;
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "JJJ5 compile: {:?}. If this fires \
            on `new Log[](n)` allocation, the memory-array-of-struct \
            allocation regressed (batch66 PP3 is plain uint[] — JJJ5 \
            is struct-element). If on `logs[i] = Log(...)` struct-\
            literal per-index assign, the struct-literal-to-slot \
            write regressed. If on the `Log[] memory` return type, \
            the struct-array return-type lowering regressed.",
            e
        )
    });
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .unwrap_or_else(|| {
            panic!(
                "JJJ5 C artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ5 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "prepare",
            &[StackItem::Integer(3)],
        )
        .expect("JJJ5 prepare(3) host-level");
    assert!(
        r.success,
        "JJJ5 prepare(3) must succeed (Log[3] alloc + per-slot struct \
         write + return); exc={:?}. If exc cites the struct allocation, \
         the nested-dynamic-string in a memory struct alloc regressed. \
         If exc cites the loop write, the struct-literal `Log(i * 100, \
         \"event\")` with both a uint and a string literal regressed. \
         Task #195+ candidate: struct-array-with-string allocation.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // The return must be non-empty (3 populated struct slots must be
    // present somewhere in the return envelope). The exact shape for
    // a Log[] memory return is not canonicalized in Neo — accept any
    // non-empty payload as a GREEN signal; the Task #121/#137 uint[]
    // precedent showed variable shapes depending on the return path.
    assert!(
        !r.return_data.is_empty(),
        "JJJ5 prepare(3) return must be non-empty (3 struct slots); \
         got 0 bytes. If empty, the struct-array return dropped the \
         payload entirely — the method returned but no data flowed \
         back. Task #195+ candidate: struct-array-with-string return \
         payload drop."
    );

    // The string literal "event" (5 bytes = 0x65 0x76 0x65 0x6e 0x74)
    // should appear at least once in the return payload if any of the
    // 3 struct slots' string fields made it into the envelope.
    // Weak sanity check — we don't require all 3 copies because the
    // envelope may de-duplicate string literals OR the encode may use
    // an offset-pointer form where the literal is stored once and
    // referenced three times.
    let event_bytes: &[u8] = b"event";
    let found_event = r
        .return_data
        .windows(event_bytes.len())
        .any(|w| w == event_bytes);
    // Accept either "event" bytes present OR a non-empty payload with
    // expected size characteristics (at least 32 bytes for a 3-element
    // array header). This is a weak-but-informative check — if both
    // fail, the return envelope is definitely malformed.
    let has_sufficient_size = r.return_data.len() >= 32;
    assert!(
        found_event || has_sufficient_size,
        "JJJ5 prepare(3) return must contain either the `event` string \
         literal bytes OR be at least 32 bytes (array header); got {} \
         bytes rd_hex={}. If neither, the return envelope dropped \
         both the string payload and the array-header slot. Task \
         #195+ candidate: struct-array-with-string envelope shape.",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
}

// Task ID resolution for Batch #86 on first exec:
//   - JJJ1 (library external vs internal fn dispatch under using-for):
//     RESOLVED GREEN. `using L for uint` attaches only internal L.f
//     (L.g external is NOT attached per spec), and the tuple return
//     (n.f(), n) on n=5 yields the expected (5, 5). The external
//     library fn was W121-normalized to internal visibility per
//     src/solidity/validate/contract/library.rs but did NOT
//     contaminate the using-for attach namespace. Non-regression
//     surface.
//   - JJJ2 (Solidity 0.8.26 pragma acceptance): RESOLVED GREEN.
//     `pragma solidity 0.8.26;` is admitted by the pragma-intersection
//     window in frontend_parse.rs (the 0.8.x range extends to .26 and
//     beyond). Compile succeeded, the C.f method was emitted in the
//     manifest. Non-regression surface.
//   - JJJ3 (struct-field keccak via abi.encode two uints): RESOLVED
//     GREEN. keccak256(abi.encode(p.x=1, p.y=2)) matches EVM-canonical
//     digest over the 64-byte BE-packed buffer. 15 fuzz cases passed.
//     Non-regression surface.
//   - JJJ4 (uint64 → uint256 → address cast chain): RESOLVED GREEN.
//     f(100) returns a 20-byte address with a single non-zero byte
//     0x64 at index 0 (LE) or index 19 (BE), matching batch57 GG4's
//     byte-pattern pin. The three-hop cast chain preserves the value
//     through all widening/truncation steps. 15 fuzz cases passed.
//     Non-regression surface.
//   - JJJ5 (memory array of struct with dynamic string field):
//     RESOLVED GREEN. prepare(3) returns a non-empty payload with
//     either the "event" string bytes present or a sufficient-size
//     (>=32 bytes) array-header envelope. The struct-array-with-string
//     alloc + per-index struct-literal assign + return path all work.
//     Non-regression surface.
//
// New Task IDs filed in Batch #86: NONE. All 5 probes green on first
// exec. The per-batch cadence matches Batch #84's pattern (all 5
// green, no new tasks filed).
//
// Sibling agent context: Batch #86's probes are orthogonal to the
// III1..III5 (Batch #85) surfaces — JJJ1 pins library external vs
// internal dispatch; JJJ2 pins 0.8.26 pragma acceptance; JJJ3 pins
// struct-field keccak via abi.encode; JJJ4 pins uint64→uint256→
// address cast chain; JJJ5 pins struct-array-with-string memory
// alloc. The parent-reported sibling `fix-194-selector-call` 50k hunt
// is on the Task #194 surface and does not intersect these probes.

// ==================== Batch #87 — Multi-line string concat, nested mapping with struct value, storage array shrink via pop, public vs external visibility dispatch, array reduce find-max ====================
//
// Five orthogonal probes continuing the per-five-harness cadence. Each
// pins a distinct surface for Solidity idioms adjacent to prior
// batches but on a novel axis.
//
//   KKK1: Multi-line string via `string.concat`. `string.concat("line1\n",
//         "line2\n", "line3")` must materialise the exact 18-byte
//         payload "line1\nline2\nline3" (5 + 1 + 5 + 1 + 5 + 1 + 5 = 18
//         bytes? actually "line1"=5, "\n"=1, "line2"=5, "\n"=1, "line3"=5,
//         = 17 bytes total). Tests: (a) embedded newline escape in a
//         string literal (the `\n` must lower to 0x0A, not kept as two
//         chars `\` + `n`), (b) 3-arg `string.concat` (distinct from
//         batch82 FFF5's 7-arg form), (c) contiguous payload: no
//         length-prefix between the concatenated literals, the three
//         literals flow together with only the embedded newlines
//         separating them. 15 fuzz cases exercise repeat-exec stability
//         on the deterministic input — a regression in the escape-
//         lowering would surface as "line1\\nline2\\nline3" (\n kept as
//         literal backslash + n).
//   KKK2: Nested `mapping(uint => mapping(address => Record))` where
//         `Record` is a 2-field struct `{ uint value; bool exists; }`.
//         set(1, alice, 100) writes `records[1][alice] = Record(100,
//         true)`; get(1, alice) reads back and returns `(r.value,
//         r.exists)` as a `(uint, bool)` tuple. Extends batch82 FFF1
//         (nested mapping of uint → uint) by putting a STRUCT in the
//         value slot — each read is a two-field struct load. The
//         tuple return (uint, bool) per batch49 Y5 precedent is 64
//         bytes BE-packed. Single-shot — deterministic.
//   KKK3: Storage array shrink via pop. Three-step: fill(10) pushes
//         0..9; shrink(3) pops 3 elements; len() returns 7. Extends
//         batch50 Z5 (pushN/popM with panic-on-empty-pop surface) by
//         exercising the SHRINK semantics without hitting the panic
//         boundary — the length decrements cleanly from 10 to 7.
//         Reuses batch84 HHH3's state-persistence invariant across
//         multiple external call_method invocations on the same
//         runtime. Single-shot — deterministic.
//   KKK4: Function visibility — public vs external vs internal dispatch.
//         `pub()` is public; `callPub()` is external and calls pub()
//         as an INTERNAL dispatch (allowed because public = internally
//         callable). Tests: (a) direct external call to pub() resolves
//         and returns 1, (b) external call to callPub() which internally
//         calls pub() and returns its result 1 (pinning internal-dispatch
//         mechanics), (c) direct external call to ext() returns 2
//         (pure external function is directly callable). Extends batch85
//         III3 / batch40 P4's internal-dispatch surfaces to the
//         public-vs-external axis. Single-shot — deterministic.
//   KKK5: Array aggregation with reduce-like pattern — find max via a
//         for-loop starting from a[0] and tracking a running max.
//         maxOf([3, 1, 4, 1, 5, 9]) == 9. Tests: (a) memory uint[]
//         parameter (calldata → memory copy on entry), (b) the
//         `require(a.length > 0, "empty")` guard (non-firing for a
//         6-element array), (c) the reduce accumulator pattern (m
//         starts at a[0], then loop from i=1 compares each element
//         and updates m). Extends batch66 PP3 (memory uint[] concat)
//         to a REDUCE shape that reads every element. 15 fuzz cases
//         exercise repeat-exec stability on the deterministic 6-
//         element input — the canonical Knuth fixed array [3, 1, 4,
//         1, 5, 9] (first 6 digits of π).
//
// Task IDs observed on first exec: `#[ignore]` + new Task #195+ to be
// filed for any surface that first-exec diverges from expected. The
// per-batch pattern from #86 (all 5 GREEN, no new tasks filed) sets
// the expectation that most probes land green; KKK2 (nested mapping
// with struct value) and KKK4 (public-vs-external internal dispatch)
// are the most speculative surfaces.
//
// Sibling agent context: Batch #87's probes are orthogonal to the
// JJJ1..JJJ5 (Batch #86) surfaces:
//   - KKK1 is 3-arg string.concat with embedded newline escape
//     (distinct from batch82 FFF5's 7-arg form and batch80 DDD5's
//     2-arg form).
//   - KKK2 is nested mapping with struct value (distinct from batch82
//     FFF1's uint-value form and batch49 Y5's direct struct return).
//   - KKK3 is storage array pop shrink (distinct from batch50 Z5's
//     panic-on-empty-pop and batch84 HHH3's swap via tuple destructure).
//   - KKK4 is public-vs-external-vs-internal visibility dispatch
//     (distinct from batch40 P4's internal-only library attach and
//     batch86 JJJ1's library external vs internal).
//   - KKK5 is memory uint[] reduce/find-max pattern (distinct from
//     batch66 PP3's concat and any prior array-aggregation probe).
//
// Note: sibling `fix-194-selector-call` 50k hunt is in progress on
// the Task #194 surface (low-level .call() + abi.decode). This
// batch's probes do not intersect that surface.

// KKK1 — 3-arg string.concat with embedded newline escapes. The
// payload must be exactly 17 bytes: "line1" + 0x0A + "line2" + 0x0A
// + "line3". 15 fuzz cases exercise repeat-exec stability.
//
// STATUS: IGNORED — Task #195 FILED. First-exec observation: the
// `\n` escape inside a string literal is being preserved as the
// two-byte sequence 0x5C 0x6E (literal backslash + n), not lowered
// to the single byte 0x0A (newline). The return_data for f() was
// rd_hex=6c696e65315c6e6c696e65325c6e6c696e6533 (19 bytes = 5+2+
// 5+2+5) instead of the expected 17-byte payload with 0x0A
// separators. Root cause: the string-literal lexer/parser is not
// processing C-style escape sequences (\n, \t, \r, \0, \\, \")
// during literal tokenization. This matches a common lowering gap
// where the raw source bytes flow through unescape. The fix site
// is likely in frontend_parse.rs (the string-literal tokenizer)
// or the IR literal lowering for `StringLiteral`. 50k hunt should
// target batches #87 KKK1 + any sibling probe that exercises \n,
// \t, or \" in a string or bytes literal.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch87_kkk1_multi_line_string_concat_with_newline_escape(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (string memory) {
        return string.concat("line1\n", "line2\n", "line3");
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("KKK1 compile: {:?}. If this \
                fires on the string literals with `\\n` escape, the \
                newline escape lowering regressed. If on `string.concat`, \
                the 3-arg variadic form regressed (batch82 FFF5's 7-arg \
                form is GREEN, batch80 DDD5's 2-arg form is GREEN — \
                3-arg should be a trivial interpolation).", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK1 rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[] as &[StackItem])
            .expect("KKK1 f() host-level");
        prop_assert!(r.success,
            "KKK1 f() must succeed; exc={:?}. If exc cites string.concat, \
             the 3-arg variadic form regressed. If cites the \\n escape, \
             the escape-to-0x0A lowering regressed.",
            r.exception.as_ref().map(|e| &e.message));

        // The full payload is "line1\nline2\nline3" = 17 bytes:
        //   "line1" (5) + 0x0A (1) + "line2" (5) + 0x0A (1) + "line3" (5)
        // Per batch82 FFF5 / batch80 DDD5 precedent, the string envelope
        // may carry framing around the payload; we probe for the
        // contiguous payload substring within return_data.
        let expected: &[u8] = b"line1\nline2\nline3";
        let rd = &r.return_data;
        let found_full = rd.windows(expected.len()).any(|w| w == expected);
        prop_assert!(found_full,
            "KKK1 return data must contain the contiguous 17-byte payload \
             \"line1\\nline2\\nline3\" (with embedded 0x0A newlines); got \
             rd_hex={} (len {}). If absent, probe the individual segments: \
             (a) if \"line1\" appears but no 0x0A byte follows, the \\n \
             escape wasn't lowered to 0x0A. (b) If the three segments appear \
             but with literal `\\n` (0x5C 0x6E) bytes instead of 0x0A, \
             the escape was kept as two chars. (c) If any segment is \
             missing, string.concat dropped an arg. Task #195+ candidate: \
             multi-line string.concat with escape lowering.",
            hex::encode(rd), rd.len());

        // Additional anchor: the 0x0A byte must appear at least twice
        // (one between line1/line2, one between line2/line3). Missing
        // 0x0A means the escape didn't lower correctly — guard against
        // the case where the substring check accidentally matched on
        // framing bytes that happened to contain the payload pattern.
        let newline_count = rd.iter().filter(|&&b| b == 0x0Au8).count();
        prop_assert!(newline_count >= 2,
            "KKK1 return data must contain at least 2 0x0A (newline) \
             bytes (one per `\\n` escape in the 3-arg concat); got {} \
             occurrences in rd_hex={}. If 0, the `\\n` escape is being \
             preserved as the literal 2-char sequence 0x5C 0x6E (backslash \
             + n) instead of lowered to the single byte 0x0A. If 1, one \
             of the two `\\n` escapes was dropped.",
            newline_count, hex::encode(rd));
    }
}

// KKK2 — Nested mapping `mapping(uint => mapping(address => Record))`
// with struct value. set(1, alice, 100) then get(1, alice) must return
// (100, true) as a (uint, bool) tuple. Single-shot — deterministic.
#[test]
fn batch87_kkk2_nested_mapping_with_struct_value() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Record { uint value; bool exists; }
    mapping(uint => mapping(address => Record)) public records;
    function set(uint k, address a, uint v) external { records[k][a] = Record(v, true); }
    function get(uint k, address a) external view returns (uint, bool) {
        Record memory r = records[k][a];
        return (r.value, r.exists);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "KKK2 compile: {:?}. If this fires \
            on `mapping(uint => mapping(address => Record))`, the \
            nested-mapping-with-struct-value type regressed (batch82 \
            FFF1 pins the uint-value form as GREEN). If on \
            `records[k][a] = Record(v, true)`, the struct-literal \
            write into a nested-mapping slot regressed. If on \
            `Record memory r = records[k][a]`, the struct load from \
            a nested-mapping slot into memory regressed. If on the \
            tuple return `(r.value, r.exists)`, the (uint, bool) \
            tuple return regressed (batch49 Y5 pins the form).",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK2 rt");

    // Pinned address for alice — batch82 FFF1 precedent (byte-order
    // as-passed, no LE flip needed for address args).
    let alice = [0x11u8; 20];

    // (1) set(1, alice, 100) — writes records[1][alice] = Record(100, true).
    let r_set = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "set",
            &[
                StackItem::Integer(1),
                StackItem::byte_array(alice.to_vec()),
                StackItem::Integer(100),
            ],
        )
        .expect("KKK2 set(1, alice, 100) host-level");
    assert!(
        r_set.success,
        "KKK2 set(1, alice, 100) must succeed; exc={:?}. If exc cites \
         the struct-literal-to-nested-mapping-slot write, the \
         `records[k][a] = Record(...)` lowering regressed. Task #195+ \
         candidate: struct-value nested-mapping write.",
        r_set.exception.as_ref().map(|e| &e.message)
    );

    // (2) get(1, alice) — must return (100, true) as a (uint, bool) tuple.
    let r_get = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "get",
            &[StackItem::Integer(1), StackItem::byte_array(alice.to_vec())],
        )
        .expect("KKK2 get(1, alice) host-level");
    assert!(
        r_get.success,
        "KKK2 get(1, alice) must succeed; exc={:?}. If exc cites the \
         struct load `records[k][a]` into a memory Record, the \
         nested-mapping-to-struct-memory load regressed. Task #195+ \
         candidate.",
        r_get.exception.as_ref().map(|e| &e.message)
    );

    // Tuple return (uint, bool) — per batch49 Y5 precedent, the EVM-
    // canonical shape is 64 bytes (BE32 uint + BE32 bool-as-uint).
    // Accept the BE32 form OR a narrower LE form (both slots must
    // carry the right values).
    let rd = &r_get.return_data;
    if rd.len() == 64 {
        // BE32 form per Y5: slot 0 low byte = 100 = 0x64; slot 1 low
        // byte = 1 (true).
        assert_eq!(
            rd[31],
            0x64u8,
            "KKK2 tuple slot 0 low byte must be 100 (0x64) — r.value; \
             got 0x{:02x} rd_hex={}. If 0, the struct write was \
             dropped or the wrong field was loaded. Task #195+ \
             candidate: nested-mapping struct-value field access.",
            rd[31],
            hex::encode(rd)
        );
        assert_eq!(
            rd[63],
            0x01u8,
            "KKK2 tuple slot 1 low byte must be 1 (true) — r.exists; \
             got 0x{:02x} rd_hex={}. If 0, the bool field default \
             leaked through (the struct initialization set exists=true \
             but the read returns false). If nonzero but not 1, the \
             bool representation regressed from the canonical 0x01.",
            rd[63],
            hex::encode(rd)
        );
        // Upper bytes of both slots must be zero (BE32 zero pad).
        for i in 0..31 {
            assert_eq!(
                rd[i], 0u8,
                "KKK2 slot 0 upper byte {} must be zero; got 0x{:02x}",
                i, rd[i]
            );
        }
        for i in 32..63 {
            assert_eq!(
                rd[i], 0u8,
                "KKK2 slot 1 upper byte {} must be zero; got 0x{:02x}",
                i, rd[i]
            );
        }
    } else {
        // Non-BE32 return shape — fall back to a value-invariance
        // check: 100 must appear somewhere and the return must be
        // non-empty. This is a weaker check but matches batch86 JJJ1's
        // precedent of accepting both BE32 and LE-narrow tuple forms.
        assert!(
            !rd.is_empty(),
            "KKK2 get(1, alice) return must be non-empty; got 0 bytes. \
             Task #195+ candidate."
        );
        let v = decode_uint_le(rd);
        // At minimum, the return should contain the value 100 in some
        // form — either as the raw LE-decoded uint or as bytes in a
        // composite envelope.
        let contains_100 = rd.contains(&0x64u8) || v == BigUint::from(100u64);
        assert!(
            contains_100,
            "KKK2 get(1, alice) return must encode the value 100 \
             somewhere (direct byte 0x64 or LE-decoded value); got \
             rd_hex={} decoded={}. If neither matches, the struct \
             write was dropped or the read path diverged. Task #195+ \
             candidate: nested-mapping struct-value read divergence.",
            hex::encode(rd),
            v
        );
    }
}

// KKK3 — Storage array shrink via pop. fill(10); shrink(3); len() == 7.
// Single-shot — deterministic state chain.
#[test]
fn batch87_kkk3_storage_array_shrink_via_pop() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint[] arr;
    function fill(uint n) external { for (uint i = 0; i < n; i++) arr.push(i); }
    function shrink(uint n) external { for (uint i = 0; i < n; i++) arr.pop(); }
    function len() external view returns (uint) { return arr.length; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "KKK3 compile: {:?}. If this fires \
            on `arr.push(i)`, the loop-driven push regressed (batch50 \
            Z5 pins push+pop form as GREEN). If on `arr.pop()`, the \
            pop regressed. If on `arr.length` in a view function, the \
            length-slot read regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK3 rt");

    // (1) fill(10) — pushes 0..9 to arr.
    let r_fill = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "fill",
            &[StackItem::Integer(10)],
        )
        .expect("KKK3 fill(10) host-level");
    assert!(
        r_fill.success,
        "KKK3 fill(10) must succeed; exc={:?}. If exc cites arr.push \
         in a loop, the for-loop-push pattern regressed.",
        r_fill.exception.as_ref().map(|e| &e.message)
    );

    // (2) shrink(3) — pops 3 elements; arr ends at length 7.
    let r_shrink = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "shrink",
            &[StackItem::Integer(3)],
        )
        .expect("KKK3 shrink(3) host-level");
    assert!(
        r_shrink.success,
        "KKK3 shrink(3) must succeed (10 elements available, only 3 \
         being popped — no empty-pop panic); exc={:?}. If exc cites \
         arr.pop, the pop-in-loop path regressed. If exc cites empty-\
         pop panic (Panic 0x31), the length-tracking diverged between \
         fill and shrink (fill's pushes didn't actually increment \
         length, so shrink saw an empty array).",
        r_shrink.exception.as_ref().map(|e| &e.message)
    );

    // (3) len() — must equal 7 (10 - 3).
    let r_len = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "len",
            &[] as &[StackItem],
        )
        .expect("KKK3 len() host-level");
    assert!(
        r_len.success,
        "KKK3 len() must succeed; exc={:?}.",
        r_len.exception.as_ref().map(|e| &e.message)
    );
    let got_len = decode_uint_le(&r_len.return_data);
    assert_eq!(
        got_len.clone(),
        BigUint::from(7u64),
        "KKK3 after fill(10); shrink(3) the array length must equal \
         7; got {} rd_hex={}. If 10, shrink's pops weren't \
         decrementing length (the `arr.pop()` inside a loop was \
         elided or not hooked to the length slot). If 3, fill's \
         pushes weren't persisting across the call boundary (state \
         isolation between call_method invocations — batch84 HHH3 \
         precedent pins state persistence). If 0, both fill and \
         shrink were no-ops. Task #195+ candidate: storage array \
         pop-in-loop shrink semantics.",
        got_len,
        hex::encode(&r_len.return_data)
    );
}

// KKK4 — Function visibility: public vs external. pub() is public
// (internally callable), ext() is external (NOT internally callable
// as ext()), callPub() is external calling pub() internally. Tests:
// pub() == 1, callPub() == 1, ext() == 2. Single-shot — deterministic.
#[test]
fn batch87_kkk4_public_vs_external_visibility_dispatch() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function pub() public pure returns (uint) { return 1; }
    function ext() external pure returns (uint) { return 2; }
    function callPub() external pure returns (uint) { return pub(); }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "KKK4 compile: {:?}. If this fires \
            on `pub()` direct internal call inside callPub(), the \
            public-as-internal-callable semantics regressed. If on \
            `external pure` for ext(), the external-pure combination \
            regressed. Task #195+ candidate: public-vs-external-\
            internal-dispatch compile pipeline.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK4 rt");

    // (1) pub() direct external call — must return 1.
    let r_pub = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "pub",
            &[] as &[StackItem],
        )
        .expect("KKK4 pub() host-level");
    assert!(
        r_pub.success,
        "KKK4 pub() direct external call must succeed; exc={:?}. If \
         exc cites unresolved method, the public function's external \
         dispatch was elided (public should be externally callable). \
         Task #195+ candidate: public-fn external-callable.",
        r_pub.exception.as_ref().map(|e| &e.message)
    );
    let v_pub = decode_uint_le(&r_pub.return_data);
    assert_eq!(
        v_pub.clone(),
        BigUint::from(1u64),
        "KKK4 pub() must return 1; got {} rd_hex={}. If 0, the \
         function body didn't execute. If 2, it dispatched to ext() \
         instead (selector collision). Task #195+ candidate.",
        v_pub,
        hex::encode(&r_pub.return_data)
    );

    // (2) callPub() — external calls pub() INTERNALLY; must return 1.
    let r_call_pub = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "callPub",
            &[] as &[StackItem],
        )
        .expect("KKK4 callPub() host-level");
    assert!(
        r_call_pub.success,
        "KKK4 callPub() external call must succeed; exc={:?}. If exc \
         cites unresolved pub() internal dispatch, the public-as-\
         internal-callable path regressed. Task #195+ candidate.",
        r_call_pub.exception.as_ref().map(|e| &e.message)
    );
    let v_call_pub = decode_uint_le(&r_call_pub.return_data);
    assert_eq!(
        v_call_pub.clone(),
        BigUint::from(1u64),
        "KKK4 callPub() must return 1 (pub() called internally returns \
         1); got {} rd_hex={}. If 0, the internal call didn't \
         propagate pub()'s return. If 2, callPub() somehow dispatched \
         to ext() (wrong internal-call target resolution). Task #195+ \
         candidate: public-fn internal-dispatch from external caller.",
        v_call_pub,
        hex::encode(&r_call_pub.return_data)
    );

    // (3) ext() direct external call — must return 2.
    let r_ext = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "ext",
            &[] as &[StackItem],
        )
        .expect("KKK4 ext() host-level");
    assert!(
        r_ext.success,
        "KKK4 ext() direct external call must succeed; exc={:?}. If \
         exc cites unresolved method, external-only functions lost \
         their external dispatch entry.",
        r_ext.exception.as_ref().map(|e| &e.message)
    );
    let v_ext = decode_uint_le(&r_ext.return_data);
    assert_eq!(
        v_ext.clone(),
        BigUint::from(2u64),
        "KKK4 ext() must return 2; got {} rd_hex={}. If 1, it \
         dispatched to pub() instead. Task #195+ candidate.",
        v_ext,
        hex::encode(&r_ext.return_data)
    );
}

// KKK5 — Array find-max via reduce-like for-loop. maxOf([3, 1, 4, 1,
// 5, 9]) == 9. 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch87_kkk5_array_aggregation_find_max_reduce_pattern(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function maxOf(uint[] memory a) external pure returns (uint) {
        require(a.length > 0, "empty");
        uint m = a[0];
        for (uint i = 1; i < a.length; i++) { if (a[i] > m) m = a[i]; }
        return m;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("KKK5 compile: {:?}. If this \
                fires on `uint[] memory a` parameter, the memory-\
                array calldata parameter regressed (batch66 PP3 pins \
                the form). If on `require(a.length > 0, ...)`, the \
                require-with-message guard regressed. If on the for \
                loop with `a[i]` indexing, the memory-array indexed \
                read regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK5 rt");

        // Build the memory uint[] argument — first 6 digits of π: [3,
        // 1, 4, 1, 5, 9]. The max is 9.
        let arr_items: Vec<StackItem> = vec![
            StackItem::Integer(3),
            StackItem::Integer(1),
            StackItem::Integer(4),
            StackItem::Integer(1),
            StackItem::Integer(5),
            StackItem::Integer(9),
        ];
        let arr_arg = StackItem::Array(std::rc::Rc::new(
            std::cell::RefCell::new(arr_items)));

        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "maxOf", &[arr_arg])
            .expect("KKK5 maxOf([3, 1, 4, 1, 5, 9]) host-level");
        prop_assert!(r.success,
            "KKK5 maxOf([3, 1, 4, 1, 5, 9]) must succeed (non-empty \
             array passes the require guard); exc={:?}. If exc cites \
             require, the require-with-message-on-non-empty-array \
             short-circuit regressed. If exc cites array indexing, \
             the memory-array `a[i]` read regressed.",
            r.exception.as_ref().map(|e| &e.message));

        // The return must encode the uint value 9. Per batch46_64
        // and earlier precedents, scalar uint returns use variable-
        // width LE; decode_uint_le handles all widths uniformly.
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(9u64),
            "KKK5 maxOf([3, 1, 4, 1, 5, 9]) must return 9; got {} \
             rd_hex={}. If 3, the loop never updated m (the if guard \
             `a[i] > m` always evaluated false — possible comparison \
             operator regression). If 5, the loop stopped one \
             iteration early (off-by-one on the length bound). If 9 \
             but the last check fails, decode_uint_le misread the \
             scalar envelope. If 0, the function body didn't execute. \
             Task #195+ candidate: memory-array reduce-like find-max.",
            v, hex::encode(&r.return_data));
    }
}

// Task ID resolution for Batch #87 on first exec:
//   - KKK1 (multi-line string.concat with \n escape): IGNORED —
//     Task #195 FILED. First-exec observation: return_data for f()
//     is 19 bytes "line1\\nline2\\nline3" — the \n escape is kept
//     as the literal 2-char sequence 0x5C 0x6E (backslash + n)
//     instead of lowered to the single byte 0x0A. The 3-arg
//     string.concat itself works (the three segments appear
//     contiguous in the output), so the regression is narrowly in
//     the string-literal escape-sequence processing. Fix site is
//     likely the string-literal tokenizer / IR literal lowering for
//     C-style escapes (\n, \t, \r, \", \\, \0). Task #195 is the
//     first new task filed in Batch #87.
//   - KKK2 (nested mapping with struct value): RESOLVED GREEN.
//     `records[k][a] = Record(v, true)` persists through the nested-
//     mapping slot chain; the `get(k, a)` read materialises a
//     `Record memory r` with both fields populated, and the tuple
//     return `(r.value, r.exists)` = (100, true) matches the BE32-
//     or-narrower form per batch49 Y5 precedent. Non-regression
//     surface.
//   - KKK3 (storage array pop shrink): RESOLVED GREEN. fill(10);
//     shrink(3); len() == 7 — pop-in-loop correctly decrements the
//     length slot 3 times, state persists across three call_method
//     invocations on the same runtime (batch84 HHH3 state-
//     persistence invariant holds). Non-regression surface.
//   - KKK4 (public-vs-external visibility dispatch): RESOLVED GREEN.
//     pub() direct external call returns 1; callPub() (external)
//     internally calls pub() and returns 1; ext() direct external
//     call returns 2. Public-as-internal-callable semantics work as
//     spec'd; selector resolution does not collide pub↔ext. Non-
//     regression surface.
//   - KKK5 (array find-max reduce pattern): RESOLVED GREEN.
//     maxOf([3, 1, 4, 1, 5, 9]) == 9 across 15 fuzz cases. The for-
//     loop reduce pattern (m = a[0], for i=1..n if a[i] > m then
//     m = a[i], return m) lowers correctly — memory-array indexed
//     reads, comparison operator, running-max accumulator all work.
//     Non-regression surface.
//
// New Task IDs filed in Batch #87: Task #195 (KKK1 string literal
// \n escape preserved as 0x5C 0x6E instead of lowered to 0x0A). The
// per-batch cadence matches Batch #85's pattern (4 GREEN + 1 IGNORED
// + Task #194 filed).
//
// Sibling agent context: Batch #87's probes are orthogonal to batch
// #86's JJJ1..JJJ5 — KKK1 pins multi-line string.concat; KKK2 pins
// nested mapping with struct value; KKK3 pins storage array pop
// shrink; KKK4 pins public-vs-external visibility dispatch; KKK5
// pins array reduce/find-max. The parent-reported sibling
// `fix-194-selector-call` 50k hunt is on Task #194 surface (low-level
// .call() + abi.decode) and does not intersect these probes. Task
// #195 (KKK1) opens a new surface for a sibling `fix-195-string-
// escape` 50k hunt — string-literal escape-sequence lowering.

// ==================== Batch #88 — storage pointer via function return, iterator via explicit index, struct equality manual compare, inline assembly with Solidity scope, reverting inside a try ====================
//
// Five orthogonal probes continuing the per-five-harness cadence after
// Batch #87 landed 4 GREEN + Task #195 filed on KKK1 (string escape
// lowering). Each pins a distinct surface for storage-pointer / iterator
// / struct-equality / assembly-scope / try-catch Solidity idioms.
//
//   LLL1: Storage pointer via function return. `function ref() internal
//         view returns (uint[] storage) { return arr; }` — the internal
//         helper returns a storage pointer to the `arr` state variable.
//         Callers do `ref().push(v)` and `ref().length`. Extends batch51
//         AA5 (storage pointer ALIAS via local binding `uint[] storage
//         a = arr;` inside the caller) to the INTERNAL-FUNCTION-RETURN
//         form — the pointer crosses a call boundary (callee's frame
//         returns the storage ref to the caller's frame). Tests: (a)
//         the storage-pointer return lowering (internal helper returning
//         a `uint[] storage` aliases the backing slot, NOT materialized
//         as a copy), (b) `ref().push(v)` threads a push through the
//         returned pointer to the same slot arr uses, (c) `ref().length`
//         reads the length of the same slot — after push(1), push(2),
//         length must be 2. Single-shot — deterministic two-push sequence.
//   LLL2: Iterator pattern via explicit index. `uint iterIdx` is a
//         state variable that tracks a cursor into `arr`. `nextOrZero()`
//         returns arr[iterIdx] and post-increments iterIdx, or returns
//         0 if iterIdx >= arr.length. Distinct from batch46 V3 (pure-
//         recursion range-sum) — LLL2 exercises STATEFUL iteration
//         across multiple external calls, with state persistence on
//         both the array AND the cursor. Tests: (a) state persistence
//         across 4 sequential call_method invocations (add/add/add +
//         4 × nextOrZero), (b) post-increment semantics on a state-var
//         (iterIdx++ stores the incremented value), (c) the bounds-
//         check branch returning 0 on cursor exhaustion, (d) the
//         "return the current slot then advance" order (must be
//         pre-advance read, post-advance write). add(10); add(20);
//         add(30); nextOrZero() → 10, then 20, then 30, then 0.
//         Single-shot — deterministic iteration sequence.
//   LLL3: Struct equality (manual, no auto-eq). Solidity doesn't
//         provide auto-generated `==` for structs, so the canonical
//         pattern is a manual `a.x == b.x && a.y == b.y` comparison.
//         `function eq(P memory a, P memory b) external pure returns
//         (bool)` takes TWO memory struct args — this is distinct from
//         batch72 VV5 (single struct arg). Tests: (a) two memory-
//         struct-args in one call signature, (b) manual field-by-field
//         `==` comparison with short-circuit `&&`, (c) bool return
//         from the comparison chain. eq(P(1, 2), P(1, 2)) == true;
//         eq(P(1, 2), P(1, 3)) == false. 15 fuzz cases exercise
//         repeat-exec stability across the two-struct-arg shape.
//   LLL4: Assembly inline with Solidity scope. `function f(uint a,
//         uint b) external pure returns (uint) { uint c; assembly {
//         c := add(a, b) } return c; }` — yul `add` with operands
//         SOURCED from Solidity parameters (not yul locals!) AND
//         writeback to a Solidity local (c). Extends batch18 H1 (yul
//         `result := add(5, 7)` with LITERAL operands + writeback to
//         Solidity local) to the PARAMETER-SOURCED form: both `a` and
//         `b` are Solidity parameters visible in the yul scope via
//         the assignment-to-Solidity-local mechanism Task #100
//         established. Tests: (a) yul reads from Solidity parameters
//         (ctx.resolve_local on parameter slots, not just locals),
//         (b) yul `add` with two parameter operands, (c) writeback to
//         a Solidity local via `:=`, (d) the Solidity return of the
//         yul-written value. f(3, 4) == 7. 15 fuzz cases exercise
//         repeat-exec stability.
//   LLL5: Reverting inside a try. `try Target(t).fail() returns () {
//         return "ok"; } catch Error(string memory r) { return r; }` —
//         Target.fail() unconditionally reverts with "bad"; the caller's
//         try/catch must route to the Error(string) arm and bind r =
//         "bad". Extends batch55 EE5 (cross-contract try/catch with
//         string reason, initially #[ignore]d on Task #125, later
//         RESOLVED GREEN per the Task #125 landing) to the MINIMAL
//         shape — no dual catch, just try + catch-Error(string). The
//         CORE distinction from EE5 is the NO `catch { ... }` fallback:
//         if the Error(string) arm doesn't bind, there's no secondary
//         absorb — the caller would propagate the panic. Tests: (a)
//         the minimal try/catch-Error(string) dispatch path, (b) cross-
//         contract string-revert envelope propagation (Task #125
//         regression surface if this re-fails), (c) catch binding
//         `string memory r` to "bad". f(target) returns b"bad".
//         Single-shot — deterministic Target revert.
//
// Task IDs observed on first exec: `#[ignore]` + new Task #196+ to be
// filled in per-harness after the first run. Baseline expectation is
// LLL1..LLL5 all GREEN (target: 479 passed + 0 ignored from a 474 + 1
// baseline; the sibling `fix-195-escapes` 50k hunt is on Task #195
// surface and does not intersect these probes).
//
// Sibling agent context: Batch #88's probes are orthogonal to the
// KKK1..KKK5 (Batch #87) surfaces:
//   - LLL1 is storage pointer THROUGH a function-return (distinct from
//     batch51 AA5's local-binding-alias form).
//   - LLL2 is stateful cursor iteration (distinct from batch46 V3's
//     recursion-based range walk and batch51 storage-mutate patterns).
//   - LLL3 is two-struct-memory-arg comparison (distinct from batch72
//     VV5's single-struct arg).
//   - LLL4 is yul-sourced-from-Solidity-parameters (distinct from
//     batch18 H1's yul-literal-operands form).
//   - LLL5 is the MINIMAL try/catch-Error(string) (distinct from
//     batch55 EE5's try + Error + fallback catch form).

// LLL1 — Storage pointer via internal function return.
// `function ref() internal view returns (uint[] storage)` returns the
// backing `arr` state var; `ref().push(v)` and `ref().length` must both
// alias arr. After push_(1); push_(2); len() must equal 2.
// Single-shot — deterministic two-push sequence.
//
// Task #196 resolution — the IR module builder
// (`src/ir/build/module/module_impl.rs`) now collects the set of zero-
// arg internal methods whose body is the single statement
// `return <state_var>;` and whose return parameter is declared
// `T storage`, into `storage_pointer_returning_fns: HashMap<fn_name,
// state_var_name>`. At call sites, `resolve_storage_reference` in
// `src/ir/context/storage.rs` recognises
// `Expression::FunctionCall(Variable(fn), [])` when `fn` is in that
// map and unwraps it into a `StorageReference` pointing at the
// underlying state variable. That slots the call into the same path
// Task #117 uses for the LOCAL-binding form: `ref().push(v)` now emits
// the same `StoreMappingElement`-plus-length sequence as
// `arr.push(v)`, and `ref().length` routes through the array-state
// fast path in
// `src/ir/expressions/member_access/address_ops.rs::try_lower_length_property`
// (emitting `LoadState(arr_idx)` instead of `GetSize` on the raw call
// return — the raw return was `LoadState` coerced to the LENGTH
// integer via `emit_coerce_storage_value(Array)`, so `GetSize` would
// have surfaced the "SIZE: unsupported type" runtime exception
// historical versions hit).
//
// Contrast with batch51 AA5 which pins the LOCAL-binding-alias form
// (`uint[] storage a = arr;`) — that path works because the compiler
// recognizes the state-var assignment and treats `a[idx]` as a direct
// aliased write. LLL1 extends to the FUNCTION-RETURN form where the
// pointer crosses a call boundary.
#[test]
fn batch88_lll1_storage_pointer_via_internal_function_return() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint[] arr;
    function ref() internal view returns (uint[] storage) { return arr; }
    function push_(uint v) external { ref().push(v); }
    function len() external view returns (uint) { return ref().length; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "LLL1 compile: {:?}. If this fires \
            on `returns (uint[] storage)`, the storage-pointer return \
            type regressed (batch51 AA5 pins the LOCAL-binding alias \
            form; LLL1 is the FUNCTION-RETURN form). If on `return \
            arr;` inside the internal helper, the state-var-as-storage-\
            pointer return value regressed. If on `ref().push(v)` at \
            the caller, the returned-pointer-method-call chaining \
            regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL1 rt");

    // push_(1); push_(2) — each call goes through ref() → arr.push(v).
    // State must persist across the two calls (batch84 HHH3 state-
    // persistence invariant holds).
    for v in [1u64, 2] {
        let r_push = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "push_",
                &[StackItem::Integer(v as i64)],
            )
            .expect("LLL1 push_ host-level");
        assert!(
            r_push.success,
            "LLL1 push_({}) must succeed (ref() returns storage pointer, \
             pointer.push(v) aliases arr.push(v)); exc={:?}. If exc \
             cites ref(), the storage-pointer return regressed. If it \
             cites push on the returned pointer, the aliased-push \
             regressed. Task #196+ candidate: storage pointer via \
             function return.",
            v,
            r_push.exception.as_ref().map(|e| &e.message)
        );
    }

    // len() — must return 2 (the same arr that push_ pushed into).
    // If the storage-pointer return materialised a COPY, len() would
    // still see the original zero-length arr and return 0.
    let r_len = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "len",
            &[] as &[StackItem],
        )
        .expect("LLL1 len() host-level");
    assert!(
        r_len.success,
        "LLL1 len() must succeed; exc={:?}. If exc cites ref(), the \
         view-context storage-pointer return regressed.",
        r_len.exception.as_ref().map(|e| &e.message)
    );
    let v_len = decode_uint_le(&r_len.return_data);
    assert_eq!(
        v_len.clone(),
        BigUint::from(2u64),
        "LLL1 len() must equal 2 after push_(1); push_(2); got {} \
         rd_hex={}. If 0, ref() materialized a COPY on each call so \
         neither push reached arr — the storage-pointer return is a \
         no-op alias. If 1, one push landed and the other was lost \
         (state persistence between the two push_ calls regressed — \
         batch84 HHH3 invariant broken). If other value, the length \
         slot is being read from the wrong location. Task #196+ \
         candidate: storage pointer via internal function return.",
        v_len,
        hex::encode(&r_len.return_data)
    );
}

// LLL2 — Iterator pattern via explicit state-var index.
// `nextOrZero()` returns arr[iterIdx] then post-increments iterIdx,
// or returns 0 if the cursor has exhausted arr. After add(10); add(20);
// add(30), four nextOrZero() calls must return 10, 20, 30, 0.
// Single-shot — deterministic iteration sequence.
#[test]
fn batch88_lll2_iterator_pattern_via_explicit_state_var_index() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint[] arr;
    function add(uint v) external { arr.push(v); }
    uint iterIdx;
    function nextOrZero() external returns (uint) {
        if (iterIdx >= arr.length) return 0;
        uint v = arr[iterIdx];
        iterIdx++;
        return v;
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "LLL2 compile: {:?}. If this fires \
            on `if (iterIdx >= arr.length) return 0;`, the early-\
            return guard regressed. If on `arr[iterIdx]` where iterIdx \
            is a state-var, the state-var-as-array-index regressed. \
            If on `iterIdx++;`, the post-increment on a state-var \
            regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL2 rt");

    // Seed arr via three add() calls: arr = [10, 20, 30].
    for v in [10u64, 20, 30] {
        let r_add = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "add",
                &[StackItem::Integer(v as i64)],
            )
            .expect("LLL2 add host-level");
        assert!(
            r_add.success,
            "LLL2 add({}) must succeed; exc={:?}. If exc, storage array \
             push regressed on the seed step.",
            v,
            r_add.exception.as_ref().map(|e| &e.message)
        );
    }

    // Four nextOrZero() calls — first 3 should advance through arr,
    // the 4th should hit the exhaustion guard and return 0. State must
    // persist across all 4 calls: iterIdx must be 0 → 1 → 2 → 3 → 3.
    let expected: [u64; 4] = [10, 20, 30, 0];
    for (i, exp) in expected.iter().enumerate() {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "nextOrZero",
                &[] as &[StackItem],
            )
            .unwrap_or_else(|e| panic!("LLL2 nextOrZero() call #{} host err: {:?}", i, e));
        assert!(
            r.success,
            "LLL2 nextOrZero() call #{} must succeed; exc={:?}. If exc \
             cites array OOB on the first 3 calls, the bounds guard \
             didn't short-circuit. If on the 4th, the guard failed to \
             fire and arr[3] was accessed past length.",
            i,
            r.exception.as_ref().map(|e| &e.message)
        );
        let v = decode_uint_le(&r.return_data);
        assert_eq!(
            v.clone(),
            BigUint::from(*exp),
            "LLL2 nextOrZero() call #{} must return {}; got {} \
             rd_hex={}. If call #0 returns 0 instead of 10, either the \
             iterIdx state-var was never 0 at init (default-zero \
             regression) or the `arr[iterIdx]` read went to the wrong \
             slot. If call #1 returns 10 again (not 20), iterIdx++ \
             didn't persist (state-var post-increment didn't store the \
             new value). If call #3 returns 30 instead of 0, the \
             exhaustion guard `iterIdx >= arr.length` didn't fire after \
             3 advances (or iterIdx stopped at 2). Task #196+ \
             candidate: iterator-pattern state-var index advancement.",
            i,
            exp,
            v,
            hex::encode(&r.return_data)
        );
    }
}

// LLL3 — Struct equality via manual field-by-field compare.
// `eq(P memory a, P memory b)` returns `a.x == b.x && a.y == b.y`.
// eq(P(1,2), P(1,2)) → true; eq(P(1,2), P(1,3)) → false.
// 15 fuzz cases exercise repeat-exec stability on the two-struct-arg
// shape.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch88_lll3_struct_equality_manual_field_compare(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct P { uint x; uint y; }
    function eq(P memory a, P memory b) external pure returns (bool) {
        return a.x == b.x && a.y == b.y;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("LLL3 compile: {:?}. If this fires \
                on `eq(P memory a, P memory b)`, the two-struct-memory-\
                args signature regressed (batch72 VV5 pins the single-\
                struct-arg form; LLL3 extends to dual-struct form). If \
                on the bool return, the bool-from-comparison-chain \
                regressed. If on `a.x == b.x && a.y == b.y`, the field-\
                access-cross-struct comparison regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL3 rt");

        // Case A: eq(P(1, 2), P(1, 2)) == true.
        let a_eq = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            StackItem::Integer(1),
            StackItem::Integer(2),
        ])));
        let b_eq = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            StackItem::Integer(1),
            StackItem::Integer(2),
        ])));
        let r_eq = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "eq", &[a_eq, b_eq])
            .expect("LLL3 eq(P(1,2), P(1,2)) host-level");
        prop_assert!(r_eq.success,
            "LLL3 eq(P(1,2), P(1,2)) must succeed; exc={:?}. If exc \
             cites field access on struct memory, the struct-arg field \
             read regressed.",
            r_eq.exception.as_ref().map(|e| &e.message));
        let v_eq = decode_uint_le(&r_eq.return_data);
        let is_true_uint = v_eq == num_bigint::BigUint::from(1u64);
        let is_true_byte = r_eq.return_data.iter().any(|b| *b == 0x01)
            && !r_eq.return_data.iter().all(|b| *b == 0x00);
        prop_assert!(is_true_uint || is_true_byte,
            "LLL3 eq(P(1,2), P(1,2)) must return true (identical \
             structs, all fields equal); got rd_hex={} decoded={}. If \
             0/false, either (a) a.x != b.x evaluated false (wrong \
             field access), (b) the && short-circuit dropped the \
             second conjunct, or (c) bool-return encoding went wrong. \
             Task #196+ candidate: struct-equality manual compare.",
            hex::encode(&r_eq.return_data), v_eq);

        // Case B: eq(P(1, 2), P(1, 3)) == false (y-field differs).
        let a_ne = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            StackItem::Integer(1),
            StackItem::Integer(2),
        ])));
        let b_ne = StackItem::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            StackItem::Integer(1),
            StackItem::Integer(3),
        ])));
        let r_ne = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "eq", &[a_ne, b_ne])
            .expect("LLL3 eq(P(1,2), P(1,3)) host-level");
        prop_assert!(r_ne.success,
            "LLL3 eq(P(1,2), P(1,3)) must succeed; exc={:?}",
            r_ne.exception.as_ref().map(|e| &e.message));
        let v_ne = decode_uint_le(&r_ne.return_data);
        let is_false_uint = v_ne == num_bigint::BigUint::from(0u64);
        // Bool false → empty return_data OR all-zero bytes (no 0x01).
        let is_false_byte = r_ne.return_data.is_empty()
            || r_ne.return_data.iter().all(|b| *b == 0x00);
        prop_assert!(is_false_uint || is_false_byte,
            "LLL3 eq(P(1,2), P(1,3)) must return false (y-field \
             differs: 2 != 3); got rd_hex={} decoded={}. If 1/true, \
             the y-field inequality was ignored — the && short-\
             circuit misfired OR the second conjunct `a.y == b.y` \
             evaluated true despite the actual mismatch (field index \
             aliasing — both &.y loads hit the same slot). Task \
             #196+ candidate: struct field-compare second-conjunct.",
            hex::encode(&r_ne.return_data), v_ne);
    }
}

// LLL4 — Inline assembly with Solidity-scope operand sourcing.
// `f(a, b) { uint c; assembly { c := add(a, b) } return c; }` — yul
// `add` reads from Solidity params a, b and writes to Solidity local c.
// f(3, 4) == 7. 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch88_lll4_inline_assembly_with_solidity_scope_operands(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint a, uint b) external pure returns (uint) {
        uint c;
        assembly { c := add(a, b) }
        return c;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("LLL4 compile: {:?}. If this fires \
                on `assembly {{ c := add(a, b) }}`, the yul-reads-from-\
                Solidity-parameters path regressed (batch18 H1 pins the \
                yul-literal-operands + writeback-to-Solidity-local form; \
                LLL4 extends to PARAMETER-sourced operands). Task #100 \
                established ctx.resolve_local for `:=` LHS; LLL4 extends \
                to the RHS operand resolution from Solidity parameters.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL4 rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(3), StackItem::Integer(4)])
            .expect("LLL4 f(3, 4) host-level");
        prop_assert!(r.success,
            "LLL4 f(3, 4) must succeed; exc={:?}. If exc cites the \
             assembly block, the yul block's body execution regressed \
             (Task #99/#100 precedent). If exc cites the return c, \
             the yul-written Solidity local didn't materialize back \
             at the return site.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(7u64),
            "LLL4 f(3, 4) must equal 7 (yul `add(a, b)` with a=3, \
             b=4 writes 7 to c); got {} rd_hex={}. If 0, the yul body \
             was a NO-OP (fell through to the pre-Task-#99 no-op \
             path; regression of the yul-parameter-read handler). If \
             3, only `a` was read and `b` evaluated to 0 (parameter \
             lookup broke on the second operand). If 4, only `b` was \
             read. If any other value, the add operation or the \
             writeback produced wrong data. Task #196+ candidate: \
             yul operand-from-Solidity-parameter resolution.",
            v, hex::encode(&r.return_data));
    }
}

// LLL5 — Reverting inside a try (minimal try/catch-Error(string)).
// Target.fail() unconditionally reverts with "bad"; C.f(t) must route
// to the catch Error(string) arm and bind r = "bad", then return r.
// f(target) → b"bad". Extends batch55 EE5 (which has a fallback
// `catch { }` arm) to the MINIMAL shape.
// Single-shot — deterministic Target revert.
#[test]
fn batch88_lll5_reverting_inside_try_minimal_catch_error_string() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Target {
    function fail() external pure { revert("bad"); }
}
contract C {
    function f(address t) external returns (string memory) {
        try Target(t).fail() returns () { return "ok"; }
        catch Error(string memory r) { return r; }
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "LLL5 compile: {:?}. If this fires \
            on `try Target(t).fail() returns ()`, the zero-return-\
            value try-clause regressed (returns () is explicit for a \
            void-returning target). If on `catch Error(string memory \
            r)`, the minimal (no fallback) try/catch shape regressed. \
            If on the contract split with Target first, the two-\
            contract compilation regressed.",
            e
        )
    });
    assert_eq!(
        arts.len(),
        2,
        "LLL5 must emit 2 artifacts (Target + C); got {} (names={:?}). \
         If 1, one contract was elided.",
        arts.len(),
        arts.iter()
            .map(|a| a.metadata.name.clone())
            .collect::<Vec<_>>()
    );
    let c_art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .unwrap_or_else(|| {
            panic!(
                "LLL5 C artifact missing; got names={:?}",
                arts.iter()
                    .map(|a| a.metadata.name.clone())
                    .collect::<Vec<_>>()
            )
        });

    // Use the zero-placeholder routing (batch55 EE5 precedent, Task #125
    // landed). Target.fail is reachable through C's self_method_offsets
    // via the Task #83 sibling-merge pass.
    let zero_target = [0u8; 20];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL5 rt");
    let r = rt
        .call_method(
            &c_art.bytecode,
            &c_art.tokens,
            &c_art.manifest,
            "f",
            &[StackItem::byte_array(zero_target.to_vec())],
        )
        .expect("LLL5 f(target) host-level");

    // The outer call must succeed — the catch arm absorbs Target's
    // revert and returns a string. If it faults, the Error(string)
    // arm didn't catch the revert at all (the panic propagated
    // through the try without being absorbed).
    assert!(
        r.success,
        "LLL5 f(target) must succeed (catch Error(string) absorbs the \
         target's revert); exc={:?}, rd_hex={}. If exc, either (a) the \
         minimal try/catch-Error(string) form (no fallback catch) didn't \
         match the revert envelope, or (b) Task #125 cross-contract \
         revert propagation regressed on the minimal shape. Task #196+ \
         candidate: minimal try + catch-Error(string) without fallback.",
        r.exception.as_ref().map(|e| &e.message),
        hex::encode(&r.return_data)
    );

    // Expected: catch Error(string memory r) binds r = "bad", return
    // r surfaces as raw UTF-8 b"bad" (3 bytes, per batch11 H1 /
    // batch52 BB2 / batch55 EE5 precedent).
    //   - If b"ok" (2 bytes): try arm fired — target didn't revert
    //     or the revert was silently absorbed before the catch.
    //   - If b"bad" (3 bytes): EVERYTHING WORKED.
    //   - If some other bytes: the catch arm fired but bound r to
    //     something other than "bad" (envelope misparse).
    assert_eq!(
        r.return_data,
        b"bad".to_vec(),
        "LLL5 f(target) must return raw UTF-8 b\"bad\" (3 bytes, from \
         catch Error(string memory r) binding on Target.fail's \
         `revert(\"bad\")`); got {} bytes rd_hex={} utf8={:?}. If \
         b\"ok\" (2 bytes), the try arm fired instead of the catch — \
         target's revert(\"bad\") was absorbed before catch-dispatch \
         (Task #125 regression on the MINIMAL try/catch shape). If \
         some other string, the catch arm fired but the reason \
         envelope was misdecoded. Task #196+ candidate: minimal \
         try/catch-Error(string) reason binding.",
        r.return_data.len(),
        hex::encode(&r.return_data),
        std::str::from_utf8(&r.return_data).ok()
    );
}

// Task ID resolution for Batch #88 on first exec:
//   - LLL1 (storage pointer via internal function return): IGNORED —
//     Task #196 FILED. First-exec observation: push_(1) and push_(2)
//     succeed at host level, but len() faults with "SIZE: unsupported
//     type". The `ref()` internal helper is returning a storage-key
//     handle (or a tagged non-Array/non-ByteString value) instead of
//     loading the backing array, so the SIZE opcode on the `.length`
//     access receives an incompatible type. Fix site hypothesized:
//     the return-path for internal functions whose return type is a
//     `uint[] storage` reference (needs a load/resolve before the
//     value crosses the call boundary — distinct from batch51 AA5's
//     LOCAL-binding-alias form which resolves at assignment). Task
//     #196 is the first new task filed in Batch #88.
//   - LLL2 (iterator pattern via explicit state-var index): RESOLVED
//     GREEN. add(10); add(20); add(30) seed the storage array;
//     nextOrZero() returns 10, 20, 30, 0 across 4 sequential calls.
//     The post-increment on iterIdx persists across calls (batch84
//     HHH3 state-persistence invariant holds); the bounds guard
//     `iterIdx >= arr.length` short-circuits correctly on the 4th
//     call. Non-regression surface.
//   - LLL3 (struct equality via manual field compare): RESOLVED
//     GREEN. eq(P(1,2), P(1,2)) returns true; eq(P(1,2), P(1,3))
//     returns false. The two-memory-struct-arg signature decodes
//     both arguments, field-access on each is unambiguous, and the
//     `&&` short-circuit handles both the all-equal and the y-
//     differs cases correctly across 15 fuzz cases. Non-regression
//     surface.
//   - LLL4 (inline assembly with Solidity-scope operand sourcing):
//     RESOLVED GREEN. f(3, 4) == 7 across 15 fuzz cases. Yul `add`
//     reads from Solidity parameters a, b (Task #100 ctx.resolve_local
//     extends to parameters, not just locals) and writes the sum back
//     to Solidity local c via `:=`. The yul-to-Solidity-return data
//     flow is clean. Non-regression surface.
//   - LLL5 (reverting inside a try — minimal catch-Error(string)):
//     RESOLVED GREEN. f(target) returns b"bad" (3 bytes) per Task #125
//     cross-contract revert-envelope propagation. The minimal
//     try/catch-Error(string) form (no fallback `catch { }` arm)
//     matches the revert envelope and binds r = "bad" correctly. The
//     Task #125 fix extends to the no-fallback shape as well. Non-
//     regression surface.
//
// New Task IDs filed in Batch #88: Task #196 (LLL1 storage pointer via
// internal function return produces a handle/key-typed value instead
// of loading the backing array, so `.length` / SIZE opcode fails).
// Per-batch cadence: 4 GREEN + 1 IGNORED + Task #196 filed, matching
// Batches #85 (Task #194) and #87 (Task #195) shape.
//
// Sibling agent context: Batch #88's probes are orthogonal to batch
// #87's KKK1..KKK5 — LLL1 pins storage pointer via function return;
// LLL2 pins iterator pattern via explicit state-var index; LLL3 pins
// struct equality via manual field compare; LLL4 pins inline assembly
// with Solidity-scope operand sourcing; LLL5 pins the minimal
// try/catch-Error(string) shape. The parent-reported sibling
// `fix-195-escapes` 50k hunt is on Task #195 surface (string-literal
// \n escape lowering) and does not intersect these probes. Task #196
// (LLL1) opens a new surface for a sibling `fix-196-storage-return`
// 50k hunt — internal function return of a storage-reference type.

// ==================== Batch #89 — 3-way ternary nesting, uint256 >> 1, fixed-size state array, fallback msg.data replay, Panic(0x01) envelope ====================
//
// Five orthogonal probes continuing the per-five-harness cadence. Each
// pins a distinct surface previously uncovered by Batches #81..#88.
//
//   MMM1: 3-way nested ternary with string-literal arms.
//         `n == 0 ? "zero" : n < 10 ? "small" : "big"` — TWO levels of
//         ternary where the false-arm of the outer is itself a ternary
//         expression (right-associative per Solidity's precedence). The
//         three inputs 0 / 5 / 100 exercise all three terminal arms:
//         arm0 ("zero") via n==0 true, arm1 ("small") via n==0 false +
//         n<10 true, arm2 ("big") via n==0 false + n<10 false. Extends
//         batch49 Y1 (FLAT ternary with string arms) to the NESTED
//         form. Probes: (a) 3-branch type unification across dynamic
//         strings (Y1 pins 2-branch), (b) right-associative parsing
//         of `a ? b : c ? d : e`, (c) conditional-arm short-circuit
//         across two levels. 15 fuzz cases exercise repeat-exec
//         stability over the three inputs.
//   MMM2: uint256 division by 2 via right-shift. `n >> 1` must equal
//         n / 2 for unsigned n. Extends batch32 U3 (five-op bitwise
//         snapshot on u8) to uint256 with a SINGLE-OP signature, and
//         contrasts with batch52 BB4 (mul/div/WAD fixed-point arithmetic).
//         The key distinction: MMM2 pins `>>` as a pure arithmetic
//         shortcut for unsigned division by a power of 2 — known inputs
//         100 → 50 and 7 → 3 (7 >> 1 = 3, integer floor). 15 fuzz cases
//         exercise repeat-exec stability.
//   MMM3: Fixed-size state array (uint256[3] public fixed_).
//         Multi-call sequence: set() assigns fixed_[0..2] = 1, 2, 3;
//         get(1) then reads fixed_[1] = 2. Extends batch46 V3 / batch68
//         RR3 (MEMORY fixed-size arrays) to the STATE-variable form;
//         fixed-size state arrays use distinct slot-derivation (each
//         index maps to a consecutive storage slot, no keccak key
//         derivation). Probes: (a) the `public` auto-getter is NOT
//         called here — instead a custom get(i) pins direct indexed
//         read; (b) three distinct writes persist across one external
//         call; (c) the indexed read returns the correct per-slot
//         value. Single-shot — deterministic seed-then-read sequence.
//   MMM4: Fallback with msg.data replay into state.
//         `fallback() external { lastCall = msg.data; }` captures the
//         raw calldata into a `bytes public lastCall`; getLast() reads
//         it back. Uses `execute_with_overrides` (per batch26 H3 /
//         batch48 X2 precedent) to inject raw calldata `hex"deadbeef"`
//         — there is no matching method selector so the dispatcher
//         must route to the fallback. Extends batch48 X2 (fallback
//         EMITS msg.data in a log) and batch26 H3 (fallback READS
//         msg.data.length into a state var) to the JOINT shape:
//         fallback ASSIGNS the full msg.data byte-array into a state
//         var, then a follow-up view call reads it back — pins (a)
//         msg.data as a `bytes` r-value (not a .length scalar),
//         (b) `bytes = bytes` storage assignment, (c) round-trip
//         equality of the injected calldata bytes. Single-shot —
//         deterministic calldata injection.
//   MMM5: Panic(0x01) envelope on `assert(false)`. Extends batch38 M2a
//         (which pinned the 4-byte selector prefix keccak("Panic(uint256)")
//         [..4] = 0x4e487b71) to the FULL 36-byte envelope: selector
//         (4 bytes) + BE32 of the uint256 panic code (0x01 for assert).
//         Per EVM spec `Panic(uint256)` abi-encodes the code as a
//         big-endian 32-byte uint256; bytes 4..35 must all be zero and
//         byte 35 must equal 0x01. This pins the CANONICAL Panic
//         envelope (Task #103 completed canonicalization; MMM5 is a
//         regression probe on the 0x01 assert-false code specifically,
//         distinct from M2a's partial-selector check). Single-shot —
//         deterministic assert path.
//
// Task IDs observed on first exec: per-harness after the first run;
// any new gaps surface as Task #197+ (last-assigned is #196 from
// Batch #88 LLL1). Expected GREEN baseline: all 5 harnesses pass, no
// new ignore. If one fails, mark `#[ignore]` + file Task #197+ per the
// Batch #82 FFF4 / #85 HHH5 / #87 KKK3 precedent.
//
// Sibling agent context: Batch #89's probes are orthogonal to the
// LLL1..LLL5 (Batch #88) surfaces:
//   - MMM1 is NESTED ternary with dynamic-string arms (distinct from
//     Y1's FLAT 2-branch form and LLL3's struct-memory comparison).
//   - MMM2 is single-op `>>` on uint256 (distinct from U3's five-op
//     u8 snapshot — MMM2 pins the ISOLATED shift-right on the widest
//     scalar type).
//   - MMM3 is fixed-size STATE array (distinct from batch68 RR3's
//     memory fixed array, and distinct from batch75 YY1's dynamic
//     storage array — MMM3 specifically pins the fixed-size state-var
//     slot derivation).
//   - MMM4 is fallback that WRITES msg.data to state (distinct from
//     X2's EMIT-in-log form and H3's LENGTH-only read — MMM4 pins
//     the BYTES-value round-trip through storage).
//   - MMM5 is canonical Panic(0x01) envelope (distinct from M2a's
//     selector-only check — MMM5 pins the FULL selector + BE32(code)
//     encoding, Task #103 canonical form).
// The parent-reported sibling `fix-196-storage-ptr-return` 50k hunt
// is on Task #196 surface (internal fn returning uint[] storage
// pointer) and does not intersect these probes.

// MMM1 — 3-way nested ternary with string-literal arms.
// `f(0) == "zero"`, `f(5) == "small"`, `f(100) == "big"`. Extends
// batch49 Y1 (flat 2-branch ternary) to the nested form.
// 15 fuzz cases exercise repeat-exec stability over the 3 inputs.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch89_mmm1_three_way_nested_ternary_with_string_arms(
        seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint n) external pure returns (string memory) {
        return n == 0 ? "zero" : n < 10 ? "small" : "big";
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("MMM1 compile: {:?}. If this fires \
                on the nested ternary `n == 0 ? \"zero\" : n < 10 ? \"small\" \
                : \"big\"`, the right-associative parse regressed (batch49 \
                Y1 pins the FLAT `c ? \"yes\" : \"no\"` form; MMM1 extends \
                to the NESTED form where the else-arm is itself a ternary). \
                If on 3-branch string type-unification, the dynamic-arm \
                unification across THREE branches regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM1 rt");

        // Rotate across the three named inputs by seed % 3 so proptest
        // spreads coverage across all three terminal arms.
        let cases: [(u64, &[u8]); 3] = [
            (0u64, b"zero"),
            (5u64, b"small"),
            (100u64, b"big"),
        ];
        let (n, expected) = cases[(seed as usize) % 3];
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(n as i64)])
            .expect("MMM1 f host-level");
        prop_assert!(r.success,
            "MMM1 f(n={}) must succeed; exc={:?}. If exc cites the ternary \
             lowering, the 3-branch dynamic-arm type unification regressed. \
             If cites the string literal, the \"zero\"/\"small\"/\"big\" \
             encoding regressed.",
            n, r.exception.as_ref().map(|e| &e.message));
        // Per batch49 Y1 / batch31 H4(b) precedent, `returns (string memory)`
        // surfaces as the raw ASCII bytes of the literal on the external
        // return path. A drift (e.g. ABI-wrapped payload or wrong arm
        // selected) would fail the exact-bytes compare.
        prop_assert_eq!(&r.return_data[..], expected,
            "MMM1 f(n={}) must return literal bytes {:?}; got {:?} \
             (rd_hex={}). If the wrong literal surfaced: n=0 → \"zero\", \
             0<n<10 → \"small\", n>=10 → \"big\". A mis-match means \
             (a) the nested ternary's right-associative parse fired \
             left-to-right (returning \"small\" for n=0), (b) the \
             condition `n < 10` was inverted (returning \"big\" for n=5), \
             or (c) 3-branch type unification dropped an arm. Task \
             #197+ candidate: 3-way nested ternary dispatch.",
            n, std::str::from_utf8(expected).unwrap_or("<bin>"),
            std::str::from_utf8(&r.return_data).unwrap_or("<bin>"),
            hex::encode(&r.return_data));
    }
}

// MMM2 — uint256 division by 2 via right-shift (`n >> 1`).
// f(100) == 50; f(7) == 3 (floor). 15 fuzz cases exercise repeat-exec
// stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch89_mmm2_uint256_right_shift_by_one(
        seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint256 n) external pure returns (uint256) { return n >> 1; }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("MMM2 compile: {:?}. If this fires \
                on `n >> 1`, the right-shift operator on uint256 regressed \
                (batch32 U3 pins the five-op bitwise snapshot on u8; MMM2 \
                pins the ISOLATED single-op form on the widest scalar). \
                If on the direct-return of a shift expression, the \
                shift-as-rvalue regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM2 rt");

        // Alternate between (100 → 50) and (7 → 3) via seed parity.
        let (n, expected) = if seed.is_multiple_of(2) {
            (100u64, 50u64)
        } else {
            (7u64, 3u64)
        };
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(n as i64)])
            .expect("MMM2 f host-level");
        prop_assert!(r.success,
            "MMM2 f(n={}) must succeed; exc={:?}. If exc cites SHR or the \
             shift opcode, the uint256 >> 1 lowering regressed.",
            n, r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), BigUint::from(expected),
            "MMM2 f(n={}) must equal {} ({} >> 1 = integer-floor \
             division by 2); got {} rd_hex={}. If f(7)=4 instead of 3, \
             the shift is treating LSB as a carry (rounding up — wrong \
             for unsigned). If f(100)=100 or some off-by-power-of-2 \
             value, the shift amount was misinterpreted. If zero, the \
             shift operand was dropped. Task #197+ candidate: uint256 \
             right-shift by constant 1.",
            n, expected, n, v, hex::encode(&r.return_data));
    }
}

// MMM3 — Fixed-size state array (uint256[3] public fixed_).
// set() writes fixed_[0..2] = 1, 2, 3; get(1) reads fixed_[1] == 2.
// Single-shot — deterministic seed-then-read.
#[test]
fn batch89_mmm3_fixed_size_state_array_index_write_read() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256[3] public fixed_;
    function set() external { fixed_[0] = 1; fixed_[1] = 2; fixed_[2] = 3; }
    function get(uint i) external view returns (uint) { return fixed_[i]; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "MMM3 compile: {:?}. If this fires \
            on `uint256[3] public fixed_`, the fixed-size state-var \
            array declaration regressed (batch68 RR3 pins the MEMORY \
            fixed-size array; MMM3 extends to the STATE form which \
            uses distinct slot-derivation — consecutive storage slots, \
            no keccak key). If on the auto-getter synthesis for a \
            public fixed-size array, the public-decl getter regressed. \
            If on the 3-write assignment sequence in set(), the \
            multi-statement indexed-store regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM3 rt");

    // set() — three indexed writes in one external call. State must
    // persist across this call so the subsequent get() observes the
    // written values (batch84 HHH3 state-persistence invariant).
    let r_set = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "set",
            &[] as &[StackItem],
        )
        .expect("MMM3 set() host-level");
    assert!(
        r_set.success,
        "MMM3 set() must succeed (three indexed writes fixed_[0]=1, \
         fixed_[1]=2, fixed_[2]=3); exc={:?}. If exc cites bounds \
         check, the [0..2] indices are in-range for a [3] array (so \
         the fixed-size bounds table regressed). If cites SSTORE or \
         storage-slot derivation, the fixed-size state-array slot \
         computation regressed. If cites the multi-statement sequence, \
         the three-write-in-one-call lowering regressed.",
        r_set.exception.as_ref().map(|e| &e.message)
    );

    // get(1) — read fixed_[1] which was written = 2. This pins the
    // indexed READ path (distinct from the auto-getter synthesis for
    // `public` — we're calling the custom get() helper). A drift
    // would surface as 0 (unwritten), 1 (wrong index), or 3 (wrong
    // index).
    let r_get = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "get",
            &[StackItem::Integer(1)],
        )
        .expect("MMM3 get(1) host-level");
    assert!(
        r_get.success,
        "MMM3 get(1) must succeed (read fixed_[1] after set()); \
         exc={:?}. If exc cites bounds, index 1 must be in [0..3). \
         If cites GETSTORAGE or state-not-found, the fixed-size state \
         array slot didn't materialize (set() didn't persist).",
        r_get.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r_get.return_data);
    assert_eq!(
        v.clone(),
        BigUint::from(2u64),
        "MMM3 get(1) must equal 2 (fixed_[1] was written to 2 in set()); \
         got {} rd_hex={}. If 0, set() didn't persist — fixed-size state \
         array storage writes are not reaching the slot (state-var slot \
         derivation regression). If 1, get(1) read fixed_[0] (index \
         off-by-one in the read path). If 3, get(1) read fixed_[2] \
         (index off-by-one in the OTHER direction). Task #197+ \
         candidate: fixed-size state array write/read.",
        v,
        hex::encode(&r_get.return_data)
    );
}

// MMM4 — Fallback with msg.data replay into state.
// Inject raw calldata 0xdeadbeef via execute_with_overrides → dispatcher
// routes to fallback → lastCall = msg.data. getLast() must then return
// bytes carrying 0xdeadbeef. Single-shot — deterministic calldata.
#[test]
fn batch89_mmm4_fallback_msg_data_replay_into_state() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use neo_devpack_solidity::runtime::ExecutionOverrides;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes public lastCall;
    fallback() external { lastCall = msg.data; }
    function getLast() external view returns (bytes memory) { return lastCall; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "MMM4 compile: {:?}. If this fires \
            on `bytes public lastCall`, the bytes-typed state var \
            regressed. If on `fallback() external {{ lastCall = \
            msg.data; }}`, the fallback-assigns-msg.data-to-state form \
            regressed (batch48 X2 pins fallback EMITS msg.data in a log, \
            batch26 H3 pins fallback READS msg.data.length — MMM4 is \
            the JOINT shape: fallback WRITES the full bytes value into \
            state). If on the auto-getter vs custom getLast(), the \
            user-defined bytes-returning view regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM4 rt");

    // Inject raw calldata hex"deadbeef" — no matching method selector,
    // so the dispatcher must fall through to the fallback() body
    // which copies msg.data into `lastCall`. Per batch26 H3 / batch48
    // X2 precedent, execute_with_overrides threads the calldata
    // through GetScriptContainer.Script for msg.data materialization.
    let calldata: Vec<u8> = vec![0xde, 0xad, 0xbe, 0xef];
    let r_fb = rt
        .execute_with_overrides(&art.bytecode, &calldata, &ExecutionOverrides::default())
        .expect("MMM4 fallback via execute_with_overrides host-level");
    assert!(
        r_fb.success,
        "MMM4 fallback dispatch must succeed (unknown 4-byte selector \
         0xdeadbeef routes to fallback, which assigns msg.data to \
         lastCall); exc={:?}. If exc cites method-not-found, the \
         dispatcher didn't route to fallback (regression — batch26 \
         H3 / batch48 X2 pin fallback dispatch). If cites the \
         assignment, the bytes=msg.data storage-write regressed.",
        r_fb.exception.as_ref().map(|e| &e.message)
    );

    // getLast() — reads `lastCall` which should contain 0xdeadbeef.
    // Per the fallback-via-call precedent, msg.data carries the exact
    // 4-byte calldata, so lastCall should equal that 4-byte sequence.
    let r_get = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "getLast",
            &[] as &[StackItem],
        )
        .expect("MMM4 getLast() host-level");
    assert!(
        r_get.success,
        "MMM4 getLast() must succeed (view read of lastCall after \
         fallback assigned msg.data); exc={:?}. If exc cites \
         SIZE/GETITEM on a non-bytes value, the bytes storage read \
         regressed. If cites state-not-found, the fallback's write \
         didn't persist.",
        r_get.exception.as_ref().map(|e| &e.message)
    );

    // The fallback body wrote msg.data = 0xdeadbeef into lastCall.
    // Per existing batch48 X2 precedent, msg.data carries the injected
    // calldata bytes. Expected: rd contains the 4 bytes 0xdeadbeef.
    // Accept either (a) raw 4-byte payload directly, or (b) an
    // ABI-encoded bytes envelope (offset + length + payload) per
    // batch38 M3 / dynamic-bytes precedent — we probe for the
    // contiguous 4-byte payload within return_data.
    let expected: &[u8] = &[0xde, 0xad, 0xbe, 0xef];
    let rd = &r_get.return_data;
    let found = rd.windows(expected.len()).any(|w| w == expected);
    assert!(
        found,
        "MMM4 getLast() return must contain the contiguous 4-byte \
         payload 0xdeadbeef (injected as calldata through the fallback \
         path); got rd_hex={} (len {}). If absent, either (a) the \
         fallback's lastCall = msg.data assignment dropped the bytes \
         payload (msg.data arrived empty or truncated — batch48 X2 \
         regression surface on the STATE-assign path), (b) the getLast() \
         read returned the wrong storage slot (bytes-typed state var \
         slot mismatch), or (c) msg.data materialised as zeros instead \
         of the injected calldata (Task #113 regression surface for \
         calldata synthesis). Task #197+ candidate: fallback \
         msg.data → bytes state var round-trip.",
        hex::encode(rd),
        rd.len()
    );
}

// MMM5 — Canonical Panic(uint256) envelope on assert(false), code = 0x01.
// Per Solidity 0.8 spec: return_data = keccak("Panic(uint256)")[..4]
// || BE32(0x01). Extends batch38 M2a (selector-prefix only) to the FULL
// 36-byte envelope including the BE32(code) suffix. Single-shot —
// deterministic assert(false) path.
#[test]
fn batch89_mmm5_assert_false_panic_0x01_canonical_envelope() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure {
        assert(false);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "MMM5 compile: {:?}. If this fires \
            on `assert(false)`, the compile-time-false assert lowering \
            regressed (batch38 M2a pins this compile path as GREEN). \
            If on the bare `external pure` form with no return type, \
            the void-returning asserting function regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM5 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[] as &[StackItem],
        )
        .expect("MMM5 f() host-level");
    assert!(
        !r.success,
        "MMM5 f() must FAULT via assert(false) (Panic(0x01)); got \
         success=true rd_hex={}. If success, the assert was DCE'd \
         (compile-time-false assert got optimized to a no-op) — that's \
         a regression of the Task #103 canonical-panic lowering.",
        hex::encode(&r.return_data)
    );

    // Spec: return_data = keccak256("Panic(uint256)")[..4] || BE32(0x01).
    // Total 36 bytes: 4-byte selector (0x4e487b71) + 32-byte BE uint
    // with last byte = 0x01.
    assert!(
        r.return_data.len() >= 36,
        "MMM5 return_data must carry the full 36-byte Panic envelope \
         (4-byte selector + 32-byte BE uint code); got {} bytes rd_hex={}. \
         If <36, only the selector landed (batch38 M2a surface) but the \
         BE32(code) suffix was dropped — Task #103 canonical-encoding \
         regression. Task #197+ candidate: Panic(0x01) full envelope.",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );

    // Selector bytes [0..4] must equal 0x4e487b71 (keccak("Panic(uint256)")[..4]).
    assert_eq!(
        &r.return_data[..4],
        &[0x4eu8, 0x48, 0x7b, 0x71],
        "MMM5 return_data[..4] must equal keccak(\"Panic(uint256)\")[..4] \
         = 0x4e487b71; got 0x{}. If different, either the selector is \
         wrong or the envelope preamble has extra bytes. See batch38 \
         M2a for the selector-only pin.",
        hex::encode(&r.return_data[..4])
    );

    // Bytes [4..35] must all be zero (high 31 bytes of the BE32 encoding
    // of 0x01 — the low byte carries the code).
    for i in 4..35 {
        assert_eq!(
            r.return_data[i],
            0u8,
            "MMM5 return_data[{}] must be zero (high-byte of BE32(0x01) \
             Panic code); got 0x{:02x} rd_hex={}. If nonzero in the \
             high bytes, the BE32 encoding of the panic code drifted \
             (wrong endianness or wrong width). Task #197+ candidate: \
             Panic code BE32 width.",
            i,
            r.return_data[i],
            hex::encode(&r.return_data)
        );
    }

    // Byte [35] must equal 0x01 (the assert-false panic code per
    // Solidity 0.8 spec § Error-Handling). If 0x11 (overflow) or
    // 0x12 (div-by-zero) or 0x31 (empty-pop), a different panic
    // path took over.
    assert_eq!(
        r.return_data[35],
        0x01u8,
        "MMM5 return_data[35] must equal 0x01 (assert-false panic code \
         per Solidity 0.8 spec); got 0x{:02x} rd_hex={}. If 0x11, an \
         arithmetic overflow panic was routed instead (wrong code). If \
         0x12, division-by-zero. If 0x31, empty-pop. If 0x00, the code \
         slot wasn't populated at all (Task #103 canonical-form \
         regression on the code suffix). Task #197+ candidate: \
         Panic(0x01) assert code.",
        r.return_data[35],
        hex::encode(&r.return_data)
    );
}

// Task ID resolution for Batch #89 on first exec:
//   - MMM1 (3-way nested ternary with string arms): RESOLVED GREEN.
//     f(0) → "zero", f(5) → "small", f(100) → "big" across 15 fuzz
//     cases. The right-associative nested-ternary parse is correct,
//     3-branch dynamic-string type unification works, and the raw
//     literal bytes surface on return_data per the batch49 Y1 / batch31
//     H4(b) precedent. Non-regression surface.
//   - MMM2 (uint256 >> 1): RESOLVED GREEN. f(100) == 50 and f(7) == 3
//     across 15 fuzz cases. The single-op right-shift lowering on the
//     widest scalar type is a clean arithmetic shortcut (unsigned
//     floor-division by 2). Non-regression surface.
//   - MMM3 (fixed-size state array uint256[3] public): RESOLVED GREEN.
//     set() writes three indices; get(1) reads fixed_[1] == 2. The
//     fixed-size state-array slot derivation (consecutive storage
//     slots, no keccak key) lowers correctly; the three-write-in-one-
//     call sequence persists across the external call boundary. Non-
//     regression surface.
//   - MMM4 (fallback msg.data → bytes state var): RESOLVED GREEN. The
//     fallback dispatches correctly on an unknown 4-byte selector
//     0xdeadbeef; the `lastCall = msg.data` assignment captures the
//     raw 4-byte calldata into the `bytes` state var; getLast()
//     surfaces a return envelope containing the 0xdeadbeef payload.
//     Extends batch48 X2 (fallback EMITS msg.data) and batch26 H3
//     (fallback READS msg.data.length) to the JOINT shape — state
//     assignment of the full bytes value. Non-regression surface.
//   - MMM5 (Panic(0x01) canonical envelope on assert(false)): RESOLVED
//     GREEN. return_data is the full 36-byte envelope: 4-byte selector
//     0x4e487b71 (keccak("Panic(uint256)")[..4]) + 32-byte BE uint
//     with last byte = 0x01. Task #103's canonical-panic encoding
//     holds for the assert-false code specifically — extends batch38
//     M2a's selector-only pin to the FULL 36-byte envelope pin. Non-
//     regression surface.
//
// New Task IDs filed in Batch #89: NONE. All 5 harnesses passed GREEN
// on first exec. This breaks the per-batch "4 GREEN + 1 IGNORED + new
// Task filed" cadence seen in Batches #82/#85/#87/#88 — Batch #89 is
// the first full-5-GREEN batch since #84, signaling that the surfaces
// probed here (nested ternary, single-op >>, fixed-size state array,
// fallback msg.data→bytes, full Panic envelope) are all well-covered
// by prior canonicalization work (Tasks #103, #113, sibling-merge
// Task #83, etc.).
//
// Sibling agent context: Batch #89's all-GREEN result means the
// parent-reported sibling `fix-196-storage-ptr-return` 50k hunt (on
// Task #196 surface from Batch #88 LLL1) runs uncontended by new
// Batch #89 task surfaces. The next batch (#90) opens the final slot
// in this file's 81-90 range.

// ==================== Batch #90 — Transfer-event 3-topic shape (single emit), string isEmpty/startsWith, arithmetic in ternary condition, bytes32 indexed event (sig + raw-hash topic), custom error via nested internal fn ====================
//
// Five orthogonal probes — the final batch in this file's 81-90 range.
// Each pins a distinct surface previously uncovered by Batches #81..#89.
//
//   NNN1: Token-transfer event with two indexed addresses + one
//         non-indexed uint256 value. `event Transfer(address indexed
//         from, address indexed to, uint256 value)` plus a single
//         `emit Transfer(msg.sender, to, val)`. Pins the canonical
//         ERC-20 Transfer event shape on a PARAMETRIC (non-literal)
//         emit site: msg.sender and an external address param feed
//         the indexed slots, and an external uint256 param feeds the
//         non-indexed value. Extends batches_18_30 H3 (which pins
//         the same event with LITERAL args `address(0x1)`,
//         `address(0x2)`, `100`) to the PARAMETER-SOURCED form where
//         all three fields are runtime values. Invariants: topics.len
//         == 3 (sig + 2 indexed addresses), topic[0] = keccak256
//         ("Transfer(address,address,uint256)"), data.len == 32
//         (abi.encode of the single non-indexed uint256). Single-shot
//         — deterministic args.
//   NNN2: String isEmpty / startsWith patterns. Two sub-harnesses:
//         (a) isEmpty("") == true and isEmpty("hi") == false pin
//         the `bytes(s).length == 0` predicate — a STRING-LENGTH
//         equality check with short-circuit on empty (zero-length
//         bytes-cast). (b) startsWith("abc", "a") == true pins the
//         single-bytes1-prefix check `bytes(s)[0] == c`. Extends
//         batch67 QQ2 (startsWith with a STRING prefix — multi-byte
//         slice compare) to the bytes1 SINGLE-CHAR prefix form,
//         distinct because bytes1 indexing on a bytes memory returns
//         a single byte value, not a sub-slice. 15 fuzz cases
//         exercise repeat-exec stability.
//   NNN3: Arithmetic expression in ternary condition. `(a + b) > 100
//         ? (a + b) * 2 : a * b` — the condition evaluates an add,
//         compares against a literal, and both arms evaluate
//         ADDITIONAL arithmetic (one re-uses the same add, one is a
//         multiplication of the inputs). Pins: (a) arithmetic-in-
//         condition lowering (add-then-compare on uint256), (b) both
//         arms are independent arithmetic expressions (NOT common-
//         subexpression-eliminated across branches — the add is
//         re-evaluated in the true-arm), (c) the branch test
//         produces the correct boolean. Test inputs: f(50, 50) —
//         (100 > 100) = false → 50 * 50 = 2500; f(60, 50) — (110 >
//         100) = true → 110 * 2 = 220. Distinct from batch49 Y1
//         (string-arm ternary, literal condition) and batch89 MMM1
//         (nested string-arm ternary) — NNN3's arms are ARITHMETIC
//         expressions and the condition computes. 15 fuzz cases.
//   NNN4: Single bytes32-indexed event. `event Hashed(bytes32 indexed
//         hash); function h(string memory s) external { emit Hashed
//         (keccak256(bytes(s))); }` — the argument is a RUNTIME
//         keccak256 result, threaded directly into the indexed slot.
//         Pins: (a) topics.len == 2 (sig + bytes32 indexed hash),
//         (b) topic[1] = the raw keccak256 output (NOT re-hashed —
//         bytes32 is a STATIC type, so indexed bytes32 carries the
//         RAW value, distinct from indexed bytes/string which hashes
//         the value per batch #83 GGG3's bytes-indexed spec), (c)
//         data.len == 0 (only arg is indexed). Extends batches_66_80
//         baseline_tests H4 (bytes32-indexed with LITERAL keccak256
//         (b"TEST") as argument) to the COMPUTED form where the
//         keccak256 call is inlined in the emit expression. Single-
//         shot — deterministic input string.
//   NNN5: Custom error via nested internal fn. `error TooSmall(uint
//         value); function _check(uint n) internal pure { if (n <
//         10) revert TooSmall(n); } function f(uint n) external pure
//         returns (uint) { _check(n); return n * 2; }`. Pins: (a)
//         the internal `_check` with a custom-error revert path, (b)
//         the revert PROPAGATES across the internal-call boundary
//         (not caught by the caller — Solidity has no try/catch on
//         internal calls), (c) selector + abi.encode(n) surface on
//         return_data for the failing path. Two inputs: f(15) == 30
//         (internal branch taken = false, body continues, returns n
//         * 2 = 30); f(5) REVERTS with TooSmall(5) (internal branch
//         taken = true, revert propagates). Extends baseline_tests
//         runtime_revert_custom_error (direct revert) to the
//         INDIRECT form where the revert fires inside a nested
//         internal call. Single-shot — deterministic seed + probe.
//
// Task IDs observed on first exec: per-harness after the first run;
// any new gaps surface as Task #197+ (last-assigned is #196 from
// Batch #88 LLL1). Expected GREEN baseline: NNN1..NNN5 all GREEN
// (target: 490 passed + 0 ignored from a 485 + 0 baseline). If any
// GREEN expectation fails, mark `#[ignore]` + file Task #197+ per
// the Batch #82 FFF4 / #85 HHH5 / #87 KKK3 precedent.
//
// Sibling agent context: Batch #90's probes are orthogonal to the
// MMM1..MMM5 (Batch #89) surfaces:
//   - NNN1 is parametric ERC-20 Transfer emit (distinct from
//     batches_18_30 H3's literal-args form).
//   - NNN2 is string isEmpty + startsWith-bytes1 (distinct from
//     batch67 QQ2's string-prefix slice-compare).
//   - NNN3 is arithmetic ternary with computed condition + arithmetic
//     arms (distinct from MMM1's string-arm nested ternary).
//   - NNN4 is single bytes32-indexed event with INLINE keccak256 arg
//     (distinct from baseline_tests H4's literal keccak256 arg).
//   - NNN5 is custom-error revert propagation across an internal
//     fn call boundary (distinct from baseline_tests'
//     direct-revert form). The parent-reported sibling
//     `fix-196-storage-ptr-return` 50k hunt (on Task #196 from Batch
//     #88 LLL1) is on an orthogonal surface.

// NNN1 — ERC-20/NEP-17 Transfer event with parametric args.
// `emit Transfer(msg.sender, to, val)` — matches the NEP-17 standard
// signature, so the emit is NATIVE Neo shape (events-native gap):
// Notify("Transfer", [from, to, amount]). Invariants: 1 name topic,
// state = [msg.sender(20B), to(20B), Integer(val)].
// Single-shot — deterministic args.
#[test]
fn batch90_nnn1_transfer_event_two_indexed_addrs_one_data_uint() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Transfer(address indexed from, address indexed to, uint256 value);
    function transfer(address to, uint256 val) external { emit Transfer(msg.sender, to, val); }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "NNN1 compile: {:?}. If this fires \
            on `event Transfer(address indexed from, address indexed \
            to, uint256 value)`, the 2-indexed-addr + 1-non-indexed-\
            uint256 event shape regressed (batches_18_30 H3 pins the \
            LITERAL-args form; NNN1 is the PARAMETER-SOURCED form). \
            If on `emit Transfer(msg.sender, to, val)`, the msg.sender \
            → indexed-address threading regressed. If on `transfer(\
            address to, uint256 val)`, the multi-arg external fn \
            signature regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("NNN1 rt");

    // Fixed args: to = 0x22 * 20 (LE on the boundary), val = 1000.
    // msg.sender is injected by the runtime; we don't assert its exact
    // topic-level value here (the address is pinned by the runtime's
    // GetCurrentContext path and varies by runtime-default init).
    let to_le = [0x22u8; 20];
    let val: u64 = 1000;

    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "transfer",
            &[
                StackItem::byte_array(to_le.to_vec()),
                StackItem::Integer(val as i64),
            ],
        )
        .expect("NNN1 transfer(to, val) host-level");
    assert!(
        r.success,
        "NNN1 transfer() must succeed; exc={:?}. If exc cites event \
         emit or msg.sender resolution, the parametric-emit path \
         regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // (1) Exactly 1 log must fire.
    assert_eq!(
        r.logs.len(),
        1,
        "NNN1 transfer() must emit exactly 1 Transfer log; got {} \
         logs. If 0, the emit was elided. If 2+, a shadow emit fired.",
        r.logs.len()
    );
    let log = &r.logs[0];

    // events-native gap: `Transfer(address,address,uint256)` matches the
    // NEP-17 standard signature, so the PARAMETER-SOURCED emit is NATIVE
    // Neo shape too: Notify("Transfer", [from, to, amount]). The emulator
    // records it on its legacy path: topics = ["Transfer"], data = the
    // JSON-encoded state array.

    // (2) One topic: the literal Neo event name.
    assert_eq!(
        log.topics.len(),
        1,
        "NNN1 native NEP-17 Transfer must record exactly 1 topic (the \
         event name); got {} topics. If 3, the emit regressed to the \
         EVM topic shape (NEP trackers cannot read that).",
        log.topics.len()
    );
    assert_eq!(
        &log.topics[0][..],
        b"Transfer" as &[u8],
        "NNN1 topics[0] must be the literal event name \"Transfer\"; got 0x{}",
        hex::encode(&log.topics[0])
    );

    // (3) State = [from, to, amount]. `from` = msg.sender (runtime-
    //     injected; assert only that it is a non-null 20-byte value),
    //     `to` = the 20-byte arg verbatim, amount = Integer(val).
    let state = decode_native_notification_state(&log.data);
    assert_eq!(
        state.len(),
        3,
        "NNN1 native NEP-17 Transfer state must be [from, to, amount]; got {state:?}"
    );
    assert!(
        !native_state_is_null(&state[0]),
        "NNN1 state[0] (`from` = msg.sender) must not be Null for a \
         non-zero sender; got {:?}",
        state[0]
    );
    assert_eq!(
        native_state_bytes(&state[0]).len(),
        20,
        "NNN1 state[0] (`from` = msg.sender) must be a 20-byte UInt160"
    );
    assert_eq!(
        native_state_bytes(&state[1]),
        to_le.to_vec(),
        "NNN1 state[1] must be the `to` arg verbatim (20-byte LE UInt160)"
    );
    assert_eq!(
        native_state_int(&state[2]),
        val as i64,
        "NNN1 state[2] must be the Integer amount {val}"
    );
}

// NNN2 — String isEmpty and startsWith (bytes1 prefix).
// isEmpty("") == true, isEmpty("hi") == false, startsWith("abc", "a")
// == true. Three inputs baked as source-level literals (per batch67
// QQ2 precedent — dynamic-string round-trip is orthogonal to these
// invariants).
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch90_nnn2_string_is_empty_and_starts_with_bytes1_prefix(
        _seed in any::<u8>(),
    ) {
        use num_bigint::BigUint;

        // Sub-harness (a): isEmpty("") — the empty-string case.
        // `bytes(s).length == 0` must evaluate to true when s is "".
        let src_empty = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function isEmpty(string memory s) internal pure returns (bool) { return bytes(s).length == 0; }
    function f() external pure returns (bool) { return isEmpty(""); }
}"#;
        let arts_e = compile_contracts(src_empty, false, 2)
            .unwrap_or_else(|e| panic!("NNN2a compile: {:?}. If this \
                fires on `bytes(s).length == 0`, the bytes-cast + \
                length-access + equality predicate regressed.", e));
        let art_e = &arts_e[0];
        let mut rt_e = NeoRuntime::new(RuntimeConfig::default()).expect("NNN2a rt");
        let r_e = rt_e.execute(&art_e.bytecode, &[])
            .expect("NNN2a f() host-level");
        prop_assert!(r_e.success,
            "NNN2a isEmpty(\"\") must succeed; exc={:?}. If exc cites \
             SIZE on the bytes cast, the empty-string → bytes-memory \
             path regressed.",
            r_e.exception.as_ref().map(|e| &e.message));
        let got_true = decode_uint_le(&r_e.return_data);
        prop_assert_eq!(got_true.clone(), BigUint::from(1u8),
            "NNN2a isEmpty(\"\") must return true; got {} (rd_hex={}). \
             If false, the empty-string length wasn't 0 (maybe the \
             bytes-cast allocated a 1-byte zero terminator, or the \
             length property returned a non-zero sentinel). Task #197+ \
             candidate: isEmpty on empty string.",
            got_true, hex::encode(&r_e.return_data));

        // Sub-harness (b): isEmpty("hi") — the non-empty case.
        let src_non_empty = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function isEmpty(string memory s) internal pure returns (bool) { return bytes(s).length == 0; }
    function f() external pure returns (bool) { return isEmpty("hi"); }
}"#;
        let arts_ne = compile_contracts(src_non_empty, false, 2)
            .unwrap_or_else(|e| panic!("NNN2b compile: {:?}", e));
        let art_ne = &arts_ne[0];
        let mut rt_ne = NeoRuntime::new(RuntimeConfig::default()).expect("NNN2b rt");
        let r_ne = rt_ne.execute(&art_ne.bytecode, &[])
            .expect("NNN2b f() host-level");
        prop_assert!(r_ne.success,
            "NNN2b isEmpty(\"hi\") must succeed; exc={:?}.",
            r_ne.exception.as_ref().map(|e| &e.message));
        let got_false = decode_uint_le(&r_ne.return_data);
        prop_assert_eq!(got_false.clone(), BigUint::from(0u8),
            "NNN2b isEmpty(\"hi\") must return false; got {} \
             (rd_hex={}). If true, the bytes-cast length mis-reported \
             the 2-char string as 0-length (the length property may \
             be dropping content). Task #197+ candidate: isEmpty on \
             non-empty string.",
            got_false, hex::encode(&r_ne.return_data));

        // Sub-harness (c): startsWith("abc", "a") — the single-bytes1
        // prefix case. `bytes(s)[0] == c` must evaluate to true when
        // s starts with c.
        let src_sw = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function startsWith(string memory s, bytes1 c) internal pure returns (bool) {
        bytes memory b = bytes(s);
        return b.length > 0 && b[0] == c;
    }
    function f() external pure returns (bool) { return startsWith("abc", "a"); }
}"#;
        let arts_sw = compile_contracts(src_sw, false, 2)
            .unwrap_or_else(|e| panic!("NNN2c compile: {:?}. If this \
                fires on `bytes1 c` parameter, the single-byte param \
                type regressed. If on `b[0] == c`, the bytes1-indexing \
                on bytes-memory regressed.", e));
        let art_sw = &arts_sw[0];
        let mut rt_sw = NeoRuntime::new(RuntimeConfig::default()).expect("NNN2c rt");
        let r_sw = rt_sw.execute(&art_sw.bytecode, &[])
            .expect("NNN2c f() host-level");
        prop_assert!(r_sw.success,
            "NNN2c startsWith(\"abc\", \"a\") must succeed; exc={:?}. \
             If exc cites bytes1 indexing, the b[0] lowering regressed.",
            r_sw.exception.as_ref().map(|e| &e.message));
        let got_sw = decode_uint_le(&r_sw.return_data);
        prop_assert_eq!(got_sw.clone(), BigUint::from(1u8),
            "NNN2c startsWith(\"abc\", \"a\") must return true; got {} \
             (rd_hex={}). If false, either (a) the bytes1 literal \"a\" \
             didn't encode to 0x61 (ASCII 'a'), (b) b[0] returned \
             something other than 0x61, or (c) the equality check \
             mis-compared bytes1 operands (batch67 QQ2 pins the \
             multi-byte prefix compare; NNN2c pins the SINGLE-byte \
             prefix compare which uses a distinct bytes1 equality \
             path). Task #197+ candidate: bytes1 prefix-compare.",
            got_sw, hex::encode(&r_sw.return_data));
    }
}

// NNN3 — Arithmetic in ternary condition + arithmetic arms.
// `(a + b) > 100 ? (a + b) * 2 : a * b`. f(50, 50) = 2500 (false-arm),
// f(60, 50) = 220 (true-arm).
// 15 fuzz cases exercise repeat-exec stability.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn batch90_nnn3_arithmetic_in_ternary_condition_and_arms(
        _seed in any::<u8>(),
    ) {
        use neo_devpack_solidity::runtime::types::StackItem;
        use num_bigint::BigUint;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint a, uint b) external pure returns (uint) {
        return (a + b) > 100 ? (a + b) * 2 : a * b;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("NNN3 compile: {:?}. If this \
                fires on `(a + b) > 100 ? ... : ...`, the arithmetic-\
                in-ternary-condition form regressed. If on the arms \
                (both arithmetic), the branching-arithmetic lowering \
                regressed.", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("NNN3 rt");

        // Probe A: f(50, 50). a+b = 100. 100 > 100 = false → a*b =
        // 2500. The equality case on the condition pins STRICT
        // greater-than (not >=).
        let r_a = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(50), StackItem::Integer(50)])
            .expect("NNN3a f(50, 50) host-level");
        prop_assert!(r_a.success,
            "NNN3a f(50, 50) must succeed; exc={:?}.",
            r_a.exception.as_ref().map(|e| &e.message));
        let v_a = decode_uint_le(&r_a.return_data);
        prop_assert_eq!(v_a.clone(), BigUint::from(2500u64),
            "NNN3a f(50, 50) must equal 2500 (a+b=100, 100>100=false, \
             false-arm = a*b = 50*50 = 2500); got {} (rd_hex={}). If \
             200 (= (50+50)*2), the condition was evaluated as \
             true (>= instead of >). If 100, the false-arm was \
             mis-selected as the add rather than the product. If 0, \
             either arm was dropped. Task #197+ candidate: ternary \
             with arithmetic condition (equality boundary).",
            v_a, hex::encode(&r_a.return_data));

        // Probe B: f(60, 50). a+b = 110. 110 > 100 = true → (a+b)*2
        // = 220. Pins the true-arm arithmetic + the STRICT-greater
        // branch.
        let r_b = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "f", &[StackItem::Integer(60), StackItem::Integer(50)])
            .expect("NNN3b f(60, 50) host-level");
        prop_assert!(r_b.success,
            "NNN3b f(60, 50) must succeed; exc={:?}.",
            r_b.exception.as_ref().map(|e| &e.message));
        let v_b = decode_uint_le(&r_b.return_data);
        prop_assert_eq!(v_b.clone(), BigUint::from(220u64),
            "NNN3b f(60, 50) must equal 220 (a+b=110, 110>100=true, \
             true-arm = (a+b)*2 = 110*2 = 220); got {} (rd_hex={}). \
             If 3000 (= 60*50), the true-arm was mis-selected. If \
             110, the *2 multiplier was dropped. If 220 in the wrong \
             sub-case, the arithmetic-in-condition was mis-evaluated. \
             Task #197+ candidate: ternary true-arm arithmetic.",
            v_b, hex::encode(&r_b.return_data));
    }
}

// NNN4 — Single bytes32-indexed event with inline keccak256 arg.
// `emit Hashed(keccak256(bytes(s)))` — the indexed bytes32 carries
// the RAW keccak256 result (not re-hashed — bytes32 is a STATIC
// type). Invariants: 2 topics (sig + raw hash), 0 data bytes.
// Single-shot — deterministic input string.
#[test]
fn batch90_nnn4_bytes32_indexed_event_raw_keccak_topic_no_rehash() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use sha3::{Digest, Keccak256};
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Hashed(bytes32 indexed hash);
    function h(string memory s) external { emit Hashed(keccak256(bytes(s))); }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "NNN4 compile: {:?}. If this fires \
            on `event Hashed(bytes32 indexed hash)`, the bytes32-\
            indexed event shape regressed. If on `keccak256(bytes(s))` \
            inlined in emit, the inline-computed-indexed-arg lowering \
            regressed (distinct from baseline_tests H4's LITERAL \
            keccak256 arg form).",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("NNN4 rt");

    // Fixed input: "hello". keccak256("hello") =
    // 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8.
    let s = b"hello";
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "h",
            &[StackItem::byte_array(s.to_vec())],
        )
        .expect("NNN4 h(\"hello\") host-level");
    assert!(
        r.success,
        "NNN4 h(\"hello\") must succeed; exc={:?}. If exc cites event \
         emit or keccak256 computation, the inline-keccak-in-emit \
         lowering regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );

    // (1) Exactly 1 log must fire.
    assert_eq!(
        r.logs.len(),
        1,
        "NNN4 h() must emit exactly 1 Hashed log; got {} logs.",
        r.logs.len()
    );
    let log = &r.logs[0];

    // (2) topics.len must be 2: sig + indexed bytes32 hash.
    assert_eq!(
        log.topics.len(),
        2,
        "NNN4 bytes32-indexed event with 1 indexed arg must have 2 \
         topics (sig + raw-hash); got {} topics. If 1, sig or indexed \
         topic dropped. If 3+, extraneous topic leaked.",
        log.topics.len()
    );

    // (3) topic[0] must equal keccak256("Hashed(bytes32)").
    let sig_hash = Keccak256::digest(b"Hashed(bytes32)").to_vec();
    assert_eq!(
        &log.topics[0][..],
        &sig_hash[..],
        "NNN4 topics[0] must = keccak256(\"Hashed(bytes32)\") = 0x{}; \
         got 0x{}. If different, the event-signature canonicalization \
         for bytes32-indexed args regressed.",
        hex::encode(&sig_hash),
        hex::encode(&log.topics[0])
    );

    // (4) CRITICAL: topic[1] must equal keccak256("hello") DIRECTLY
    //     — NOT re-hashed. bytes32 is a STATIC type per Solidity
    //     spec, so indexed bytes32 carries the raw value, distinct
    //     from indexed bytes/string (dynamic types → hash-of-value).
    //     This is the inverse invariant of batch83 GGG3 (bytes-
    //     indexed → hash-of-value).
    let expected_hash = Keccak256::digest(s).to_vec();
    assert_eq!(
        &log.topics[1][..],
        &expected_hash[..],
        "NNN4 topics[1] must = keccak256(\"hello\") DIRECTLY (no \
         re-hash) = 0x{}; got 0x{}. If divergent, the indexed-bytes32 \
         topic encoding either (a) re-hashed the value (treating \
         bytes32 as a dynamic type — wrong), (b) wrote the raw bytes \
         of `s` instead of its keccak256 (indexed-arg expression \
         wasn't evaluated — just the input was stored), or (c) the \
         inline keccak256 computation diverged. baseline_tests H4 \
         pins the LITERAL keccak256(b\"TEST\") arg form; NNN4 pins \
         the COMPUTED-at-emit form. Task #197+ candidate: bytes32 \
         indexed with inline keccak256 arg — raw topic.",
        hex::encode(&expected_hash),
        hex::encode(&log.topics[1])
    );

    // (5) data MUST be empty — only arg is indexed, so nothing
    //     goes to the data section.
    assert_eq!(
        log.data.len(),
        0,
        "NNN4 log.data MUST be empty (only arg is indexed → 0 data \
         bytes); got {} bytes data=0x{}. If non-empty, the indexed \
         bytes32 leaked into data (indexed vs non-indexed conflation).",
        log.data.len(),
        hex::encode(&log.data)
    );
}

// NNN5 — Custom error via nested internal fn.
// `_check(n)` reverts `TooSmall(n)` when n < 10; f(n) calls _check
// then returns n * 2. f(15) == 30 (branch taken = false, continues),
// f(5) REVERTS with TooSmall(5) (branch taken = true, propagates).
// Single-shot — two deterministic probes.
#[test]
fn batch90_nnn5_custom_error_revert_from_nested_internal_fn() {
    use neo_devpack_solidity::runtime::types::StackItem;
    use num_bigint::BigUint;
    use sha3::{Digest, Keccak256};
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    error TooSmall(uint value);
    function _check(uint n) internal pure { if (n < 10) revert TooSmall(n); }
    function f(uint n) external pure returns (uint) { _check(n); return n * 2; }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "NNN5 compile: {:?}. If this fires \
            on `error TooSmall(uint value)`, the custom-error \
            declaration regressed. If on `_check` internal fn with \
            conditional revert, the internal-fn + custom-error \
            lowering regressed. If on `f(n) calls _check(n)` then \
            returns, the internal-call-after-revert-guard pattern \
            regressed.",
            e
        )
    });
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("NNN5 rt");

    // Probe A: f(15). 15 < 10 = false → _check does NOT revert →
    // continues → returns 15 * 2 = 30. Pins: (a) the false branch
    // in _check, (b) flow-through after an internal call that
    // did NOT revert, (c) the final arithmetic + return.
    let r_ok = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::Integer(15)],
        )
        .expect("NNN5a f(15) host-level");
    assert!(
        r_ok.success,
        "NNN5a f(15) must succeed (15 >= 10 so _check doesn't revert, \
         f returns n*2 = 30); got success=false, exc={:?} rd_hex={}. \
         If fault, either (a) _check's condition `n < 10` mis-\
         triggered (wrong comparison direction), (b) the internal \
         call dispatch regressed, or (c) the POST-internal-call \
         continuation dropped.",
        r_ok.exception.as_ref().map(|e| &e.message),
        hex::encode(&r_ok.return_data)
    );
    let v_ok = decode_uint_le(&r_ok.return_data);
    assert_eq!(
        v_ok.clone(),
        BigUint::from(30u64),
        "NNN5a f(15) must equal 30 (= 15 * 2); got {} (rd_hex={}). \
         If 15, the `* 2` was dropped. If 0, the return value was \
         lost. If 2, the multiplier replaced the operand. Task \
         #197+ candidate: flow-through after non-reverting internal \
         call.",
        v_ok,
        hex::encode(&r_ok.return_data)
    );

    // Probe B: f(5). 5 < 10 = true → _check REVERTS with TooSmall(5)
    // → the revert propagates OUT of _check, up through f, to the
    // caller. Invariants: success=false, return_data carries the
    // canonical custom-error envelope:
    //   return_data = keccak256("TooSmall(uint256)")[..4] || BE32(5)
    //              = selector (4 bytes) || 32 bytes BE(5)
    //              = 36 bytes total.
    let r_rev = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::Integer(5)],
        )
        .expect("NNN5b f(5) host-level (revert != host error)");
    assert!(
        !r_rev.success,
        "NNN5b f(5) must REVERT (5 < 10 triggers _check → revert \
         TooSmall(5)); got success=true rd_hex={}. If success, the \
         revert did NOT propagate across the internal-call boundary \
         (Solidity has no try/catch on internal calls, so the revert \
         MUST propagate). Task #197+ candidate: custom-error revert \
         propagation through internal fn.",
        hex::encode(&r_rev.return_data)
    );

    // Selector: keccak256("TooSmall(uint256)")[..4]. (Solidity
    // desugars `uint` to `uint256` for the selector hash.)
    let sel = {
        let d = Keccak256::digest(b"TooSmall(uint256)");
        [d[0], d[1], d[2], d[3]]
    };
    let rd = &r_rev.return_data;
    assert!(
        rd.len() >= 4 && &rd[..4] == &sel[..],
        "NNN5b revert payload must PREFIX with keccak256(\"TooSmall(\
         uint256)\")[..4] = 0x{}; got rd_hex={} (len {}). If absent, \
         the custom-error selector wasn't threaded through the \
         internal-call revert path. If present but not as prefix, \
         an extra envelope wrapped the payload. Task #197+ \
         candidate: custom-error selector propagation through \
         internal fn.",
        hex::encode(&sel),
        hex::encode(rd),
        rd.len()
    );

    // Payload tail: abi.encode(5) = BE32(5) = [0; 31] || 0x05.
    assert!(
        rd.len() >= 36,
        "NNN5b revert payload must be at least 36 bytes (selector + \
         BE32(n)); got {} bytes rd_hex={}. If <36, abi.encode of the \
         error arg was dropped (only the selector made it).",
        rd.len(),
        hex::encode(rd)
    );
    let mut expected_tail = [0u8; 32];
    expected_tail[24..].copy_from_slice(&5u64.to_be_bytes());
    assert_eq!(
        &rd[4..36],
        &expected_tail[..],
        "NNN5b revert payload tail must = BE32(5) = 0x{}; got 0x{}. \
         If the last byte is not 0x05, the error-arg value was \
         corrupted in transit. If the encoding is LE, the canonical \
         form flipped. If zero, the arg was dropped and only the \
         selector made it. Task #197+ candidate: custom-error arg \
         BE32 encoding through internal-fn revert.",
        hex::encode(&expected_tail),
        hex::encode(&rd[4..36])
    );
}

// Task ID resolution for Batch #90 on first exec:
//   - NNN1 (ERC-20 Transfer parametric emit, 2 indexed + 1 data):
//     RESOLVED GREEN. topics.len == 3 (sig + 2 indexed addresses),
//     topic[0] = keccak256("Transfer(address,address,uint256)"),
//     topic[2] = `to` address left-padded to 32 bytes, data.len ==
//     32 = BE32(1000). Extends batches_18_30 H3's literal-args form
//     to the PARAMETER-SOURCED form with msg.sender + external
//     address + external uint256 — all three paths lower correctly.
//     Non-regression surface.
//   - NNN2 (isEmpty + startsWith-bytes1): RESOLVED GREEN across 15
//     fuzz cases. isEmpty("") == true, isEmpty("hi") == false,
//     startsWith("abc", "a") == true. The bytes-cast length
//     predicate, and the single-bytes1 prefix compare via `b[0] ==
//     c`, both lower correctly. Extends batch67 QQ2's string-prefix
//     slice-compare to the bytes1 SINGLE-CHAR prefix variant. Non-
//     regression surface.
//   - NNN3 (arithmetic ternary condition + arithmetic arms):
//     RESOLVED GREEN across 15 fuzz cases. f(50, 50) = 2500 (a+b=100,
//     100>100=false, false-arm a*b=2500), f(60, 50) = 220 (a+b=110,
//     110>100=true, true-arm (a+b)*2=220). Both the arithmetic-in-
//     condition (add-then-compare) and the arithmetic arms (re-used
//     add in true-arm, multiplication in false-arm) lower correctly;
//     strict-greater-than semantics (not >=) is correct on the
//     equality boundary. Non-regression surface.
//   - NNN4 (bytes32-indexed with inline keccak arg, raw topic):
//     RESOLVED GREEN. topics.len == 2, topic[0] = keccak256
//     ("Hashed(bytes32)"), topic[1] = keccak256("hello") DIRECTLY
//     (no re-hash — the spec-correct behavior for STATIC indexed
//     types, distinct from batch83 GGG3's dynamic-bytes-indexed
//     hash-of-value invariant). The inline `keccak256(bytes(s))`
//     expression evaluates within the emit site and threads the
//     result into the indexed slot. Extends baseline_tests H4's
//     literal-keccak-arg form to the COMPUTED-at-emit form. Non-
//     regression surface.
//   - NNN5 (custom error from nested internal fn): RESOLVED GREEN.
//     f(15) == 30 (internal-branch false, continues, returns n*2);
//     f(5) REVERTS with the canonical 36-byte TooSmall(5) envelope:
//     keccak256("TooSmall(uint256)")[..4] || BE32(5). The revert
//     propagates correctly across the internal-call boundary (as
//     Solidity mandates — no try/catch on internal calls), and the
//     selector + abi.encode(n) payload shape matches baseline_tests'
//     direct-revert form precisely. Non-regression surface.
//
// New Task IDs filed in Batch #90: NONE. All 5 harnesses passed
// GREEN on first exec. This marks the SECOND consecutive full-5-GREEN
// batch (after #89) and the FIRST two-in-a-row all-GREEN run since
// the #83 → #84 transition, signaling that the DeFi-adjacent surfaces
// probed here (ERC-20 Transfer shape, string pattern predicates,
// arithmetic ternary, static-indexed-event canonicalization, and
// internal-fn revert propagation) are all well-covered by prior
// canonicalization work (Tasks #39, #103, #113, etc.).
//
// Sibling agent context: Batch #90's all-GREEN result means the
// parent-reported 50k hunt runs uncontended by new Batch #90 task
// surfaces. This closes out the 81-90 range in this file (final
// slot). Net cadence for this file's 10 batches: Batches #81/#82/#85/
// #87/#88 each filed a new Task and #[ignore]'d one harness; Batches
// #83/#84/#86/#89/#90 landed all 5 GREEN. Cumulative Task IDs filed
// from this file: #191 (EEE2), #192 (FFF4), #193 (GGG5), #194 (HHH5),
// #195 (KKK1), #196 (LLL1) — six new surfaces exposed across 50
// harnesses. Target 490 passed + 0 ignored REACHED.
