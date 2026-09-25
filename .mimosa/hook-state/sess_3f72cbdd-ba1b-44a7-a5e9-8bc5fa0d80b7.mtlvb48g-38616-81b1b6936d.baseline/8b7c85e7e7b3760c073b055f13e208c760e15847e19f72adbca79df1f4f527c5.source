//! Property-based tests for the under-covered arithmetic helpers in the
//! Neo DevPack for Solidity runtime: POW, SQRT, SIGN, ABS, NEGATE, INC, DEC.
//!
//! These helpers live in `src/runtime/execution/helpers/arithmetic/` and were
//! identified as 0-12% covered per the coverage analysis. The functions
//! themselves are private (`fn`, not `pub fn`), so the tests drive them
//! through the opcode dispatch path: push StackItem operands onto an
//! `ExecutionContext` evaluation stack, then `step()` a single-byte
//! bytecode containing the target opcode and observe the post-execution
//! stack top.
//!
//! Opcode numbers (from `src/runtime/spec/opcodes.rs`):
//!   0x99 SIGN, 0x9A ABS, 0x9B NEGATE, 0x9C INC, 0x9D DEC,
//!   0xA3 POW, 0xA4 SQRT.

#![allow(unused_imports)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]

use neo_devpack_solidity::runtime::execution::{ExecutionContext, StackItem};
use neo_devpack_solidity::runtime::RuntimeConfig;
use proptest::prelude::*;

const OP_SIGN: u8 = 0x99;
const OP_ABS: u8 = 0x9A;
const OP_NEGATE: u8 = 0x9B;
const OP_INC: u8 = 0x9C;
const OP_DEC: u8 = 0x9D;
const OP_POW: u8 = 0xA3;
const OP_SQRT: u8 = 0xA4;

/// Build a fresh `ExecutionContext` initialised against a single-byte script
/// containing `opcode`. The default `RuntimeConfig` has `strict_mode = true`,
/// which enables `strict_arithmetic` for INC / DEC overflow checking.
fn ctx_for_opcode(opcode: u8) -> ExecutionContext {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&[opcode], &[]).expect("init bytecode");
    ctx
}

/// Push operands then step a single instruction. Returns the popped result on
/// success, or the runtime error on failure. Used for unary ops; the operand
/// stack is `[value]` at top.
fn step_unary(opcode: u8, value: StackItem) -> Result<StackItem, String> {
    let mut ctx = ctx_for_opcode(opcode);
    ctx.push_stack(value).map_err(|e| e.to_string())?;
    ctx.step().map_err(|e| e.to_string())?;
    let top = ctx.peek_stack().map_err(|e| e.to_string())?.clone();
    Ok(top)
}

/// Push `[a, b]` (b on top) then step a binary op. POW pops `b` (exponent)
/// first then `a` (base), which mirrors how `instruction/arithmetic/binary.rs`
/// dispatches to `pow_stack_items(a, b)`.
fn step_binary(opcode: u8, a: StackItem, b: StackItem) -> Result<StackItem, String> {
    let mut ctx = ctx_for_opcode(opcode);
    ctx.push_stack(a).map_err(|e| e.to_string())?;
    ctx.push_stack(b).map_err(|e| e.to_string())?;
    ctx.step().map_err(|e| e.to_string())?;
    let top = ctx.peek_stack().map_err(|e| e.to_string())?.clone();
    Ok(top)
}

/// Extract an `i64` from a `StackItem` for assertion convenience.
fn as_i64(item: &StackItem) -> Option<i64> {
    match item {
        StackItem::Integer(v) => Some(*v),
        StackItem::UnsignedInteger(v) => i64::try_from(*v).ok(),
        StackItem::Boolean(b) => Some(if *b { 1 } else { 0 }),
        _ => None,
    }
}

