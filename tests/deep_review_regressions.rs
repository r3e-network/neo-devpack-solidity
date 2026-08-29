//! Regression tests for the deep-review correctness fixes.
//!
//! Each test pins a behavior that previously miscompiled or faulted on a real
//! Neo node while passing in the in-tree simulator (which masked the bug).

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

/// #1 — An external function taking a struct parameter must reserve ONE arg
/// slot (the struct arrives as a single `Array` StackItem, per the manifest's
/// single `Array` parameter). Previously INITSLOT over-counted the flattened
/// field count, so a conformant single-`Array` call underflowed the frame and
/// faulted. Here the call passes one Array and must succeed with the right sum.
#[test]
fn external_struct_param_is_callable_with_single_array_arg() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct P { uint256 a; uint256 b; }
    function f(P memory p) external pure returns (uint256) { return p.a + p.b; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::array(vec![
                StackItem::Integer(11),
                StackItem::Integer(31),
            ])],
        )
        .expect("call");
    assert!(
        r.success,
        "struct-param external fn faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    // 11 + 31 = 42.
    assert_eq!(r.return_data.first().copied(), Some(42u8), "sum must be 42");
}

/// #3 — A contract function declared without an explicit visibility specifier
/// is a hard error in Solidity 0.5.0+. Previously it silently defaulted to
/// `internal` and vanished from the ABI, producing a contract that "compiles"
/// but exposes none of its intended entrypoints.
#[test]
fn missing_visibility_specifier_is_a_hard_error() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Tok {
    mapping(address => uint256) bal;
    function balanceOf(address a) returns (uint256) { return bal[a]; }
}"#;
    let err = compile_contracts(src, false, 2).expect_err("missing visibility must be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("visibility") || msg.contains("NO_VISIBILITY_SPECIFIER"),
        "diagnostic should mention the missing visibility; got: {msg}"
    );
}

/// #3 — explicit visibility still compiles and the method appears in the ABI.
#[test]
fn explicit_visibility_keeps_method_in_abi() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Tok {
    mapping(address => uint256) bal;
    function balanceOf(address a) public view returns (uint256) { return bal[a]; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let methods = arts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods");
    assert!(
        methods
            .iter()
            .any(|m| m["name"].as_str() == Some("balanceOf")),
        "balanceOf must appear in the ABI"
    );
}

/// #5 — `int256.min / -1` overflows. Native NeoVM DIV would fault UNCATCHABLY
/// (the quotient 2^255 needs 33 bytes); the compiler now pre-checks the pair.
/// In an `unchecked` block the result wraps to `int256.min` instead of faulting.
#[test]
fn int256_min_div_minus_one_unchecked_wraps_without_faulting() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(int256 a, int256 b) external pure returns (int256) {
        unchecked { return a / b; }
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    // int256.min = -2^255 -> 32-byte little-endian two's complement: 0x80 in the
    // most-significant (last LE) byte, all others zero.
    let mut min_le = vec![0u8; 32];
    min_le[31] = 0x80;
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[
                StackItem::byte_array(min_le.clone()),
                StackItem::Integer(-1),
            ],
        )
        .expect("call");
    assert!(
        r.success,
        "unchecked int256.min / -1 must not fault: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    // Result wraps to int256.min, whose 32-byte little-endian form is `min_le`.
    let mut got = r.return_data.clone();
    got.resize(32, 0);
    assert_eq!(
        got, min_le,
        "unchecked int256.min / -1 must wrap to int256.min"
    );
}

/// #4 — `abi.encode` of a `bytesN` value must produce a correct fixed-width,
/// left-aligned, big-endian 32-byte slot regardless of how the value is backed.
/// Fixed: an Integer-backed `bytesN` static arg (hex literal / named constant)
/// is now resolved to its big-endian bytes and emitted as a left-aligned slot,
/// instead of the byte-reversed (N==32) / `SIZE`-faulting (N<32) encoder path;
/// ByteArray-backed values (keccak output) keep their correct path. (A bytesN
/// constant as a struct FIELD, or alongside a dynamic arg via the head/tail
/// path, still uses the encoder — rarer, see the abi_encode.rs note.)
#[test]
fn abi_encode_bytesn_produces_correct_fixed_width_slot() {
    // `z` is a runtime arg so the multi-arg static-slot path is exercised
    // (and not constant-folded away); the `bytesN` constant flows through the
    // bytesN ABI-slot encoder under test.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes32 constant FULL  = 0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff;
    bytes32 constant TRAIL = 0xaabbccdd00000000000000000000000000000000000000000000000000000000;
    bytes4  constant SEL   = 0x01ffc9a7;
    function encFull(uint8 z)  external pure returns (bytes memory) { return abi.encode(z, FULL); }
    function encTrail(uint8 z) external pure returns (bytes memory) { return abi.encode(z, TRAIL); }
    function encSel(uint8 z)   external pure returns (bytes memory) { return abi.encode(z, SEL); }
    // keccak output is ByteArray-backed (already big-endian) — must round-trip
    // unchanged (no reversal).
    function encKeccak(bytes memory d) external pure returns (bytes memory) {
        return abi.encode(uint8(0), keccak256(d));
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];

    let call = |method: &str, arg: StackItem| -> Vec<u8> {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, method, &[arg])
            .expect("call");
        assert!(
            r.success,
            "{method} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data
    };

    // Helper: bytes32 slot is bytes [32..64] of `abi.encode(uint8, bytes32)`.
    let hexb = |s: &str| hex::decode(s).unwrap();

    let full = call("encFull", StackItem::Integer(0));
    assert_eq!(full.len(), 64, "abi.encode(uint8,bytes32) is 64 bytes");
    assert_eq!(
        &full[32..64],
        hexb("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff").as_slice(),
        "bytes32 (full) slot must be the value, big-endian, left-aligned"
    );

    let trail = call("encTrail", StackItem::Integer(0));
    assert_eq!(
        &trail[32..64],
        hexb("aabbccdd00000000000000000000000000000000000000000000000000000000").as_slice(),
        "bytes32 with trailing zeros must keep its full 32-byte width (was truncated)"
    );

    let sel = call("encSel", StackItem::Integer(0));
    let mut expected_sel = vec![0u8; 32];
    expected_sel[..4].copy_from_slice(&hexb("01ffc9a7"));
    assert_eq!(
        &sel[32..64],
        expected_sel.as_slice(),
        "bytes4 must be left-aligned + zero-padded to 32 bytes (was a SIZE fault)"
    );

    // keccak256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
    let kec = call("encKeccak", StackItem::byte_array(Vec::new()));
    assert_eq!(
        &kec[32..64],
        hexb("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").as_slice(),
        "keccak256 (ByteArray-backed) slot must round-trip unchanged (no reversal)"
    );
}

/// #8 — uintN multiply (N >= 128) must not fault uncatchably: the full product
/// exceeds NeoVM's 32-byte integer. Checked overflow → catchable Panic(0x11);
/// unchecked → wraps mod 2^N. Verified for uint128 via runtime execution.
#[test]
fn uint128_mul_checked_panics_and_unchecked_wraps_without_faulting() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function mc(uint128 a, uint128 b) external pure returns (uint128) { return a * b; }
    function mu(uint128 a, uint128 b) external pure returns (uint128) { unchecked { return a * b; } }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    // 2^e (e < 128) as a 16-byte little-endian uint128 argument; for e <= 100
    // the high bit is clear, so it's an unambiguous positive value.
    let pow2_u128 = |e: usize| -> StackItem {
        let mut le = vec![0u8; 16];
        le[e / 8] = 1u8 << (e % 8);
        StackItem::byte_array(le)
    };
    let runc = |m: &str, a: StackItem, b: StackItem| {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[a, b])
            .expect("call")
    };
    // checked: 2^100 * 2^100 = 2^200 > 2^128-1 -> overflow Panic (not success).
    let r = runc("mc", pow2_u128(100), pow2_u128(100));
    assert!(
        !r.success,
        "checked uint128 overflow must Panic, not succeed"
    );
    // checked: 3 * 5 = 15 fits.
    let r = runc("mc", StackItem::Integer(3), StackItem::Integer(5));
    assert!(
        r.success && r.return_data.first().copied() == Some(15),
        "checked 3*5 must be 15"
    );
    // unchecked: 2^100 * 2^100 = 2^200 -> mod 2^128 = 0, must NOT fault.
    let r = runc("mu", pow2_u128(100), pow2_u128(100));
    assert!(
        r.success,
        "unchecked uint128 mul must not fault: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert!(
        r.return_data.iter().all(|&x| x == 0),
        "2^100 * 2^100 mod 2^128 == 0"
    );
}

/// deep-review #4-delete — `delete` of a struct-element storage array must clear
/// the per-field element slots, not just the length. Previously only the length
/// slot was zeroed, so `ps[i].field` (which reads via the per-field slot and
/// skips the array bounds guard) resurrected the pre-delete value.
#[test]
fn delete_struct_element_array_clears_field_slots() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct P { uint256 a; uint256 b; }
    P[] ps;
    function test() external returns (uint256) {
        ps.push(P(7, 9));   // ps[0].a = 7
        delete ps;          // must clear the ps[0].a field slot too
        return ps[0].a;     // length 0; reads the field slot directly
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "test", &[])
        .expect("call");
    assert!(
        r.success,
        "test() faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    // After `delete ps`, ps[0].a must read as the default 0, not the stale 7.
    assert!(
        r.return_data.iter().all(|&x| x == 0),
        "deleted struct-element field must read as 0, got {:?}",
        r.return_data
    );
}

/// deep-review #1-itoa (simulator) — `StdLib.itoa` must format the full
/// arbitrary-precision value. The old i64 decode truncated wide (>64-bit)
/// integers to their low 8 bytes, formatting the wrong number and giving false
/// confidence to any test asserting itoa output for large values.
#[test]
fn stdlib_itoa_formats_full_wide_integer() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
import "devpack/contracts/Syscalls.sol";
contract C {
    function f(int256 v) external view returns (bytes memory) { return bytes(Syscalls.itoa(v)); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .expect("contract C");
    // 2^100 as a 32-byte little-endian int256 (positive: bit 100, high bit clear).
    let mut le = vec![0u8; 32];
    le[12] = 0x10; // bit 100 = byte 12, bit 4
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[StackItem::byte_array(le)],
        )
        .expect("call");
    assert!(
        r.success,
        "itoa faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        String::from_utf8_lossy(&r.return_data),
        "1267650600228229401496703205376", // 2^100 in decimal
        "itoa must format the full 2^100, not a low-8-byte truncation"
    );
}

/// deep-review bytesN comparison — a `bytesN` named constant is Integer-backed
/// (pushed little-endian) while a ByteArray-backed value (keccak/cast/storage/
/// param) is big-endian, so a raw EQUAL compared them byte-reversed and equal
/// values tested UNEQUAL (e.g. `require(role == ADMIN_ROLE)` silently failing).
/// Fixed for the common `runtime_value == CONSTANT` shape by canonicalizing the
/// constant to big-endian bytes when the other operand is not integer-backed.
/// (Indexing `b[i]` and bitwise on Integer-backed bytesN remain — see the
/// bytesN-backing note in `binary.rs` / the v0.25 memory.)
#[test]
fn bytesn_constant_compares_equal_to_runtime_value() {
    // A bytes32 param (ByteArray-backed) compared against a bytes32 constant
    // with the same value must be equal.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes32 constant ROLE  = 0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff;
    bytes32 constant ROLE2 = 0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff;
    function eq(bytes32 x) external pure returns (bool) { return x == ROLE; }
    function ne(bytes32 x) external pure returns (bool) { return x != ROLE; }
    function constEq() external pure returns (bool) { return ROLE == ROLE2; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let role_be =
        hex::decode("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff").unwrap();
    let call1 = |m: &str, x: Vec<u8>| -> Vec<u8> {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                m,
                &[StackItem::byte_array(x)],
            )
            .expect("call");
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data
    };
    // runtime (ByteArray, big-endian) == CONSTANT (Integer-backed) for equal values.
    assert_eq!(
        call1("eq", role_be.clone()).first().copied(),
        Some(1u8),
        "x == ROLE must be true"
    );
    // != is the logical negation.
    assert_eq!(
        call1("ne", role_be.clone()).first().copied(),
        Some(0u8),
        "x != ROLE must be false"
    );
    // a DIFFERENT value must not be equal.
    let mut other = role_be.clone();
    other[0] ^= 0xff;
    assert_eq!(
        call1("eq", other).first().copied(),
        Some(0u8),
        "x == ROLE must be false for a different value"
    );
    // CONSTANT == CONSTANT (both Integer-backed) must remain correct.
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "constEq", &[])
        .expect("call");
    assert!(
        r.success && r.return_data.first().copied() == Some(1u8),
        "ROLE == ROLE2 must stay true"
    );
}

