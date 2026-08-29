//! Regression tests for the ABI-interop fixes (agent key: abi).
//!
//! Covers:
//! 1. `abi.decode` of multi-slot static types (all-static structs, struct
//!    arrays, tuples containing structs) round-trips canonically instead of
//!    falling back to `StdLib.deserialize` and faulting.
//! 2. `abi.decode` of unsigned integers >= 2^255 stays positive (the signed
//!    little-endian CONVERT no longer reinterprets `type(uint256).max` as -1).
//! 3. Custom-error selectors are computed from the DECLARED `error`
//!    signature, and named arguments encode in declaration order.
//! 4. The manifest reports `ByteArray` for abi-encoded returns (multi-value
//!    and Solidity array returns) while struct returns stay `Array`.
//! 5. `abi.encodePacked(intN)` sign-extends negative values.
//! 6. `require(cond, <non-literal string>)` throws the canonical
//!    `Error(string)` envelope, matching `revert(msg)`, so
//!    `catch Error(string)` selector guards see it.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{ExecutionResult, NeoRuntime, RuntimeConfig};
use sha3::{Digest, Keccak256};

/// Compile a single-contract source and execute the method at offset 0
/// (the first declared public function) with no arguments.
fn compile_and_execute(source: &str) -> ExecutionResult {
    let artifacts = compile_contracts(source, false, 2)
        .unwrap_or_else(|e| panic!("compile failed: {e:?}\nsource:\n{source}"));
    assert!(!artifacts.is_empty(), "compile produced no artifacts");
    let mut runtime =
        NeoRuntime::new(RuntimeConfig::default()).expect("runtime construction must not fail");
    runtime
        .execute(&artifacts[0].bytecode, &[])
        .expect("execute must not fail at host level (a fault != host error)")
}

fn selector(signature: &str) -> [u8; 4] {
    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    [digest[0], digest[1], digest[2], digest[3]]
}

fn assert_returns_true(source: &str) {
    let result = compile_and_execute(source);
    assert!(
        result.success,
        "execution must succeed; exception={:?} return_data={}",
        result.exception,
        hex::encode(&result.return_data)
    );
    assert_eq!(
        result.return_data,
        vec![0x01],
        "expected `true`; return_data={}",
        hex::encode(&result.return_data)
    );
}

// ---------------------------------------------------------------------------
// 1. Multi-slot static decode (structs)
// ---------------------------------------------------------------------------

#[test]
fn abi_decode_static_struct_roundtrip() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract StructRoundtrip {
    struct S { uint256 a; uint256 b; }
    function check() public pure returns (bool) {
        S memory s = S(7, 9);
        bytes memory enc = abi.encode(s);
        if (enc.length != 64) return false;
        S memory d = abi.decode(enc, (S));
        return d.a == 7 && d.b == 9;
    }
}"#,
    );
}

#[test]
fn abi_decode_struct_with_negative_and_max_fields_roundtrip() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract StructSignedRoundtrip {
    struct P { uint256 big; int64 neg; }
    function check() public pure returns (bool) {
        P memory p = P(type(uint256).max, -5);
        bytes memory enc = abi.encode(p);
        if (enc.length != 64) return false;
        P memory d = abi.decode(enc, (P));
        return d.big == type(uint256).max && d.neg == -5 && d.neg < 0;
    }
}"#,
    );
}

#[test]
fn abi_decode_struct_array_roundtrip() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract StructArrayRoundtrip {
    struct S { uint256 a; uint256 b; }
    function check() public pure returns (bool) {
        S[] memory arr = new S[](2);
        arr[0] = S(1, 2);
        arr[1] = S(3, 4);
        bytes memory enc = abi.encode(arr);
        // offset(32) + length(32) + 2 elements x 2 slots x 32 = 192.
        if (enc.length != 192) return false;
        S[] memory d = abi.decode(enc, (S[]));
        return d.length == 2
            && d[0].a == 1 && d[0].b == 2
            && d[1].a == 3 && d[1].b == 4;
    }
}"#,
    );
}

