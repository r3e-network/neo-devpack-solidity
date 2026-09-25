//! Storage-soundness regression tests (agent key: storage).
//!
//! Covers the confirmed findings from the storage-soundness review:
//!  1. Mapping keys are canonicalized (CONVERT) before `StdLib.serialize`,
//!     so Buffer vs ByteString representations of the same value derive the
//!     same storage slot on real Neo N3 (structural bytecode assertion —
//!     the bundled runtime's unified ByteArray type cannot observe the
//!     divergence behaviorally).
//!  2. `delete` on a fixed-size storage array zeroes every element.
//!  3. `delete` on a dynamic storage array zeroes elements + length and the
//!     array remains usable afterwards.
//!  4. Struct-field array subscripts (`s.arr[i]`) respect a Panic(0x32)
//!     bounds guard: out-of-range reads and post-`delete s` reads panic
//!     instead of silently returning stale/zero data; fixed-size struct
//!     fields (`uint256[3]` inside a struct) keep working in-range.
//!  5. `delete` on a struct containing a mapping member does not emit a
//!     `Storage.Put` for the mapping field (which would Put a Null value
//!     and fault on real Neo N3); the mapping entries survive per Solidity
//!     semantics while value members read back as zero.
//!  6. Sibling-merge rejects same-named state variables with conflicting
//!     types instead of silently aliasing one storage slot.

#![allow(clippy::uninlined_format_args)]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::interop::interop_id_bytes;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{ExecutionResult, NeoRuntime, RuntimeConfig};
use num_bigint::BigUint;

/// Test-harness gas budget. The production `RuntimeConfig::default().gas_limit`
/// is 10M (Neo N3 mainnet-scale); after the S2 fix `Storage.Put` charges the
/// mainnet-aligned 100_000/byte rate, so storage-heavy contracts written by
/// these tests can legitimately exceed 10M. These tests are not gas-asserting,
/// so they use a generous 1B budget. Gas-asserting tests construct their own
/// `RuntimeConfig` explicitly.
fn test_runtime() -> NeoRuntime {
    NeoRuntime::new(RuntimeConfig {
        gas_limit: 1_000_000_000,
        ..RuntimeConfig::default()
    })
    .expect("runtime")
}

fn decode_uint_le(bytes: &[u8]) -> BigUint {
    if bytes.is_empty() {
        BigUint::from(0u8)
    } else {
        BigUint::from_bytes_le(bytes)
    }
}

fn returned_uint(result: &ExecutionResult, what: &str) -> BigUint {
    assert!(
        result.success,
        "{what} must succeed; exception={:?}",
        result.exception.as_ref().map(|e| &e.message)
    );
    decode_uint_le(&result.return_data)
}

/// Count non-overlapping occurrences of `needle` in `haystack`.
fn count_occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    let mut count = 0;
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        if &haystack[i..i + needle.len()] == needle {
            count += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    count
}

/// Byte positions where `needle` starts in `haystack`.
fn find_positions(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    if needle.is_empty() || haystack.len() < needle.len() {
        return out;
    }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle {
            out.push(i);
        }
    }
    out
}

// =====================================================================
// Finding 1 — mapping-key canonicalization: every StdLib `serialize`
// call used for slot derivation must be preceded by a CONVERT to the
// key's canonical stack-item type.
// =====================================================================

