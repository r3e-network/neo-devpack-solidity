use super::*;

// ---- Inline software uint256 routines (32-byte two's-complement) -------------
//
// NeoVM integers are signed two's-complement capped at 32 bytes, so a `uint256`
// value >= 2^255 is stored as its negative-looking two's-complement and a native
// ADD/SUB/MUL can both fault (a 33-byte intermediate) and wrap in a way the
// `GetSize > 32` overflow heuristic cannot see. These helpers compute the
// UNSIGNED result over 128-bit (add/sub) or 64-bit (mul) limbs so no intermediate
// ever exceeds 32 bytes — the same routines validated in a now-removed
// bytecode-level prototype (`cli/bytecode/uint256_ops.rs`; see git history)
// against a faithful reference VM — emitted here as IR over a shared
// scratch-local pool.

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

/// `[a, b] -> [a <op> b mod 2^256]` over two 128-bit limbs (no 33-byte
/// intermediate). `op` is `Add` or `Sub`; both share the same limb
/// scaffolding — the only delta is the operator on the low and high
/// limbs. The carry/borrow propagates through `lo >> 128` so the high
/// limb gets the spill unconditionally (addition adds the spill,
/// subtraction folds it back into the result).
pub(crate) fn emit_u256_limb_op(
    op: BinaryOperator,
    ctx: &mut LoweringContext,
    ins: &mut Vec<Instruction>,
) {
    let s = ctx.u256_scratch_locals(3);
    let (al, bl, lo) = (s[0], s[1], s[2]);
    ins.push(Instruction::StoreLocal(bl));
    ins.push(Instruction::StoreLocal(al));
    // lo = (a & M128) <op> (b & M128)
    ins.push(Instruction::LoadLocal(al));
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    ins.push(Instruction::LoadLocal(bl));
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_bop(ins, op);
    ins.push(Instruction::StoreLocal(lo));
    // hi = (a>>128 & M128) <op> (b>>128 & M128) + (lo>>128)
    emit_u256_hi_limb(ins, al);
    emit_u256_hi_limb(ins, bl);
    u256_bop(ins, op);
    ins.push(Instruction::LoadLocal(lo));
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_bop(ins, BinaryOperator::Add);
    emit_u256_combine_limbs(ins, lo);
}

/// `[a, b] -> [a + b mod 2^256]` — thin wrapper for the Add case.
pub(crate) fn emit_u256_unchecked_add_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    emit_u256_limb_op(BinaryOperator::Add, ctx, ins);
}

/// `[a, b] -> [a - b mod 2^256]` — thin wrapper for the Sub case.
pub(crate) fn emit_u256_unchecked_sub_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    emit_u256_limb_op(BinaryOperator::Sub, ctx, ins);
}

/// Push `(loc >> 128) & M128` (the unsigned high 128-bit limb of `loc`).
pub(crate) fn emit_u256_hi_limb(ins: &mut Vec<Instruction>, loc: usize) {
    ins.push(Instruction::LoadLocal(loc));
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
}

/// Given a FULL high limb `hi` on the stack and the low sum in `lo`, leave
/// `sign_ext128(hi & M128) << 128 + (lo & M128)` — the 32-byte two's-complement
/// result, where `sign_ext128(x) = (x ^ 2^127) - 2^127`.
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

/// Run the 64-bit-limb schoolbook columns. Consumes `[a, b]`; leaves limbs
/// `a0..a3 -> s[0..3]`, `b0..b3 -> s[4..7]`, low result limbs `r0..r3 -> s[9..12]`,
/// and the carry into column 4 in `s[8]`. Returns the 15-slot scratch vector.
pub(crate) fn emit_u256_mul_columns_ir(
    ctx: &mut LoweringContext,
    ins: &mut Vec<Instruction>,
) -> Vec<usize> {
    let s = ctx.u256_scratch_locals(15);
    // 0..3 a0..a3, 4..7 b0..b3, 8 acc, 9..12 r0..r3, 13 a, 14 b
    ins.push(Instruction::StoreLocal(s[14]));
    ins.push(Instruction::StoreLocal(s[13]));
    for i in 0..4usize {
        ins.push(Instruction::LoadLocal(s[13]));
        if i > 0 {
            u256_push(ins, BigInt::from(64u32 * i as u32));
            u256_bop(ins, BinaryOperator::Shr);
        }
        u256_push(ins, u256_mask64());
        u256_bop(ins, BinaryOperator::BitAnd);
        ins.push(Instruction::StoreLocal(s[i]));
    }
    for j in 0..4usize {
        ins.push(Instruction::LoadLocal(s[14]));
        if j > 0 {
            u256_push(ins, BigInt::from(64u32 * j as u32));
            u256_bop(ins, BinaryOperator::Shr);
        }
        u256_push(ins, u256_mask64());
        u256_bop(ins, BinaryOperator::BitAnd);
        ins.push(Instruction::StoreLocal(s[4 + j]));
    }
    u256_push(ins, BigInt::zero());
    ins.push(Instruction::StoreLocal(s[8]));
    for k in 0..4usize {
        ins.push(Instruction::LoadLocal(s[8]));
        for i in 0..=k {
            let j = k - i;
            ins.push(Instruction::LoadLocal(s[i]));
            ins.push(Instruction::LoadLocal(s[4 + j]));
            u256_bop(ins, BinaryOperator::Mul);
            u256_bop(ins, BinaryOperator::Add);
        }
        ins.push(Instruction::Dup);
        u256_push(ins, u256_mask64());
        u256_bop(ins, BinaryOperator::BitAnd);
        ins.push(Instruction::StoreLocal(s[9 + k]));
        u256_push(ins, BigInt::from(64u32));
        u256_bop(ins, BinaryOperator::Shr);
        ins.push(Instruction::StoreLocal(s[8]));
    }
    s
}

