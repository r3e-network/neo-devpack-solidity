use super::*;

pub(crate) fn try_lower_expression_unary(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::Not(_, inner) => Some(if lower_expression(inner, ctx, instructions) {
            instructions.push(Instruction::LogicalNot);
            true
        } else {
            false
        }),
        Expression::BitwiseNot(_, inner) => {
            // Solidity `~x` is the bitwise complement WITHIN the operand's bit
            // width: `~uint8(0) == 255`, `~uint16(0) == 65535`, etc. NeoVM
            // INVERT computes the full-precision two's-complement `-x-1`, so a
            // narrow/uint256 result must be re-truncated to the declared width
            // — otherwise `~uint8(0)` yields -1 and `~uint256(small)` truncates
            // to u64 via the runtime narrow-Integer path.
            let ty = infer_type_from_expression(inner, ctx);
            if !lower_expression(inner, ctx, instructions) {
                return Some(false);
            }
            match ty {
                Some(ValueType::Integer {
                    signed: false,
                    bits: 256,
                }) => {
                    // ~x for uint256 == x XOR (2^256-1). The 256-bit all-ones
                    // literal makes the runtime XOR take its wide BigInt path,
                    // which canonicalizes the result (`u256_bigint_to_stack_item`)
                    // — unlike INVERT, whose narrow negative result the
                    // zero-padding emit_truncate_u256 cannot widen correctly.
                    let max_u256: BigInt = (BigInt::one() << 256usize) - BigInt::one();
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(max_u256)));
                    instructions.push(Instruction::BinaryOp(BinaryOperator::BitXor));
                }
                Some(ValueType::Integer {
                    signed: false,
                    bits,
                }) if bits < 256 => {
                    instructions.push(Instruction::BitwiseNot);
                    emit_truncate_narrow_unsigned(instructions, bits);
                }
                Some(ValueType::Integer { signed: true, bits }) if bits < 256 => {
                    instructions.push(Instruction::BitwiseNot);
                    emit_truncate_narrow_signed(ctx, instructions, bits);
                }
                // int256: full-width `-x-1` is already the correct complement.
                // Non-integer operands keep the bare op.
                _ => {
                    instructions.push(Instruction::BitwiseNot);
                }
            }
            Some(true)
        }
        Expression::Power(_, left, right) => Some(lower_power_expression(
            left.as_ref(),
            right.as_ref(),
            ctx,
            instructions,
        )),
        Expression::UnaryPlus(_, inner) => Some(lower_expression(inner, ctx, instructions)),
        Expression::Negate(_, inner) => Some(lower_negate_expression(inner, ctx, instructions)),
        _ => None,
    }
}