#[test]
fn mapping_key_serialize_is_preceded_by_canonicalizing_convert() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract MapKeys {
    mapping(bytes => uint256) m;
    function put(bytes memory a, bytes memory b, uint256 v) public {
        m[bytes.concat(a, b)] = v;
    }
    function get(bytes memory k) public view returns (uint256) {
        return m[k];
    }
}"#;
    // Optimizer level 0 keeps the raw emission layout.
    let arts = compile_contracts(src, false, 0).expect("MapKeys must compile");
    assert!(!arts.is_empty());
    let bytecode = &arts[0].bytecode;

    // Every `serialize` native-call site (PUSHDATA1 len=9 "serialize")
    // belongs to mapping-slot derivation; the canonicalizing CONVERT
    // (0xDB 0x28 for a `bytes` key) must appear shortly before it (the
    // intervening bytes are the arg PACK + call-flags push).
    let mut needle = vec![0x0Cu8, 0x09];
    needle.extend_from_slice(b"serialize");
    let sites = find_positions(bytecode, &needle);
    assert!(
        sites.len() >= 2,
        "expected serialize call sites for put+get, found {}",
        sites.len()
    );
    for pos in &sites {
        let window_start = pos.saturating_sub(16);
        let window = &bytecode[window_start..*pos];
        assert!(
            count_occurrences(window, &[0xDB, 0x28]) >= 1,
            "serialize call at byte offset {pos} is not preceded by CONVERT \
             ByteString (0xDB 0x28); window={:?}",
            window
        );
    }

    // Behavioral sanity on the bundled runtime: write via bytes.concat,
    // read via an ABI parameter with the identical bytes.
    let art = &arts[0];
    let mut rt = test_runtime();
    let a = StackItem::byte_array(vec![0xAA, 0xBB]);
    let b = StackItem::byte_array(vec![0xCC]);
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "put",
            &[a, b, StackItem::Integer(42)],
        )
        .expect("put host call");
    assert!(
        r.success,
        "put failed: {:?}",
        r.exception.map(|e| e.message)
    );
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "get",
            &[StackItem::byte_array(vec![0xAA, 0xBB, 0xCC])],
        )
        .expect("get host call");
    assert_eq!(returned_uint(&r, "get"), BigUint::from(42u8));
}

#[test]
fn storage_put_enforces_neo_n3_value_size_limit() {
    // Neo N3 MaxStorageValueSize is 65535; a Storage.Put of a larger value
    // FAULTs on-chain. The simulator must model that rather than silently
    // succeeding (it would pass the 10 MiB overlay cap otherwise).
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes data;
    function set(bytes memory v) external { data = v; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = test_runtime();

    // 70000-byte value exceeds MaxStorageValueSize -> must FAULT.
    let big = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "set",
            &[StackItem::byte_array(vec![0x5A; 70_000])],
        )
        .expect("host call");
    assert!(
        !big.success,
        "storing a 70000-byte value must FAULT (exceeds Neo N3 MaxStorageValueSize)"
    );

    // A comfortably-under-limit value still stores fine.
    let mut rt2 = test_runtime();
    let ok = rt2
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "set",
            &[StackItem::byte_array(vec![0x5A; 1000])],
        )
        .expect("host call");
    assert!(
        ok.success,
        "storing a 1000-byte value must succeed: {:?}",
        ok.exception.map(|e| e.message)
    );
}

#[test]
fn integer_mapping_key_serialize_uses_integer_convert() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract IntKeys {
    mapping(uint256 => uint256) m;
    function put(uint256 k, uint256 v) public { m[k] = v; }
    function get(uint256 k) public view returns (uint256) { return m[k]; }
}"#;
    let arts = compile_contracts(src, false, 0).expect("IntKeys must compile");
    let bytecode = &arts[0].bytecode;
    let mut needle = vec![0x0Cu8, 0x09];
    needle.extend_from_slice(b"serialize");
    let sites = find_positions(bytecode, &needle);
    assert!(
        sites.len() >= 2,
        "expected serialize sites, found {}",
        sites.len()
    );
    for pos in &sites {
        let window = &bytecode[pos.saturating_sub(16)..*pos];
        assert!(
            count_occurrences(window, &[0xDB, 0x21]) >= 1,
            "serialize call at offset {pos} missing CONVERT Integer (0xDB 0x21)"
        );
    }
}

// =====================================================================
// Finding 2 — delete on a fixed-size storage array clears every element.
// =====================================================================