/// Build the 32-byte two's-complement result from `r0..r3` (`s[9..12]`):
/// `sign_ext128(r2 + (r3<<64)) << 128 + (r0 + (r1<<64))`. Reuses `s[13]`.
pub(crate) fn emit_u256_mul_build_result_ir(ins: &mut Vec<Instruction>, s: &[usize]) {
    // lo128 = r0 + (r1 << 64) -> reuse s[13]
    ins.push(Instruction::LoadLocal(s[9]));
    ins.push(Instruction::LoadLocal(s[10]));
    u256_push(ins, BigInt::from(64u32));
    u256_bop(ins, BinaryOperator::Shl);
    u256_bop(ins, BinaryOperator::Add);
    ins.push(Instruction::StoreLocal(s[13]));
    // hi128 = r2 + (r3 << 64)
    ins.push(Instruction::LoadLocal(s[11]));
    ins.push(Instruction::LoadLocal(s[12]));
    u256_push(ins, BigInt::from(64u32));
    u256_bop(ins, BinaryOperator::Shl);
    u256_bop(ins, BinaryOperator::Add);
    // result = sign_ext128(hi128) << 128 + lo128
    u256_push(ins, u256_bias127());
    u256_bop(ins, BinaryOperator::BitXor);
    u256_push(ins, u256_bias127());
    u256_bop(ins, BinaryOperator::Sub);
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shl);
    ins.push(Instruction::LoadLocal(s[13]));
    u256_bop(ins, BinaryOperator::Add);
}

/// `[a, b] -> [a * b mod 2^256]` via 64-bit-limb schoolbook (low 256 bits).
pub(crate) fn emit_u256_unchecked_mul_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    let s = emit_u256_mul_columns_ir(ctx, ins);
    emit_u256_mul_build_result_ir(ins, &s);
}

/// Conformant uint256 CHECKED `add`/`sub`/`mul` for operands `[a, b]`. Panics
/// (0x11) on unsigned overflow/underflow. `JumpIf { target }` branches when the
/// popped condition is FALSE, so each guard pushes the OVERFLOW predicate and
/// jumps PAST the panic when it is false.
pub(crate) fn emit_u256_checked_arith(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    operator: BinaryOperator,
) {
    match operator {
        BinaryOperator::Add => {
            // result = a + b (mod 2^256); overflow iff result <u a.
            emit_u256_unchecked_add_ir(ctx, instructions); // [result], scratch s[0]=a
            let s = ctx.u256_scratch_locals(3);
            instructions.push(Instruction::StoreLocal(s[2])); // res (reuse lo slot)
            instructions.push(Instruction::LoadLocal(s[2]));
            instructions.push(Instruction::LoadLocal(s[0])); // a
            emit_u256_unsigned_compare(instructions, BinaryOperator::Lt); // [res <u a]
            let done = ctx.next_label();
            instructions.push(Instruction::JumpIf { target: done });
            emit_panic(0x11, instructions);
            instructions.push(Instruction::Label(done));
            instructions.push(Instruction::LoadLocal(s[2]));
        }
        BinaryOperator::Sub => {
            // underflow iff a <u b; else result = a - b.
            let s = ctx.u256_scratch_locals(3);
            instructions.push(Instruction::StoreLocal(s[1])); // b
            instructions.push(Instruction::StoreLocal(s[0])); // a
            instructions.push(Instruction::LoadLocal(s[0]));
            instructions.push(Instruction::LoadLocal(s[1]));
            emit_u256_unsigned_compare(instructions, BinaryOperator::Lt); // [a <u b]
            let safe = ctx.next_label();
            instructions.push(Instruction::JumpIf { target: safe });
            emit_panic(0x11, instructions);
            instructions.push(Instruction::Label(safe));
            instructions.push(Instruction::LoadLocal(s[0]));
            instructions.push(Instruction::LoadLocal(s[1]));
            emit_u256_unchecked_sub_ir(ctx, instructions); // [result]
        }
        BinaryOperator::Mul => {
            // overflow iff any high-column term or the column-3 carry is non-zero.
            let s = emit_u256_mul_columns_ir(ctx, instructions);
            instructions.push(Instruction::LoadLocal(s[8])); // acc (carry into col 4)
            for (i, j) in [(1usize, 3usize), (2, 2), (3, 1), (2, 3), (3, 2), (3, 3)] {
                instructions.push(Instruction::LoadLocal(s[i]));
                instructions.push(Instruction::LoadLocal(s[4 + j]));
                u256_bop(instructions, BinaryOperator::Mul);
                u256_bop(instructions, BinaryOperator::Add);
            }
            // [high]; overflow iff high != 0.
            let no_overflow = ctx.next_label();
            instructions.push(Instruction::JumpIf {
                target: no_overflow,
            }); // jumps if high == 0
            emit_panic(0x11, instructions);
            instructions.push(Instruction::Label(no_overflow));
            emit_u256_mul_build_result_ir(instructions, &s);
        }
        _ => {
            // Only Add/Sub/Mul are supported. The sole caller
            // (`emit_checked_arith_guard`) filters the operator with
            // `matches!(Add|Sub|Mul)`, so this arm is unreachable today.
            // Under `panic = "abort"` a future caller that skipped that
            // guard would abort the process; degrade to a diagnostic
            // instead of crashing. The operand stack is left unchanged;
            // the recorded error stops this compilation.
            ctx.record_error(format!(
                "internal: emit_u256_checked_arith only handles Add/Sub/Mul, got {operator:?}"
            ));
        }
    }
}