#[test]
fn abi_decode_tuple_with_struct_member_shifts_slots() {
    // A 2-slot struct in tuple position 0 must shift the following member
    // to head slot 2 (byte 64) — pins the running head-slot accounting.
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract TupleWithStruct {
    struct S { uint256 a; uint256 b; }
    function check() public pure returns (bool) {
        S memory s = S(5, 6);
        bytes memory enc = abi.encode(s, uint256(77));
        if (enc.length != 96) return false;
        (S memory d, uint256 v) = abi.decode(enc, (S, uint256));
        return d.a == 5 && d.b == 6 && v == 77;
    }
}"#,
    );
}

// ---------------------------------------------------------------------------
// 1b. Dynamic struct decode (struct with string/bytes/array fields)
// ---------------------------------------------------------------------------

#[test]
fn abi_decode_dynamic_struct_roundtrip() {
    // `struct D { uint256 a; string b; bytes c; }` encodes as a tuple:
    // head = [a | off_b | off_c] (96 bytes), tail = [b_tail | c_tail]
    // (64 + 64). The single top-level value adds a 0x20 offset word ⇒ 256.
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract DynStruct {
    struct D { uint256 a; string b; bytes c; }
    function check() public pure returns (bool) {
        D memory d = D(7, "hi", hex"abcd");
        bytes memory enc = abi.encode(d);
        if (enc.length != 256) return false;
        D memory r = abi.decode(enc, (D));
        return r.a == 7
            && keccak256(bytes(r.b)) == keccak256(bytes("hi"))
            && keccak256(r.c) == keccak256(hex"abcd");
    }
}"#,
    );
}

#[test]
fn abi_decode_dynamic_struct_array_roundtrip() {
    // `D[]` where `D` is dynamic — a nested head/tail array whose elements
    // are themselves dynamic structs (deepest recursion this exercises).
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract DynStructArray {
    struct D { uint256 a; string b; }
    function check() public pure returns (bool) {
        D[] memory arr = new D[](2);
        arr[0] = D(1, "x");
        arr[1] = D(2, "yz");
        bytes memory enc = abi.encode(arr);
        D[] memory r = abi.decode(enc, (D[]));
        return r.length == 2
            && r[0].a == 1 && keccak256(bytes(r[0].b)) == keccak256(bytes("x"))
            && r[1].a == 2 && keccak256(bytes(r[1].b)) == keccak256(bytes("yz"));
    }
}"#,
    );
}

#[test]
fn abi_decode_tuple_with_dynamic_struct_member() {
    // A dynamic struct in a tuple position forces the tuple itself into the
    // head/tail layout: the struct member is an offset word, trailing
    // statics stay inline.
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract TupleDynStruct {
    struct D { uint256 a; bytes b; }
    function check() public pure returns (bool) {
        D memory d = D(5, hex"c0ffee");
        bytes memory enc = abi.encode(d, uint256(99));
        (D memory r, uint256 v) = abi.decode(enc, (D, uint256));
        return r.a == 5
            && keccak256(r.b) == keccak256(hex"c0ffee")
            && v == 99;
    }
}"#,
    );
}

#[test]
fn abi_decode_rejects_noncanonical_length_offset_slot() {
    // The top-level offset word here has a nonzero HIGH byte (byte 0 = 0x01)
    // with a benign low-8 (0x20). Reading only the low 8 bytes would silently
    // truncate the crafted value and decode the wrong region; the high-bits
    // guard must revert instead (Panic 0x41).
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function check() public pure returns (uint256) {
        bytes memory enc = hex"0100000000000000000000000000000000000000000000000000000000000020";
        string[] memory b = abi.decode(enc, (string[]));
        return b.length;
    }
}"#,
    );
    assert!(
        !result.success,
        "a non-canonical offset slot (nonzero high bytes) must revert, not silently truncate"
    );
}