#[test]
fn delete_fixed_size_storage_array_zeroes_all_elements() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract FixedDelete {
    uint256[3] arr;
    function setAll() public {
        arr[0] = 11;
        arr[1] = 22;
        arr[2] = 33;
    }
    function wipe() public {
        delete arr;
    }
    function get(uint256 i) public view returns (uint256) {
        return arr[i];
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("FixedDelete must compile");
    let art = &arts[0];
    let mut rt = test_runtime();

    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "setAll", &[])
        .expect("setAll");
    assert!(
        r.success,
        "setAll failed: {:?}",
        r.exception.map(|e| e.message)
    );

    for (i, expected) in [(0i64, 11u64), (1, 22), (2, 33)] {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "get",
                &[StackItem::Integer(i)],
            )
            .expect("get pre-wipe");
        assert_eq!(
            returned_uint(&r, "get pre-wipe"),
            BigUint::from(expected),
            "arr[{i}] before delete"
        );
    }

    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "wipe", &[])
        .expect("wipe");
    assert!(
        r.success,
        "wipe failed: {:?}",
        r.exception.map(|e| e.message)
    );

    for i in 0i64..3 {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "get",
                &[StackItem::Integer(i)],
            )
            .expect("get post-wipe");
        assert_eq!(
            returned_uint(&r, "get post-wipe"),
            BigUint::from(0u8),
            "arr[{i}] must be zero after delete"
        );
    }
}

// =====================================================================
// Finding 2 (dynamic case) — delete resets length, clears slots, and the
// array remains usable.
// =====================================================================

#[test]
fn delete_dynamic_storage_array_resets_length_and_stays_usable() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract DynDelete {
    uint256[] arr;
    function pushVal(uint256 v) public { arr.push(v); }
    function wipe() public { delete arr; }
    function len() public view returns (uint256) { return arr.length; }
    function get(uint256 i) public view returns (uint256) { return arr[i]; }
    function readAt(uint256 i) public returns (uint256) {
        try this.get(i) returns (uint256 v) {
            return v;
        } catch Panic(uint256) {
            return 0xdead;
        }
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("DynDelete must compile");
    let art = &arts[0];
    let mut rt = test_runtime();
    let call = |rt: &mut NeoRuntime, name: &str, args: &[StackItem]| {
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, name, args)
            .unwrap_or_else(|e| panic!("{name} host call failed: {e:?}"))
    };

    assert!(call(&mut rt, "pushVal", &[StackItem::Integer(7)]).success);
    assert!(call(&mut rt, "pushVal", &[StackItem::Integer(9)]).success);
    let r = call(&mut rt, "len", &[]);
    assert_eq!(returned_uint(&r, "len pre-wipe"), BigUint::from(2u8));
    let r = call(&mut rt, "get", &[StackItem::Integer(0)]);
    assert_eq!(returned_uint(&r, "get(0) pre-wipe"), BigUint::from(7u8));

    assert!(call(&mut rt, "wipe", &[]).success);

    let r = call(&mut rt, "len", &[]);
    assert_eq!(returned_uint(&r, "len post-wipe"), BigUint::from(0u8));
    // Reads past the (now zero) length must Panic(0x32) -> caught -> 0xdead.
    let r = call(&mut rt, "readAt", &[StackItem::Integer(0)]);
    assert_eq!(
        returned_uint(&r, "readAt(0) post-wipe"),
        BigUint::from(0xdeadu32),
        "post-delete read must panic, not return stale data"
    );

    // The array stays usable after delete.
    assert!(call(&mut rt, "pushVal", &[StackItem::Integer(5)]).success);
    let r = call(&mut rt, "len", &[]);
    assert_eq!(returned_uint(&r, "len after re-push"), BigUint::from(1u8));
    let r = call(&mut rt, "get", &[StackItem::Integer(0)]);
    assert_eq!(
        returned_uint(&r, "get(0) after re-push"),
        BigUint::from(5u8)
    );
}

// =====================================================================
// Finding 3 — struct-field array subscripts respect the Panic(0x32)
// bounds guard (OOB reads and post-`delete s` reads).
// =====================================================================

