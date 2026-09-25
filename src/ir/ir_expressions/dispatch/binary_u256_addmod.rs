use super::*;

/// Emit `addmod(a, b, m)` for uint256 operands held in the dedicated locals
/// `a_slot`, `b_slot`, `m_slot` (the caller guarantees `m != 0`). Leaves the
/// result `(a + b) mod m` on the stack and clobbers `a_slot`/`b_slot`.
///
/// The naive `(a + b) % m` is WRONG when `a + b >= 2^256`: a native NeoVM ADD
/// adds the two 32-byte words as SIGNED two's-complement and discards the carry
/// out of bit 255, so the 257th bit of the true unsigned sum is lost before the
/// modulo (e.g. `addmod(2^256-1, 2^256-1, 5)` yielded 4 instead of 0).
///
/// Correct algorithm, using only carry-safe limb arithmetic and unsigned
/// divmod/compare: reduce both operands mod m first (each becomes `< m < 2^256`),
/// add them (the limb add gives the correct low 256 bits), detect the single
/// possible carry out of bit 255, and conditionally subtract `m` once. Because
/// `a' + b' < 2m < 2^257`, one correction suffices — no recursion needed.
pub(crate) fn emit_u256_addmod_ir(
    ctx: &mut LoweringContext,
    ins: &mut Vec<Instruction>,
    a_slot: usize,
    b_slot: usize,
    m_slot: usize,
) {
    // Dedicated locals (NOT scratch): the nested divmod/limb routines reuse the
    // shared u256 scratch pool, so cross-call state must live in real locals.
    let uid = ctx.next_label();
    let s_slot = ctx.allocate_local(format!("__addmod_sum_{uid}"), None);

    // a' = a mod m ; b' = b mod m  (both now < m < 2^256)
    ins.push(Instruction::LoadLocal(a_slot));
    ins.push(Instruction::LoadLocal(m_slot));
    emit_u256_divmod_ir(ctx, ins, true);
    ins.push(Instruction::StoreLocal(a_slot));
    ins.push(Instruction::LoadLocal(b_slot));
    ins.push(Instruction::LoadLocal(m_slot));
    emit_u256_divmod_ir(ctx, ins, true);
    ins.push(Instruction::StoreLocal(b_slot));

    // s = (a' + b') mod 2^256 — limb add yields the correct low 256 bits.
    ins.push(Instruction::LoadLocal(a_slot));
    ins.push(Instruction::LoadLocal(b_slot));
    emit_u256_unchecked_add_ir(ctx, ins);
    ins.push(Instruction::StoreLocal(s_slot));

    // Subtract m once iff the true sum >= m, i.e. iff the add overflowed past
    // 2^256 (carry) OR the in-range sum is still >= m. The two comparisons each
    // yield a NeoVM Boolean (not a valid BitOr operand), so branch on them
    // separately rather than OR-combining. In this IR, JumpIf branches when the
    // condition is FALSE.
    let check_ge = ctx.next_label();
    let do_sub = ctx.next_label();
    let do_else = ctx.next_label();
    let done = ctx.next_label();

    // carry = (s <u a') — the limb add wrapped past 2^256.
    ins.push(Instruction::LoadLocal(s_slot));
    ins.push(Instruction::LoadLocal(a_slot));
    emit_u256_unsigned_compare(ins, BinaryOperator::Lt);
    ins.push(Instruction::JumpIf { target: check_ge }); // carry false -> test s>=m
    ins.push(Instruction::Jump { target: do_sub }); // carry true -> subtract

    ins.push(Instruction::Label(check_ge));
    ins.push(Instruction::LoadLocal(s_slot));
    ins.push(Instruction::LoadLocal(m_slot));
    emit_u256_unsigned_compare(ins, BinaryOperator::Ge);
    ins.push(Instruction::JumpIf { target: do_else }); // s>=m false -> result = s
                                                       // s>=m true -> fall through to subtract.

    ins.push(Instruction::Label(do_sub));
    // result = (s - m) mod 2^256 — exact, since the true (a'+b') - m is in
    // [0, m) so the low-256 limb sub recovers it.
    ins.push(Instruction::LoadLocal(s_slot));
    ins.push(Instruction::LoadLocal(m_slot));
    emit_u256_unchecked_sub_ir(ctx, ins);
    ins.push(Instruction::Jump { target: done });

    ins.push(Instruction::Label(do_else));
    ins.push(Instruction::LoadLocal(s_slot));
    ins.push(Instruction::Label(done));
}