#[test]
fn same_arity_overloads_dispatch_by_argument_type() {
    // `pick(uint256)` and `pick(address)` share (name, arity)=(pick,1).
    // Each call must dispatch to the overload matching its argument type;
    // previously the dispatch table collapsed to the last declaration, so one
    // of them ran the wrong body. pick(uint256(5))=105, pick(address(0))=7.
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function pick(uint256 x) internal pure returns (uint256) { return 100 + x; }
    function pick(address a) internal pure returns (uint256) { return a == address(0) ? 7 : 9; }
    function check() public pure returns (bool) {
        return pick(uint256(5)) == 105 && pick(address(0)) == 7;
    }
}"#,
    );
}

#[test]
fn negative_signed_multi_value_return_sign_extends() {
    // A multi-value static return with a NEGATIVE signed integer must
    // sign-extend that slot to 32 bytes of 0xFF (EVM ABI canonical), not
    // zero-extend it. `(-1, 5)` -> slot0 = 0xFF*32, slot1 = uint256(5).
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() public pure returns (int256, uint256) {
        return (-1, 5);
    }
}"#,
    );
    assert!(
        result.success,
        "multi-return must succeed; exc={:?}",
        result.exception
    );
    assert_eq!(
        result.return_data.len(),
        64,
        "two static slots = 64 bytes; got {}",
        hex::encode(&result.return_data)
    );
    assert!(
        result.return_data[..32].iter().all(|b| *b == 0xFF),
        "int256(-1) must sign-extend to 32 0xFF bytes; got {}",
        hex::encode(&result.return_data[..32])
    );
    let mut slot1 = [0u8; 32];
    slot1[31] = 5;
    assert_eq!(
        &result.return_data[32..64],
        &slot1,
        "uint256(5) slot must be big-endian 5"
    );
}

// ---------------------------------------------------------------------------
// 2. Unsigned decode >= 2^255
// ---------------------------------------------------------------------------

#[test]
fn abi_decode_uint256_max_roundtrip() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract UintMaxRoundtrip {
    function check() public pure returns (bool) {
        uint256 x = type(uint256).max;
        uint256 d = abi.decode(abi.encode(x), (uint256));
        return d == x && d >= 1;
    }
}"#,
    );
}

#[test]
fn abi_decode_uint256_half_roundtrip() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract UintHalfRoundtrip {
    function check() public pure returns (bool) {
        uint256 x = 2 ** 255;
        uint256 d = abi.decode(abi.encode(x), (uint256));
        return d == x && d > 2 ** 254;
    }
}"#,
    );
}

#[test]
fn abi_decode_uint256_array_with_high_bit_elements_roundtrip() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract UintArrayRoundtrip {
    function check() public pure returns (bool) {
        uint256[] memory a = new uint256[](2);
        a[0] = type(uint256).max;
        a[1] = 12345;
        uint256[] memory d = abi.decode(abi.encode(a), (uint256[]));
        return d.length == 2 && d[0] == type(uint256).max && d[1] == 12345;
    }
}"#,
    );
}

#[test]
fn abi_decode_negative_int256_still_signed() {
    // Control: the sign-byte fix must NOT break signed decode.
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract IntRoundtrip {
    function check() public pure returns (bool) {
        int256 x = -42;
        int256 d = abi.decode(abi.encode(x), (int256));
        return d == x && d < 0;
    }
}"#,
    );
}

// ---------------------------------------------------------------------------
// 3. Declared custom-error selectors
// ---------------------------------------------------------------------------

