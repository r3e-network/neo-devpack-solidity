use super::*;

pub(crate) fn lower_post_inc_dec(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    increment: bool,
) -> bool {
    // Post-increment/decrement must evaluate the lvalue (and any index/key
    // sub-expressions) EXACTLY ONCE — Solidity semantics. The previous lowering
    // lowered `expr` separately to read the old value AND again inside the
    // compound assignment, so `m[f()]++` / `arr[g()]++` ran the index expression
    // twice (wrong slot + duplicate side effects).
    //
    // Instead: perform the compound (which evaluates the index once, stores
    // `new = old +/- 1`, and leaves `new` on the stack), then RECOVER the old
    // value as `old = new -/+ 1`. The compound's checked guard still Panics on
    // overflow before any recovery runs.
    let one = Expression::NumberLiteral(Default::default(), "1".to_string(), "".to_string(), None);
    let op = if increment {
        BinaryOperator::Add
    } else {
        BinaryOperator::Sub
    };

    if !lower_compound_assignment(expr, &one, ctx, instructions, op) {
        return false;
    }

    // Recover `old` from `new` (on the stack). `++` -> old = new - 1; `--` -> new + 1.
    let inferred = infer_type_from_expression(expr, ctx);
    let is_u256 = matches!(
        inferred,
        Some(ValueType::Integer {
            signed: false,
            bits: 256
        })
    );
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    if is_u256 {
        // Use the limb routine so a uint256 result >= 2^255 (e.g. recovering the
        // old value of an unchecked `x++` that wrapped) does not fault on a
        // 33-byte native intermediate.
        if increment {
            emit_u256_unchecked_sub_ir(ctx, instructions);
        } else {
            emit_u256_unchecked_add_ir(ctx, instructions);
        }
    } else {
        let recover = if increment {
            BinaryOperator::Sub
        } else {
            BinaryOperator::Add
        };
        instructions.push(Instruction::BinaryOp(recover));
        // Re-truncate the recovered value to the operand width. When the
        // compound WRAPPED in an `unchecked` block (e.g. `uint8 x = 255; x++`
        // stores new = 0), the naive `new - 1` recovers -1 instead of the
        // pre-increment 255; masking restores the correct width-bounded old
        // value. (In checked mode the compound Panics before reaching here, so
        // this only ever corrects a genuine wrap; non-wrapping values are
        // unchanged by the mask.)
        match inferred {
            Some(ValueType::Integer {
                signed: false,
                bits,
            }) if bits < 256 => {
                emit_truncate_narrow_unsigned(instructions, bits);
            }
            Some(ValueType::Integer { signed: true, bits }) if bits < 256 => {
                emit_truncate_narrow_signed(ctx, instructions, bits);
            }
            _ => {}
        }
    }
    true
}

pub(crate) fn lower_pre_inc_dec(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    increment: bool,
) -> bool {
    let one = Expression::NumberLiteral(Default::default(), "1".to_string(), "".to_string(), None);
    let op = if increment {
        BinaryOperator::Add
    } else {
        BinaryOperator::Sub
    };

    if !lower_compound_assignment(expr, &one, ctx, instructions, op) {
        return false;
    }

    // lower_compound_assignment returns the updated value as the expression result.
    true
}