// ============================================================================
// POW
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// POW with small non-negative exponents matches `i64::checked_pow`.
    #[test]
    fn pow_small_exponent_matches_reference(
        a in -100i64..=100i64,
        b in 0u32..=10,
    ) {
        let result = step_binary(OP_POW, StackItem::Integer(a), StackItem::Integer(b as i64));
        match a.checked_pow(b) {
            Some(expected) => {
                let v = result.expect("POW must succeed for in-range inputs");
                prop_assert_eq!(as_i64(&v), Some(expected),
                    "POW({}, {}) returned {:?} (expected {})", a, b, v, expected);
            }
            None => {
                prop_assert!(result.is_err(),
                    "POW({}, {}) should overflow but returned {:?}", a, b, result);
            }
        }
    }

    /// POW with a negative exponent must error.
    #[test]
    fn pow_negative_exponent_errors(
        a in -1000i64..=1000i64,
        b in i64::MIN..0i64,
    ) {
        let result = step_binary(OP_POW, StackItem::Integer(a), StackItem::Integer(b));
        prop_assert!(result.is_err(),
            "POW with negative exponent {} must error, got {:?}", b, result);
    }

    /// POW with a u32-overflowing exponent must error rather than OOM.
    /// We feed an `UnsignedInteger` exponent because `Integer` exponents
    /// cap at `i64::MAX` which still casts; an unsigned exponent above
    /// `u32::MAX` exercises the dedicated try_into guard in
    /// `pow_stack_items`.
    #[test]
    fn pow_huge_exponent_errors(
        a in 2i64..=10i64,
        b in (u32::MAX as u64 + 1)..u64::MAX,
    ) {
        let result = step_binary(OP_POW, StackItem::Integer(a), StackItem::UnsignedInteger(b));
        prop_assert!(result.is_err(),
            "POW with exponent > u32::MAX ({}) must error, got {:?}", b, result);
    }

    /// POW(a, 0) == 1 for any integer base in range.
    #[test]
    fn pow_zero_exponent_is_one(
        a in any::<i64>(),
    ) {
        let result = step_binary(OP_POW, StackItem::Integer(a), StackItem::Integer(0));
        let v = result.expect("POW(a, 0) must succeed");
        prop_assert_eq!(as_i64(&v), Some(1));
    }

    /// POW also accepts `UnsignedInteger` bases and matches `u64::checked_pow`.
    #[test]
    fn pow_unsigned_base_matches_reference(
        a in 0u64..=1_000_000,
        b in 0u32..=5,
    ) {
        let result = step_binary(OP_POW, StackItem::UnsignedInteger(a), StackItem::Integer(b as i64));
        match a.checked_pow(b) {
            Some(expected) => {
                let v = result.expect("POW(unsigned) must succeed");
                match v {
                    StackItem::UnsignedInteger(got) => prop_assert_eq!(got, expected),
                    other => prop_assert!(false,
                        "POW(unsigned base) returned non-unsigned variant {:?}", other),
                }
            }
            None => {
                prop_assert!(result.is_err(),
                    "POW(unsigned) overflow must error");
            }
        }
    }

    /// Non-numeric POW operands must error (invalid base / invalid exponent).
    /// Coercion path: Boolean(true) is NOT treated as 1 here — the impl
    /// rejects it with "Invalid base for POW" / "Invalid exponent for POW".
    #[test]
    fn pow_rejects_non_integer_operands(
        b in 0i64..=10,
    ) {
        let cases: Vec<StackItem> = vec![
            StackItem::Boolean(true),
            StackItem::Null,
            StackItem::byte_array(vec![1, 2, 3]),
        ];
        for op in cases {
            let r_base = step_binary(OP_POW, op.clone(), StackItem::Integer(b));
            prop_assert!(r_base.is_err(),
                "POW with non-integer base {:?} must error, got {:?}", op, r_base);
            let r_exp = step_binary(OP_POW, StackItem::Integer(2), op.clone());
            prop_assert!(r_exp.is_err(),
                "POW with non-integer exponent {:?} must error, got {:?}", op, r_exp);
        }
    }
}