/// deep-review bytesN indexing — `b[i]` on an Integer-backed `bytesN` (a hex
/// literal / named constant) indexed the little-endian byte span, returning the
/// byte from the wrong end. Solidity `b[0]` is the MOST-significant byte. Fixed
/// by canonicalizing the constant to big-endian bytes before PICKITEM.
#[test]
fn bytesn_constant_indexing_uses_solidity_byte_order() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes32 constant ROLE = 0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff;
    function first() external pure returns (bytes1) { return ROLE[0]; }
    function last()  external pure returns (bytes1) { return ROLE[31]; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call0 = |m: &str| -> Vec<u8> {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data
    };
    // ROLE[0] is the most-significant byte 0x00; ROLE[31] is the least, 0xff.
    assert_eq!(call0("first"), vec![0x00u8], "ROLE[0] must be the MSB 0x00");
    assert_eq!(call0("last"), vec![0xffu8], "ROLE[31] must be the LSB 0xff");
}

/// deep-review bytesN bitwise — `&`/`|`/`^` between a runtime bytesN value
/// (big-endian) and an Integer-backed constant (little-endian) combined bytes
/// from opposite ends, producing a wrong mask. Fixed by canonicalizing the
/// constant operand to big-endian bytes (reducing it to the correct
/// runtime-op-runtime case).
#[test]
fn bytesn_bitwise_with_constant_mask_is_correct() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes32 constant MASK = 0x00000000000000000000000000000000000000000000000000000000000000ff;
    function withConst(bytes32 x) external pure returns (bytes32) { return x & MASK; }
    function withRuntime(bytes32 x, bytes32 m) external pure returns (bytes32) { return x & m; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let x: Vec<u8> = (0..32u8).map(|i| 0xA0u8.wrapping_add(i)).collect(); // distinct bytes, BE
    let mut mask = vec![0u8; 32];
    mask[31] = 0xff;
    let mut expected = vec![0u8; 32];
    expected[31] = x[31]; // only last byte survives
    let runc = |m: &str, args: Vec<StackItem>| -> (bool, Vec<u8>) {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &args)
            .expect("call");
        (r.success, r.return_data)
    };
    let (rs, rr) = runc(
        "withRuntime",
        vec![
            StackItem::byte_array(x.clone()),
            StackItem::byte_array(mask.clone()),
        ],
    );
    let mut rr32 = rr.clone();
    rr32.resize(32, 0);
    assert!(
        rs && rr32 == expected,
        "runtime & runtime must mask correctly: got {}",
        hex::encode(&rr32)
    );
    let (cs, cr) = runc("withConst", vec![StackItem::byte_array(x.clone())]);
    let mut cr32 = cr.clone();
    cr32.resize(32, 0);
    assert!(
        cs && cr32 == expected,
        "runtime & CONSTANT must mask correctly (not byte-reversed): got {}",
        hex::encode(&cr32)
    );
}

/// deep-review #2 (CRITICAL) — a modifier with an epilogue (e.g. a
/// `nonReentrant` guard's `locked = 0;` reset) applied to a VOID function must
/// still run the epilogue on an early `return;`. Previously the redirect was
/// only installed for functions with declared returns, so a bare `return;` in a
/// void guarded body emitted ReturnVoid directly and SKIPPED the epilogue —
/// leaving the guard engaged and bricking the contract.
#[test]
fn modifier_epilogue_runs_on_early_return_in_void_function() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 locked;
    uint256 public count;
    modifier guard() { require(locked == 0, "REENTRANT"); locked = 1; _; locked = 0; }
    function doit(bool early) external guard {
        count += 1;
        if (early) return;
        count += 10;
    }
    function lockedVal() external view returns (uint256) { return locked; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    // First call with an early return — the guard epilogue must reset `locked`.
    let r1 = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "doit",
            &[StackItem::Boolean(true)],
        )
        .expect("call doit#1");
    assert!(
        r1.success,
        "doit(true)#1 must succeed: {:?}",
        r1.exception.as_ref().map(|e| &e.message)
    );
    let lv = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "lockedVal", &[])
        .expect("call lockedVal");
    assert!(
        lv.return_data.iter().all(|&b| b == 0),
        "locked must be reset to 0 after early return"
    );
    // Second call must NOT revert on require(locked == 0) — proves the guard reset.
    let r2 = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "doit",
            &[StackItem::Boolean(true)],
        )
        .expect("call doit#2");
    assert!(
        r2.success,
        "doit(true)#2 must succeed (guard was reset), got fault: {:?}",
        r2.exception.as_ref().map(|e| &e.message)
    );
}

/// deep-review #7 — a bare hex literal implicitly returned as `bytesN` (in a
/// multi-return tuple) must be LEFT-aligned in its 32-byte ABI slot (bytesN),
/// not RIGHT-aligned (uint-style). Previously the literal was encoded as a
/// uint, corrupting the returned tuple for any ABI decoder.
#[test]
fn multi_return_bytesn_literal_is_left_aligned() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bool, bytes4) { return (true, 0x11223344); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
        .expect("call");
    assert!(
        r.success,
        "f faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.len(),
        64,
        "(bool, bytes4) ABI-encodes to 64 bytes"
    );
    // slot 0: bool true, right-aligned.
    let mut expect_bool = vec![0u8; 32];
    expect_bool[31] = 1;
    assert_eq!(&r.return_data[0..32], expect_bool.as_slice(), "bool slot");
    // slot 1: bytes4 0x11223344 LEFT-aligned (not 0x00..0011223344).
    let mut expect_b4 = vec![0u8; 32];
    expect_b4[..4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(
        &r.return_data[32..64],
        expect_b4.as_slice(),
        "bytes4 must be left-aligned"
    );
}

/// Regression for finding #7 (indexed-event variant): a bare hex-number literal
/// emitted as an `indexed bytesN` topic was ABI-encoded RIGHT-aligned (uint
/// style) by the generic static-slot encoder, producing a corrupt EVM topic.
/// `bytesN` topics must be LEFT-aligned. Verifies the fix in
/// `lower_emit_evm_shape`.
#[test]
fn indexed_event_bytesn_literal_topic_is_left_aligned() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Tag(bytes4 indexed t);
    function f() external { emit Tag(0x11223344); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
        .expect("call");
    assert!(
        r.success,
        "f faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let log = r.logs.first().expect("an event must be emitted");
    // topics[0] = keccak(signature); topics[1] = the indexed bytes4 arg.
    assert!(
        log.topics.len() >= 2,
        "expected topic0 + 1 indexed topic, got {}",
        log.topics.len()
    );
    let mut expect = vec![0u8; 32];
    expect[..4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(
        log.topics[1], expect,
        "indexed bytes4 topic must be left-aligned"
    );
}

/// A `returns (bytes memory)` method returns the byte payload directly (the
/// NeoVM ByteString), so the returned `bytes` IS the value under test.
fn run_returns_bytes(src: &str) -> Vec<u8> {
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
        .expect("call");
    assert!(
        r.success,
        "f faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    r.return_data
}

/// #7b (struct-field variant): an integer-backed `bytesN` (hex literal) as a
/// STRUCT FIELD must be ABI-encoded LEFT-aligned. For N<32 the old encoder
/// FAULTed (SIZE on an Integer stack item); for N==32 it byte-reversed the
/// literal. Both are fixed by canonicalizing the literal to its big-endian
/// ByteArray at struct construction.
#[test]
fn struct_field_bytes4_literal_is_left_aligned() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct S { bytes4 b; uint256 x; }
    function f() external pure returns (bytes memory) { return abi.encode(S(0x01020304, 1)); }
}"#;
    let inner = run_returns_bytes(src);
    assert_eq!(
        inner.len(),
        64,
        "struct (bytes4, uint256) encodes to 64 bytes"
    );
    let mut expect_b = vec![0u8; 32];
    expect_b[..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
    assert_eq!(
        &inner[0..32],
        expect_b.as_slice(),
        "bytes4 field must be left-aligned"
    );
    let mut expect_x = vec![0u8; 32];
    expect_x[31] = 1;
    assert_eq!(&inner[32..64], expect_x.as_slice(), "uint256 field");
}

#[test]
fn struct_field_bytes32_literal_is_not_reversed() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct S { bytes32 b; uint256 x; }
    function f() external pure returns (bytes memory) {
        return abi.encode(S(0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20, 1));
    }
}"#;
    let inner = run_returns_bytes(src);
    assert_eq!(inner.len(), 64);
    let expect_b: Vec<u8> = (1u8..=0x20).collect();
    assert_eq!(
        &inner[0..32],
        expect_b.as_slice(),
        "bytes32 field must be verbatim, not reversed"
    );
}

/// #7c (array-element variant): an integer-backed `bytesN` assigned to a
/// dynamic-array element must encode LEFT-aligned, not fault.
#[test]
fn array_element_bytes4_literal_is_left_aligned() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bytes memory) {
        bytes4[] memory a = new bytes4[](1);
        a[0] = 0x01020304;
        return abi.encode(a);
    }
}"#;
    let inner = run_returns_bytes(src);
    // abi.encode(bytes4[]) = [offset 0x20][count 1][elem0 left-aligned 32B]
    assert!(inner.len() >= 96, "got {}", inner.len());
    let mut expect_e = vec![0u8; 32];
    expect_e[..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
    assert_eq!(
        &inner[64..96],
        expect_e.as_slice(),
        "array bytes4 element must be left-aligned"
    );
}

/// #7d (encodePacked variant): a `bytesN` constant packed via
/// `abi.encodePacked` must emit exactly its N big-endian bytes, not the
/// little-endian integer backing (which was 8 reversed bytes).
#[test]
fn encode_packed_bytes4_constant_is_exact_width_be() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes4 constant SEL = 0x01020304;
    function f() external pure returns (bytes memory) { return abi.encodePacked(SEL); }
}"#;
    let inner = run_returns_bytes(src);
    assert_eq!(
        inner,
        vec![0x01, 0x02, 0x03, 0x04],
        "packed bytes4 must be exactly 4 BE bytes"
    );
}

