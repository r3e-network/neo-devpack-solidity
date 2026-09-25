use super::*;

/// NeoVM-specific peephole optimization: removes redundant stack operations
pub(crate) fn neovm_peephole_optimize(block: &mut ir::BasicBlock) {
    let mut optimized = Vec::with_capacity(block.instructions.len());
    let mut i = 0;

    while i < block.instructions.len() {
        // Pattern: PUSH x followed by DROP → remove both
        if i + 1 < block.instructions.len() {
            if matches!(&block.instructions[i], ir::Instruction::PushLiteral(_))
                && matches!(&block.instructions[i + 1], ir::Instruction::Drop(_))
            {
                i += 2;
                continue;
            }

            // Pattern: LoadLocal x followed by DROP → remove both
            if matches!(&block.instructions[i], ir::Instruction::LoadLocal(_))
                && matches!(&block.instructions[i + 1], ir::Instruction::Drop(_))
            {
                i += 2;
                continue;
            }

            // Pattern: DUP followed by DROP → remove both
            if matches!(&block.instructions[i], ir::Instruction::Dup)
                && matches!(&block.instructions[i + 1], ir::Instruction::Drop(_))
            {
                i += 2;
                continue;
            }
        }

        // Pattern: StoreLocal x, LoadLocal x → DUP, StoreLocal x
        if i + 1 < block.instructions.len() {
            if let (ir::Instruction::StoreLocal(store_idx), ir::Instruction::LoadLocal(load_idx)) =
                (&block.instructions[i], &block.instructions[i + 1])
            {
                if store_idx == load_idx {
                    optimized.push(ir::Instruction::Dup);
                    optimized.push(ir::Instruction::StoreLocal(*store_idx));
                    i += 2;
                    continue;
                }
            }
        }

        // Pattern: SWAP followed by SWAP → remove both
        if i + 1 < block.instructions.len()
            && matches!(&block.instructions[i], ir::Instruction::Swap)
            && matches!(&block.instructions[i + 1], ir::Instruction::Swap)
        {
            i += 2;
            continue;
        }

        optimized.push(block.instructions[i].clone());
        i += 1;
    }

    block.instructions = optimized;
}

/// NeoVM-specific: simplify identity operations (x + 0, x * 1, x & MAX, etc.)
pub(crate) fn neovm_simplify_identity_ops(block: &mut ir::BasicBlock) {
    use num_bigint::BigInt;
    use num_traits::Zero;

    let mut optimized = Vec::with_capacity(block.instructions.len());
    let mut i = 0;

    while i < block.instructions.len() {
        // Pattern: PUSH 0, BinaryOp::Add → remove (x + 0 = x)
        if i + 1 < block.instructions.len() {
            if let ir::Instruction::PushLiteral(ir::LiteralValue::Integer(val)) =
                &block.instructions[i]
            {
                if val.is_zero() {
                    if let ir::Instruction::BinaryOp(ir::BinaryOperator::Add) =
                        &block.instructions[i + 1]
                    {
                        // Skip both instructions, identity operation
                        i += 2;
                        continue;
                    }
                    if let ir::Instruction::BinaryOp(ir::BinaryOperator::Sub) =
                        &block.instructions[i + 1]
                    {
                        // x - 0 = x, skip both
                        i += 2;
                        continue;
                    }
                    if let ir::Instruction::BinaryOp(ir::BinaryOperator::BitOr) =
                        &block.instructions[i + 1]
                    {
                        // x | 0 = x, skip both
                        i += 2;
                        continue;
                    }
                    if let ir::Instruction::BinaryOp(ir::BinaryOperator::BitXor) =
                        &block.instructions[i + 1]
                    {
                        // x ^ 0 = x, skip both
                        i += 2;
                        continue;
                    }
                }
                // Pattern: PUSH 1, MUL → identity (x * 1 = x)
                if *val == BigInt::from(1) {
                    if let ir::Instruction::BinaryOp(ir::BinaryOperator::Mul) =
                        &block.instructions[i + 1]
                    {
                        i += 2;
                        continue;
                    }
                    if let ir::Instruction::BinaryOp(ir::BinaryOperator::Div) =
                        &block.instructions[i + 1]
                    {
                        // x / 1 = x
                        i += 2;
                        continue;
                    }
                }
            }
        }

        optimized.push(block.instructions[i].clone());
        i += 1;
    }

    block.instructions = optimized;
}

/// NeoVM-specific: optimize boolean patterns
pub(crate) fn neovm_bool_optimize(block: &mut ir::BasicBlock) {
    let mut optimized = Vec::with_capacity(block.instructions.len());
    let mut i = 0;

    while i < block.instructions.len() {
        // M-BC2 fix — REMOVED the `PUSH true, EQ → identity` rewrite. For a
        // non-boolean operand (e.g. NeoVM Integer 5), `(5 == true)` must
        // evaluate to `false`, but the rewrite left `5` on the stack —
        // corrupting any downstream consumer expecting a 0/1 boolean. The
        // frontend guarantees Solidity `bool` operands for `==`, but Yul /
        // assembly-injected values or a future type-inference miss could
        // reach here with a non-bool, so the rewrite was unsound. The EQ
        // instruction is cheap; correctness wins.

        // NOTE — the `PUSH false, NE → identity` ("x != false = x") rewrite was
        // REMOVED for the same reason the symmetric `PUSH true, EQ → identity`
        // was dropped under M-BC2: for a non-boolean operand (e.g. NeoVM Integer
        // 5 written via inline assembly), `(5 != false)` must normalize to the
        // boolean `true` (1), but the rewrite left the raw `5` on the stack,
        // diverging from a real NOTEQUAL against a Boolean ABI return. The NE
        // instruction is cheap; correctness wins.

        // Pattern: BitwiseNot followed by BitwiseNot → identity (removes both)
        if i + 1 < block.instructions.len()
            && matches!(&block.instructions[i], ir::Instruction::BitwiseNot)
            && matches!(&block.instructions[i + 1], ir::Instruction::BitwiseNot)
        {
            i += 2;
            continue;
        }

        optimized.push(block.instructions[i].clone());
        i += 1;
    }

    block.instructions = optimized;
}