// ============================================================================
// SQRT
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// SQRT of any non-negative i64 matches `(n as f64).sqrt().floor() as i64`
    /// for values up to `u32::MAX` (where f64 precision is lossless for
    /// integer sqrt). We validate the integer-newton implementation against
    /// the floating-point reference.
    #[test]
    fn sqrt_matches_floor_sqrt(
        n in 0i64..=(u32::MAX as i64),
    ) {
        let result = step_unary(OP_SQRT, StackItem::Integer(n));
        let v = result.expect("SQRT must succeed for non-negative input");
        let got = as_i64(&v).expect("SQRT result must be integer-shaped");
        let expected = (n as f64).sqrt().floor() as i64;
        prop_assert_eq!(got, expected,
            "SQRT({}) = {} (expected {})", n, got, expected);
        // Independent invariants: got*got <= n < (got+1)*(got+1).
        prop_assert!(got >= 0);
        prop_assert!(got.saturating_mul(got) <= n,
            "SQRT({}) lower bound: {}^2 = {} > {}", n, got, got.saturating_mul(got), n);
        let next = got + 1;
        prop_assert!(next.saturating_mul(next) > n || next.saturating_mul(next) < 0,
            "SQRT({}) upper bound: ({}+1)^2 = {} <= {}", n, got, next.saturating_mul(next), n);
    }

    /// SQRT of a negative input must error.
    #[test]
    fn sqrt_negative_errors(
        n in i64::MIN..0i64,
    ) {
        let result = step_unary(OP_SQRT, StackItem::Integer(n));
        prop_assert!(result.is_err(),
            "SQRT({}) must error, got {:?}", n, result);
    }

    /// Non-numeric operands must error.
    #[test]
    fn sqrt_rejects_non_integer(_dummy in any::<bool>()) {
        for op in [
            StackItem::Boolean(true),
            StackItem::Null,
            StackItem::byte_array(vec![1, 2, 3]),
        ] {
            let result = step_unary(OP_SQRT, op.clone());
            prop_assert!(result.is_err(),
                "SQRT({:?}) must error, got {:?}", op, result);
        }
    }
}

#[test]
fn sqrt_zero_is_zero() {
    let v = step_unary(OP_SQRT, StackItem::Integer(0)).expect("ok");
    assert_eq!(as_i64(&v), Some(0));
}

#[test]
fn sqrt_one_is_one() {
    let v = step_unary(OP_SQRT, StackItem::Integer(1)).expect("ok");
    assert_eq!(as_i64(&v), Some(1));
}

// ============================================================================
// SIGN
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// SIGN matches `i64::signum` across the full i64 range.
    #[test]
    fn sign_signed_matches_signum(
        v in any::<i64>(),
    ) {
        let result = step_unary(OP_SIGN, StackItem::Integer(v));
        let item = result.expect("SIGN must succeed");
        prop_assert_eq!(as_i64(&item), Some(v.signum()));
    }

    /// SIGN(unsigned) is 0 if zero, else 1.
    #[test]
    fn sign_unsigned_is_zero_or_one(
        v in any::<u64>(),
    ) {
        let result = step_unary(OP_SIGN, StackItem::UnsignedInteger(v));
        let item = result.expect("SIGN(unsigned) must succeed");
        let expected = if v == 0 { 0i64 } else { 1 };
        prop_assert_eq!(as_i64(&item), Some(expected));
    }

    /// SIGN(Boolean) is 0/1.
    #[test]
    fn sign_boolean(
        b in any::<bool>(),
    ) {
        let result = step_unary(OP_SIGN, StackItem::Boolean(b));
        let item = result.expect("SIGN(bool) must succeed");
        let expected = if b { 1i64 } else { 0 };
        prop_assert_eq!(as_i64(&item), Some(expected));
    }

    /// SIGN(Null) and SIGN(ByteArray) must error per the helper.
    #[test]
    fn sign_rejects_unsupported(_dummy in any::<bool>()) {
        for op in [
            StackItem::Null,
            StackItem::byte_array(vec![]),
            StackItem::byte_array(vec![0u8; 32]),
        ] {
            let r = step_unary(OP_SIGN, op.clone());
            prop_assert!(r.is_err(),
                "SIGN({:?}) must error, got {:?}", op, r);
        }
    }
}

