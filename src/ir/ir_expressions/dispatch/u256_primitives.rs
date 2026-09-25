use super::*;

pub(crate) fn u256_push(ins: &mut Vec<Instruction>, v: BigInt) {
    ins.push(Instruction::PushLiteral(LiteralValue::Integer(v)));
}
pub(crate) fn u256_bop(ins: &mut Vec<Instruction>, op: BinaryOperator) {
    ins.push(Instruction::BinaryOp(op));
}
pub(crate) fn u256_mask128() -> BigInt {
    (BigInt::one() << 128usize) - BigInt::one()
}
pub(crate) fn u256_bias127() -> BigInt {
    BigInt::one() << 127usize
}
pub(crate) fn u256_mask64() -> BigInt {
    (BigInt::one() << 64usize) - BigInt::one()
}

pub(crate) fn emit_u256_limb_op(
    op: BinaryOperator,
    ctx: &mut LoweringContext,
    ins: &mut Vec<Instruction>,
) {
    let s = ctx.u256_scratch_locals(3);
    let (al, bl, lo) = (s[0], s[1], s[2]);
    ins.push(Instruction::StoreLocal(bl));
    ins.push(Instruction::StoreLocal(al));
    ins.push(Instruction::LoadLocal(al));
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    ins.push(Instruction::LoadLocal(bl));
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_bop(ins, op);
    ins.push(Instruction::StoreLocal(lo));
    emit_u256_hi_limb(ins, al);
    emit_u256_hi_limb(ins, bl);
    u256_bop(ins, op);
    ins.push(Instruction::LoadLocal(lo));
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_bop(ins, BinaryOperator::Add);
    emit_u256_combine_limbs(ins, lo);
}

pub(crate) fn emit_u256_unchecked_add_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    emit_u256_limb_op(BinaryOperator::Add, ctx, ins);
}

pub(crate) fn emit_u256_unchecked_sub_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    emit_u256_limb_op(BinaryOperator::Sub, ctx, ins);
}

pub(crate) fn emit_u256_hi_limb(ins: &mut Vec<Instruction>, loc: usize) {
    ins.push(Instruction::LoadLocal(loc));
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
}

pub(crate) fn emit_u256_combine_limbs(ins: &mut Vec<Instruction>, lo: usize) {
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_push(ins, u256_bias127());
    u256_bop(ins, BinaryOperator::BitXor);
    u256_push(ins, u256_bias127());
    u256_bop(ins, BinaryOperator::Sub);
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shl);
    ins.push(Instruction::LoadLocal(lo));
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_bop(ins, BinaryOperator::Add);
}
