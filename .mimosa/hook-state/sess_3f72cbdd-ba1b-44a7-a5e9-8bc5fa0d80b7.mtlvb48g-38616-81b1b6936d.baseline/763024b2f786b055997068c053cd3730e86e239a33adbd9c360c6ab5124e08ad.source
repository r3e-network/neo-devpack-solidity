//! Regression tests for narrow-integer (uintN/intN, N<256) checked-arithmetic
//! when one operand is an untyped number literal — and for compound assignment,
//! ++/--, `unchecked` truncation, `<<` truncation, and `**`.
//!
//! Before the fix, an untyped literal defaulted to `uint256`, which (a) routed
//! `uint8 x; x + 1` through the 256-bit overflow guard (which only trips above
//! 2^256 and so never fired for a narrow overflow) and (b) suppressed the
//! narrow-width guard. Compound/`++`/`--` never consulted the narrow guard at
//! all; `<<` and `**` applied no width handling. The result was silent overflow
//! (e.g. `uint8(255) + 1 == 256`) instead of the Solidity-0.8 `Panic(0x11)`, and
//! no mod-2^N truncation in `unchecked`. See `binary.rs`/`compound.rs`/`power.rs`.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

/// Compile a single-contract source and execute its (no-arg, entry) `run()`,
/// returning `Ok(value)` on success (return data decoded as little-endian
/// unsigned) or `Err(panic_code)` when the contract faulted with a Solidity
/// `Panic(uint256)` envelope.
fn run_u(src: &str) -> Result<u128, u8> {
    let arts = compile_contracts(src, false, 2).expect("compile must succeed");
    assert!(!arts.is_empty(), "no artifacts");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let res = rt
        .execute(&arts[0].bytecode, &[])
        .expect("host-level execute must not error");
    if res.success {
        let mut v: u128 = 0;
        for (i, b) in res.return_data.iter().enumerate().take(16) {
            v |= (*b as u128) << (8 * i);
        }
        Ok(v)
    } else {
        // Canonical EVM Panic(uint256) envelope: selector 0x4e487b71 || 32-byte BE code.
        if res.return_data.len() >= 36 && res.return_data[..4] == [0x4e, 0x48, 0x7b, 0x71] {
            Err(res.return_data[35])
        } else {
            // legacy "Panic: 0x11" message path
            let msg = res
                .exception
                .as_ref()
                .map(|e| e.message.clone())
                .unwrap_or_default();
            if let Some(i) = msg.find("Panic: 0x") {
                if let Ok(c) = u8::from_str_radix(&msg[i + 9..i + 11], 16) {
                    return Err(c);
                }
            }
            Err(0xff)
        }
    }
}

fn contract(body: &str, ret: &str) -> String {
    format!(
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\ncontract C {{ function run() public pure returns ({ret}) {{ {body} }} }}"
    )
}

// ---- checked overflow with a literal operand must Panic(0x11) ----

#[test]
fn narrow_uint8_plus_literal_overflows() {
    assert_eq!(
        run_u(&contract("uint8 x = 255; x = x + 1; return x;", "uint8")),
        Err(0x11)
    );
}

#[test]
fn narrow_uint8_mul_literal_overflows() {
    assert_eq!(
        run_u(&contract("uint8 x = 16; x = x * 16; return x;", "uint8")),
        Err(0x11)
    );
}

#[test]
fn narrow_uint8_compound_add_overflows() {
    assert_eq!(
        run_u(&contract("uint8 x = 255; x += 1; return x;", "uint8")),
        Err(0x11)
    );
}

#[test]
fn narrow_uint8_post_increment_overflows() {
    assert_eq!(
        run_u(&contract("uint8 x = 255; x++; return x;", "uint8")),
        Err(0x11)
    );
}

#[test]
fn narrow_uint8_pre_increment_overflows() {
    assert_eq!(
        run_u(&contract("uint8 x = 255; ++x; return x;", "uint8")),
        Err(0x11)
    );
}

#[test]
fn narrow_uint16_compound_sub_underflows() {
    assert_eq!(
        run_u(&contract("uint16 x = 0; x -= 1; return x;", "uint16")),
        Err(0x11)
    );
}

#[test]
fn narrow_uint8_power_literal_overflows() {
    assert_eq!(
        run_u(&contract(
            "uint8 a = 2; uint8 r = a ** 8; return r;",
            "uint8"
        )),
        Err(0x11)
    );
}

// ---- non-overflowing narrow ops still produce the correct value ----

#[test]
fn narrow_uint8_in_range_add_ok() {
    assert_eq!(
        run_u(&contract("uint8 x = 100; x = x + 27; return x;", "uint8")),
        Ok(127)
    );
}