/// #8 — `addmod(a, b, m)` must reduce the TRUE (up to 257-bit) sum mod m. The
/// old lowering used a native NeoVM Add, which adds the two 32-byte words as
/// signed two's-complement and discards the carry out of bit 255, so any sum
/// reaching 2^256 produced a wrong residue.
#[test]
fn addmod_reduces_true_sum_not_truncated_sum() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint256 a, uint256 b, uint256 m) external pure returns (uint256) {
        return addmod(a, b, m);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    // uint256 max = 2^256-1 — the 32-byte all-ones two's-complement word.
    let max = StackItem::byte_array(vec![0xFFu8; 32]);
    let call = |a: StackItem, b: StackItem, m: StackItem| -> Vec<u8> {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[a, b, m])
            .expect("call");
        assert!(
            r.success,
            "addmod faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data
    };
    // (2^256-1)+(2^256-1) = 2^257-2 ≡ 0 (mod 5). Truncated sum would give 4.
    let r = call(max.clone(), max.clone(), StackItem::Integer(5));
    assert!(
        r.iter().all(|&x| x == 0),
        "addmod(max,max,5) must be 0, got {r:?}"
    );
    // (2^256-1)+1 = 2^256 ≡ 2 (mod 7). Truncated sum would give 0.
    let r = call(max.clone(), StackItem::Integer(1), StackItem::Integer(7));
    assert_eq!(
        r.first().copied(),
        Some(2),
        "addmod(max,1,7) must be 2, got {r:?}"
    );
    // Non-overflowing control: 30 mod 7 == 2.
    let r = call(
        StackItem::Integer(10),
        StackItem::Integer(20),
        StackItem::Integer(7),
    );
    assert_eq!(
        r.first().copied(),
        Some(2),
        "addmod(10,20,7) must be 2, got {r:?}"
    );
    // m == 0 → Panic(0x12). Solidity >= 0.5.0 asserts the modulus is non-zero:
    // addmod/mulmod-by-zero REVERTS (division/modulo by zero), it does NOT
    // return 0 like the raw EVM ADDMOD/MULMOD opcodes. (bug-hunt #3)
    let mut rt0 = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r0 = rt0
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "f",
            &[
                StackItem::Integer(5),
                StackItem::Integer(5),
                StackItem::Integer(0),
            ],
        )
        .expect("call");
    assert!(
        !r0.success,
        "addmod(_,_,0) must Panic(0x12), not return 0 (Solidity >= 0.5 semantics)"
    );
}

/// #10 — `intN(bytesN)` must reverse big-endian↔little-endian like `uintN(bytesN)`
/// does. The signed-cast branch omitted the reversal, so `int256(bytes32(1))`
/// decoded as 2^248 and diverged from `uint256(bytes32(1))`.
#[test]
fn int_cast_of_bytesn_reverses_like_uint_cast() {
    // int256(b) must equal uint256(b) reinterpreted, and both must equal 1 for
    // b = bytes32(uint256(1)).
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function asInt(bytes32 b) external pure returns (int256) { return int256(b); }
    function asUint(bytes32 b) external pure returns (uint256) { return uint256(b); }
    function narrow(bytes4 b) external pure returns (int32) { return int32(b); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call = |m: &str, arg: StackItem| -> (bool, Vec<u8>) {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[arg])
            .expect("call");
        (r.success, r.return_data)
    };
    // bytes32(uint256(1)) big-endian = 31 zero bytes then 0x01.
    let mut b32 = vec![0u8; 32];
    b32[31] = 1;
    let (ok, rd) = call("asInt", StackItem::byte_array(b32.clone()));
    assert!(ok, "int256(bytes32) must succeed");
    assert_eq!(le_u128(&rd), 1, "int256(bytes32(1)) must be 1, not 2^248");
    let (ok, rd) = call("asUint", StackItem::byte_array(b32));
    assert!(ok);
    assert_eq!(le_u128(&rd), 1, "uint256(bytes32(1)) must be 1 (control)");
    // bytes4 0xFFFFFFFF as int32 == -1.
    let (ok, rd) = call("narrow", StackItem::byte_array(vec![0xFFu8; 4]));
    assert!(ok, "int32(bytes4) must succeed");
    // -1 little-endian minimal form is 0xFF (one byte); interpret signed.
    let signed = if rd.is_empty() {
        0i128
    } else {
        let mut v = 0i128;
        for (i, &b) in rd.iter().take(16).enumerate() {
            v |= (b as i128) << (8 * i);
        }
        // sign-extend from the top set byte
        let bits = rd.len().min(16) * 8;
        if bits < 128 && (v >> (bits - 1)) & 1 == 1 {
            v |= -1i128 << bits;
        }
        v
    };
    assert_eq!(
        signed, -1,
        "int32(bytes4(0xFFFFFFFF)) must be -1, got {rd:?}"
    );
}

/// #15 — a bare integer-backed `bytesN` literal/constant returned DIRECTLY
/// (`return 0x01020304;`) must come back as a big-endian ByteString matching the
/// manifest's `ByteArray` return type — not the little-endian Integer form.
/// (Real-node verified: pre-fix `return 0x01020304` returned Integer 16909060.)
#[test]
fn bare_bytesn_literal_return_is_bytestring() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function b4() external pure returns (bytes4) { return 0x01020304; }
    bytes4 constant SEL = 0xaabbccdd;
    function sel() external pure returns (bytes4) { return SEL; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call = |m: &str| -> Vec<u8> {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data
    };
    // Big-endian ByteString [01,02,03,04] — NOT the LE Integer form [04,03,02,01].
    assert_eq!(
        call("b4"),
        vec![0x01, 0x02, 0x03, 0x04],
        "bytes4 literal return must be BE ByteString"
    );
    assert_eq!(
        call("sel"),
        vec![0xaa, 0xbb, 0xcc, 0xdd],
        "bytes4 constant return must be BE ByteString"
    );
}

/// #14 — try/catch must keep the NeoVM TRY frame balanced: a matched
/// non-fallback catch now exits via ENDTRY (not a JMP past it), and a `return`
/// inside a catch body pops the active frame before the RET. The in-repo
/// simulator tolerates an unbalanced frame, so these assert correct BEHAVIOR
/// (value + no fault) across all three catch exit shapes — the balance fix is
/// reasoned + structural (real C# NeoVM faults at RET with a live try-stack).
#[test]
fn try_catch_frame_balanced_across_exit_shapes() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function willPanic() public pure returns (uint256) { uint256 a = 1; uint256 b = 0; return a / b; }
    function ok() public pure returns (uint256) { return 7; }
    // matched non-fallback catch with NO return (falls through, then returns).
    function fallThrough() external returns (uint256) {
        uint256 total = 0;
        try this.willPanic() returns (uint256 r) { total += r; }
        catch Panic(uint256 code) { total += code; }
        catch (bytes memory) { total += 100; }
        return total;
    }
    // matched non-fallback catch that RETURNS (return-in-catch).
    function returnInCatch() external returns (uint256) {
        try this.willPanic() returns (uint256 r) { return r; }
        catch Panic(uint256 code) { return code + 1; }
        catch (bytes memory) { return 100; }
    }
    // success path: handler returns (frame already popped on the success edge).
    function successReturns() external returns (uint256) {
        try this.ok() returns (uint256 r) { return r + 1; }
        catch { return 0; }
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .expect("C artifact");
    let call = |m: &str| -> (bool, Vec<u8>) {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        (r.success, r.return_data)
    };
    // div-by-zero → Panic 0x12 (=18). fallThrough: total += 18 → 18.
    let (ok, rd) = call("fallThrough");
    assert!(ok, "fallThrough must succeed");
    assert_eq!(
        rd.first().copied(),
        Some(18),
        "fallThrough total must be 0x12 (got {rd:?})"
    );
    // returnInCatch: code(18) + 1 = 19, returned from inside the catch.
    let (ok, rd) = call("returnInCatch");
    assert!(ok, "returnInCatch must succeed");
    assert_eq!(
        rd.first().copied(),
        Some(19),
        "returnInCatch must return code+1 (got {rd:?})"
    );
    // successReturns: ok() returns 7 → handler returns 8.
    let (ok, rd) = call("successReturns");
    assert!(ok, "successReturns must succeed");
    assert_eq!(
        rd.first().copied(),
        Some(8),
        "successReturns must return 8 (got {rd:?})"
    );
}

/// #13 — a side-effecting MODIFIER argument must be evaluated exactly ONCE.
/// Parameter substitution cloned the argument expression at every parameter
/// use, so `check(tick())` ran `tick()` once per `v` reference (counter ended
/// at 2 instead of 1).
#[test]
fn modifier_argument_evaluated_once() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counter;
    function tick() internal returns (uint256) { counter += 1; return counter; }
    modifier check(uint256 v) { require(v < 1000000, "x"); require(v < 1000000, "y"); _; }
    function run() public check(tick()) returns (uint256) { return counter; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .expect("C artifact");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "run faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.first().copied(),
        Some(1),
        "modifier arg tick() must run exactly once (counter==1); got {:?}",
        r.return_data
    );
}

/// #13b — a side-effecting BASE-CONSTRUCTOR argument must be evaluated exactly
/// ONCE. `Base(side())` ran `side()` once per use of the base parameter (p=1
/// but q=2 instead of q=1).
#[test]
fn base_constructor_argument_evaluated_once() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Base { uint256 public p; uint256 public q; constructor(uint256 v) { p = v; q = v; } }
contract Derived is Base {
    uint256 public counter;
    function side() internal returns (uint256) { counter += 1; return counter; }
    constructor() Base(side()) {}
    function pv() external view returns (uint256) { return p; }
    function qv() external view returns (uint256) { return q; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "Derived")
        .expect("Derived artifact");
    // The `_deploy` constructor auto-fires on each call (fresh rt → runs once),
    // setting p and q. With single-eval, side() runs once so p == q == 1; with
    // the bug, side() ran twice so q == 2.
    let getv = |m: &str| -> u8 {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data.first().copied().unwrap_or(0)
    };
    assert_eq!(getv("pv"), 1, "p must be 1");
    assert_eq!(
        getv("qv"),
        1,
        "q must be 1 — base ctor arg side() must run once, not per use"
    );
}

/// #12 — storage `bytes` `.push(b)` / `.pop()` previously compiled to a SILENT
/// no-op (the lowering required `ValueType::Array`, bailed for `bytes`, and the
/// fallback dropped the args and wrote nothing). They must mutate the stored
/// ByteString and revert Panic(0x31) on empty-pop.
#[test]
fn storage_bytes_push_pop_mutates_and_underflows() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes public data;
    function add(bytes1 b) external { data.push(b); }
    function popOne() external { data.pop(); }
    function len() external view returns (uint256) { return data.length; }
    function at(uint256 i) external view returns (bytes1) { return data[i]; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "C")
        .expect("C artifact");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let call = |rt: &mut NeoRuntime, m: &str, args: &[StackItem]| -> (bool, Vec<u8>) {
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, args)
            .expect("call");
        (r.success, r.return_data)
    };
    // push 0xAB → len 1, data[0] == 0xAB
    let (ok, _) = call(&mut rt, "add", &[StackItem::byte_array(vec![0xAB])]);
    assert!(ok, "push 0xAB must succeed");
    let (_, rd) = call(&mut rt, "len", &[]);
    assert_eq!(
        rd.first().copied(),
        Some(1),
        "len after one push must be 1 (was a silent no-op)"
    );
    let (_, rd) = call(&mut rt, "at", &[StackItem::Integer(0)]);
    assert_eq!(rd, vec![0xAB], "data[0] must be the pushed byte");
    // push 0xCD → len 2, data[1] == 0xCD
    call(&mut rt, "add", &[StackItem::byte_array(vec![0xCD])]);
    let (_, rd) = call(&mut rt, "len", &[]);
    assert_eq!(
        rd.first().copied(),
        Some(2),
        "len after two pushes must be 2"
    );
    let (_, rd) = call(&mut rt, "at", &[StackItem::Integer(1)]);
    assert_eq!(rd, vec![0xCD], "data[1] must be the second pushed byte");
    // pop → len 1
    let (ok, _) = call(&mut rt, "popOne", &[]);
    assert!(ok, "pop must succeed");
    let (_, rd) = call(&mut rt, "len", &[]);
    assert_eq!(rd.first().copied(), Some(1), "len after pop must be 1");
    // pop last → len 0
    call(&mut rt, "popOne", &[]);
    let (_, rd) = call(&mut rt, "len", &[]);
    assert!(
        rd.iter().all(|&x| x == 0),
        "len after popping all must be 0"
    );
    // pop on empty → Panic(0x31) revert
    let (ok, _) = call(&mut rt, "popOne", &[]);
    assert!(!ok, "pop on empty bytes must revert Panic(0x31)");
}