#[test]
fn custom_error_selector_uses_declared_uint8() {
    // `error E1(uint8); revert E1(1);` — the literal infers uint256, but the
    // selector must hash the DECLARED signature `E1(uint8)`.
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract DeclaredSelector {
    error E1(uint8 code);
    function boom() public pure {
        revert E1(1);
    }
}"#,
    );
    assert!(!result.success, "boom() must revert");
    let expected = selector("E1(uint8)");
    assert!(
        result.return_data.len() >= 36,
        "revert payload must be selector + one 32-byte slot; got {}",
        hex::encode(&result.return_data)
    );
    assert_eq!(
        &result.return_data[..4],
        &expected[..],
        "selector must be keccak(\"E1(uint8)\")[..4]; got {}",
        hex::encode(&result.return_data[..4])
    );
    // abi.encode(1): 32-byte BE slot with last byte 1.
    assert_eq!(result.return_data[35], 1, "encoded arg must be 1");
    assert!(
        result.return_data[4..35].iter().all(|b| *b == 0),
        "high bytes of the slot must be zero"
    );
}

#[test]
fn custom_error_selector_uses_declared_uint256_for_narrow_arg() {
    // `error EW(uint256); uint64 x; revert EW(x);` — the variable infers
    // uint64, but the selector must hash `EW(uint256)`.
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract DeclaredSelectorWide {
    error EW(uint256 code);
    function boom() public pure {
        uint64 x = 7;
        revert EW(x);
    }
}"#,
    );
    assert!(!result.success, "boom() must revert");
    let expected = selector("EW(uint256)");
    assert_eq!(
        &result.return_data[..4],
        &expected[..],
        "selector must be keccak(\"EW(uint256)\")[..4]; got {}",
        hex::encode(&result.return_data[..4])
    );
    assert_eq!(result.return_data[35], 7, "encoded arg must be 7");
}

#[test]
fn custom_error_named_args_encode_in_declaration_order() {
    // solang preserves SOURCE order for named args; the payload must follow
    // DECLARATION order: a=1 in slot 0, b=2 in slot 1, and the selector must
    // hash the declared types `E2(uint256,uint8)`.
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract NamedOrder {
    error E2(uint256 a, uint8 b);
    function boom() public pure {
        revert E2({b: 2, a: 1});
    }
}"#,
    );
    assert!(!result.success, "boom() must revert");
    let expected = selector("E2(uint256,uint8)");
    assert_eq!(
        &result.return_data[..4],
        &expected[..],
        "selector must be keccak(\"E2(uint256,uint8)\")[..4]; got {}",
        hex::encode(&result.return_data[..4])
    );
    assert!(
        result.return_data.len() >= 68,
        "payload must carry two 32-byte slots; got {}",
        hex::encode(&result.return_data)
    );
    assert_eq!(result.return_data[35], 1, "slot 0 must hold a = 1");
    assert_eq!(result.return_data[67], 2, "slot 1 must hold b = 2");
}

#[test]
fn require_with_custom_error_uses_declared_signature() {
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract RequireDeclared {
    error Nope(uint8 code);
    function boom() public pure {
        require(false, Nope(3));
    }
}"#,
    );
    assert!(!result.success, "boom() must revert");
    let expected = selector("Nope(uint8)");
    assert_eq!(
        &result.return_data[..4],
        &expected[..],
        "require custom-error selector must be keccak(\"Nope(uint8)\")[..4]; got {}",
        hex::encode(&result.return_data[..4])
    );
}

// ---------------------------------------------------------------------------
// 4. Manifest returntype for abi-encoded returns
// ---------------------------------------------------------------------------

#[test]
fn manifest_returntype_bytearray_for_encoded_returns_array_for_structs() {
    let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract ManifestShapes {
    struct S { uint256 a; uint256 b; }
    S public s;
    function pair() public pure returns (uint256, uint256) {
        return (1, 2);
    }
    function arr() public pure returns (uint256[] memory) {
        uint256[] memory a = new uint256[](1);
        a[0] = 1;
        return a;
    }
    function getS() public view returns (S memory) {
        return s;
    }
}"#;
    let artifacts = compile_contracts(source, false, 2).expect("compile failed");
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("abi.methods array");
    let returntype = |name: &str| -> String {
        methods
            .iter()
            .find(|m| m["name"] == name)
            .unwrap_or_else(|| panic!("method {name} missing from manifest"))["returntype"]
            .as_str()
            .unwrap()
            .to_string()
    };
    // Multi-value and Solidity-array returns lower to abi-encoded bytes.
    assert_eq!(
        returntype("pair"),
        "ByteArray",
        "multi-return is encoded bytes"
    );
    assert_eq!(
        returntype("arr"),
        "ByteArray",
        "array return is encoded bytes"
    );
    // A DECLARED single struct return genuinely pushes a StackItem::Array.
    assert_eq!(returntype("getS"), "Array", "struct return stays Array");
    // The auto-getter returns the struct FIELDS as a multi-value tuple,
    // which the return lowering abi-encodes into bytes.
    assert_eq!(
        returntype("s"),
        "ByteArray",
        "struct auto-getter is encoded bytes"
    );
}