/// Task #30 slice 2: unary-minus lowering with a `-type(intN).min` guard for
/// signed-integer operands. For `intN`, negating the min value produces
/// `2^(N-1)`, which is not representable — Solidity 0.8.x must revert with
/// Panic(0x11).
///
/// The guard emits:
/// 1. Evaluate the inner expression.
/// 2. If the operand is signed intN and not inside `unchecked { }` and not a
///    literal number, emit:
///    DUP
///    Push(type(intN).min)
///    Eq
///    JumpIf (branch on FALSE — skip THROW when condition FALSE (safe))
///    emit_panic(0x11)  // canonical EVM Panic(uint256) envelope
///    leaving the original operand on the stack for the subsequent negation.
/// 3. Multiply by -1 to compute the negation.
pub(crate) fn lower_negate_expression(
    inner: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Check whether to emit the guard before lowering (since inference needs
    // the pre-lowered AST).
    let emit_guard = should_emit_negate_guard(inner, ctx);
    let intn_min = signed_intn_min_literal(inner, ctx);
    // Signed operand width, needed to pick the unchecked wrap strategy below.
    let signed_bits = match infer_type_from_expression(inner, ctx) {
        Some(ValueType::Integer { signed: true, bits }) => Some(bits),
        _ => None,
    };
    let unchecked = ctx.in_unchecked_block();

    if !lower_expression(inner, ctx, instructions) {
        return false;
    }

    if emit_guard {
        if let Some(min_value) = intn_min {
            // DUP the operand so the post-check can compare it without
            // consuming the value (we still need it for the Mul below).
            instructions.push(Instruction::Dup);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(min_value)));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
            // JumpIf-on-false: safe path; if operand != intN::min, skip the
            // THROW. Fall through to THROW when operand == intN::min.
            // Task #107 — canonical EVM Panic(uint256) envelope so
            // `catch Panic(uint code)` can bind code = 0x11.
            let safe_label = ctx.next_label();
            instructions.push(Instruction::JumpIf { target: safe_label });
            emit_panic(0x11, instructions);
            instructions.push(Instruction::Label(safe_label));
        }
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(-1),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
        return true;
    }

    // UNCHECKED signed negation must WRAP two's-complement, matching EVM
    // (`unchecked { -x }` where `x == type(intN).min` yields `intN.min`, not a
    // Panic or the out-of-range positive `+2^(N-1)`). Bug-hunt #2/#25/#31.
    if unchecked {
        if let (Some(bits), Some(min_value)) = (signed_bits, intn_min.clone()) {
            if bits >= 256 {
                // int256: `-(int256.min)` mathematically is `+2^255`, which is
                // OUT of the int256 range AND exceeds NeoVM's 256-bit signed
                // integer capacity (it would fault the MUL on a real node). The
                // wrapped result is `int256.min` itself, so SUBSTITUTE it
                // directly for that one input instead of computing `x * -1`.
                // NOTE: IR `JumpIf` branches when the condition is FALSE
                // (JMPIFNOT_L), so `x == min` FALLS THROUGH and `x != min`
                // jumps to the negate path.
                //   DUP; PUSH min; EQ; JMPIFNOT negate;  (x != min -> negate)
                //   JMP end;                              (x == min -> leave x)
                //   negate: PUSH -1; MUL;
                //   end:
                let negate = ctx.next_label();
                let end = ctx.next_label();
                instructions.push(Instruction::Dup);
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(min_value)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
                instructions.push(Instruction::JumpIf { target: negate });
                instructions.push(Instruction::Jump { target: end });
                instructions.push(Instruction::Label(negate));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(-1),
                )));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
                instructions.push(Instruction::Label(end));
                return true;
            } else {
                // Narrow intN: `x * -1` is in NeoVM range (|result| <= 2^(N-1)
                // < 2^255), but `-(intN.min) = +2^(N-1)` overflows the declared
                // width. Truncate/sign-extend back into range so it wraps to
                // `intN.min`, matching the already-correct narrow `0 - x` path.
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(-1),
                )));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
                emit_truncate_narrow_signed(ctx, instructions, bits);
                return true;
            }
        }
    }

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(-1),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    true
}

/// Gate for Task #30 slice 2 unary-minus guard emission.
pub(crate) fn should_emit_negate_guard(inner: &Expression, ctx: &LoweringContext) -> bool {
    if ctx.in_unchecked_block() {
        return false;
    }
    if is_literal_number(inner) {
        return false;
    }
    matches!(
        infer_type_from_expression(inner, ctx),
        Some(ValueType::Integer { signed: true, .. })
    )
}

/// Returns the `type(intN).min` literal value for the inferred signed-int
/// type of the operand, or `None` for unsigned / unknown types.
pub(crate) fn signed_intn_min_literal(inner: &Expression, ctx: &LoweringContext) -> Option<BigInt> {
    if let Some(ValueType::Integer { signed: true, bits }) = infer_type_from_expression(inner, ctx)
    {
        // intN::min = -2^(N-1).
        let one: BigInt = BigInt::from(1);
        let shifted: BigInt = one << (bits as u32 - 1);
        Some(-shifted)
    } else {
        None
    }
}
