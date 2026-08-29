//! Regression test for the uint256-literal diagnostic (#12, A4).
//!
//! The bytecode emitter (`push_integer_bigint`) represents every integer in
//! `[-2^255, 2^256-1]` as a conformant 32-byte `PUSHINT256`. In particular a
//! `uint256` value in `[2^255, 2^256-1]` — notably `type(uint256).max` — is
//! stored as its two's-complement "negative-looking" face value, which is this
//! compiler's established `uint256` representation and is handled by the
//! unsigned-aware limb routines and comparisons. Such literals therefore must
//! NOT warn.
//!
//! The pre-fix diagnostic falsely warned for `type(uint256).max` claiming the
//! emitted bytecode "faults on-chain". These tests pin the corrected behavior:
//! representable uint256 literals (incl. the full `2^256-1`) are accepted
//! silently. The genuinely-out-of-range case (magnitude beyond the window) is
//! exercised directly against `neovm_integer_limit_warning` in the crate's unit
//! tests (`src/ir/build/literals.rs`), since valid Solidity source cannot carry
//! a literal that large.

use neo_devpack_solidity::cli::compile_contracts;

fn warns_about_neovm_limit(src: &str) -> bool {
    let arts = compile_contracts(src, false, 2).expect("compile");
    arts.iter().any(|a| {
        a.warnings
            .iter()
            .any(|w| w.message.contains("representable NeoVM 256-bit range"))
    })
}

#[test]
fn type_uint256_max_is_supported_and_does_not_warn() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C { function f() public pure returns (uint256) { return type(uint256).max; } }"#;
    assert!(
        !warns_about_neovm_limit(src),
        "type(uint256).max is representable as a 32-byte two's-complement push and must NOT warn (A4)"
    );
}

#[test]
fn large_hex_literal_2_256_minus_1_does_not_warn() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function f() public pure returns (uint256) {
        return 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    }
}"#;
    assert!(
        !warns_about_neovm_limit(src),
        "2^256-1 (all-ones uint256) is representable and must NOT warn (A4)"
    );
}

#[test]
fn uint256_max_used_in_unsigned_ops_does_not_warn() {
    // max - amount and max > x are the exact integer-operation shapes the old
    // warning claimed would "fault on-chain"; confirm they compile silently.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function f(uint256 a) public pure returns (uint256) { uint256 x = type(uint256).max; unchecked { return x - a; } }
    function g(uint256 a) public pure returns (bool)   { return type(uint256).max > a; }
}"#;
    assert!(
        !warns_about_neovm_limit(src),
        "using type(uint256).max in uint256 add/sub/compare must not warn (A4)"
    );
}

#[test]
fn normal_uint256_values_do_not_warn() {
    // Values that fit a 32-byte signed integer (< 2^255) must NOT warn either.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function f() public pure returns (uint128) { return type(uint128).max; }
    function g() public pure returns (uint256) { return 123456789; }
}"#;
    assert!(
        !warns_about_neovm_limit(src),
        "uint128.max / ordinary values fit 32 bytes and must not warn"
    );
}
