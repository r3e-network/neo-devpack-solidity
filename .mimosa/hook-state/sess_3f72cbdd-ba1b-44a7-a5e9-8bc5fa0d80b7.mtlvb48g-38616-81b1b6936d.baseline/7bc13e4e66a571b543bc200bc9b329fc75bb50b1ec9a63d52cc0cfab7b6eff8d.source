//! Regression tests for wave-#28 bug:
//! same-contract `this.method()` calls returning `T[] memory` left the
//! caller with a corrupted Solidity-level array — `.length` read the wire
//! byte count (e.g. 224 = 32 + 32 + 5*32) instead of the element count (5).
//!
//! Background. Bug #25 wired up `abi.decode(buf, (T[]))` correctly in
//! `src/ir/expressions/calls/builtins/helpers.rs::lower_abi_decode_direct`.
//! That fix only covered explicit `abi.decode(...)` callsites; same-contract
//! method-return assignment (`uint256[] memory got = this.makeArray();`)
//! went through a different lowering path in
//! `src/ir/statements/assignments/lower_assignment.rs` which only handled
//! tuple destructure (`(a, b) = this.pair();`) for static-1-slot members.
//! A single-variable LHS bound to a dynamic-array RHS fell through to the
//! generic `Variable` branch which stored the raw ABI ByteString into the
//! local — so subsequent `.length` reads returned the byte length of the
//! ABI-encoded payload rather than the element count.
//!
//! The fix: extend the dynamic-decode hook in `assign_optional_tuple` and
//! its sibling single-variable path so single-LHS `T[] memory got =
//! this.method()` (and the interface-cast / address-typed shapes covered
//! by `is_this_external_tuple_call`) routes the returned ByteString
//! through `emit_abi_decode_dynamic_top_level`.

#![allow(clippy::uninlined_format_args)]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

/// Wave-#28 minimal repro: a `T[] memory` returned from `this.method()`
/// must yield an array whose `.length` is the element count and whose
/// elements are the encoded values, not the raw ABI byte string.
#[test]
fn wave28_this_uint_array_return_length_and_elements() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function makeArray() external pure returns (uint256[] memory) {
        uint256[] memory r = new uint256[](5);
        r[0]=1; r[1]=2; r[2]=3; r[3]=4; r[4]=5;
        return r;
    }
    function check() external view returns (bool) {
        uint256[] memory got = this.makeArray();
        return got.length == 5 && got[2] == 3;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("wave28 compile: {:?}", e));
    assert!(!arts.is_empty(), "wave28: no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("wave28 runtime");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "check", &[])
        .expect("wave28 check()");
    assert!(
        r.success,
        "wave28 check() must succeed; exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    // bool true is encoded as a non-empty LE byte sequence whose first byte
    // is 0x01. Match the runtime's existing decode_bool habit by checking
    // the raw return_data.
    assert!(
        !r.return_data.is_empty() && r.return_data[0] == 1,
        "wave28: this.makeArray() returned to a `uint256[] memory got` \
         local must produce got.length==5 && got[2]==3; got rd_hex={} \
         (false). If got.length read the byte-length (224) instead of the \
         element count (5), the same-contract dynamic-array decode in \
         lower_assignment.rs regressed.",
        hex::encode(&r.return_data)
    );
}

/// Direct length probe. Avoids any short-circuit interaction in the
/// boolean-AND check above so a regression where `got.length` reads
/// bytes-not-count surfaces as a numeric mismatch.
#[test]
fn wave28_this_uint_array_return_length_only() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function makeArray() external pure returns (uint256[] memory) {
        uint256[] memory r = new uint256[](5);
        r[0]=10; r[1]=20; r[2]=30; r[3]=40; r[4]=50;
        return r;
    }
    function lenOf() external view returns (uint256) {
        uint256[] memory got = this.makeArray();
        return got.length;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("wave28 len compile: {:?}", e));
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("wave28 len rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "lenOf", &[])
        .expect("wave28 lenOf()");
    assert!(
        r.success,
        "wave28 lenOf() must succeed; exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    // The runtime returns the value as a min-width LE byte sequence.
    // For a u256 value of 5 that is a single-byte 0x05 (or empty for 0).
    let value = if r.return_data.is_empty() {
        0u64
    } else {
        let mut acc: u64 = 0;
        for (i, b) in r.return_data.iter().enumerate().take(8) {
            acc |= (*b as u64) << (i * 8);
        }
        acc
    };
    assert_eq!(
        value,
        5,
        "wave28: got.length must be 5; got {} (rd_hex={}). \
         If this returns 224, it's reading the wire byte-length (32 head + \
         32 length-prefix + 5*32 elements) instead of the abi-decoded \
         element count.",
        value,
        hex::encode(&r.return_data)
    );
}

/// A single element read after the `this.<method>()` array assignment must
/// see the encoded value, not garbage. Distinct from the `.length` probe so
/// a partial-fix regression (length right, elements wrong) is also caught.
#[test]
fn wave28_this_uint_array_return_element_read() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function makeArray() external pure returns (uint256[] memory) {
        uint256[] memory r = new uint256[](5);
        r[0]=10; r[1]=20; r[2]=30; r[3]=40; r[4]=50;
        return r;
    }
    function third() external view returns (uint256) {
        uint256[] memory got = this.makeArray();
        return got[2];
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("wave28 elem compile: {:?}", e));
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("wave28 elem rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "third", &[])
        .expect("wave28 third()");
    assert!(
        r.success,
        "wave28 third() must succeed; exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    let value = if r.return_data.is_empty() {
        0u64
    } else {
        let mut acc: u64 = 0;
        for (i, b) in r.return_data.iter().enumerate().take(8) {
            acc |= (*b as u64) << (i * 8);
        }
        acc
    };
    assert_eq!(
        value,
        30,
        "wave28: got[2] must be 30 (the third encoded element); got {} \
         (rd_hex={}). A 0 here means element reads are decoding the raw \
         ABI bytes per index instead of the abi-decoded element values.",
        value,
        hex::encode(&r.return_data)
    );
}