/// #11 — a single-arg interface method named `transfer`/`send` (e.g.
/// `IToken(t).transfer(100)`) was hijacked into a GAS native value transfer
/// because interface handles infer as `Address`. It must lower to a
/// cross-contract CALL of the interface method instead. A plain
/// `payable(t).send(x)` must STILL be a GAS transfer.
#[test]
fn interface_send_is_contract_call_not_gas_native_send() {
    const GAS_LE: [u8; 20] = [
        0xcf, 0x76, 0xe2, 0x8b, 0xd0, 0x06, 0x2c, 0x4a, 0x47, 0x8e, 0xe3, 0x55, 0x61, 0x01, 0x13,
        0x19, 0xf3, 0xcf, 0xa4, 0xd2,
    ];
    let references_gas = |src: &str| -> bool {
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = arts
            .iter()
            .find(|a| a.metadata.name == "C")
            .expect("C artifact");
        art.bytecode.windows(20).any(|w| w == GAS_LE) || art.tokens.iter().any(|t| t.hash == GAS_LE)
    };
    let interface_form = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
interface IMail { function send(uint256 x) external returns (bool); }
contract C { function run(address t) external returns (bool) { return IMail(t).send(100); } }"#;
    let plain_form = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function run(address t) external returns (bool) { return payable(t).send(100); } }"#;
    assert!(
        !references_gas(interface_form),
        "IMail(t).send(x) must be a cross-contract call, not a GAS native transfer"
    );
    assert!(
        references_gas(plain_form),
        "payable(t).send(x) must remain a GAS native transfer (control)"
    );
}

/// Read `return_data` as a little-endian unsigned integer (for moderate values).
fn le_u128(rd: &[u8]) -> u128 {
    let mut v = 0u128;
    for (i, &b) in rd.iter().take(16).enumerate() {
        v |= (b as u128) << (8 * i);
    }
    v
}

/// #9 (int256 `**`): previously `int256` exponentiation fell through the
/// type-gating with NO overflow check (checked) and NO mod-2^256 wrap
/// (unchecked) — only uint256 and narrow widths were handled. Checked overflow
/// must Panic(0x11); unchecked must wrap; in-range powers must be exact.
#[test]
fn int256_pow_checked_panics_unchecked_wraps_inrange_exact() {
    let checked = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f(int256 a, uint256 b) external pure returns (int256) { return a ** b; } }"#;
    let unchecked = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f(int256 a, uint256 b) external pure returns (int256) { unchecked { return a ** b; } } }"#;
    let run = |src: &str, a: i64, b: i64| -> (bool, Vec<u8>) {
        let arts = compile_contracts(src, false, 2).expect("compile");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "f",
                &[StackItem::Integer(a), StackItem::Integer(b)],
            )
            .expect("call");
        (r.success, r.return_data)
    };
    // in-range positive: 2**10 = 1024
    let (ok, rd) = run(checked, 2, 10);
    assert!(ok, "2**10 must succeed");
    assert_eq!(le_u128(&rd), 1024, "2**10 == 1024");
    // in-range negative base: (-2)**3 = -8 (must not fault)
    let (ok, _) = run(checked, -2, 3);
    assert!(ok, "(-2)**3 must succeed");
    // checked overflow: 2**255 > int256 max (2^255-1) -> Panic
    let (ok, _) = run(checked, 2, 255);
    assert!(
        !ok,
        "checked int256 2**255 must Panic (overflow), not return"
    );
    // unchecked overflow: 2**255 wraps to int256.min, must NOT fault
    let (ok, _) = run(unchecked, 2, 255);
    assert!(ok, "unchecked int256 2**255 must wrap without faulting");
}

/// #16b — int256 `**` via the unsigned-magnitude approach: exponentiate |base|
/// with the soft-arith mul, then re-apply the sign. Validates negative bases
/// (even/odd exponents), the unchecked overflow wrap, and the int256.min
/// boundary (`(-2)**255 == type(int256).min`, in range — must NOT Panic).
#[test]
fn int256_pow_magnitude_sign_and_boundary() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function neg3()   external pure returns (bool) { int256 b = -2; return b ** 3 == -8; }
    function neg4()   external pure returns (bool) { int256 b = -2; return b ** 4 == 16; }
    function negOdd() external pure returns (bool) { int256 b = -3; return b ** 5 == -243; }
    function uWrap()  external pure returns (bool) { unchecked { int256 b = 2; return b ** 255 == type(int256).min; } }
    function minPow() external pure returns (bool) { int256 b = -2; return b ** 255 == type(int256).min; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call = |m: &str| -> bool {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data.first().copied() == Some(1)
    };
    assert!(call("neg3"), "(-2)**3 == -8");
    assert!(call("neg4"), "(-2)**4 == 16");
    assert!(call("negOdd"), "(-3)**5 == -243");
    assert!(call("uWrap"), "unchecked 2**255 wraps to int256.min");
    assert!(
        call("minPow"),
        "(-2)**255 == int256.min (in range, no panic)"
    );
}

/// #16 — uint256 `**` now uses the soft-arith 256-bit multiply in the loop so
/// an overflowing intermediate never materializes a >32-byte integer (which
/// faults uncatchably on a real node). Validates the in-range result and the
/// unchecked mod-2^256 wrap (the catchable-Panic-vs-fault distinction is only
/// observable on a real node — see examples/test_neoxp_arith_smoke.sh).
#[test]
fn uint256_pow_soft_mul_in_range_and_unchecked_wrap() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function p10() external pure returns (uint256) { uint256 b = 2; return b ** 10; }
    function wrap() external pure returns (uint256) { unchecked { uint256 b = 2; return b ** 256; } }
    function p1e18() external pure returns (uint256) { uint256 b = 10; return b ** 18; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call = |m: &str| -> Vec<u8> {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        r.return_data
    };
    assert_eq!(
        le_u128(&call("p10")),
        1024,
        "2**10 == 1024 via soft-arith mul"
    );
    assert!(
        call("wrap").iter().all(|&x| x == 0),
        "unchecked 2**256 wraps to 0"
    );
    assert_eq!(le_u128(&call("p1e18")), 1_000_000_000_000_000_000, "10**18");
}

/// #9b (square-and-multiply restructure): the squaring-skip fix that avoids the
/// overflowing final squaring must not change any result value.
#[test]
fn pow_values_preserved_after_squaring_skip() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C { function f(uint256 b, uint256 e) external pure returns (uint256) { return b ** e; } }"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let run = |b: i64, e: i64| -> u128 {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "f",
                &[StackItem::Integer(b), StackItem::Integer(e)],
            )
            .expect("call");
        assert!(
            r.success,
            "{b}**{e} faulted: {:?}",
            r.exception.as_ref().map(|x| &x.message)
        );
        le_u128(&r.return_data)
    };
    assert_eq!(run(2, 0), 1, "b**0 == 1");
    assert_eq!(run(7, 1), 7, "b**1 == b");
    assert_eq!(run(2, 10), 1024);
    assert_eq!(run(3, 5), 243);
    assert_eq!(run(2, 64), 1u128 << 64, "2**64");
    assert_eq!(
        run(10, 18),
        10u128.pow(18),
        "10**18 (1e18, common token scale)"
    );
}