// ============================================================================
// ABS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// ABS over signed integers, excluding `i64::MIN` (which is asserted
    /// separately to error).
    #[test]
    fn abs_signed_matches_wrapping_abs(
        v in (i64::MIN + 1)..=i64::MAX,
    ) {
        let result = step_unary(OP_ABS, StackItem::Integer(v));
        let item = result.expect("ABS must succeed when not i64::MIN");
        let got = as_i64(&item).expect("ABS produces an integer");
        prop_assert!(got >= 0, "ABS({}) = {} must be non-negative", v, got);
        prop_assert_eq!(got, v.wrapping_abs(),
            "ABS({}) = {} (expected {})", v, got, v.wrapping_abs());
    }

    /// ABS over unsigned integers is identity.
    #[test]
    fn abs_unsigned_identity(
        v in any::<u64>(),
    ) {
        let result = step_unary(OP_ABS, StackItem::UnsignedInteger(v));
        let item = result.expect("ABS(unsigned) must succeed");
        match item {
            StackItem::UnsignedInteger(got) => prop_assert_eq!(got, v),
            other => prop_assert!(false, "ABS(unsigned) returned non-unsigned {:?}", other),
        }
    }
}

#[test]
fn abs_i64_min_must_error() {
    let r = step_unary(OP_ABS, StackItem::Integer(i64::MIN));
    assert!(r.is_err(), "ABS(i64::MIN) must error, got {:?}", r);
}

#[test]
fn abs_rejects_non_numeric() {
    for op in [
        StackItem::Boolean(true),
        StackItem::Null,
        StackItem::byte_array(vec![]),
    ] {
        let r = step_unary(OP_ABS, op.clone());
        assert!(r.is_err(), "ABS({:?}) must error, got {:?}", op, r);
    }
}

// ============================================================================
// NEGATE
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// NEGATE(a) == -a for all `a != i64::MIN`.
    #[test]
    fn negate_signed_matches(
        v in (i64::MIN + 1)..=i64::MAX,
    ) {
        let result = step_unary(OP_NEGATE, StackItem::Integer(v));
        let item = result.expect("NEGATE must succeed when not i64::MIN");
        prop_assert_eq!(as_i64(&item), Some(-v),
            "NEGATE({}) failed", v);
    }

    /// NEGATE(unsigned u) where u <= i64::MAX yields -u as Integer.
    #[test]
    fn negate_unsigned_in_signed_range(
        v in 0u64..=(i64::MAX as u64),
    ) {
        let result = step_unary(OP_NEGATE, StackItem::UnsignedInteger(v));
        let item = result.expect("NEGATE(unsigned in range) must succeed");
        prop_assert_eq!(as_i64(&item), Some(-(v as i64)));
    }

    /// NEGATE(unsigned u) where u > i64::MAX must error (cannot fit `-u`
    /// in i64).
    #[test]
    fn negate_unsigned_out_of_range_errors(
        v in (i64::MAX as u64 + 1)..=u64::MAX,
    ) {
        let result = step_unary(OP_NEGATE, StackItem::UnsignedInteger(v));
        prop_assert!(result.is_err(),
            "NEGATE(unsigned {}) must error, got {:?}", v, result);
    }
}

#[test]
fn negate_i64_min_must_error() {
    let r = step_unary(OP_NEGATE, StackItem::Integer(i64::MIN));
    assert!(r.is_err(), "NEGATE(i64::MIN) must error, got {:?}", r);
}