#[test]
fn struct_field_dynamic_array_subscript_is_bounds_guarded() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract StructArr {
    struct S { uint256 a; uint256[] arr; }
    S s;
    function setup() public {
        s.a = 1;
        s.arr.push(11);
        s.arr.push(22);
    }
    function len() public view returns (uint256) { return s.arr.length; }
    function get(uint256 i) public view returns (uint256) { return s.arr[i]; }
    function readAt(uint256 i) public returns (uint256) {
        try this.get(i) returns (uint256 v) {
            return v;
        } catch Panic(uint256) {
            return 0xdead;
        }
    }
    function wipe() public { delete s; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("StructArr must compile");
    let art = &arts[0];
    let mut rt = test_runtime();
    let call = |rt: &mut NeoRuntime, name: &str, args: &[StackItem]| {
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, name, args)
            .unwrap_or_else(|e| panic!("{name} host call failed: {e:?}"))
    };

    assert!(call(&mut rt, "setup", &[]).success);
    let r = call(&mut rt, "len", &[]);
    assert_eq!(returned_uint(&r, "len"), BigUint::from(2u8));

    // In-range reads still work.
    let r = call(&mut rt, "readAt", &[StackItem::Integer(0)]);
    assert_eq!(returned_uint(&r, "readAt(0)"), BigUint::from(11u8));
    let r = call(&mut rt, "readAt", &[StackItem::Integer(1)]);
    assert_eq!(returned_uint(&r, "readAt(1)"), BigUint::from(22u8));

    // Out-of-range read must Panic(0x32), not silently return 0.
    let r = call(&mut rt, "readAt", &[StackItem::Integer(5)]);
    assert_eq!(
        returned_uint(&r, "readAt(5)"),
        BigUint::from(0xdeadu32),
        "OOB struct-field array read must Panic(0x32)"
    );

    // After `delete s`, the stale element slots must be unreachable.
    assert!(call(&mut rt, "wipe", &[]).success);
    let r = call(&mut rt, "len", &[]);
    assert_eq!(returned_uint(&r, "len post-delete"), BigUint::from(0u8));
    for i in 0i64..2 {
        let r = call(&mut rt, "readAt", &[StackItem::Integer(i)]);
        assert_eq!(
            returned_uint(&r, "readAt post-delete"),
            BigUint::from(0xdeadu32),
            "deleted struct-array element {i} must not be readable"
        );
    }
}

#[test]
fn struct_field_fixed_size_array_uses_declared_bound() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract StructFixed {
    struct F { uint256 x; uint256[3] vals; }
    F f;
    function setVals() public {
        f.vals[0] = 5;
        f.vals[1] = 6;
        f.vals[2] = 7;
    }
    function get(uint256 i) public view returns (uint256) { return f.vals[i]; }
    function readAt(uint256 i) public returns (uint256) {
        try this.get(i) returns (uint256 v) {
            return v;
        } catch Panic(uint256) {
            return 0xdead;
        }
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("StructFixed must compile");
    let art = &arts[0];
    let mut rt = test_runtime();
    let call = |rt: &mut NeoRuntime, name: &str, args: &[StackItem]| {
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, name, args)
            .unwrap_or_else(|e| panic!("{name} host call failed: {e:?}"))
    };

    assert!(call(&mut rt, "setVals", &[]).success);
    // Fixed-size struct fields maintain no length slot — in-range reads must
    // NOT spuriously panic (the guard uses the declared compile-time bound).
    for (i, expected) in [(0i64, 5u64), (1, 6), (2, 7)] {
        let r = call(&mut rt, "readAt", &[StackItem::Integer(i)]);
        assert_eq!(
            returned_uint(&r, "readAt in-range"),
            BigUint::from(expected),
            "f.vals[{i}] in-range read"
        );
    }
    // Past the declared bound -> Panic(0x32).
    let r = call(&mut rt, "readAt", &[StackItem::Integer(3)]);
    assert_eq!(
        returned_uint(&r, "readAt(3)"),
        BigUint::from(0xdeadu32),
        "read past the fixed bound must Panic(0x32)"
    );
}

// =====================================================================
// Finding 4 — delete on a struct with a mapping member: no Put for the
// mapping field, value members zeroed, mapping entries survive.
// =====================================================================