/// Regression: a VOID native call via the dynamic `System.Contract.Call` path
/// (e.g. `NativeCalls.updateContract`) must NOT emit a `PUSH1` placeholder
/// after the syscall. Neo's dynamic call ALWAYS leaves exactly one item on the
/// caller's stack (a `Null` for a void callee), which the statement-position
/// `DROP` already balances. A spurious `PUSH1` leaves a second item that the
/// strict real-node return-count check rejects ("Return value count mismatch:
/// expected 0, but got 1") — the lenient in-tree simulator masks this, so the
/// guard is a bytecode-shape assertion, not a runtime call.
///
/// (The CALLT path is genuinely different: the method token's `has_return_value`
/// flag sets RVCount=0 for a void method, so there a `PUSH1` placeholder IS
/// required. Default CLI optimization does not use CALLT.)
#[test]
fn void_native_call_default_path_has_no_push1_placeholder() {
    use neo_devpack_solidity::cli::disassemble_neovm_bytecode;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract U {
    function upd(bytes calldata nef, string calldata manifest) public {
        NativeCalls.updateContract(nef, manifest, "");
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let asm = disassemble_neovm_bytecode(&arts[0].bytecode);
    let lines: Vec<&str> = asm.lines().collect();
    let i = lines
        .iter()
        .position(|l| l.contains("SYSCALL System.Contract.Call"))
        .unwrap_or_else(|| panic!("no System.Contract.Call in disasm:\n{asm}"));
    // The instruction immediately after the void call must be the statement
    // DROP, never a PUSH1 placeholder.
    let next = lines.get(i + 1).copied().unwrap_or("");
    assert!(
        next.contains("DROP"),
        "void native call must be followed directly by DROP (no PUSH1 placeholder), got:\n  {}\n  {}\nfull:\n{asm}",
        lines[i], next
    );
}

/// Regression (type-strictness audit, root cause A): a ternary `bytesN`
/// operand compared to a hex-number literal must canonicalize the literal to a
/// full-width big-endian ByteString (`PUSHDATA1 len=32`), NOT leave it as an
/// Integer (`PUSHINT`). Before the fix, `infer_type_from_expression` had no
/// `ConditionalOperator` arm, so `(c ? a : b)` inferred `None`, the
/// canonicalization was skipped, and the bytecode emitted a type-strict
/// `EQUAL(ByteString[32], Integer)` that is always false on a real node
/// (defeating sentinel/role `!=` guards). The simulator's lenient EQUAL masks
/// it, so this is a bytecode-shape assertion.
#[test]
fn ternary_bytesn_eq_hex_literal_canonicalizes_to_bytestring() {
    use neo_devpack_solidity::cli::disassemble_neovm_bytecode;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function f(bool c, bytes32 a, bytes32 b) external pure returns (bool) {
        return (c ? a : b) == 0x00000000000000000000000000000000000000000000000000000000000000ff;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let asm = disassemble_neovm_bytecode(&arts[0].bytecode);
    assert!(
        asm.contains("PUSHDATA1 len=32"),
        "ternary bytesN==hex literal must push the literal as a 32-byte ByteString \
         (PUSHDATA1 len=32), not an Integer; disasm:\n{asm}"
    );
}

/// Regression (type-strictness audit, root cause B): `abi.encodePacked`/
/// `abi.encode` results are built with NeoVM `CAT`, which yields a *Buffer*
/// (0x30). Comparing that with `==` against a ByteString literal — or returning
/// it where the manifest declares `ByteArray` — mis-behaves on a real node
/// (`EQUAL` is type-strict). The `BytesConcat` lowering now normalizes the
/// result to a ByteString (`CONVERT 0x28`) at its single chokepoint. Bytecode-
/// shape assertion: a `CONVERT` follows the `CAT` chain.
#[test]
fn abi_encode_packed_result_is_converted_to_bytestring() {
    use neo_devpack_solidity::cli::disassemble_neovm_bytecode;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function f() external pure returns (bool) {
        return abi.encodePacked(uint8(1), uint8(2)) == hex"0102";
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let asm = disassemble_neovm_bytecode(&arts[0].bytecode);
    let lines: Vec<&str> = asm.lines().collect();
    let last_cat = lines.iter().rposition(|l| l.contains("CAT"));
    assert!(
        last_cat.is_some(),
        "expected a CAT in abi.encodePacked lowering:\n{asm}"
    );
    let after_cat = &lines[last_cat.unwrap()..];
    assert!(
        after_cat.iter().any(|l| l.contains("CONVERT")),
        "abi.encodePacked result (CAT Buffer) must be CONVERTed to ByteString before use; disasm:\n{asm}"
    );
}

/// Regression (type-strictness audit, root causes D + E): a byte index `b[i]`
/// of a `bytes`/`bytesN` is a Solidity `bytes1`, but lowers to ArrayGet/PICKITEM
/// — a NeoVM Integer (the byte value). Because NeoVM `EQUAL` is type-strict,
/// leaving it an Integer made `b[i] == bytes1(x)` / `!=`, `return b[i]`, and
/// assign-then-compare all wrong on a real node (the lenient simulator masks
/// it). `try_lower_expression_primary` now coerces a byte index to a 1-byte
/// ByteString AT THE SOURCE (`Convert ByteArray` + the `bytes1(uintN)` cast
/// helper `coerce_to_fixed_bytes`, correct for all byte values incl. 0x00 and
/// ≥0x80), so every consumer sees the right type. Bytecode-shape assertion: a
/// byte-index `return` is coerced (NEWBUFFER + CONVERT), not a bare PICKITEM→RET.
#[test]
fn return_byte_index_as_bytes1_is_coerced_to_bytestring() {
    use neo_devpack_solidity::cli::disassemble_neovm_bytecode;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function f(bytes32 b) external pure returns (bytes1) { return b[0]; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let asm = disassemble_neovm_bytecode(&arts[0].bytecode);
    assert!(
        asm.contains("NEWBUFFER") && asm.contains("CONVERT"),
        "return of a byte index as bytes1 must be coerced to a ByteString \
         (NEWBUFFER + CONVERT from the fixed-byte coercion), not returned as a raw \
         PICKITEM Integer; disasm:\n{asm}"
    );
}

/// Control: a `bytesN` local assigned from a hex literal, then `abi.encode`d.
/// Confirms whether local-variable binding already canonicalizes (informs the
/// scope of the fix).
#[test]
fn local_bytes4_literal_abi_encode_is_left_aligned() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() external pure returns (bytes memory) {
        bytes4 x = 0x01020304;
        return abi.encode(x);
    }
}"#;
    let inner = run_returns_bytes(src);
    assert_eq!(inner.len(), 32, "abi.encode(bytes4) is one 32-byte slot");
    let mut expect = vec![0u8; 32];
    expect[..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
    assert_eq!(inner, expect, "local bytes4 must be left-aligned");
}

/// Regression (fuzz, runtime_exec OOM / DoS): a bytecode that stacks `TRY`
/// opcodes without a matching `ENDTRY` must NOT grow the try-stack without
/// bound. NeoVM caps try-nesting at `MaxTryNestingDepth = 16`; before the cap,
/// the `runtime_exec` libFuzzer target drove a 155-byte input of stacked TRYs
/// to a 1.4-second execution that OOM'd under a 100k-gas budget — a host-level
/// DoS for any operator running compiled contracts. The VM must now fault
/// quickly at depth 16 instead of hanging / exhausting memory.
#[test]
fn deeply_nested_try_faults_not_ooms() {
    // 64 back-to-back `TRY catch=+3 finally=0` (3 bytes each), no ENDTRY.
    let mut bytecode = Vec::with_capacity(64 * 3);
    for _ in 0..64 {
        bytecode.push(0x3B); // OpCode::TRY
        bytecode.push(0x03); // catchOffset = +3 (forward, non-zero)
        bytecode.push(0x00); // finallyOffset = 0
    }
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    // Must return promptly (the 17th TRY exceeds the nesting cap and faults);
    // a pre-fix build would spend ~1.4s growing the try-stack and could OOM.
    let result = rt.execute(&bytecode, &[]);
    // A graceful runtime error is also acceptable; only a reported SUCCESS is wrong.
    if let Ok(r) = result {
        assert!(
            !r.success,
            "deeply nested TRY must fault (MaxTryNestingDepth), not succeed"
        );
    }
}

/// Regression (fuzz, runtime_exec): unbounded `CALL` recursion must fault at the
/// call-stack limit (NeoVM MaxInvocationStackSize) as an UNCATCHABLE VM fault,
/// not hang/OOM, and not be routed into an enclosing TRY (which would let
/// bytecode build a fault-storm). `CALL +0` calls itself.
#[test]
fn infinite_call_recursion_faults_not_hangs() {
    let bytecode = vec![0x34u8, 0x00]; // OpCode::CALL, relative offset 0 (self)
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let result = rt.execute(&bytecode, &[]);
    // A graceful runtime error is also acceptable; only a reported SUCCESS is wrong.
    if let Ok(r) = result {
        assert!(!r.success, "infinite recursion must fault, not succeed");
    }
}

/// Regression (dev-env / neo-test): a VOID library function invoked via a
/// `using ... for` directive as a STATEMENT (`x.check(4);`) must not emit a
/// trailing DROP against an empty stack. Before the fix the using-for receiver
/// path in member_calls.rs returned `Some(true)` for a void library function,
/// so the statement-position lowering dropped a value that was never pushed →
/// a "Stack underflow" VM fault. (Found by the `neo-test` runner exercising an
/// `assertEq`-style helper called as `x.assertEq(4)`.)
#[test]
fn using_for_void_library_call_statement_no_stack_underflow() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
library L { function check(uint256 a, uint256 b) internal pure { require(a == b, "ne"); } }
contract C {
    using L for uint256;
    function ok() public pure returns (bool) { uint256 x = 4; x.check(4); return true; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &arts[0].bytecode,
            &arts[0].tokens,
            &arts[0].manifest,
            "ok",
            &[],
        )
        .expect("call");
    assert!(
        r.success,
        "void using-for library call as a statement must not underflow: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
}

/// Regression (cross-contract `msg.sender` for `new`-deployed contracts):
/// a contract you `new` and then call must observe the CALLING contract's
/// `address(this)` as its `msg.sender` — not a synthetic hash, and not
/// `tx.origin`. This is what makes the canonical test-runner pattern
/// `B b = new B(); b.deposit();` (where `deposit` does `bal[msg.sender] += 1`)
/// credit the caller, so a later `b.balanceOf(address(this))` reads the same
/// slot. The fix splits the entry-script hash (`GetEntryScriptHash`) from the
/// executing hash (`GetExecutingScriptHash` / `address(this)`) so the
/// `msg.sender` lowering's `CallingScriptHash == EntryScriptHash` top-level
/// short-circuit no longer misfires on an inner contract-to-contract hop —
/// while `tx.origin` (`Transaction.Sender`) stays pinned to the entry caller.
#[test]
fn new_deployed_callee_sees_caller_address_this_as_msg_sender() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Probe {
    function ctx() external view returns (address, address) {
        return (msg.sender, tx.origin);
    }
}
contract Host {
    // forward routes through the 20-byte zero-placeholder self-offsets path.
    function forward(address c) external view returns (bool) {
        (address s, address o) = Probe(c).ctx();
        // msg.sender inside the callee == THIS caller's address(this), and it
        // diverges from tx.origin (the entry signer).
        return s == address(this) && s != o;
    }
    // Direct entry call: msg.sender and tx.origin coincide (no inner hop).
    function directEntry() external view returns (bool) {
        return msg.sender == tx.origin;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let host = arts
        .iter()
        .find(|a| a.metadata.name == "Host")
        .expect("Host artifact");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

    // Nested: forward(0x00..00) → Probe(zero).ctx() via self-offsets dispatch.
    let zero = [0u8; 20];
    let r = rt
        .call_method(
            &host.bytecode,
            &host.tokens,
            &host.manifest,
            "forward",
            &[StackItem::byte_array(zero.to_vec())],
        )
        .expect("forward host-level");
    assert!(
        r.success,
        "forward must succeed: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert!(
        r.return_data.iter().any(|&b| b != 0),
        "nested callee msg.sender must equal the caller's address(this) AND \
         differ from tx.origin; forward returned false (rd={:?})",
        r.return_data
    );

    // Direct: msg.sender == tx.origin at the genuine entry frame.
    let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).expect("rt2");
    let r2 = rt2
        .call_method(
            &host.bytecode,
            &host.tokens,
            &host.manifest,
            "directEntry",
            &[],
        )
        .expect("directEntry host-level");
    assert!(
        r2.success,
        "directEntry must succeed: {:?}",
        r2.exception.as_ref().map(|e| &e.message)
    );
    assert!(
        r2.return_data.iter().any(|&b| b != 0),
        "direct entry: msg.sender must equal tx.origin (rd={:?})",
        r2.return_data
    );
}

/// Regression (partial tuple destructure from a cross-contract call): binding
/// a subset of a multi-return external call — `(uint256 a, ) = c.f()` or
/// `( , uint256 b) = c.f()` — must select the correct ABI slot. The
/// `this`/external multi-return decode path keyed its slot types off the LHS
/// tuple parameters; a SKIPPED slot (`,`) has no LHS parameter, so the
/// `param.as_ref()?` short-circuited the whole `collect()` to `None`, skipping
/// the ABI-decode and letting the legacy `PICKITEM`-on-a-ByteString path read
/// the ABI buffer as an Array (garbage). Skipped slots are now represented as
/// `None` entries and left undecoded; each kept static slot is decoded at its
/// own `index * 32` head offset. Full destructure already worked.
#[test]
fn partial_tuple_destructure_from_cross_contract_call_selects_right_slot() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Probe {
    function pair() external pure returns (uint256, uint256) { return (11, 22); }
}
contract Host {
    function firstOf(address c) external view returns (uint256) {
        (uint256 a, ) = Probe(c).pair();
        return a;
    }
    function secondOf(address c) external view returns (uint256) {
        ( , uint256 b) = Probe(c).pair();
        return b;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let host = arts
        .iter()
        .find(|a| a.metadata.name == "Host")
        .expect("Host artifact");
    let zero = [0u8; 20];
    // Host-level uint256 returns come back as raw little-endian NeoVM integer
    // bytes (low byte first), e.g. 11 -> [0x0B, 0x00, ...]. Decode LE.
    let le = |rd: &[u8]| {
        rd.iter()
            .take(16)
            .enumerate()
            .fold(0u128, |acc, (i, &b)| acc + ((b as u128) << (8 * i)))
    };

    for (method, want) in [("firstOf", 11u128), ("secondOf", 22u128)] {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &host.bytecode,
                &host.tokens,
                &host.manifest,
                method,
                &[StackItem::byte_array(zero.to_vec())],
            )
            .unwrap_or_else(|e| panic!("{method} call: {e:?}"));
        assert!(
            r.success,
            "{method} must succeed: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        assert_eq!(
            le(&r.return_data),
            want,
            "{method}: partial destructure bound the wrong cross-contract \
             return slot (rd={:?})",
            r.return_data
        );
    }
}

/// Regression (neo-test cheatcodes): Foundry-style `vm.*` cheats are serviced
/// by the runtime when a contract calls the well-known HEVM cheatcode address
/// (0x7109709ECfa91a80626fF3989D68f67F5b1DD12D). `vm.prank` rewrites the next
/// call's msg.sender; `vm.warp`/`vm.roll` set block.timestamp/number;
/// `vm.deal` sets a GAS balance. Validated end-to-end: a host contract drives a
/// `new`-deployed Target and observes the cheat effects.
#[test]
fn cheatcodes_prank_warp_roll_deal() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
interface Vm {
    function warp(uint256) external;
    function roll(uint256) external;
    function prank(address) external;
    function deal(address, uint256) external;
}
contract Target {
    function ping() external view returns (address) { return msg.sender; }
    function ts() external view returns (uint256) { return block.timestamp; }
    function bn() external view returns (uint256) { return block.number; }
    function bal(address a) external view returns (uint256) { return a.balance; }
}
contract Host {
    Vm constant vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
    Target t;
    constructor() { t = new Target(); }
    function prankOk(address who) external returns (bool) {
        vm.prank(who);
        return t.ping() == who;
    }
    function warpOk(uint256 s) external returns (bool) {
        vm.warp(s);
        return t.ts() == s;
    }
    function rollOk(uint256 n) external returns (bool) {
        vm.roll(n);
        return t.bn() == n;
    }
    function dealOk(address who, uint256 amt) external returns (bool) {
        vm.deal(who, amt);
        return t.bal(who) == amt;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let host = arts
        .iter()
        .find(|a| a.metadata.name == "Host")
        .expect("Host artifact");
    let alice = vec![0x11u8; 20];
    let truthy = |rd: &[u8]| rd.iter().any(|&b| b != 0);

    let cases: Vec<(&str, Vec<StackItem>)> = vec![
        ("prankOk", vec![StackItem::byte_array(alice.clone())]),
        ("warpOk", vec![StackItem::Integer(123_456)]),
        ("rollOk", vec![StackItem::Integer(999)]),
        (
            "dealOk",
            vec![
                StackItem::byte_array(alice.clone()),
                StackItem::Integer(5_000_000),
            ],
        ),
    ];
    for (method, args) in cases {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&host.bytecode, &host.tokens, &host.manifest, method, &args)
            .unwrap_or_else(|e| panic!("{method}: {e:?}"));
        assert!(
            r.success,
            "{method} must succeed: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        assert!(
            truthy(&r.return_data),
            "{method}: cheatcode effect not observed (rd={:?})",
            r.return_data
        );
    }
}

/// Regression (loop / mapping / dynamic-array codegen): these constructs are
/// now generated by the `fuzz_target_structured_sol` grammar, so pin their
/// runtime correctness (the fuzzer only catches panics, not miscompiles). A
/// `for` loop counts; a `while` loop with a monotonic guard terminates; a
/// mapping round-trips; and `push`/index/`pop`/`length` on a dynamic array
/// agree.
#[test]
fn loop_mapping_array_codegen_executes_correctly() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 state;
    mapping(uint256 => uint256) sm;
    uint256[] sa;
    function forSum() external returns (uint256) {
        for (uint256 i = 0; i < 4; i++) { state += 1; }
        return state;
    }
    function whileSum() external returns (uint256) {
        while (state < 3) { state += 1; }
        return state;
    }
    function mapRoundtrip(uint256 k, uint256 v) external returns (uint256) {
        sm[k & 0xff] = v & 0xff;
        return sm[k & 0xff];
    }
    function arrPushPop() external returns (uint256) {
        sa.push(7); sa.push(9);
        uint256 last = sa[sa.length - 1];
        sa.pop();
        return last + sa.length;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let be = |rd: &[u8]| {
        rd.iter()
            .take(16)
            .enumerate()
            .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)))
    };
    let cases: Vec<(&str, Vec<StackItem>, u128)> = vec![
        ("forSum", vec![], 4),
        ("whileSum", vec![], 3),
        (
            "mapRoundtrip",
            vec![StackItem::Integer(300), StackItem::Integer(500)],
            500 & 0xff,
        ),
        ("arrPushPop", vec![], 10),
    ];
    for (method, args, want) in cases {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, method, &args)
            .unwrap_or_else(|e| panic!("{method}: {e:?}"));
        assert!(
            r.success,
            "{method} must succeed: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        assert_eq!(
            be(&r.return_data),
            want,
            "{method}: wrong result (rd={:?})",
            r.return_data
        );
    }
}

/// Regression (neo-test `vm.expectRevert`): arming the cheat makes the next
/// cross-contract call's revert be SWALLOWED (execution continues) and rolls
/// back that call's storage; if the guarded call does NOT revert, it's a
/// violation that faults. The hooks live in the exception path
/// (try_frames.rs::try_satisfy_expect_revert) + return_from_function, gated on
/// a per-frame guard that is `None` for every non-expectRevert call.
#[test]
fn expect_revert_swallows_and_continues_else_faults() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
interface Vm { function expectRevert() external; }
contract Target {
    uint256 public x;
    function boom() external { require(false, "kaboom"); }
    function ok() external returns (uint256) { x = 7; return 7; }
}
contract Host {
    Vm constant vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
    Target t;
    constructor() { t = new Target(); }
    function expectAndContinue() external returns (uint256) {
        vm.expectRevert();
        t.boom();            // reverts -> swallowed by expectRevert
        return t.ok();       // execution continues -> 7
    }
    function expectButNoRevert() external returns (uint256) {
        vm.expectRevert();
        return t.ok();       // does NOT revert -> violation -> fault
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let host = arts
        .iter()
        .find(|a| a.metadata.name == "Host")
        .expect("Host");
    let be = |rd: &[u8]| {
        rd.iter()
            .take(16)
            .enumerate()
            .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)))
    };

    // expectAndContinue: succeeds, returns 7 (continued past the swallow).
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &host.bytecode,
            &host.tokens,
            &host.manifest,
            "expectAndContinue",
            &[],
        )
        .expect("expectAndContinue");
    assert!(
        r.success,
        "expectAndContinue must succeed (revert swallowed): {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        be(&r.return_data),
        7,
        "must continue and return 7 (rd={:?})",
        r.return_data
    );

    // expectButNoRevert: the guarded call did not revert -> violation -> fault.
    let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).expect("rt2");
    let r2 = rt2
        .call_method(
            &host.bytecode,
            &host.tokens,
            &host.manifest,
            "expectButNoRevert",
            &[],
        )
        .expect("expectButNoRevert host-level");
    assert!(
        !r2.success,
        "expectButNoRevert must FAIL: a guarded call that does not revert is a violation"
    );
}

