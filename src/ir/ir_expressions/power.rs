use super::*;

pub(crate) fn lower_power_expression(
    left: &Expression,
    right: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let (Some(LiteralValue::Integer(base)), Some(LiteralValue::Integer(exp_lit))) = (
        literal_from_expression(left),
        literal_from_expression(right),
    ) {
        if let Some(exp) = exp_lit.to_u32() {
            // Constant-fold `base ** exp` at compile time, but cap the
            // iteration count so a pathological source like
            // `uint x = 2 ** 4294967295;` doesn't spin the compiler in
            // a 4-billion-iter BigInt loop. Real Solidity literals that
            // fit in a uint256 slot have `exp < 256` for `base >= 2`, so
            // any legal use is well under the cap. Above the cap we
            // fall through to the runtime-loop lowering below, which is
            // bounded by the executing contract's gas budget rather than
            // the compiler's memory. Reuses `MAX_DECIMAL_EXPONENT` from
            // `ir::ir_build` (the re-export of `build/literals.rs`) so
            // the two exponent limits stay in lockstep — a single source
            // of truth.
            if exp <= crate::ir::ir_build::MAX_DECIMAL_EXPONENT {
                let mut result = BigInt::one();
                for _ in 0..exp {
                    result *= &base;
                }
                // Only constant-fold when the result fits a 32-byte signed push.
                // A folded value in [2^255, 2^256-1] (a valid uint256) is a
                // 33-byte POSITIVE BigInt that real NeoVM rejects when pushed as
                // a literal. Route those (and any out-of-range result) to the
                // runtime loop below, which applies the width-aware checked
                // (Panic 0x11) / unchecked (mod 2^N wrap) semantics correctly.
                let two_255 = BigInt::one() << 255usize;
                if result < two_255 && result >= -(&two_255) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(result)));
                    return true;
                }
            }
            // exp > MAX_LITERAL_POW_EXP: fall through to the runtime
            // exponentiation loop below. The compiler stays bounded.
        }
    }

    let base_local = ctx.allocate_local("__pow_base".to_string(), None);
    let exp_local = ctx.allocate_local("__pow_exp".to_string(), None);
    let result_local = ctx.allocate_local("__pow_result".to_string(), None);

    if !lower_expression(left, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(base_local));

    if !lower_expression(right, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(exp_local));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::StoreLocal(result_local));

    // For 256-bit types the square-and-multiply loop uses the soft-arith 256-bit
    // multiply (mod 2^256, kept in 32-byte two's-complement form) instead of a
    // native MUL: a native MUL of an overflowing intermediate produces a >32-byte
    // BigInteger that real NeoVM FAULTS on (uncatchably), before the post-loop
    // checked/unchecked handling can run. The CHECKED variant Panics 0x11
    // (catchable) on a product >= 2^256; the unchecked one wraps. int256 is
    // handled by exponentiating the UNSIGNED magnitude |base| (so the same
    // unsigned soft-arith path serves both) and re-applying the sign afterward.
    // Narrow widths keep native MUL (their intermediates stay <= 32 bytes) + the
    // post-loop range-check/truncate ladder.
    let pow_ty = infer_type_from_expression(left, ctx);
    let pow_soft256 = matches!(pow_ty, Some(ValueType::Integer { bits: 256, .. }));
    let pow_int256 = matches!(
        pow_ty,
        Some(ValueType::Integer {
            bits: 256,
            signed: true
        })
    );
    let pow_checked = !ctx.in_unchecked_block();

    // int256: replace `base` with |base| and record whether the final result is
    // negative (base < 0 AND exp odd) — computed BEFORE the loop consumes `exp`.
    // `0 - base` (soft-arith) is the magnitude even for int256.min: 0 - (-2^255)
    // wraps to 2^255 = 0x80..00, avoiding a NEGATE-of-int_min fault.
    let result_neg_local = if pow_int256 {
        let neg = ctx.allocate_local("__pow_result_neg".to_string(), None);
        let neg_done = ctx.next_label();
        let keep_zero = ctx.next_label();
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::StoreLocal(neg));
        instructions.push(Instruction::LoadLocal(base_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Lt)); // base < 0 ?
        instructions.push(Instruction::JumpIf { target: neg_done }); // base >= 0 -> neg stays 0
                                                                     // base < 0: result_neg = 1 iff exp is odd.
        instructions.push(Instruction::LoadLocal(exp_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::one(),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Ne)); // exp odd ?
        instructions.push(Instruction::JumpIf { target: keep_zero }); // exp even -> neg stays 0
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::one(),
        )));
        instructions.push(Instruction::StoreLocal(neg));
        instructions.push(Instruction::Label(keep_zero));
        // base = |base| = 0 - base (base is < 0 here).
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::LoadLocal(base_local));
        emit_u256_unchecked_sub_ir(ctx, instructions);
        instructions.push(Instruction::StoreLocal(base_local));
        instructions.push(Instruction::Label(neg_done));
        Some(neg)
    } else {
        None
    };

    let loop_label = ctx.next_label();
    let end_label = ctx.next_label();
    let skip_mul_label = ctx.next_label();

    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(exp_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ne));
    instructions.push(Instruction::JumpIf { target: end_label });

    instructions.push(Instruction::LoadLocal(exp_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ne));
    // JumpIf branches when the condition is false; skip multiply when exp is even.
    instructions.push(Instruction::JumpIf {
        target: skip_mul_label,
    });
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::LoadLocal(base_local));
    emit_pow_mul(pow_soft256, pow_checked, ctx, instructions);
    instructions.push(Instruction::StoreLocal(result_local));

    instructions.push(Instruction::Label(skip_mul_label));
    // Square `base` ONLY while more exponent bits remain (current exp > 1).
    // The final iteration (exp == 1) does its `result *= base` above and then
    // must NOT square again: that wasted squaring can overshoot NeoVM's 32-byte
    // integer limit and FAULT on-chain even when the final result is in range
    // (e.g. `2 ** 200` — result 2^200 fits, but `(2^128)^2 = 2^256` would
    // fault). The simulator's arbitrary-precision BigInt masked this.
    let after_square_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(exp_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
    // JumpIf branches when FALSE — skip the squaring when exp <= 1.
    instructions.push(Instruction::JumpIf {
        target: after_square_label,
    });
    instructions.push(Instruction::LoadLocal(base_local));
    instructions.push(Instruction::LoadLocal(base_local));
    emit_pow_mul(pow_soft256, pow_checked, ctx, instructions);
    instructions.push(Instruction::StoreLocal(base_local));
    instructions.push(Instruction::Label(after_square_label));

    instructions.push(Instruction::LoadLocal(exp_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Shr));
    instructions.push(Instruction::StoreLocal(exp_local));

    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));

    // Solidity `**` is a CHECKED operation: the result must fit the base type
    // (the result type of `a ** b` is the type of `a`). The square-and-multiply
    // loop above computes the un-truncated BigInt, so apply the same width-aware
    // overflow handling as Add/Sub/Mul: Panic(0x11) on overflow in checked mode,
    // wrap mod 2^N in `unchecked`. Narrow widths (N<256) are the silent-overflow
    // case — `uint8 a = 2; a ** 8` yields 256, which fits NeoVM's 256-bit integer
    // but overflows uint8. (uint256/int256 powers that exceed 256 bits already
    // fault on NeoVM's integer-size limit inside the loop, so they are not
    // silently wrong.) Constant-folded literal powers are handled above.
    if let Some(ValueType::Integer { signed, bits }) = infer_type_from_expression(left, ctx) {
        if bits == 256 && !signed {
            // uint256 `**` is fully handled INSIDE the loop now: the soft-arith
            // CHECKED multiply already Panicked 0x11 on overflow, the unchecked
            // multiply already wrapped mod 2^256, and both leave `result_local`
            // as a canonical 32-byte two's-complement Integer. Nothing to do.
        } else if bits == 256 && signed {
            // int256 `**`: the loop ran on the unsigned magnitude |base|, so
            // `result_local` holds mag = |base|^exp mod 2^256 (the soft-arith
            // CHECKED multiply already Panicked if any product reached 2^256, so
            // mag < 2^256). Re-apply the sign and (checked) range-check.
            let neg = result_neg_local.expect("int256 pow tracks the result sign");
            if pow_checked {
                // int256 range: a NEGATIVE result allows |result| <= 2^255
                // (int256.min); a POSITIVE result allows |result| <= 2^255-1.
                // Detect via the high bit (mag >> 255) and, for the negative
                // boundary, whether mag is EXACTLY 2^255 (low 255 bits zero) —
                // which avoids pushing the 33-byte literal 2^255.
                let ok = ctx.next_label();
                let overflow = ctx.next_label();
                instructions.push(Instruction::LoadLocal(result_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(255u64),
                )));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Shr)); // hi: 0 if mag<2^255
                instructions.push(Instruction::JumpIf { target: ok }); // hi == 0 -> in range
                                                                       // mag >= 2^255: a positive result always overflows; a negative
                                                                       // result is in range only if mag == 2^255 (low 255 bits zero).
                instructions.push(Instruction::LoadLocal(neg));
                instructions.push(Instruction::JumpIf { target: overflow }); // neg == 0 -> overflow
                instructions.push(Instruction::LoadLocal(result_local));
                let low_mask = (BigInt::one() << 255usize) - BigInt::one(); // 0x7f.. (32-byte, safe)
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(low_mask)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Eq)); // low == 0 ?
                instructions.push(Instruction::JumpIf { target: overflow }); // low != 0 -> overflow
                instructions.push(Instruction::Jump { target: ok }); // mag == 2^255 -> int256.min
                instructions.push(Instruction::Label(overflow));
                emit_panic(0x11, instructions);
                instructions.push(Instruction::Label(ok));
            }
            // Re-apply the sign: result = result_neg ? (0 - mag) : mag.
            let sign_done = ctx.next_label();
            instructions.push(Instruction::LoadLocal(neg));
            instructions.push(Instruction::JumpIf { target: sign_done }); // neg == 0 -> keep mag
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::LoadLocal(result_local));
            emit_u256_unchecked_sub_ir(ctx, instructions); // 0 - mag
            instructions.push(Instruction::StoreLocal(result_local));
            instructions.push(Instruction::Label(sign_done));
        } else if matches!(bits, 8 | 16 | 32 | 64 | 128) {
            let bits_usize = bits as usize;
            if ctx.in_unchecked_block() {
                instructions.push(Instruction::LoadLocal(result_local));
                if signed {
                    emit_truncate_narrow_signed(ctx, instructions, bits);
                } else {
                    emit_truncate_narrow_unsigned(instructions, bits);
                }
                instructions.push(Instruction::StoreLocal(result_local));
            } else if signed {
                // checked intN: [-2^(bits-1), 2^(bits-1) - 1].
                let int_max = (BigInt::one() << (bits_usize - 1)) - BigInt::one();
                let int_min = -(BigInt::one() << (bits_usize - 1));
                let after_max = ctx.next_label();
                instructions.push(Instruction::LoadLocal(result_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(int_max)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
                instructions.push(Instruction::JumpIf { target: after_max });
                emit_panic(0x11, instructions);
                instructions.push(Instruction::Label(after_max));
                let after_min = ctx.next_label();
                instructions.push(Instruction::LoadLocal(result_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(int_min)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
                instructions.push(Instruction::JumpIf { target: after_min });
                emit_panic(0x11, instructions);
                instructions.push(Instruction::Label(after_min));
            } else {
                // checked uintN: [0, 2^bits - 1]. A power of a non-negative base
                // is always >= 0, so only the upper bound can be violated.
                let uint_max = (BigInt::one() << bits_usize) - BigInt::one();
                let after_max = ctx.next_label();
                instructions.push(Instruction::LoadLocal(result_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(uint_max)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
                instructions.push(Instruction::JumpIf { target: after_max });
                emit_panic(0x11, instructions);
                instructions.push(Instruction::Label(after_max));
            }
        }
    }

    instructions.push(Instruction::LoadLocal(result_local));
    true
}

/// Emit one square-and-multiply step `[x, y] -> [x * y]` for the pow loop.
///
/// For uint256 the native NeoVM `MUL` is replaced with the soft-arith 256-bit
/// multiply, which keeps the value in 32-byte two's-complement form so an
/// overflowing intermediate never materializes a >32-byte BigInteger (which a
/// real node faults on, uncatchably). The CHECKED variant Panics 0x11 on
/// overflow (catchable); the unchecked variant wraps mod 2^256. Narrow widths
/// keep the native `MUL` (their intermediates stay within 32 bytes) and are
/// range-checked/truncated after the loop.
fn emit_pow_mul(
    unsigned256: bool,
    checked: bool,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    if unsigned256 {
        if checked {
            emit_u256_checked_arith(ctx, instructions, BinaryOperator::Mul);
        } else {
            emit_u256_unchecked_mul_ir(ctx, instructions);
        }
    } else {
        instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    }
}