/// Emit `<op>` for two operands already on the stack (`[.., lhs, rhs]`), applying
/// the full Solidity-0.8 checked-arithmetic / unchecked-truncation ladder:
/// uint256, int256, narrow uintN, narrow intN overflow guards (checked mode);
/// mod-2^256 / mod-2^N truncation (`unchecked` mode); narrow `<<` width
/// truncation; plain op otherwise. Shared by `lower_binary_expr` and the
/// compound-assignment / ++/-- paths so `x <op>= y`, `x++`, `--x` are
/// byte-for-byte consistent with `x = x <op> y`. The gate predicates inspect the
/// Emit an UNSIGNED 256-bit comparison for operands already on the stack as
/// `[.., a, b]`. Uses the order-preserving map `x -> x ^ 2^255`, after which a
/// native (signed) compare yields the unsigned result. `2^255` is pushed as a
/// `uint256` literal, which lowers to the 32-byte two's-complement sign bit.
pub(crate) fn emit_u256_unsigned_compare(
    instructions: &mut Vec<Instruction>,
    operator: BinaryOperator,
) {
    let sign_bit: BigInt = BigInt::one() << 255usize; // 2^255
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        sign_bit.clone(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::BitXor)); // [a, b^S]
    instructions.push(Instruction::Swap); // [b^S, a]
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(sign_bit)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::BitXor)); // [b^S, a^S]
    instructions.push(Instruction::Swap); // [a^S, b^S]
    instructions.push(Instruction::BinaryOp(operator)); // (a^S) <s (b^S) == a <u b
}

/// Emit a LOGICAL (zero-filling) uint256 right shift for operands `[a, n]`
/// (n on top). Native NeoVM SHR is arithmetic, so for a uint256 `a >= 2^255`
/// (stored as a 32-byte two's-complement word) the sign bit propagates. Solidity
/// `>>` on an unsigned type is logical, reproduced as:
///   n == 0  ->  a
///   n >= 1  ->  ((a >>arith 1) & (2^255-1)) >>arith (n-1)
/// The `& (2^255-1)` clears the bit the first arithmetic shift pushed into
/// position 255, turning the whole sequence into a zero-fill. (Mirrors a
/// bytecode-level `emit_uint256_logical_shr` from the now-removed
/// cli/bytecode/uint256_ops.rs; see git history.)
/// Uses scratch slots s[0..1]; it performs only native shift/and/sub ops (no
/// nested limb routines), so it cannot collide with an in-flight u256 op.
pub(crate) fn emit_u256_logical_shr_ir(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let scratch = ctx.u256_scratch_locals(2);
    let n_local = scratch[0];
    let a_local = scratch[1];
    instructions.push(Instruction::StoreLocal(n_local)); // pop n
    instructions.push(Instruction::StoreLocal(a_local)); // pop a

    let zero_label = ctx.next_label();
    let nonzero_label = ctx.next_label();
    let end_label = ctx.next_label();

    // n >=u 256 -> result 0. Besides being the Solidity answer, this guard is
    // REQUIRED for on-chain conformance: the recursive form below executes a
    // native `SHR (n-1)`, and a shift amount > 256 trips the node's
    // `ExecutionEngineLimits.AssertShift` and FAULTS.
    instructions.push(Instruction::LoadLocal(n_local));
    u256_push(instructions, BigInt::from(256u32));
    emit_u256_unsigned_compare(instructions, BinaryOperator::Ge); // [n >=u 256]
                                                                  // JumpIf branches on FALSE -> fall through to the zero path when n >= 256.
    instructions.push(Instruction::JumpIf {
        target: nonzero_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::Jump { target: end_label });

    // if n == 0 -> result = a
    instructions.push(Instruction::Label(nonzero_label));
    instructions.push(Instruction::LoadLocal(n_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    // JumpIf -> JMPIFNOT: jump to the n>=1 path when (n == 0) is FALSE.
    instructions.push(Instruction::JumpIf { target: zero_label });
    instructions.push(Instruction::LoadLocal(a_local));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(zero_label));
    let max_int256: BigInt = (BigInt::one() << 255usize) - BigInt::one(); // 2^255 - 1
    instructions.push(Instruction::LoadLocal(a_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Shr)); // a >>arith 1
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(max_int256)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd)); // logical (a>>1)
    instructions.push(Instruction::LoadLocal(n_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub)); // n - 1
    instructions.push(Instruction::BinaryOp(BinaryOperator::Shr)); // >> (n-1)
    instructions.push(Instruction::Label(end_label));
}