/// Regression (overload resolution + address member inference): an
/// `<address>.balance` argument must infer to `uint256` so a same-arity
/// overload set resolves to the uint256 candidate. Previously `.balance`
/// inferred to `None`, so `resolve_overload([None, …])` matched nothing and
/// the call trapped at runtime with "no compiled body". (Surfaced by neo-test's
/// `assertEq(addr.balance, x, "msg")` against the value-rich assertion set.)
#[test]
fn address_balance_arg_resolves_same_arity_overload() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    // Sentinel returns make the result independent of the actual balance.
    function pick(uint256 x) internal pure returns (uint256) { x; return 7; }
    function pick(address a) internal pure returns (uint256) { a; return 100; }
    function run() external view returns (uint256) {
        return pick(address(this).balance);   // uint256 arg -> pick(uint256) -> 7
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("run");
    assert!(
        r.success,
        "run() must succeed (no 'no compiled body' trap): {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(
        v, 7,
        "must resolve to pick(uint256) (sentinel 7), not pick(address) (100); got {v}"
    );
}

/// Regression (bug-hunt #1): a `mapping(K => uintN)` value read (N < 256) must
/// carry its declared width into arithmetic — checked overflow Panics(0x11),
/// unchecked wraps mod 2^N. Previously the mapping-index-access type inferred
/// to `None`, so the operand was treated as full-width and BOTH the checked
/// guard and the unchecked truncation were skipped (silent overflow).
#[test]
fn mapping_narrow_value_arithmetic_carries_width() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    mapping(uint256 => uint8) m;
    function checkedOverflow() external returns (uint256) { m[1] = 255; return m[1] + 1; }
    function uncheckedWrap() external returns (uint256) { m[1] = 255; unchecked { return m[1] + 1; } }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "checkedOverflow",
            &[],
        )
        .expect("call");
    assert!(
        !r.success,
        "checked mapping uint8 255+1 must Panic(0x11), not silently yield 256"
    );
    let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).expect("rt2");
    let r2 = rt2
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "uncheckedWrap",
            &[],
        )
        .expect("call2");
    assert!(
        r2.success,
        "unchecked wrap must not fault: {:?}",
        r2.exception.as_ref().map(|e| &e.message)
    );
    assert!(
        r2.return_data.iter().all(|&b| b == 0),
        "unchecked uint8 255+1 must wrap to 0 (rd={:?})",
        r2.return_data
    );
}

/// Regression (bug-hunt #5/#7): a `bytesN` hex literal assigned to a `bytesN`
/// STATE variable must be canonicalized to its big-endian ByteString before
/// StoreState. Previously the hex literal lowered to a little-endian Integer
/// and was persisted byte-reversed, so `uint256(v)` / byte indexing read back a
/// corrupted value.
#[test]
fn bytesn_literal_to_storage_is_big_endian() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct S { bytes32 h; }
    bytes32 b32;
    bytes4 b4;
    S s;
    bytes32[] arr;
    function set32() external returns (uint256) {
        b32 = 0x00000000000000000000000000000000000000000000000000000000000000aa;
        return uint256(b32);
    }
    function set4hi() external returns (uint256) {
        b4 = 0xaabbccdd;
        return uint32(b4);
    }
    function setStructField() external returns (uint256) {
        s.h = 0x00000000000000000000000000000000000000000000000000000000000000aa;
        return uint256(s.h);
    }
    function pushArr() external returns (uint256) {
        arr.push(0x0000000000000000000000000000000000000000000000000000000000000005);
        return uint256(arr[0]);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let be = |rd: &[u8]| {
        rd.iter()
            .take(16)
            .enumerate()
            .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)))
    };
    let call = |method: &str| -> (bool, u128) {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, method, &[])
            .expect("call");
        (r.success, be(&r.return_data))
    };
    // scalar state var, struct field, and dynamic-array element bytesN literals
    // all persist big-endian (read back as their face value, not byte-reversed).
    for (m, want) in [
        ("set32", 0xaau128),
        ("set4hi", 0xaabbccdd),
        ("setStructField", 0xaa),
        ("pushArr", 5),
    ] {
        let (ok, v) = call(m);
        assert!(ok, "{m} must succeed");
        assert_eq!(
            v, want,
            "{m}: bytesN storage literal not big-endian; got {v:#x} want {want:#x}"
        );
    }
}

/// Regression (bug-hunt #3): `addmod`/`mulmod` with a ZERO modulus REVERT with
/// Panic(0x12) (division/modulo by zero) per Solidity >= 0.5.0 — they do NOT
/// return 0 like the raw EVM opcodes. A non-zero modulus (including full-width
/// 2^256-1 operands) still computes the correct residue.
#[test]
fn addmod_mulmod_by_zero_panics_else_computes() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function am(uint256 a, uint256 b, uint256 m) external pure returns (uint256) { return addmod(a, b, m); }
    function mm(uint256 a, uint256 b, uint256 m) external pure returns (uint256) { return mulmod(a, b, m); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let max = || StackItem::byte_array(vec![0xFFu8; 32]);
    let run = |method: &str, a: StackItem, b: StackItem, m: StackItem| {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        rt.call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            method,
            &[a, b, m],
        )
        .expect("call")
    };
    // Zero modulus -> Panic (both addmod and mulmod).
    for method in ["am", "mm"] {
        let r = run(
            method,
            StackItem::Integer(5),
            StackItem::Integer(5),
            StackItem::Integer(0),
        );
        assert!(!r.success, "{method}(_,_,0) must Panic(0x12), not return 0");
    }
    // Non-zero modulus still computes, including full-width operands:
    // addmod(2^256-1, 2^256-1, 5) = (2^257-2) % 5 = 0.
    let r = run("am", max(), max(), StackItem::Integer(5));
    assert!(
        r.success && r.return_data.iter().all(|&x| x == 0),
        "addmod(max,max,5) must be 0 (success={}, rd={:?})",
        r.success,
        r.return_data
    );
    // mulmod(2^256-1, 2^256-1, 7): 2^256 ≡ 2 (mod 7) so (2^256-1) ≡ 1 (mod 7),
    // and 1*1 ≡ 1 (mod 7).
    let r = run("mm", max(), max(), StackItem::Integer(7));
    assert!(
        r.success,
        "mulmod(max,max,7) must succeed: rd={:?}",
        r.return_data
    );
    assert_eq!(
        r.return_data.first().copied(),
        Some(1),
        "mulmod(max,max,7) must be 1, got {:?}",
        r.return_data
    );
}

