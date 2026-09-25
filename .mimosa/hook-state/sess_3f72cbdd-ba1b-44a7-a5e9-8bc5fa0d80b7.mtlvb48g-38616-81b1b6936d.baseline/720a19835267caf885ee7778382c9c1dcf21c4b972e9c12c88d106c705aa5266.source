use super::*;

pub(crate) fn fold_constant_binary_ops(block: &mut ir::BasicBlock) {
    let mut optimized = Vec::with_capacity(block.instructions.len());
    let mut i = 0;

    while i < block.instructions.len() {
        // Task #88 — fold `PushLiteral(const); JumpIf(target)` into an
        // unconditional Jump or a no-op. IR `JumpIf` branches when the
        // stack-top is FALSE (emitted as NeoVM JMPIFNOT_L); collapsing
        // the constant-condition guard lets the downstream terminator
        // pruner drop the dead arm's instructions, including any
        // `LoadState`/storage reads the compiler emitted for a
        // statically-unreachable `return s;` tail (see fuzz harness
        // batch35_k3_view_dead_branch_storage_read_not_eliminated).
        if i + 1 < block.instructions.len() {
            if let (ir::Instruction::PushLiteral(lit), ir::Instruction::JumpIf { target }) =
                (&block.instructions[i], &block.instructions[i + 1])
            {
                let branch_taken = match lit {
                    ir::LiteralValue::Boolean(b) => Some(!*b),
                    ir::LiteralValue::Integer(n) => Some(n.is_zero()),
                    _ => None,
                };
                match branch_taken {
                    Some(true) => {
                        // Constant condition is false → JumpIf always taken.
                        optimized.push(ir::Instruction::Jump { target: *target });
                        i += 2;
                        continue;
                    }
                    Some(false) => {
                        // Constant condition is true → JumpIf never taken.
                        i += 2;
                        continue;
                    }
                    None => {}
                }
            }
        }

        // Try constant folding for binary ops
        if i + 2 < block.instructions.len() {
            if let (
                ir::Instruction::PushLiteral(lhs),
                ir::Instruction::PushLiteral(rhs),
                ir::Instruction::BinaryOp(op),
            ) = (
                &block.instructions[i],
                &block.instructions[i + 1],
                &block.instructions[i + 2],
            ) {
                if let Some(result) = evaluate_binary_literal(lhs, rhs, *op) {
                    optimized.push(ir::Instruction::PushLiteral(result));
                    i += 3;
                    continue;
                }
            }
        }

        // Try identity elimination: x + 0, x * 1, etc.
        if i + 1 < block.instructions.len() {
            if let Some(simplified) =
                try_identity_elimination(&block.instructions[i], &block.instructions[i + 1])
            {
                if let Some(instr) = simplified {
                    optimized.push(instr);
                }
                // Skip both instructions (or just drop if None)
                i += 2;
                continue;
            }
        }

        optimized.push(block.instructions[i].clone());
        i += 1;
    }

    block.instructions = optimized;
}

/// Try to eliminate identity operations
pub(crate) fn try_identity_elimination(
    first: &ir::Instruction,
    second: &ir::Instruction,
) -> Option<Option<ir::Instruction>> {
    use ir::{
        Instruction::{BinaryOp, PushLiteral},
        LiteralValue::Integer,
    };

    match (first, second) {
        // PUSH 0, ADD -> no-op (keep original value)
        (PushLiteral(Integer(n)), BinaryOp(ir::BinaryOperator::Add)) if n.is_zero() => Some(None),
        // PUSH 1, MUL -> no-op
        (PushLiteral(Integer(n)), BinaryOp(ir::BinaryOperator::Mul)) if *n == 1.into() => {
            Some(None)
        }
        // PUSH 0, MUL is intentionally NOT folded to `PUSH 0`. The multiplicand
        // is already on the evaluation stack beneath the literal, and the
        // single-instruction replacement framework here cannot also drop it, so
        // rewriting `<x>; PUSH 0; MUL` → `<x>; PUSH 0` would leak `x` on the
        // stack (corrupting subsequent stack positions and, inside a loop,
        // eventually faulting on MAXSTACKSIZE). Leave it to normal emission
        // (`PUSH 0; MUL`), which correctly consumes `x`.
        // PUSH 1, DIV -> no-op
        (PushLiteral(Integer(n)), BinaryOp(ir::BinaryOperator::Div)) if *n == 1.into() => {
            Some(None)
        }
        _ => None,
    }
}