/// Emit Solidity `a << n` for a 256-bit left operand `[a, n]` (n on top).
/// Solidity's `<<` NEVER reverts: it is `a * 2^n` reduced mod 2^256, with the
/// result reinterpreted in the operand's type — so uint256 and int256 share the
/// exact same bit operation. Native NeoVM `SHL` instead computes the
/// full-precision product and FAULTS as soon as it leaves the 32-byte integer
/// range (any result >= 2^255, or a shift amount > 256). Reproduce the wrapping
/// semantics as `a * 2^n mod 2^256` through the conformant unchecked-mul limb
/// routine, clamping `n >= 256` to 0. Uses fresh locals for `n`/`a`/`pow2`; the
/// values are on the stack before the nested mul runs, so its scratch pool
/// cannot clobber them.
pub(crate) fn emit_u256_shl_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    let id = ctx.next_label();
    let n_local = ctx.allocate_local(format!("__shl_n_{id}"), None);
    let a_local = ctx.allocate_local(format!("__shl_a_{id}"), None);
    let pow_local = ctx.allocate_local(format!("__shl_pow_{id}"), None);
    ins.push(Instruction::StoreLocal(n_local)); // pop n
    ins.push(Instruction::StoreLocal(a_local)); // pop a

    let compute = ctx.next_label();
    let small_n = ctx.next_label();
    let have_pow = ctx.next_label();
    let end = ctx.next_label();

    // n >= 256 (UNSIGNED) -> result 0. `JumpIf` branches when the predicate is
    // FALSE, so it jumps to `compute` when n < 256 and falls through to push 0
    // otherwise. An unsigned compare is required because a shift amount
    // >= 2^255 is stored as a negative-looking word.
    ins.push(Instruction::LoadLocal(n_local));
    u256_push(ins, BigInt::from(256u32));
    emit_u256_unsigned_compare(ins, BinaryOperator::Ge); // [n >=u 256]
    ins.push(Instruction::JumpIf { target: compute });
    u256_push(ins, BigInt::zero());
    ins.push(Instruction::Jump { target: end });

    // n in [0, 255]: pow2 = 2^n.
    ins.push(Instruction::Label(compute));
    // `1 << 255` overflows NeoVM's signed 32-byte integer, so special-case it
    // with the literal 2^255 (the uint256 literal path already emits it as the
    // 0x80…00 two's-complement sign word). n in [0, 254] uses a native SHL.
    ins.push(Instruction::LoadLocal(n_local));
    u256_push(ins, BigInt::from(255u32));
    u256_bop(ins, BinaryOperator::Eq); // [n == 255]
    ins.push(Instruction::JumpIf { target: small_n }); // jump when n != 255
    u256_push(ins, BigInt::one() << 255usize); // pow2 = 2^255
    ins.push(Instruction::Jump { target: have_pow });

    ins.push(Instruction::Label(small_n));
    u256_push(ins, BigInt::one());
    ins.push(Instruction::LoadLocal(n_local));
    u256_bop(ins, BinaryOperator::Shl); // 1 << n (n <= 254 -> result <= 2^254, safe)

    ins.push(Instruction::Label(have_pow));
    ins.push(Instruction::StoreLocal(pow_local));
    // result = a * pow2 mod 2^256
    ins.push(Instruction::LoadLocal(a_local));
    ins.push(Instruction::LoadLocal(pow_local));
    emit_u256_unchecked_mul_ir(ctx, ins); // [result]
    ins.push(Instruction::Label(end));
}

/// Emit Solidity `a >> n` for a SIGNED 256-bit left operand `[a, n]` (n on
/// top). Solidity signed `>>` is arithmetic; native NeoVM SHR on BigInteger is
/// arithmetic too, so for shift amounts <= 256 the raw opcode is conformant.
/// A shift amount > 256 trips the node's `ExecutionEngineLimits.AssertShift`
/// and FAULTS, while Solidity yields `a < 0 ? -1 : 0`, so large amounts are
/// short-circuited with that value. `n` is compared UNSIGNED because an
/// amount >= 2^255 is stored as a negative-looking two's-complement word.
pub(crate) fn emit_i256_arith_shr_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    let id = ctx.next_label();
    let n_local = ctx.allocate_local(format!("__ishr_n_{id}"), None);
    let a_local = ctx.allocate_local(format!("__ishr_a_{id}"), None);
    ins.push(Instruction::StoreLocal(n_local)); // pop n
    ins.push(Instruction::StoreLocal(a_local)); // pop a

    let native_path = ctx.next_label();
    let zero_case = ctx.next_label();
    let end = ctx.next_label();

    // n >=u 257 -> native SHR would FAULT; result = a <s 0 ? -1 : 0.
    // (n == 256 stays native: arithmetic shift of a 32-bit word by 256 gives
    // exactly -1 / 0, matching Solidity.)
    ins.push(Instruction::LoadLocal(n_local));
    u256_push(ins, BigInt::from(257u32));
    emit_u256_unsigned_compare(ins, BinaryOperator::Ge); // [n >=u 257]
                                                         // JumpIf branches on FALSE -> jump to the native path when n < 257.
    ins.push(Instruction::JumpIf {
        target: native_path,
    });

    ins.push(Instruction::LoadLocal(a_local));
    u256_push(ins, BigInt::zero());
    ins.push(Instruction::BinaryOp(BinaryOperator::Lt)); // signed: a < 0
    ins.push(Instruction::JumpIf { target: zero_case }); // a >= 0 -> 0
    ins.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(-1),
    )));
    ins.push(Instruction::Jump { target: end });
    ins.push(Instruction::Label(zero_case));
    ins.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    ins.push(Instruction::Jump { target: end });

    ins.push(Instruction::Label(native_path));
    ins.push(Instruction::LoadLocal(a_local));
    ins.push(Instruction::LoadLocal(n_local));
    ins.push(Instruction::BinaryOp(BinaryOperator::Shr));
    ins.push(Instruction::Label(end));
}