/// Regression (bug-hunt #4/#24): a positive integer LITERAL arg is convertible
/// to any integer type, so `f(intVar, 3)` must resolve to the `int` overload
/// (and `f(uintVar, 3)` to the `uint` overload) rather than leaving
/// `[int256, uint256]` — which matched NEITHER same-arity overload and trapped
/// at runtime with "no compiled body". Fix: resolve_overload treats an integer
/// literal as matching any integer param; non-literal args still disambiguate.
#[test]
fn overload_resolution_integer_literal_matches_any_int() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function g(uint256 a, uint256 b) internal pure returns (uint256) { return a + b; }
    function g(int256 a, int256 b) internal pure returns (int256)  { return a - b; }
    function runInt() external pure returns (int256)  { int256 x = 10;  int256 r = g(x, 3);  return r; }
    function runUint() external pure returns (uint256) { uint256 y = 10; uint256 r = g(y, 3); return r; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let le = |rd: &[u8]| {
        rd.iter()
            .take(16)
            .enumerate()
            .fold(0i128, |a, (i, &b)| a + ((b as i128) << (8 * i)))
    };
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let ri = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "runInt", &[])
        .expect("runInt");
    assert!(
        ri.success,
        "runInt must not trap 'no compiled body': {:?}",
        ri.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        le(&ri.return_data),
        7,
        "g(int x=10, 3) must pick int overload -> 10-3=7 (rd={:?})",
        ri.return_data
    );
    let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).expect("rt2");
    let ru = rt2
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "runUint", &[])
        .expect("runUint");
    assert!(
        ru.success,
        "runUint must not trap: {:?}",
        ru.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        le(&ru.return_data),
        13,
        "g(uint y=10, 3) must pick uint overload -> 10+3=13 (rd={:?})",
        ru.return_data
    );
}

/// Regression (bug-hunt #22): a `bytesN` CONSTANT initialized with an implicit
/// (un-cast) hex literal must fold to its big-endian ByteString, so
/// `uint16(B2)` / `uint32(B4)` read the face value — not the byte-reversed
/// little-endian Integer the raw hex literal lowers to.
#[test]
fn bytesn_constant_implicit_hex_is_big_endian() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    bytes2 constant B2 = 0x1234;
    bytes4 constant B4 = 0xdeadbeef;
    function b2() external pure returns (uint256) { return uint256(uint16(B2)); }
    function b4() external pure returns (uint256) { return uint256(uint32(B4)); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let be = |rd: &[u8]| {
        rd.iter()
            .take(16)
            .enumerate()
            .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)))
    };
    for (m, want) in [("b2", 0x1234u128), ("b4", 0xdeadbeef)] {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call");
        assert!(
            r.success,
            "{m}: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        assert_eq!(
            be(&r.return_data),
            want,
            "{m}: bytesN constant not big-endian; got {:#x} want {want:#x}",
            be(&r.return_data)
        );
    }
}

/// Regression (bug-hunt #26/#27): zero-arg `arr.push()` (Solidity >= 0.6)
/// appends a zero-initialized element to a state-var dynamic array, rather
/// than being rejected at compile time.
#[test]
fn zero_arg_array_push_appends_default() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256[] arr;
    function run() external returns (uint256) {
        arr.push();
        arr.push();
        arr[1] = 7;
        return arr.length * 100 + arr[0] * 10 + arr[1];
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "zero-arg push must compile+run: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(
        v, 207,
        "arr.push();push();arr[1]=7 -> len=2,arr[0]=0,arr[1]=7 -> 207; got {v}"
    );
}

/// Regression (bug-hunt #23): a contract's own event declarations must survive
/// when another primary contract references it by type (`new Emitter()`). The
/// sibling-merge pass pulls `Emitter.doSimple` into the referencing host `E`;
/// without carrying `Emitter`'s event declarations too, the merged `doSimple`
/// body's `emit Simple(...)` fails the "no resolved declaration" guard and the
/// whole compile errors. Both the owner (`Emitter`) AND the host (`E`) must
/// declare `Simple` in their manifests so the notification validates on Neo
/// nodes >= 3.6.
#[test]
fn sibling_referenced_contract_keeps_its_events() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Emitter {
    event Simple(uint256 a);
    uint256 public counter;
    function doSimple() public { emit Simple(42); counter += 1; }
}
contract E {
    Emitter e;
    function run() public returns (uint256) { e = new Emitter(); e.doSimple(); return e.counter(); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("sibling-referenced events must compile");
    let has_simple = |name: &str| {
        arts.iter()
            .find(|a| a.manifest["name"].as_str() == Some(name))
            .and_then(|a| a.manifest["abi"]["events"].as_array())
            .map(|evs| evs.iter().any(|e| e["name"].as_str() == Some("Simple")))
            .unwrap_or(false)
    };
    assert!(
        has_simple("Emitter"),
        "Emitter must keep its own Simple event"
    );
    assert!(
        has_simple("E"),
        "host E (merged doSimple) must declare Simple so the notification validates"
    );
}

/// Regression (bug-hunt #2/#25/#31): `unchecked` negation of `type(intN).min`
/// must WRAP two's-complement to `intN.min` (matching EVM), not fault or yield
/// the out-of-range `+2^(N-1)`. Covers unary `-x` at int256 (substitute-min,
/// since `+2^255` overflows NeoVM's 256-bit integer) and narrow widths
/// (truncate/sign-extend), plus the `0 - x` idiom which reroutes to the same
/// unary wrap. The checked path must still Panic(0x11) on `-int256.min`.
#[test]
fn unchecked_intn_min_negation_wraps() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function negMin256() external pure returns (bool) {
        unchecked { int256 x = type(int256).min; return -x == type(int256).min; }
    }
    function zeroSubMin256() external pure returns (bool) {
        unchecked { int256 x = type(int256).min; return int256(0) - x == type(int256).min; }
    }
    function negMin128() external pure returns (bool) {
        unchecked { int128 x = type(int128).min; return -x == type(int128).min; }
    }
    function negMin8() external pure returns (bool) {
        unchecked { int8 x = type(int8).min; return -x == type(int8).min; }
    }
    function checkedNegMin256() external pure returns (int256) {
        int256 x = type(int256).min; return -x; // must Panic(0x11)
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call = |m: &str| {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call")
    };
    for m in ["negMin256", "zeroSubMin256", "negMin128", "negMin8"] {
        let r = call(m);
        assert!(
            r.success && r.return_data.first().copied() == Some(1u8),
            "{m}: unchecked -min must wrap to min (true); success={} data={:?} exc={:?}",
            r.success,
            r.return_data.first(),
            r.exception.as_ref().map(|e| &e.message)
        );
    }
    // The checked path is unchanged: negating int256.min still faults (Panic 0x11).
    let rc = call("checkedNegMin256");
    assert!(!rc.success, "checked -int256.min must Panic, not succeed");
}

/// Regression (bug-hunt #14): bitwise `&`/`|`/`^` on `bytes32` values read back
/// through `uint256(...)` must yield the true value, not a byte-reversed word.
/// NeoVM AND/OR/XOR treat the big-endian ByteString as a little-endian integer;
/// the `uint256(bytesN)` cast's reverse recovers it, but only when `a|b` infers
/// to `bytesN` (the fix adds that inference arm).
#[test]
fn bytes32_bitwise_read_as_uint256_is_true_value() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function orv() external pure returns (uint256) { bytes32 a=bytes32(uint256(1)); bytes32 b=bytes32(uint256(4)); return uint256(a | b); }
    function andv() external pure returns (uint256) { bytes32 a=bytes32(uint256(7)); bytes32 b=bytes32(uint256(5)); return uint256(a & b); }
    function xorv() external pure returns (uint256) { bytes32 a=bytes32(uint256(7)); bytes32 b=bytes32(uint256(5)); return uint256(a ^ b); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let call = |m: &str| {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        rt.call_method(&art.bytecode, &art.tokens, &art.manifest, m, &[])
            .expect("call")
    };
    for (m, want) in [("orv", 5u8), ("andv", 5), ("xorv", 2)] {
        let r = call(m);
        assert!(
            r.success,
            "{m} faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        assert_eq!(
            r.return_data.first().copied(),
            Some(want),
            "{m} must equal {want}"
        );
    }
}

/// Regression (bug-hunt #8): a storage struct passed by reference to an internal
/// function must ALIAS the slot — the callee's field mutation persists. Was
/// lost because the positional call materialized a detached struct copy.
#[test]
fn storage_struct_ref_param_aliases_and_persists() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct Acct { uint256 bal; uint256 nonce; }
    mapping(address => Acct) accts;
    function _bump(Acct storage x) internal { x.nonce += 1; }
    function run() external returns (uint256) {
        address a = address(0x3);
        accts[a].nonce = 5;
        _bump(accts[a]);
        return accts[a].nonce; // must be 6
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.first().copied(),
        Some(6u8),
        "storage ref param mutation must persist"
    );
}

/// Regression (bug-hunt #18/#20): a merged public method that CALL_Ls into its
/// own contract's concrete internal helper must reach a real body — the helper
/// used to be dropped from the sibling-merge and lowered to PUSH0, so its state
/// write vanished. `new H(); w.pub()` (pub calls internal helper() { c += 10 })
/// must leave c == 10.
#[test]
fn sibling_merge_includes_concrete_internal_helpers() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract H {
    uint256 public c;
    function helper() internal { c += 10; }
    function pub() public { helper(); }
}
contract T {
    function run() external returns (uint256) {
        H w = new H();
        w.pub();
        return w.c(); // must be 10
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.manifest["name"].as_str() == Some("T"))
        .expect("T artifact");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.first().copied(),
        Some(10u8),
        "merged internal helper write must persist"
    );
}

/// Regression (bug-hunt #11/#13): element write to a storage `bytes`
/// (`data[i] = v`) must persist. `bytes` is ByteArray{fixed_len:None}, which
/// resolve_storage_reference doesn't model, so it took a copy-and-discard path;
/// now it does a read-modify-write into the slot.
#[test]
fn storage_bytes_element_write_persists() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    bytes data;
    function run() external returns (uint256) {
        data.push(0x11);
        data.push(0x22);
        data[0] = 0xaa;
        data[1] = 0xbb;
        return uint256(uint8(data[0])) * 256 + uint256(uint8(data[1])); // 0xaabb = 43707
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(
        v, 0xaabb,
        "storage bytes element writes must persist (got {v:#x})"
    );
}

/// Regression (bug-hunt #21/#28/#29): a low-level `(bool ok, bytes data) =
/// addr.call(...)` to a reverting callee must report `ok == false` (not be
/// re-decoded as a this.method() ABI buffer), and its returndata must be the
/// RAW EVM envelope (reason-carrying revert starts with the Error selector
/// 0x08c379a0 and has no Neo serialize framing; a no-reason revert is empty).
#[test]
fn low_level_call_revert_ok_false_and_raw_returndata() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract Callee {
    function reqMsg() external pure { require(false, "hi"); }
    function reqNoMsg() external pure { require(false); }
    function ok() external pure returns (uint256) { return 7; }
}
contract T {
    // returns: bit0 = (msg-revert ok==false), bit1 = data starts 0x08c3,
    //          bit2 = (no-msg revert ok==false), bit3 = (no-msg data empty),
    //          bit4 = (success ok==true)
    function run() external returns (uint256) {
        Callee c = new Callee();
        uint256 flags = 0;
        (bool ok1, bytes memory d1) = address(c).call(abi.encodeWithSignature("reqMsg()"));
        if (!ok1) flags |= 1;
        if (d1.length >= 2 && uint8(d1[0]) == 0x08 && uint8(d1[1]) == 0xc3) flags |= 2;
        (bool ok2, bytes memory d2) = address(c).call(abi.encodeWithSignature("reqNoMsg()"));
        if (!ok2) flags |= 4;
        if (d2.length == 0) flags |= 8;
        (bool ok3, ) = address(c).call(abi.encodeWithSignature("ok()"));
        if (ok3) flags |= 16;
        return flags;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.manifest["name"].as_str() == Some("T"))
        .expect("T artifact");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.first().copied(),
        Some(0b11111u8),
        "all low-level-call revert-handling flags must hold (got {:?})",
        r.return_data.first()
    );
}

/// Regression (bug-hunt #10): `T[] memory m = storageArr;` deep-copies. A
/// storage dynamic array holds only its length in the state slot, so the old
/// path bound the Integer length to `m` and `m[i] = v` faulted `SETITEM:
/// unsupported target Integer`. Now it materializes a fresh array; writes to
/// `m` must not touch the storage array.
#[test]
fn storage_array_to_memory_is_deep_copied() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    uint256[] arr;
    function run() external returns (uint256) {
        arr.push(1);
        arr.push(2);
        uint256[] memory m = arr;
        m[0] = 999;
        // arr[0] must stay 1 (deep copy); m[0]==999, m[1]==2
        return arr[0] * 1_000_000 + m[0] * 1000 + m[1];
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(
        v, 1_999_002,
        "deep copy: arr[0]=1, m[0]=999, m[1]=2 (got {v})"
    );
}