/// Ceiling (in bits) on the magnitude of any literal produced by constant
/// folding. Mirrors `MAX_DECIMAL_EXPONENT` in src/ir/build/literals.rs (also
/// reused by `ir::expressions::power`): without a bound, a one-line source
/// like `return 1 << 18000000000000000000;` makes the `Shl` fold allocate an
/// exabyte-scale BigInt and abort the whole
/// compiler (SIGABRT) at the default `-O2`, and even sub-abort shifts like
/// `1 << 200000000` balloon the emitted .nef to tens of megabytes. Any legal
/// uint256/int256 constant (and 512-bit intermediate products) is far below
/// this ceiling; oversized expressions are simply left to the runtime ops —
/// the exact code `-O0`/`-O1` already emit for the same source.
const MAX_FOLDED_LITERAL_BITS: u64 = 4096;

/// Decline a folded INTEGER result that does not fit any 32-byte NeoVM integer.
/// The default representable range is the union of `uint256` (stored as 32-byte
/// two's-complement, so up to `2^256-1`) and `int256` (down to `-2^255`):
/// `[-2^255, 2^256-1]`. A value outside it would be emitted as a >32-byte
/// literal that FAULTS the moment it is used as an integer on a real node
/// (`MaxSize of Integer is exceeded`) — e.g. `uint256(1 << 300)` folding to a
/// 38-byte `2^300`. Returning `None` leaves the op to the runtime lowering,
/// which applies the proper mod-2^256 wrap (unchecked) or overflow guard
/// (checked). Values INSIDE the range (incl. `type(uint256).max` and any
/// `[2^255, 2^256-1]` magnitude) still fold — `push_integer_bigint` emits them
/// as the correct 32-byte two's-complement word. Non-integer results pass through.
///
/// A1 fix — the magnitude-growing checked ops (`Add`/`Sub`/`Mul`) use a NARROWER
/// ceiling of `int256::MAX` (`2^255 - 1`) instead of `uint256::MAX`. This fold
/// runs in the optimizer over the *untyped* stack IR, so it cannot tell an
/// `int256` result of `2^255` (a real overflow that MUST be trapped by the
/// runtime `Panic(0x11)` guard) from a legal `uint256` value of `2^255`. Folding
/// either collapses the operands into one literal and the downstream guard then
/// reads the already-wrapped `0x8000…` (`int256::MIN`), so `type(int256).max + 1`
/// silently returned `int256::MIN` at the default `-O2` instead of reverting.
/// The band `[2^255, 2^256-1]` is exactly that ambiguous region: declining it for
/// Add/Sub/Mul leaves `type(uint256).max & type(uint256).max`-style non-growing
/// folds (which cannot overflow a checked op into this band) and the typed
/// uint256/int256 `<<`/`>>`/`/`/`%` soft routines — intercepted earlier at
/// lowering — untouched, while every Add/Sub/Mul that folds provably lands in
/// `int256`'s range and is therefore correct for *any* operand type.
fn decline_oversized_integer_fold(
    op: ir::BinaryOperator,
    folded: Option<ir::LiteralValue>,
) -> Option<ir::LiteralValue> {
    if let Some(ir::LiteralValue::Integer(v)) = &folded {
        let one = num_bigint::BigInt::from(1u32);
        let min_i256 = -(&one << 255u32); // -2^255
                                          // int256::MAX = 2^255 - 1 for Add/Sub/Mul; uint256::MAX = 2^256 - 1
                                          // for the non-growing ops (see the doc comment above).
        let max_int = if matches!(
            op,
            ir::BinaryOperator::Add | ir::BinaryOperator::Sub | ir::BinaryOperator::Mul
        ) {
            (&one << 255u32) - &one // 2^255 - 1
        } else {
            (&one << 256u32) - &one // 2^256 - 1
        };
        if *v > max_int || *v < min_i256 {
            return None;
        }
    }
    folded
}