#[test]
fn manifest_returntype_omits_mapping_member_from_struct_getter() {
    // Solidity's auto-generated public getter for a struct state variable OMITS
    // mapping (and array) members — there is no ABI representation for them. So
    // `slots(uint256)` returns ONLY the scalar `id` (a single uint256), whose
    // manifest returntype is "Integer". Previously the mapping member was
    // wrongly included, producing a non-encodable multi-value tuple that fell
    // back to the legacy "Array" shape and advertised an invalid getter.
    let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract MapStruct {
    struct S { uint256 id; mapping(address => uint256) balances; }
    mapping(uint256 => S) public slots;
    function init(uint256 k, uint256 v) external { slots[k].id = v; }
}"#;
    let artifacts = compile_contracts(source, false, 2).expect("compile failed");
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("abi.methods array");
    let getter = methods
        .iter()
        .find(|m| m["name"] == "slots")
        .expect("slots getter missing");
    assert_eq!(
        getter["returntype"].as_str(),
        Some("Integer"),
        "mapping member must be omitted, leaving a single uint256 (Integer) return"
    );
    // The getter takes the outer mapping key only; no struct-member params leak.
    assert_eq!(
        getter["parameters"].as_array().map(|p| p.len()),
        Some(1),
        "slots getter takes exactly the outer mapping key"
    );
}

// ---------------------------------------------------------------------------
// 5. encodePacked sign extension
// ---------------------------------------------------------------------------

#[test]
fn encode_packed_int128_minus_one_sign_extends() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract PackedSigned {
    function check() public pure returns (bool) {
        bytes memory p = abi.encodePacked(int128(-1));
        if (p.length != 16) return false;
        return keccak256(p) == keccak256(hex"ffffffffffffffffffffffffffffffff");
    }
}"#,
    );
}

#[test]
fn encode_packed_int16_negative_sign_extends() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract PackedSignedNarrow {
    function check() public pure returns (bool) {
        bytes memory p = abi.encodePacked(int16(-2));
        if (p.length != 2) return false;
        return keccak256(p) == keccak256(hex"fffe");
    }
}"#,
    );
}

#[test]
fn encode_packed_positive_signed_and_unsigned_unchanged() {
    assert_returns_true(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract PackedControl {
    function check() public pure returns (bool) {
        bytes memory a = abi.encodePacked(int16(300));
        bytes memory b = abi.encodePacked(uint16(65535));
        return a.length == 2
            && keccak256(a) == keccak256(hex"012c")
            && b.length == 2
            && keccak256(b) == keccak256(hex"ffff");
    }
}"#,
    );
}

// ---------------------------------------------------------------------------
// 6. require(cond, <dynamic string>) Error(string) envelope
// ---------------------------------------------------------------------------