/// Regression (bug-hunt #9): a plain internal call to a PUBLIC function
/// returning `T[] memory` must yield the array, not the ABI-encoded blob. The
/// public callee ABI-encodes its single dynamic return; the caller now decodes
/// it (reusing the this.method() decode chain). An `internal` callee already
/// worked (no ABI encoding).
#[test]
fn internal_call_to_public_array_returner_decodes() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function makeMem() public pure returns (uint256[] memory) {
        uint256[] memory m = new uint256[](3);
        m[0] = 100; m[1] = 200; m[2] = 300;
        return m;
    }
    function run() external pure returns (uint256) {
        uint256[] memory m = makeMem();
        return m.length * 1000 + m[1]; // 3*1000 + 200 = 3200
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(
        v, 3200,
        "internal call to public array returner must decode (len=3, m[1]=200)"
    );
}

/// Regression (famous-contracts eval, library-event-merge — extends #23): a
/// `library` that DECLARES its own event and emits it (Aave v3 ReserveLogic
/// declares `event ReserveDataUpdated` and emits it unqualified) must carry
/// that event into the consuming contract's manifest when the library helper
/// is inlined. Previously the merged emit failed "event ... has no resolved
/// declaration" and the whole compile errored, blocking ~15 Aave contracts.
#[test]
fn library_declared_event_is_carried_into_consuming_contract() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.10;
library ReserveLogic {
    event ReserveDataUpdated(address indexed reserve, uint256 rate);
    function updateRates(address r) internal { emit ReserveDataUpdated(r, 100); }
}
contract Pool {
    using ReserveLogic for address;
    function touch(address r) external { ReserveLogic.updateRates(r); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("library-emitted event must compile");
    let pool = arts
        .iter()
        .find(|a| a.manifest["name"].as_str() == Some("Pool"))
        .expect("Pool artifact");
    let has_event = pool.manifest["abi"]["events"]
        .as_array()
        .map(|evs| {
            evs.iter()
                .any(|e| e["name"].as_str() == Some("ReserveDataUpdated"))
        })
        .unwrap_or(false);
    assert!(
        has_event,
        "Pool manifest must declare the inlined library's ReserveDataUpdated event"
    );
}

/// Regression (famous-contracts eval, abstract-virtual): a concrete contract
/// that merely HOLDS a field of an `abstract contract` type must NOT be forced
/// to implement that abstract's virtuals. Compound v2 (CToken/Comptroller hold
/// PriceOracle / InterestRateModel abstract-typed fields) was rejected with
/// "N unimplemented function(s) but not declared abstract" because the
/// sibling-merge injected the abstract's bodyless virtual into the host's
/// method table. Bodyless externals are no longer merged (no body to dispatch).
#[test]
fn concrete_contract_referencing_abstract_typed_field_is_not_abstract() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
abstract contract PriceOracle { function getPrice(address a) external virtual returns (uint256); }
contract Market {
    PriceOracle oracle;
    function px(address a) external returns (uint256) { return oracle.getPrice(a); }
}"#;
    let arts = compile_contracts(src, false, 2)
        .expect("concrete contract with abstract-typed field must compile");
    let market = arts
        .iter()
        .find(|a| a.manifest["name"].as_str() == Some("Market"))
        .expect("Market artifact");
    let methods: Vec<&str> = market.manifest["abi"]["methods"]
        .as_array()
        .expect("methods")
        .iter()
        .filter_map(|m| m["name"].as_str())
        .collect();
    assert!(methods.contains(&"px"), "Market must expose px");
    assert!(
        !methods.contains(&"getPrice"),
        "Market must NOT absorb the abstract's virtual getPrice"
    );
}

/// Regression (famous-contracts eval, inherited-event sibling-merge — completes
/// #23 + 8fe413b): when a host does `new A()` and A's method is merged in, the
/// merge must carry events A INHERITS (from interface/base declarations), not
/// just A's directly-declared events. `contract A is IERC20` emits IERC20's
/// `Approval`; the sibling-event map now walks A's FULL base closure (including
/// interface bases, which the storage/ctor linearization omits). Blocked
/// USDC/FRAX/MasterChef/PublicResolver-style contracts.
#[test]
fn sibling_merge_carries_inherited_events_from_interface_base() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
interface IERC20 { event Approval(address indexed o, address indexed s, uint256 v); }
contract A is IERC20 { function approve(address s, uint256 v) external { emit Approval(msg.sender, s, v); } }
contract B { A token; constructor() { token = new A(); } function go(address s) external { token.approve(s, 1); } }"#;
    let arts = compile_contracts(src, false, 2)
        .expect("sibling that emits an inherited interface event must compile");
    let has_approval = |name: &str| {
        arts.iter()
            .find(|a| a.manifest["name"].as_str() == Some(name))
            .and_then(|a| a.manifest["abi"]["events"].as_array())
            .map(|evs| evs.iter().any(|e| e["name"].as_str() == Some("Approval")))
            .unwrap_or(false)
    };
    assert!(
        has_approval("A"),
        "A must declare its inherited Approval event"
    );
    assert!(
        has_approval("B"),
        "host B (merged A.approve) must declare Approval so the notification validates"
    );
}

/// Regression (famous-contracts eval): `new X{salt: s}(args)` (CREATE2-style
/// salted creation, used by Uniswap Trident / 0x / Seaport ConduitController
/// factories) must compile. Neo N3 has no CREATE2, so the salt option is
/// evaluated for side effects then ignored and the contract is deployed as a
/// normal `new` — rather than failing "unsupported `new` expression".
#[test]
fn new_with_salt_option_compiles() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract Pair { uint256 public x; function init(uint256 v) external { x = v; } }
contract Factory {
    function create(bytes32 salt) external returns (address) {
        Pair p = new Pair{salt: salt}();
        p.init(7);
        return address(p);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("new X{salt: s}() must compile");
    assert!(
        arts.iter()
            .any(|a| a.manifest["name"].as_str() == Some("Factory")),
        "Factory must be produced"
    );
}

/// Regression (latest-feature audit): a contract using Solidity 0.8.19
/// user-defined operators (`using {add as +} for T global`) still COMPILES
/// (neo-solc surfaces a W_USER_DEFINED_OPERATOR warning that the operators are
/// not dispatched — verified separately — but must not fail the compile). This
/// pins the using-directive metadata plumbing that carries the operator
/// symbols to the validator.
#[test]
fn user_defined_operator_directive_still_compiles() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
type Int is int256;
using {addI as +} for Int global;
function addI(Int a, Int b) pure returns (Int) { return Int.wrap(Int.unwrap(a) + Int.unwrap(b)); }
contract C {
    function f(Int a, Int b) external pure returns (Int) { return a + b; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("UDVT-operator contract must still compile");
    assert!(
        arts.iter()
            .any(|a| a.manifest["name"].as_str() == Some("C")),
        "contract C must be produced"
    );
}

/// Regression (feature audit): `.length` of a FIXED-SIZE storage array `T[N]`
/// must be the declared compile-time bound N — not a read of the (never
/// written) base slot, which returned 0 and silently no-op'd
/// `for (i = 0; i < a.length; i++)` loops. Dynamic arrays keep the slot read.
#[test]
fn fixed_size_storage_array_length_is_declared_bound() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    uint256[3] fixedArr;
    uint256[] dynArr;
    function run() external returns (uint256) {
        for (uint256 i = 0; i < fixedArr.length; i++) { fixedArr[i] = i + 1; }
        dynArr.push(9);
        // fixed len=3, sum=6, dyn len=1 -> 3*1000 + 6*10 + 1 = 3061
        return fixedArr.length * 1000 + (fixedArr[0] + fixedArr[1] + fixedArr[2]) * 10 + dynArr.length;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(
        v, 3061,
        "fixed .length must be 3, loop must run, dyn .length must stay slot-backed (got {v})"
    );
}

/// Regression (feature audit): bytesN shifts operate on the BIG-endian face
/// value with EVM truncation. Previously the BE ByteString reached native
/// SHL/SHR, which rejected it ("Invalid operands", N<32) or read it as a
/// LITTLE-endian integer (bytes32(0x100) >> 8 returned 2^232). Also pins the
/// shift-expression type inference (uint256(b >> 8) must reverse the result).
#[test]
fn bytesn_shifts_use_big_endian_face_value() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    bytes32 stored;
    function run() external returns (uint256) {
        stored = bytes32(uint256(1));
        bytes32 a = bytes32(uint256(0x100));
        bytes1 b = bytes1(uint8(1));
        uint256 shl = uint256(stored << 8);   // 256
        uint256 shr = uint256(a >> 8);        // 1
        uint256 b1 = uint8(b << 1);           // 2
        uint256 trunc = uint8(bytes1(uint8(0x80)) << 1); // 0x100 truncated to 1 byte -> 0
        return shl * 1000 + shr * 100 + b1 * 10 + trunc; // 256120
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(v, 256120, "shl=256 shr=1 b1<<1=2 trunc=0 (got {v})");
}

/// Regression (feature audit): `unchecked` int256 Add/Sub/Mul wraps
/// two's-complement mod 2^256 (EVM). Previously it fell through to the bare
/// native op whose unbounded-BigInt result never wraps (max+1 produced the
/// unrepresentable +2^255). Routed through the u256 limb emitters, whose
/// canonical sign-extended 32-byte output IS the signed wrap.
#[test]
fn unchecked_int256_add_sub_mul_wrap_twos_complement() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function run() external pure returns (uint256) {
        uint256 flags = 0;
        int256 mx = type(int256).max;
        int256 mn = type(int256).min;
        unchecked {
            if (mx + 1 == mn) flags |= 1;          // max+1 -> min
            if (mn - 1 == mx) flags |= 2;          // min-1 -> max
            if (mx * 2 == -2) flags |= 4;          // max*2 -> -2
            int256 a = -3;
            if (a * mn == mn) flags |= 8;          // odd * min -> min (mod 2^256)
            if (a + 5 == 2) flags |= 16;           // plain small values unaffected
        }
        return flags;
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.first().copied(),
        Some(0b11111u8),
        "all unchecked int256 wrap identities must hold (got {:?})",
        r.return_data.first()
    );
}

/// Regression (feature audit): a `transient` state variable (Solidity 0.8.28+,
/// EIP-1153) still COMPILES — NeoVM has no transient store, so it persists —
/// and the compile must remain green while the W_TRANSIENT_PERSISTED warning
/// (verified manually via CLI stderr) informs the user of the semantic
/// difference. Pins the attr-capture plumbing (is_transient) end to end.
#[test]
fn transient_state_variable_compiles_with_persist_semantics() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;
contract C {
    uint256 transient t;
    function set(uint256 v) external { t = v; }
    function get() external view returns (uint256) { return t; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("transient state var must compile");
    assert!(
        arts.iter()
            .any(|a| a.manifest["name"].as_str() == Some("C")),
        "contract C must be produced"
    );
}

/// Regression (feature audit): file-scope free-function OVERLOADS must all be
/// callable. The injection pass deduped by bare NAME, so every overload after
/// the first was dropped and calls aborted "'pick'/2 has no compiled body".
#[test]
fn free_function_overloads_all_compile() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
function pick(uint256 a) pure returns (uint256) { return a + 1; }
function pick(uint256 a, uint256 b) pure returns (uint256) { return a + b; }
contract C {
    function run() external pure returns (uint256) {
        return pick(41) * 100 + pick(40, 2); // 42*100 + 42 = 4242
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[])
        .expect("call");
    assert!(
        r.success,
        "faulted: {:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = r
        .return_data
        .iter()
        .take(16)
        .enumerate()
        .fold(0u128, |a, (i, &b)| a + ((b as u128) << (8 * i)));
    assert_eq!(v, 4242, "both free-fn overloads must dispatch (got {v})");
}