pub(crate) fn evaluate_binary_literal(
    lhs: &ir::LiteralValue,
    rhs: &ir::LiteralValue,
    op: ir::BinaryOperator,
) -> Option<ir::LiteralValue> {
    use ir::LiteralValue::{Boolean, Integer};

    match (lhs, rhs) {
        (Integer(a), Integer(b)) => decline_oversized_integer_fold(
            op,
            match op {
                ir::BinaryOperator::Add => Some(Integer(a + b)),
                ir::BinaryOperator::Sub => Some(Integer(a - b)),
                ir::BinaryOperator::Mul => {
                    // The product has at most `a.bits() + b.bits()` bits; decline
                    // to fold when that would exceed the literal-size ceiling so
                    // chained huge-literal multiplies can't balloon compile-time
                    // memory or the emitted bytecode.
                    if a.bits().saturating_add(b.bits()) > MAX_FOLDED_LITERAL_BITS {
                        None
                    } else {
                        Some(Integer(a * b))
                    }
                }
                ir::BinaryOperator::Div => {
                    if b.is_zero() {
                        None
                    } else {
                        Some(Integer(a / b))
                    }
                }
                ir::BinaryOperator::Mod => {
                    if b.is_zero() {
                        None
                    } else {
                        Some(Integer(a % b))
                    }
                }
                ir::BinaryOperator::BitAnd => Some(Integer(a & b)),
                ir::BinaryOperator::BitOr => Some(Integer(a | b)),
                ir::BinaryOperator::BitXor => Some(Integer(a ^ b)),
                ir::BinaryOperator::Shl => {
                    let shift = b.to_u64()?;
                    // `a << shift` materialises `shift` extra bits of BigInt with
                    // no relation to the source size; leave oversized shifts to
                    // the runtime Shl op instead of allocating them here.
                    if a.bits().saturating_add(shift) > MAX_FOLDED_LITERAL_BITS {
                        None
                    } else {
                        Some(Integer(a << shift))
                    }
                }
                ir::BinaryOperator::Shr => {
                    let shift = b.to_u64()?;
                    Some(Integer(a >> shift))
                }
                ir::BinaryOperator::Lt => Some(Boolean(a < b)),
                ir::BinaryOperator::Le => Some(Boolean(a <= b)),
                ir::BinaryOperator::Gt => Some(Boolean(a > b)),
                ir::BinaryOperator::Ge => Some(Boolean(a >= b)),
                ir::BinaryOperator::Eq => Some(Boolean(a == b)),
                ir::BinaryOperator::Ne => Some(Boolean(a != b)),
            },
        ),
        (Boolean(a), Boolean(b)) => match op {
            ir::BinaryOperator::Eq => Some(Boolean(a == b)),
            ir::BinaryOperator::Ne => Some(Boolean(a != b)),
            _ => None,
        },
        _ => None,
    }
}