#[test]
fn narrow_uint8_compound_in_range_ok() {
    assert_eq!(
        run_u(&contract("uint8 x = 10; x += 5; return x;", "uint8")),
        Ok(15)
    );
}

#[test]
fn narrow_uint8_power_in_range_ok() {
    assert_eq!(
        run_u(&contract(
            "uint8 a = 2; uint8 r = a ** 7; return r;",
            "uint8"
        )),
        Ok(128)
    );
}

// ---- unchecked narrow wraps mod 2^N (no Panic) ----

#[test]
fn narrow_uint8_unchecked_compound_wraps() {
    assert_eq!(
        run_u(&contract(
            "uint8 x = 255; unchecked { x += 1; } return x;",
            "uint8"
        )),
        Ok(0)
    );
}

#[test]
fn narrow_uint8_unchecked_add_literal_wraps() {
    assert_eq!(
        run_u(&contract(
            "uint8 x = 255; unchecked { x = x + 2; } return x;",
            "uint8"
        )),
        Ok(1)
    );
}

// ---- shift-left truncates to operand width (never panics) ----

#[test]
fn narrow_uint8_shl_truncates() {
    assert_eq!(
        run_u(&contract("uint8 x = 200; x = x << 1; return x;", "uint8")),
        Ok(144)
    );
}

#[test]
fn narrow_uint8_shl_out_shifts_to_zero() {
    assert_eq!(
        run_u(&contract("uint8 x = 1; x = x << 8; return x;", "uint8")),
        Ok(0)
    );
}

// ---- wide (uint256) arithmetic is unaffected (no spurious guard / wrong path) ----

#[test]
fn uint256_in_range_add_literal_ok() {
    assert_eq!(
        run_u(&contract(
            "uint256 x = 1000; x = x + 1; return x;",
            "uint256"
        )),
        Ok(1001)
    );
}

#[test]
fn uint256_unchecked_add_literal_ok() {
    assert_eq!(
        run_u(&contract(
            "uint256 x = 1000; unchecked { x = x + 1; } return x;",
            "uint256"
        )),
        Ok(1001)
    );
}

// ---- mixed-width: `uint256 OP uintN` is uint256 arithmetic (narrow operand
// widens); it must NOT be truncated to the narrow width. Regression for the
// `is_narrow_result` precedence (a narrow operand present alongside a genuine
// uint256 must stay on the 256-bit path). ----

#[test]
fn mixed_uint256_plus_uint32_is_wide_unchecked() {
    // s (uint256) += a (uint32 = 2^32-1); unchecked. Result must be the true
    // sum 1 + (2^32-1) = 2^32 (NOT truncated mod 2^32 to 0).
    let src = contract(
        "uint256 s = 1; uint32 a = type(uint32).max; unchecked { s += a; } return s;",
        "uint256",
    );
    assert_eq!(run_u(&src), Ok(4_294_967_296));
}

#[test]
fn mixed_uint256_plus_uint32_in_range_checked() {
    let src = contract("uint256 s = 1; uint32 a = 41; s += a; return s;", "uint256");
    assert_eq!(run_u(&src), Ok(42));
}

// ---- narrow signed division overflow: `intN.min / -1` must Panic(0x11) ----

#[test]
fn narrow_int8_min_div_neg_one_overflows() {
    let src = contract(
        "int8 a = type(int8).min; int8 b = -1; return a / b;",
        "int8",
    );
    assert_eq!(run_u(&src), Err(0x11));
}

#[test]
fn narrow_int8_div_in_range_ok() {
    let src = contract("int8 a = 100; int8 b = 5; return a / b;", "int8");
    assert_eq!(run_u(&src), Ok(20));
}

#[test]
fn unsigned_narrow_div_unaffected() {
    let src = contract("uint8 a = 200; uint8 b = 3; return a / b;", "uint8");
    assert_eq!(run_u(&src), Ok(66));
}

#[test]
fn narrow_unchecked_post_inc_dec_recovers_wrapped_old_value() {
    // `y = x++` binds y to the PRE-increment value. At uint8 max in an unchecked
    // block the increment wraps x to 0; the recovered old value must be the
    // width-bounded 255, not -1 from a width-unaware `new - 1`.
    assert_eq!(
        run_u(&contract(
            "unchecked { uint8 x = 255; uint8 y = x++; return y; }",
            "uint8"
        )),
        Ok(255)
    );
    // post-decrement at the min wraps x to 255; old value is 0.
    assert_eq!(
        run_u(&contract(
            "unchecked { uint8 x = 0; uint8 y = x--; return y; }",
            "uint8"
        )),
        Ok(0)
    );
    // the stored variable also wrapped correctly.
    assert_eq!(
        run_u(&contract(
            "unchecked { uint8 x = 255; x++; return x; }",
            "uint8"
        )),
        Ok(0)
    );
}