// Emit unsigned `a / b` (or `a % b` if `want_remainder`) for uint256 operands
// `[a, b]`. Native NeoVM DIV/MOD are signed, so they are wrong for operands at
// or above 2^255. Reduction (Hacker's Delight 9-3): when the divisor is at or
// above 2^255 the quotient is 0 or 1 by an unsigned compare; otherwise reduce to
// one signed DIV/MOD on the provably-non-negative `(a>>1, b)` and correct by one
// step. The limb-unsafe steps (`2t`, `2*rem`, `r-b`, `a-b`) reuse the inline
// add/sub helpers (which use scratch slots `s[0..3]`), so divmod keeps its own
// state in `s[8..15]`. Caller guarantees `b != 0` (div/mod-by-zero panics upstream).
pub(crate) fn emit_u256_divmod_ir(
    ctx: &mut LoweringContext,
    ins: &mut Vec<Instruction>,
    want_remainder: bool,
) {
    let s = ctx.u256_scratch_locals(15);
    let (a, b, q, r, m, t, rem) = (s[8], s[9], s[10], s[11], s[12], s[13], s[14]);
    let max_int256 = (BigInt::one() << 255usize) - BigInt::one();
    ins.push(Instruction::StoreLocal(b));
    ins.push(Instruction::StoreLocal(a));

    let big_b = ctx.next_label();
    let done = ctx.next_label();
    // jump to big_b when b < 0 (i.e. b >= 2^255 unsigned) == NOT (b >= 0).
    ins.push(Instruction::LoadLocal(b));
    u256_push(ins, BigInt::zero());
    u256_bop(ins, BinaryOperator::Ge);
    ins.push(Instruction::JumpIf { target: big_b });

    // ---- small divisor: b in [1, 2^255) ----
    // m = (a >>arith 1) & (2^255-1)   [logical shift right by 1]
    ins.push(Instruction::LoadLocal(a));
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::Shr);
    u256_push(ins, max_int256.clone());
    u256_bop(ins, BinaryOperator::BitAnd);
    ins.push(Instruction::StoreLocal(m));
    // t = m / b ; rem = m % b   (both non-negative -> signed == unsigned)
    ins.push(Instruction::LoadLocal(m));
    ins.push(Instruction::LoadLocal(b));
    u256_bop(ins, BinaryOperator::Div);
    ins.push(Instruction::StoreLocal(t));
    ins.push(Instruction::LoadLocal(m));
    ins.push(Instruction::LoadLocal(b));
    u256_bop(ins, BinaryOperator::Mod);
    ins.push(Instruction::StoreLocal(rem));
    // q = 2t
    ins.push(Instruction::LoadLocal(t));
    ins.push(Instruction::LoadLocal(t));
    emit_u256_unchecked_add_ir(ctx, ins);
    ins.push(Instruction::StoreLocal(q));
    // r = 2*rem + (a & 1)
    ins.push(Instruction::LoadLocal(rem));
    ins.push(Instruction::LoadLocal(rem));
    emit_u256_unchecked_add_ir(ctx, ins);
    ins.push(Instruction::LoadLocal(a));
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_bop(ins, BinaryOperator::Add);
    ins.push(Instruction::StoreLocal(r));
    // if r >=u b: q = q+1; r = r-b
    ins.push(Instruction::LoadLocal(r));
    ins.push(Instruction::LoadLocal(b));
    emit_u256_unsigned_compare(ins, BinaryOperator::Ge);
    let skip_corr = ctx.next_label();
    ins.push(Instruction::JumpIf { target: skip_corr });
    ins.push(Instruction::LoadLocal(q));
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::Add);
    ins.push(Instruction::StoreLocal(q));
    ins.push(Instruction::LoadLocal(r));
    ins.push(Instruction::LoadLocal(b));
    emit_u256_unchecked_sub_ir(ctx, ins);
    ins.push(Instruction::StoreLocal(r));
    ins.push(Instruction::Label(skip_corr));
    ins.push(Instruction::Jump { target: done });

    // ---- big divisor: b >= 2^255 ----
    ins.push(Instruction::Label(big_b));
    // q = (a >=u b) ? 1 : 0
    ins.push(Instruction::LoadLocal(a));
    ins.push(Instruction::LoadLocal(b));
    emit_u256_unsigned_compare(ins, BinaryOperator::Ge);
    ins.push(Instruction::StoreLocal(q));
    // r = q == 1 ? a - b : a
    ins.push(Instruction::LoadLocal(q));
    let q_zero = ctx.next_label();
    let big_done = ctx.next_label();
    ins.push(Instruction::JumpIf { target: q_zero });
    ins.push(Instruction::LoadLocal(a));
    ins.push(Instruction::LoadLocal(b));
    emit_u256_unchecked_sub_ir(ctx, ins);
    ins.push(Instruction::StoreLocal(r));
    ins.push(Instruction::Jump { target: big_done });
    ins.push(Instruction::Label(q_zero));
    ins.push(Instruction::LoadLocal(a));
    ins.push(Instruction::StoreLocal(r));
    ins.push(Instruction::Label(big_done));

    ins.push(Instruction::Label(done));
    ins.push(Instruction::LoadLocal(if want_remainder { r } else { q }));
}