#[test]
fn negate_rejects_non_numeric() {
    for op in [
        StackItem::Boolean(true),
        StackItem::Null,
        StackItem::byte_array(vec![]),
    ] {
        let r = step_unary(OP_NEGATE, op.clone());
        assert!(r.is_err(), "NEGATE({:?}) must error, got {:?}", op, r);
    }
}

// ============================================================================
// INC / DEC (under strict_arithmetic = true, the default)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// INC(a) == a + 1 for a < i64::MAX under strict mode.
    #[test]
    fn inc_signed_in_range(
        v in i64::MIN..i64::MAX,
    ) {
        let result = step_unary(OP_INC, StackItem::Integer(v));
        let item = result.expect("INC must succeed in range");
        prop_assert_eq!(as_i64(&item), Some(v + 1),
            "INC({}) failed", v);
    }

    /// DEC(a) == a - 1 for a > i64::MIN under strict mode.
    #[test]
    fn dec_signed_in_range(
        v in (i64::MIN + 1)..=i64::MAX,
    ) {
        let result = step_unary(OP_DEC, StackItem::Integer(v));
        let item = result.expect("DEC must succeed in range");
        prop_assert_eq!(as_i64(&item), Some(v - 1),
            "DEC({}) failed", v);
    }

    /// INC over UnsignedInteger in the safe range.
    #[test]
    fn inc_unsigned_in_range(
        v in 0u64..u64::MAX,
    ) {
        let result = step_unary(OP_INC, StackItem::UnsignedInteger(v));
        let item = result.expect("INC(unsigned in range) must succeed");
        match item {
            StackItem::UnsignedInteger(got) => prop_assert_eq!(got, v + 1),
            other => prop_assert!(false,
                "INC(unsigned) returned non-unsigned {:?}", other),
        }
    }

    /// DEC over UnsignedInteger in the safe range.
    #[test]
    fn dec_unsigned_in_range(
        v in 1u64..=u64::MAX,
    ) {
        let result = step_unary(OP_DEC, StackItem::UnsignedInteger(v));
        let item = result.expect("DEC(unsigned in range) must succeed");
        match item {
            StackItem::UnsignedInteger(got) => prop_assert_eq!(got, v - 1),
            other => prop_assert!(false,
                "DEC(unsigned) returned non-unsigned {:?}", other),
        }
    }
}

#[test]
fn inc_i64_max_must_error_strict() {
    let r = step_unary(OP_INC, StackItem::Integer(i64::MAX));
    assert!(
        r.is_err(),
        "INC(i64::MAX) under strict mode must error, got {:?}",
        r
    );
}

#[test]
fn dec_i64_min_must_error_strict() {
    let r = step_unary(OP_DEC, StackItem::Integer(i64::MIN));
    assert!(
        r.is_err(),
        "DEC(i64::MIN) under strict mode must error, got {:?}",
        r
    );
}

#[test]
fn inc_unsigned_max_must_error_strict() {
    let r = step_unary(OP_INC, StackItem::UnsignedInteger(u64::MAX));
    assert!(
        r.is_err(),
        "INC(u64::MAX) under strict mode must error, got {:?}",
        r
    );
}

#[test]
fn dec_unsigned_zero_must_error_strict() {
    let r = step_unary(OP_DEC, StackItem::UnsignedInteger(0));
    assert!(
        r.is_err(),
        "DEC(0u64) under strict mode must error, got {:?}",
        r
    );
}

#[test]
fn inc_rejects_non_numeric() {
    for op in [
        StackItem::Boolean(true),
        StackItem::Null,
        StackItem::byte_array(vec![]),
    ] {
        let r = step_unary(OP_INC, op.clone());
        assert!(r.is_err(), "INC({:?}) must error, got {:?}", op, r);
    }
}

#[test]
fn dec_rejects_non_numeric() {
    for op in [
        StackItem::Boolean(true),
        StackItem::Null,
        StackItem::byte_array(vec![]),
    ] {
        let r = step_unary(OP_DEC, op.clone());
        assert!(r.is_err(), "DEC({:?}) must error, got {:?}", op, r);
    }
}