#[test]
fn require_dynamic_string_message_gets_error_envelope() {
    let result = compile_and_execute(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract RequireDyn {
    function boom() public pure {
        string memory m = string.concat("dyn", "amic");
        require(false, m);
    }
}"#,
    );
    assert!(!result.success, "boom() must revert");
    let expected = selector("Error(string)");
    assert!(
        result.return_data.len() >= 68 + 7,
        "payload must be the full Error(string) envelope; got {}",
        hex::encode(&result.return_data)
    );
    assert_eq!(
        &result.return_data[..4],
        &expected[..],
        "selector must be keccak(\"Error(string)\")[..4] (0x08c379a0); got {}",
        hex::encode(&result.return_data[..4])
    );
    // offset slot = 0x20.
    assert_eq!(result.return_data[35], 0x20, "offset slot must be 0x20");
    // length slot = 7.
    assert_eq!(result.return_data[67], 7, "length slot must be 7");
    assert_eq!(
        &result.return_data[68..75],
        b"dynamic",
        "payload must carry the message bytes"
    );
}

#[test]
fn catch_error_string_matches_dynamic_require_message() {
    // Before the fix this returned 3 (fell through to the bare-bytes catch);
    // the Error(string) arm must match and decode the 7-byte message.
    // `this.fail()` needs the manifest-driven self-method dispatch table, so
    // route through `call_method` instead of raw offset-0 execution.
    let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract RequireDynCatch {
    function check() public returns (uint256) {
        try this.fail() {
            return 9;
        } catch Error(string memory r) {
            if (bytes(r).length == 7) {
                return 1;
            }
            return 2;
        } catch (bytes memory) {
            return 3;
        }
    }
    function fail() external pure {
        string memory m = string.concat("dyn", "amic");
        require(false, m);
    }
}"#;
    let artifacts = compile_contracts(source, false, 2).expect("compile failed");
    let art = &artifacts[0];
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let result = runtime
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "check", &[])
        .expect("call_method check");
    assert!(
        result.success,
        "check() must succeed; exception={:?}",
        result.exception
    );
    // Scalar uint returns surface as variable-width LE bytes.
    assert_eq!(
        num_bigint::BigUint::from_bytes_le(&result.return_data),
        num_bigint::BigUint::from(1u8),
        "catch Error(string) arm must match the dynamic require message (1); got {}",
        hex::encode(&result.return_data)
    );
}

#[test]
fn modexp_precompile_faults_on_unsupported_multibyte_shape() {
    // The 0x05 precompile is the 1-byte-operand variant. A multi-byte shape
    // (base_len=2 here) must FAULT loudly via the length-header gate, instead
    // of silently mis-reading operand bytes and returning a wrong result.
    let mut input = vec![0u8; 96];
    input[31] = 2; // base_len = 2 (unsupported)
    input[63] = 1; // exp_len = 1
    input[95] = 1; // mod_len = 1
    input.extend_from_slice(&[0x01, 0x02, 0x03, 0x05]); // base(2) exp(1) mod(1)
    let hexstr = hex::encode(&input);
    let src = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function check() public view returns (uint256) {{
        bytes memory input = hex"{hexstr}";
        (bool ok, bytes memory out) = address(0x05).staticcall(input);
        return ok ? out.length : 999;
    }}
}}"#
    );
    let result = compile_and_execute(&src);
    assert!(
        !result.success,
        "a multi-byte modexp shape must fault, not silently mis-read operands"
    );

    // The supported 1-byte shape still computes: 3^2 mod 7 = 2.
    let mut ok_input = vec![0u8; 96];
    ok_input[31] = 1;
    ok_input[63] = 1;
    ok_input[95] = 1;
    ok_input.extend_from_slice(&[0x03, 0x02, 0x07]); // base=3 exp=2 mod=7
    let ok_hex = hex::encode(&ok_input);
    let ok_src = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function check() public view returns (bool) {{
        bytes memory input = hex"{ok_hex}";
        (bool ok, bytes memory out) = address(0x05).staticcall(input);
        return ok && out.length == 32 && uint8(out[31]) == 2;
    }}
}}"#
    );
    let ok_result = compile_and_execute(&ok_src);
    assert!(
        ok_result.success && ok_result.return_data == vec![0x01],
        "1-byte modexp 3^2 mod 7 must still compute 2; rd={}",
        hex::encode(&ok_result.return_data)
    );
}