/// Build a canonical 32-byte uint256 from four 64-bit limbs and leave it on
/// the stack: `sign_ext128(l2 + (l3<<64)) << 128 + ((l0 + (l1<<64)) & M128)`.
/// Mirrors `emit_u256_mul_build_result_ir` / `emit_u256_combine_limbs` but
/// takes explicit limb slots so the 512-bit mul can reuse it for BOTH the
/// low and high 256-bit halves. `tmp` is a scratch slot for the lo128
/// intermediate (written and read within this routine only).
fn emit_u256_build_256_ir(
    ins: &mut Vec<Instruction>,
    l0: usize,
    l1: usize,
    l2: usize,
    l3: usize,
    tmp: usize,
) {
    // lo128 = l0 + (l1 << 64)  (< 2^128, fits, no 33-byte intermediate)
    ins.push(Instruction::LoadLocal(l0));
    ins.push(Instruction::LoadLocal(l1));
    u256_push(ins, BigInt::from(64u32));
    u256_bop(ins, BinaryOperator::Shl);
    u256_bop(ins, BinaryOperator::Add);
    ins.push(Instruction::StoreLocal(tmp));
    // hi128 = l2 + (l3 << 64); sign_ext128(hi128) << 128 + (lo128 & M128)
    ins.push(Instruction::LoadLocal(l2));
    ins.push(Instruction::LoadLocal(l3));
    u256_push(ins, BigInt::from(64u32));
    u256_bop(ins, BinaryOperator::Shl);
    u256_bop(ins, BinaryOperator::Add);
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_push(ins, u256_bias127());
    u256_bop(ins, BinaryOperator::BitXor);
    u256_push(ins, u256_bias127());
    u256_bop(ins, BinaryOperator::Sub);
    u256_push(ins, BigInt::from(128u32));
    u256_bop(ins, BinaryOperator::Shl);
    ins.push(Instruction::LoadLocal(tmp));
    u256_push(ins, u256_mask128());
    u256_bop(ins, BinaryOperator::BitAnd);
    u256_bop(ins, BinaryOperator::Add);
}

