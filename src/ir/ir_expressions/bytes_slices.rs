use super::*;

/// Task #95: Returns true when the sliced expression's IR type is a
/// Solidity `bytes` (`ValueType::ByteArray`) — a contiguous byte-string —
/// rather than a `T[]` dynamic array (`ValueType::Array(_)`). Covers
/// `bytes memory`, `bytes calldata`, `bytes storage`, and byte-string
/// literals such as `hex"..."`.
pub(crate) fn is_bytes_slice_target(array: &Expression, ctx: &LoweringContext) -> bool {
    matches!(
        infer_type_from_expression(array, ctx),
        Some(ValueType::ByteArray { .. })
    )
}

/// Task #95: Lower `bytes b[start:end]` to a contiguous ByteString using
/// NeoVM SUBSTR (opcode 0x8C). SUBSTR stack order (bottom -> top):
/// `[bytes, index, count]` → `[bytes.sub(index, count)]`.
///
/// Both `start` and `end` are clamped to `[0, len(bytes)]` and `end` is
/// further clamped to `>= start` so out-of-range slices yield an empty
/// ByteString instead of trapping — matches Solidity's saturating
/// semantics on `bytes` views and avoids a NeoVM "SUBSTR: out of bounds"
/// fault for degenerate ranges.
pub(crate) fn lower_bytes_slice_expression(
    array: &Expression,
    start: Option<&Expression>,
    end: Option<&Expression>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Evaluate the source bytes once and stash in a local — we need the
    // length both for end-clamping and for the final SUBSTR push.
    let bytes_local = ctx.allocate_local(
        "__bytes_slice_src".to_string(),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    if !lower_expression(array, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(bytes_local));

    // len = SIZE(bytes)
    let size_local = ctx.allocate_local("__bytes_slice_size".to_string(), None);
    instructions.push(Instruction::LoadLocal(bytes_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    // Evaluate `start` (default 0) and clamp to [0, len].
    let start_local = ctx.allocate_local("__bytes_slice_start".to_string(), None);
    if let Some(start_expr) = start {
        if !lower_expression(start_expr, ctx, instructions) {
            return false;
        }
    } else {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
    }
    instructions.push(Instruction::StoreLocal(start_local));

    // NOTE on branch shape: IR `JumpIf { target }` jumps when the cond on
    // the stack is FALSE (lowers to JMPIFNOT_L). So the pattern
    //     push <cond>; JumpIf clamp; Jump done; Label clamp; <fix-up>; Label done
    // runs <fix-up> iff <cond> is false.

    // start < 0 → start = 0
    let start_clamp_lo = ctx.next_label();
    let start_clamp_lo_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf {
        target: start_clamp_lo,
    });
    instructions.push(Instruction::Jump {
        target: start_clamp_lo_done,
    });
    instructions.push(Instruction::Label(start_clamp_lo));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(start_local));
    instructions.push(Instruction::Label(start_clamp_lo_done));

    // start > len → start = len
    let start_clamp_hi = ctx.next_label();
    let start_clamp_hi_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Le));
    instructions.push(Instruction::JumpIf {
        target: start_clamp_hi,
    });
    instructions.push(Instruction::Jump {
        target: start_clamp_hi_done,
    });
    instructions.push(Instruction::Label(start_clamp_hi));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(start_local));
    instructions.push(Instruction::Label(start_clamp_hi_done));

    // Evaluate `end` (default = len) and clamp to [start, len].
    let end_local = ctx.allocate_local("__bytes_slice_end".to_string(), None);
    if let Some(end_expr) = end {
        if !lower_expression(end_expr, ctx, instructions) {
            return false;
        }
    } else {
        instructions.push(Instruction::LoadLocal(size_local));
    }
    instructions.push(Instruction::StoreLocal(end_local));

    // end > len → end = len
    let end_clamp_hi = ctx.next_label();
    let end_clamp_hi_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Le));
    instructions.push(Instruction::JumpIf {
        target: end_clamp_hi,
    });
    instructions.push(Instruction::Jump {
        target: end_clamp_hi_done,
    });
    instructions.push(Instruction::Label(end_clamp_hi));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(end_local));
    instructions.push(Instruction::Label(end_clamp_hi_done));

    // end < start → end = start (produces a zero-length slice rather than
    // a negative count that would trap SUBSTR).
    let end_clamp_lo = ctx.next_label();
    let end_clamp_lo_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf {
        target: end_clamp_lo,
    });
    instructions.push(Instruction::Jump {
        target: end_clamp_lo_done,
    });
    instructions.push(Instruction::Label(end_clamp_lo));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::StoreLocal(end_local));
    instructions.push(Instruction::Label(end_clamp_lo_done));

    // SUBSTR expects [bytes, index, count] on the stack.
    instructions.push(Instruction::LoadLocal(bytes_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
    instructions.push(Instruction::Substr);
    // SUBSTR yields a Buffer on NeoVM; coerce to ByteString so equality /
    // ABI-return layers see a canonical contiguous bytes value.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    true
}