#[test]
fn delete_struct_with_mapping_member_behaves_like_solidity() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract SlotDelete {
    struct Slot { uint256 amount; mapping(address => uint256) balances; }
    mapping(uint256 => Slot) slots;
    function setAmount(uint256 k, uint256 v) public { slots[k].amount = v; }
    function setBal(uint256 k, address a, uint256 v) public { slots[k].balances[a] = v; }
    function del(uint256 k) public { delete slots[k]; }
    function getAmount(uint256 k) public view returns (uint256) { return slots[k].amount; }
    function getBal(uint256 k, address a) public view returns (uint256) { return slots[k].balances[a]; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("SlotDelete must compile");
    let art = &arts[0];
    let mut rt = test_runtime();
    let addr = StackItem::byte_array(vec![0x11u8; 20]);
    let call = |rt: &mut NeoRuntime, name: &str, args: &[StackItem]| {
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, name, args)
            .unwrap_or_else(|e| panic!("{name} host call failed: {e:?}"))
    };

    assert!(
        call(
            &mut rt,
            "setAmount",
            &[StackItem::Integer(1), StackItem::Integer(42)]
        )
        .success
    );
    assert!(
        call(
            &mut rt,
            "setBal",
            &[StackItem::Integer(1), addr.clone(), StackItem::Integer(99)]
        )
        .success
    );

    let r = call(&mut rt, "del", &[StackItem::Integer(1)]);
    assert!(
        r.success,
        "delete slots[k] must not fault: {:?}",
        r.exception.map(|e| e.message)
    );

    let r = call(&mut rt, "getAmount", &[StackItem::Integer(1)]);
    assert_eq!(returned_uint(&r, "getAmount"), BigUint::from(0u8));
    // Solidity: delete skips mapping members — entries survive.
    let r = call(&mut rt, "getBal", &[StackItem::Integer(1), addr]);
    assert_eq!(returned_uint(&r, "getBal"), BigUint::from(99u8));
}

#[test]
fn delete_struct_with_mapping_member_emits_no_put_for_mapping_field() {
    // A contract whose ONLY function deletes a struct with one value member
    // and one mapping member: the whole program must contain exactly one
    // System.Storage.Put (the `amount` field). Pre-fix it contained two —
    // the second was a Put of a Null value for the mapping field, which
    // faults on real Neo N3.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract DelOnly {
    struct Slot { uint256 amount; mapping(address => uint256) balances; }
    mapping(uint256 => Slot) slots;
    function del(uint256 k) public { delete slots[k]; }
}"#;
    let arts = compile_contracts(src, false, 0).expect("DelOnly must compile");
    let bytecode = &arts[0].bytecode;
    let mut put_needle = vec![0x41u8]; // SYSCALL
    put_needle.extend_from_slice(&interop_id_bytes("System.Storage.Put"));
    let puts = count_occurrences(bytecode, &put_needle);
    assert_eq!(
        puts, 1,
        "delete on a struct with one value member + one mapping member must \
         emit exactly one Storage.Put (the value member); found {puts}"
    );
}

// =====================================================================
// Finding 5 — sibling-merge rejects same-named state variables with
// conflicting types (and keeps allowing identical types).
// =====================================================================

#[test]
fn sibling_merge_rejects_conflicting_state_variable_types() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Mock {
    mapping(address => uint256) _bal;
    function mint(address a, uint256 v) public { _bal[a] += v; }
    function balanceOf(address a) public view returns (uint256) { return _bal[a]; }
}
contract Client {
    uint256 _bal;
    Mock public m;
    function go() public { m = new Mock(); }
    function setHostBal(uint256 v) public { _bal = v; }
}"#;
    let result = compile_contracts(src, false, 2);
    let err = match result {
        Err(e) => format!("{e:?}"),
        Ok(_) => panic!(
            "conflicting same-named state variables across merged contracts \
             must be a hard compile error (storage slots are name-keyed)"
        ),
    };
    assert!(
        err.contains("_bal"),
        "error should name the conflicting variable; got: {err}"
    );
}

#[test]
fn sibling_merge_keeps_allowing_identical_state_variable_types() {
    // Task #197 design: same-name SAME-type slot sharing is intentional.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Mock {
    mapping(address => uint256) _bal;
    function mint(address a, uint256 v) public { _bal[a] += v; }
    function balanceOf(address a) public view returns (uint256) { return _bal[a]; }
}
contract Client {
    mapping(address => uint256) _bal;
    Mock public m;
    function go() public { m = new Mock(); }
}"#;
    let arts = compile_contracts(src, false, 2)
        .expect("identical-type same-name sibling state vars must keep compiling");
    assert!(!arts.is_empty());
}