/// `[a, b, m] -> [(a*b) mod m]` with the FULL 512-bit intermediate product.
///
/// Solidity `mulmod(a, b, m)` requires `a*b` to be computed at 512-bit width
/// before reducing mod m; the native NeoVM `MUL` truncates to 256 bits, so
/// any `mulmod` whose product can reach `2^256` (always possible for uint256
/// operands) is wrong on the naive path.
///
/// Algorithm — bit-serial MSB-first shift/subtract (simple & correct; an
/// optimization to Knuth Algorithm D can follow if the runtime gas cost of
/// the 512-iteration loop ever matters):
///
/// 1. P = a*b via 64-bit-limb schoolbook (8 columns -> r0..r7), packed into
///    two 256-bit halves PHI (high) and PLO (low). The product of two
///    `< 2^256` values is strictly `< 2^512`, so columns 0..7 suffice and
///    the column-7 carry-out is zero (ignored).
/// 2. R = 0; repeat 512 times (one product bit per iteration, MSB first):
///
///    ```text
///    bit      = bit 255 of PHI                  (the next product bit)
///    carry_lo = bit 255 of PLO
///    PLO      = (PLO + PLO) mod 2^256           (left shift, MSB dropped)
///    PHI      = ((PHI + PHI) mod 2^256) | carry_lo
///    carry_R  = bit 255 of R
///    low      = ((R + R) mod 2^256) | bit
///    if carry_R OR (low >=u m): R = low - m
///    else                     : R = low
///    ```
///
/// 3. R in [0, m) is the result.
///
/// Every `+` is the limb-based `emit_u256_unchecked_add_ir` (NOT a native
/// `<< 1`): the runtime auto-narrows small positive results to an i64, so a
/// native `<< 1` would store `2^63` as `i64::MIN` (negative) and corrupt the
/// next iteration's BigInt bitwise/shift ops. The limb-based add always
/// emits the canonical 32-byte two's-complement word. Likewise `low - m`
/// goes through `emit_u256_unchecked_sub_ir`: native SUB does not mask to
/// 256 bits, so a result in `[2^255, 2^256)` would surface as a 33-byte
/// positive ByteArray. Bit-255 extraction `(X >> 255) & 1` and `low >=u m`
/// are reliable on canonical operands. The wrapping `mod 2^256` is exactly
/// the high-bit-discarding left shift we need (the dropped bit was already
/// captured as the corresponding `carry`/`bit`).
///
/// Why `low - m` is always the correct next remainder (even when `low < m`):
/// the true shifted value is `R' = carry_R*2^256 + low`. If `carry_R = 1`
/// then `R' >= 2^256 > m` and `low < m` holds, so the wrapping 256-bit
/// subtract `low - m` yields `low - m + 2^256 = R' - m`. If `carry_R = 0`
/// and `low >=u m` the plain subtract `low - m` is `R' - m`. In both cases
/// the result is the correct `< m` remainder.
///
/// `low - m` goes through `emit_u256_unchecked_sub_ir` (NOT native SUB):
/// the runtime's arithmetic SUB path does not mask to 256 bits, so a result
/// in `[2^255, 2^256)` would surface as a 33-byte positive ByteArray and
/// corrupt the next iteration. The limb-based unchecked-sub always emits the
/// canonical 32-byte two's-complement word. `low >=u m` uses the slotless
/// `emit_u256_unsigned_compare`; all `& (2^255-1)`, `<< 1`, `| bit`, `>> 255`
/// ops are canonicalized to 32 bytes by the runtime's bitwise/shift handlers.
///
/// SCRATCH PARTITION (shared `u256_scratch_locals` pool, grown to index 29).
/// Loop-persistent state lives in `s[3..10]` so it survives the in-loop
/// `emit_u256_unchecked_sub_ir` call, which transiently reuses `s[0..3]`:
///   s[3]=m  s[4]=R  s[5]=PHI  s[6]=PLO  s[7]=CTR
///   s[8]=T_bit  s[9]=T_carry  s[10]=T_low
/// Preamble (512-bit mul) temporaries occupy `s[11..29]` (and `s[0]` as a
/// one-shot combine temp) and are dead once PHI/PLO are written. No other
/// u256 helper is invoked inside the loop, so there is no nested-slot
/// collision.
pub(crate) fn emit_u256_mulmod_512bit_ir(ctx: &mut LoweringContext, ins: &mut Vec<Instruction>) {
    let s = ctx.u256_scratch_locals(30);
    // Loop-persistent state (kept out of s[0..3] so unchecked_sub is safe).
    let m = s[3];
    let r = s[4];
    let phi = s[5];
    let plo = s[6];
    let ctr = s[7];
    let t_bit = s[8];
    let t_carry = s[9];
    let t_low = s[10];
    // Preamble temporaries (dead after PHI/PLO are built).
    let a = s[11];
    let b = s[12];
    let a0 = s[13];
    let a1 = s[14];
    let a2 = s[15];
    let a3 = s[16];
    let b0 = s[17];
    let b1 = s[18];
    let b2 = s[19];
    let b3 = s[20];
    let acc = s[21];
    let r0 = s[22];
    let r1 = s[23];
    let r2 = s[24];
    let r3 = s[25];
    let r4 = s[26];
    let r5 = s[27];
    let r6 = s[28];
    let r7 = s[29];
    let lo128_tmp = s[0]; // preamble-only; unchecked_sub is not called until the loop.

    let mask64 = u256_mask64();
    let a_limbs = [a0, a1, a2, a3];
    let b_limbs = [b0, b1, b2, b3];
    let r_limbs = [r0, r1, r2, r3, r4, r5, r6, r7];

    // ---- pop operands: entry stack [a, b, m] (m on top) ----
    ins.push(Instruction::StoreLocal(m));
    ins.push(Instruction::StoreLocal(b));
    ins.push(Instruction::StoreLocal(a));

    // ---- extract 64-bit limbs a0..a3, b0..b3 ----
    for (i, &slot) in a_limbs.iter().enumerate() {
        ins.push(Instruction::LoadLocal(a));
        if i > 0 {
            u256_push(ins, BigInt::from(64u32 * i as u32));
            u256_bop(ins, BinaryOperator::Shr);
        }
        u256_push(ins, mask64.clone());
        u256_bop(ins, BinaryOperator::BitAnd);
        ins.push(Instruction::StoreLocal(slot));
    }
    for (j, &slot) in b_limbs.iter().enumerate() {
        ins.push(Instruction::LoadLocal(b));
        if j > 0 {
            u256_push(ins, BigInt::from(64u32 * j as u32));
            u256_bop(ins, BinaryOperator::Shr);
        }
        u256_push(ins, mask64.clone());
        u256_bop(ins, BinaryOperator::BitAnd);
        ins.push(Instruction::StoreLocal(slot));
    }

    // ---- 8-column schoolbook: r_k = (acc + Σ a[i]*b[k-i]) & M64, acc = carry ----
    u256_push(ins, BigInt::zero());
    ins.push(Instruction::StoreLocal(acc));
    for (k, &rslot) in r_limbs.iter().enumerate() {
        ins.push(Instruction::LoadLocal(acc));
        // i ranges over the valid partial-product indices for column k
        // (empty for k == 7: column 7 is carry-only).
        let lo = k.saturating_sub(3);
        let hi = k.min(3);
        #[allow(clippy::needless_range_loop)]
        for i in lo..=hi {
            let j = k - i;
            ins.push(Instruction::LoadLocal(a_limbs[i]));
            ins.push(Instruction::LoadLocal(b_limbs[j]));
            u256_bop(ins, BinaryOperator::Mul);
            u256_bop(ins, BinaryOperator::Add);
        }
        // [col_sum] on stack; split into low 64 bits (r_k) and carry (new acc).
        ins.push(Instruction::Dup);
        u256_push(ins, mask64.clone());
        u256_bop(ins, BinaryOperator::BitAnd);
        ins.push(Instruction::StoreLocal(rslot));
        u256_push(ins, BigInt::from(64u32));
        u256_bop(ins, BinaryOperator::Shr);
        ins.push(Instruction::StoreLocal(acc));
    }
    // acc is now the column-7 carry-out; product < 2^512 guarantees it is 0.

    // ---- pack r0..r3 -> PLO, r4..r7 -> PHI (canonical 32-byte words) ----
    emit_u256_build_256_ir(ins, r0, r1, r2, r3, lo128_tmp);
    ins.push(Instruction::StoreLocal(plo));
    emit_u256_build_256_ir(ins, r4, r5, r6, r7, lo128_tmp);
    ins.push(Instruction::StoreLocal(phi));

    // ---- R = 0, CTR = 512 ----
    u256_push(ins, BigInt::zero());
    ins.push(Instruction::StoreLocal(r));
    u256_push(ins, BigInt::from(512u32));
    ins.push(Instruction::StoreLocal(ctr));

    // ---- 512-iteration bit-serial reduction ----
    let loop_top = ctx.next_label();
    let body = ctx.next_label();
    let check_ge = ctx.next_label();
    let subtract = ctx.next_label();
    let keep = ctx.next_label();
    let after = ctx.next_label();
    let end = ctx.next_label();

    ins.push(Instruction::Label(loop_top));
    // Continue while CTR != 0. JumpIf branches when FALSE => jump to body
    // exactly when (CTR == 0) is false.
    ins.push(Instruction::LoadLocal(ctr));
    u256_push(ins, BigInt::zero());
    u256_bop(ins, BinaryOperator::Eq);
    ins.push(Instruction::JumpIf { target: body });
    ins.push(Instruction::Jump { target: end });

    ins.push(Instruction::Label(body));
    ins.push(Instruction::LoadLocal(ctr));
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::Sub);
    ins.push(Instruction::StoreLocal(ctr));

    // bit = MSB(PHI) = (PHI >> 255) & 1
    ins.push(Instruction::LoadLocal(phi));
    u256_push(ins, BigInt::from(255u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::BitAnd);
    ins.push(Instruction::StoreLocal(t_bit));

    // carry_lo = (PLO >> 255) & 1
    ins.push(Instruction::LoadLocal(plo));
    u256_push(ins, BigInt::from(255u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::BitAnd);
    ins.push(Instruction::StoreLocal(t_carry));

    // PLO = (PLO + PLO) mod 2^256   (== PLO<<1 with MSB dropped = carry_lo)
    // Done via the limb-based unchecked-add so the result is the canonical
    // 32-byte two's-complement word; a native `<< 1` would, for narrow
    // Integer intermediates, store 2^63 as i64::MIN (negative) and corrupt
    // the next iteration's BigInt bitwise/shift ops.
    ins.push(Instruction::LoadLocal(plo));
    ins.push(Instruction::LoadLocal(plo));
    emit_u256_unchecked_add_ir(ctx, ins);
    ins.push(Instruction::StoreLocal(plo));

    // PHI = ((PHI + PHI) mod 2^256) | carry_lo   (PHI<<1 with MSB=bit dropped)
    ins.push(Instruction::LoadLocal(phi));
    ins.push(Instruction::LoadLocal(phi));
    emit_u256_unchecked_add_ir(ctx, ins);
    ins.push(Instruction::LoadLocal(t_carry));
    u256_bop(ins, BinaryOperator::BitOr);
    ins.push(Instruction::StoreLocal(phi));

    // carry_R = (R >> 255) & 1
    ins.push(Instruction::LoadLocal(r));
    u256_push(ins, BigInt::from(255u32));
    u256_bop(ins, BinaryOperator::Shr);
    u256_push(ins, BigInt::one());
    u256_bop(ins, BinaryOperator::BitAnd);
    ins.push(Instruction::StoreLocal(t_carry));

    // low = ((R + R) mod 2^256) | bit   (R<<1 with MSB=carry_R dropped)
    ins.push(Instruction::LoadLocal(r));
    ins.push(Instruction::LoadLocal(r));
    emit_u256_unchecked_add_ir(ctx, ins);
    ins.push(Instruction::LoadLocal(t_bit));
    u256_bop(ins, BinaryOperator::BitOr);
    ins.push(Instruction::StoreLocal(t_low));

    // need_sub = carry_R OR (low >=u m). carry_R truthy => subtract; else ge.
    ins.push(Instruction::LoadLocal(t_carry));
    ins.push(Instruction::JumpIf { target: check_ge }); // jump when carry_R == 0
    ins.push(Instruction::Jump { target: subtract });
    ins.push(Instruction::Label(check_ge));
    ins.push(Instruction::LoadLocal(t_low));
    ins.push(Instruction::LoadLocal(m));
    emit_u256_unsigned_compare(ins, BinaryOperator::Ge);
    ins.push(Instruction::JumpIf { target: keep }); // jump when (low >=u m) is false
                                                    // fall through (low >=u m) into the subtract branch.
    ins.push(Instruction::Label(subtract));
    ins.push(Instruction::LoadLocal(t_low));
    ins.push(Instruction::LoadLocal(m));
    emit_u256_unchecked_sub_ir(ctx, ins); // [low - m], canonical 32-byte word
    ins.push(Instruction::StoreLocal(r));
    ins.push(Instruction::Jump { target: after });
    ins.push(Instruction::Label(keep));
    ins.push(Instruction::LoadLocal(t_low));
    ins.push(Instruction::StoreLocal(r));
    ins.push(Instruction::Label(after));

    ins.push(Instruction::Jump { target: loop_top });
    ins.push(Instruction::Label(end));

    ins.push(Instruction::LoadLocal(r));
}

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