// Regression tests for the MAX_FOLDED_LITERAL_BITS guard (agent key:
// robustness). `1 << 18000000000000000000`-style folds used to abort the
// compiler with an exabyte-scale BigInt allocation at the default -O2.
// `ir_optimize.rs` includes further fragments after this one, so items from
// `labels.rs` would otherwise trip clippy::items_after_test_module.
#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod fold_literal_bits_guard_tests {
    use super::*;
    use num_bigint::BigInt;

    fn int(value: impl Into<BigInt>) -> ir::LiteralValue {
        ir::LiteralValue::Integer(value.into())
    }

    #[test]
    fn shl_declines_to_fold_oversized_shift() {
        // `1 << 18000000000000000000` — would allocate ~2.25 EB if folded.
        let result = evaluate_binary_literal(
            &int(1),
            &int(18_000_000_000_000_000_000u64),
            ir::BinaryOperator::Shl,
        );
        assert_eq!(result, None, "oversized shift must be left to runtime");

        // Sub-abort DoS shape: `1 << 200000000` used to emit a 25 MB .nef.
        let result = evaluate_binary_literal(&int(1), &int(200_000_000), ir::BinaryOperator::Shl);
        assert_eq!(result, None);
    }

    #[test]
    fn shl_still_folds_legal_uint256_constants() {
        let result = evaluate_binary_literal(&int(1), &int(255), ir::BinaryOperator::Shl)
            .expect("legal uint256 shift must keep folding");
        assert_eq!(result, int(BigInt::from(1) << 255u32));
    }

    #[test]
    fn mul_declines_to_fold_when_product_exceeds_ceiling() {
        let big = BigInt::from(1) << 4000u32; // 4001 bits
        let result = evaluate_binary_literal(&int(big.clone()), &int(big), ir::BinaryOperator::Mul);
        assert_eq!(result, None, "huge-literal product must be left to runtime");
    }

    #[test]
    fn mul_folds_only_when_the_product_fits_a_32_byte_integer() {
        // An in-range product (2^100 * 2^100 = 2^200 < 2^256) must keep folding.
        let a: BigInt = BigInt::from(1) << 100u32;
        let folded =
            evaluate_binary_literal(&int(a.clone()), &int(a.clone()), ir::BinaryOperator::Mul)
                .expect("in-range product must keep folding");
        assert_eq!(folded, int(&a * &a));

        // Two max-width uint256 operands: the 512-bit product does NOT fit
        // NeoVM's 32-byte integer and is never a valid uint256 result (a checked
        // op panics, an unchecked op wraps mod 2^256). It MUST decline to fold
        // so the runtime applies the proper wrap/guard rather than the emitter
        // baking a >32-byte literal that faults on a real node.
        let max256: BigInt = (BigInt::from(1) << 256u32) - 1;
        assert!(
            evaluate_binary_literal(&int(max256.clone()), &int(max256), ir::BinaryOperator::Mul)
                .is_none(),
            "an over-32-byte product must NOT fold"
        );

        // type(uint256).max itself (exactly 2^256-1) is a valid 32-byte value
        // and must still fold through (e.g. as `max & max`).
        let max256: BigInt = (BigInt::from(1) << 256u32) - 1;
        assert!(
            evaluate_binary_literal(
                &int(max256.clone()),
                &int(max256),
                ir::BinaryOperator::BitAnd
            )
            .is_some(),
            "in-range uint256 max must still fold"
        );
    }

    // ---- A1: magnitude-growing checked ops (Add/Sub/Mul) must NOT fold into the
    // ambiguous high band [2^255, 2^256-1], which the typeless optimizer cannot
    // tell apart from an int256 overflow (a real `Panic(0x11)` the runtime guard
    // must still observe). ----

    #[test]
    fn add_int256_max_plus_one_declines_to_fold() {
        // `type(int256).max + 1` == 2^255: legal as a uint256 magnitude but an
        // int256 overflow. The fold must decline so the runtime ADD + overflow
        // guard still reverts (before the A1 fix -O2 returned int256::MIN).
        let int_max: BigInt = (BigInt::from(1) << 255u32) - 1;
        assert_eq!(
            evaluate_binary_literal(&int(int_max), &int(1), ir::BinaryOperator::Add),
            None,
            "int256::MAX + 1 must be left to the runtime guard, not folded"
        );
    }

    #[test]
    fn add_in_int256_range_still_folds() {
        let int_max: BigInt = (BigInt::from(1) << 255u32) - 1;
        let lhs = &int_max - 10i32;
        let folded =
            evaluate_binary_literal(&int(lhs.clone()), &int(5i32), ir::BinaryOperator::Add)
                .expect("in-int256-range Add must keep folding");
        assert_eq!(folded, int(&lhs + 5i32));
    }

    #[test]
    fn sub_to_negative_still_folds_guard_still_trips() {
        // A negative constant result is in-range for int256 and folds; for a
        // checked uint this is still a value the emitted underflow guard (which
        // compares `result < 0`) correctly rejects at runtime. Shrinking is safe.
        let folded =
            evaluate_binary_literal(&int(5), &int(10), ir::BinaryOperator::Sub).expect("folds");
        assert_eq!(folded, int(BigInt::from(-5i32)));
    }

    #[test]
    fn sub_below_int256_min_declines_to_fold() {
        // `int256::MIN - 1` == -(2^255 + 1) < int256::MIN → below the window.
        let int_min: BigInt = -(BigInt::from(1) << 255u32);
        assert_eq!(
            evaluate_binary_literal(&int(int_min), &int(1), ir::BinaryOperator::Sub),
            None
        );
    }

    #[test]
    fn mul_into_high_band_declines_to_fold() {
        // 2^254 * 2 == 2^255: in the ambiguous band → decline for Mul.
        let a: BigInt = BigInt::from(1) << 254u32;
        assert_eq!(
            evaluate_binary_literal(&int(a), &int(2), ir::BinaryOperator::Mul),
            None
        );
    }

    #[test]
    fn non_growing_ops_keep_uint256_window() {
        // Shr / Div / BitAnd cannot overflow a checked op into the band, so they
        // still fold results up to 2^256-1 (unchanged from before the A1 fix).
        let max256: BigInt = (BigInt::from(1) << 256u32) - 1;
        assert_eq!(
            evaluate_binary_literal(&int(max256.clone()), &int(1), ir::BinaryOperator::Div)
                .expect("uint256::MAX / 1 folds"),
            int(max256.clone())
        );
        assert!(
            evaluate_binary_literal(&int(max256), &int(1), ir::BinaryOperator::BitAnd).is_some()
        );
    }
}
