use super::*;

/// Batch-#30 H1: emit the narrow-width overflow/underflow check for uintN
/// Add/Sub/Mul. Consumes `[lhs, rhs]` and pushes the result. The runtime's
/// i64/BigInt arithmetic doesn't wrap at `uintN`, so we compute the result
/// and then range-check against `[0, 2^bits - 1]`; out-of-range → Panic(0x11).
pub(crate) fn emit_checked_arith_guard_narrow_u(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    operator: BinaryOperator,
    bits: u16,
) {
    // uintN multiply with bits >= 128 overflows NeoVM's 32-byte integer: the
    // full product `(2^bits-1)^2` needs up to `2*bits+1` signed bits (> 256),
    // so a bare MUL FAULTS UNCATCHABLY before the range-check below can run.
    // Detect overflow with a PRE-check that never materializes the full product
    // — `a != 0 && b > uintN_max / a` — emitting a catchable Panic(0x11). When
    // it does not overflow the product fits in `bits < 256` and the plain MUL is
    // safe. (bits < 128 keep the cheaper compute-then-check path below.)
    if matches!(operator, BinaryOperator::Mul) && bits >= 128 {
        emit_checked_mul_guard_wide_narrow_u(ctx, instructions, bits);
        return;
    }

    let tmp_id = ctx.next_label();
    let result_local = ctx.allocate_local(format!("__narith_res_{tmp_id}"), None);

    // uintN_max = (1 << bits) - 1, encoded as a literal BigInt.
    let uint_max = (BigInt::one() << bits as usize) - BigInt::one();

    // Compute result = lhs op rhs.
    instructions.push(Instruction::BinaryOp(operator));
    instructions.push(Instruction::StoreLocal(result_local));

    // Check: result > uintN_max → Panic(0x11). Catches Add/Mul overflow.
    // Task #107 — route through canonical EVM Panic(uint256) envelope so
    // `catch Panic(uint code)` can bind code = 0x11.
    let after_max_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(uint_max)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
    instructions.push(Instruction::JumpIf {
        target: after_max_label,
    });
    emit_panic(0x11, instructions);
    instructions.push(Instruction::Label(after_max_label));

    // Check: result < 0 → Panic(0x11). Catches Sub underflow (lhs < rhs).
    let done_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: done_label });
    emit_panic(0x11, instructions);

    // Done: push result back onto the stack.
    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::LoadLocal(result_local));
}

/// Checked uintN multiply for bits at or above 128, where a bare MUL would
/// fault on the over-32-byte full product. Operands `[a, b]`; pushes `a * b` or
/// Panics(0x11) on overflow. Overflow is detected WITHOUT the full product:
/// `a != 0 && b > (uintN_max / a)`. Both operands and `uintN_max` are positive
/// and below 2^255, so the DIV and the compare are plain (unsigned equals signed
/// here), and the surviving product is at most `uintN_max` (below 2^255), which
/// fits a 32-byte integer.
fn emit_checked_mul_guard_wide_narrow_u(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    bits: u16,
) {
    let tmp_id = ctx.next_label();
    let a_local = ctx.allocate_local(format!("__wmul_a_{tmp_id}"), None);
    let b_local = ctx.allocate_local(format!("__wmul_b_{tmp_id}"), None);
    let uint_max = (BigInt::one() << bits as usize) - BigInt::one();

    instructions.push(Instruction::StoreLocal(b_local)); // pop b
    instructions.push(Instruction::StoreLocal(a_local)); // pop a

    let a_nonzero = ctx.next_label();
    let do_mul = ctx.next_label();

    // if a == 0 -> product is 0, no overflow possible; go straight to MUL.
    // `JumpIf` branches when the condition is FALSE, so it jumps to `a_nonzero`
    // exactly when `a != 0`.
    instructions.push(Instruction::LoadLocal(a_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf { target: a_nonzero });
    instructions.push(Instruction::Jump { target: do_mul });

    // a != 0: overflow iff b > uintN_max / a.
    instructions.push(Instruction::Label(a_nonzero));
    instructions.push(Instruction::LoadLocal(b_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(uint_max)));
    instructions.push(Instruction::LoadLocal(a_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Div)); // uintN_max / a
    instructions.push(Instruction::BinaryOp(BinaryOperator::Gt)); // b > limit
    instructions.push(Instruction::JumpIf { target: do_mul }); // no overflow -> MUL
    emit_panic(0x11, instructions); // throws

    instructions.push(Instruction::Label(do_mul));
    instructions.push(Instruction::LoadLocal(a_local));
    instructions.push(Instruction::LoadLocal(b_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
}

/// Bug #16: widen the top-of-stack uint256 operand into a >8-byte ByteArray
/// whose `from_signed_bytes_le` decoding is **non-negative** — i.e. an
/// unsigned-magnitude representation. This routes the runtime through its
/// wide-BigInt arithmetic path (`cmp_needs_bigint_path` triggers when either
/// operand is a ByteArray > 8 bytes) without the signed-BigInt accident that
/// reinterprets `[0xFF; 32]` as `-1`.
///
/// Strategy:
///   1. CONVERT operand to ByteArray (signed-LE encoding from the runtime).
///   2. Append a single 0x00 byte. In LE, the LAST byte's high bit is the
///      sign bit for `from_signed_bytes_le`; a trailing 0x00 forces a
///      positive interpretation regardless of the original payload.
///
/// Stack transformation:
///   [..., x] -> [..., bytes_le(x) ++ 0x00]
pub(crate) fn emit_widen_to_u256_unsigned(instructions: &mut Vec<Instruction>) {
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![0u8])));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
}

/// Bug #16: widen both operands currently on the stack ([lhs, rhs]) using the
/// unsigned-magnitude representation (see `emit_widen_to_u256_unsigned`). Used
/// by the `unchecked { ... }` uint256 Add/Sub/Mul lowering so all four
/// optimizer levels behave the same way.
pub(crate) fn emit_widen_both_u256_unsigned(instructions: &mut Vec<Instruction>) {
    emit_widen_to_u256_unsigned(instructions);
    instructions.push(Instruction::Swap);
    emit_widen_to_u256_unsigned(instructions);
    instructions.push(Instruction::Swap);
}

/// Bug #16: post-op truncate the top-of-stack result to its low 32 bytes
/// (mod 2^256). Pairs with `emit_widen_both_u256_unsigned` for unchecked
/// uint256 Add/Sub/Mul: after the BigInt-wide op, a result like 2^256 sits
/// on the stack as a 33-byte signed-LE ByteArray — without this truncation,
/// `decode_uint_le` would yield 2^256 instead of 0.
///
/// Strategy:
///   1. CONVERT result to ByteArray.
///   2. Append 32 zero bytes (so a too-narrow result becomes ≥ 32 bytes).
///   3. SUBSTR(0, 32) — take the low 32 bytes.
///
/// Stack transformation:
///   [..., x] -> [..., bytes_le(x)[0..32]]
pub(crate) fn emit_truncate_u256(instructions: &mut Vec<Instruction>) {
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![
        0u8;
        32
    ])));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::Substr);
}

/// Task #67: emit the int256 checked-arithmetic range-check guard for
/// Add/Sub/Mul. Consumes [lhs, rhs] and pushes the result.
///
/// Strategy: rely on the runtime's true signed-BigInt arithmetic (no wrap at
/// the bigint layer — see `bigint_to_stack_item`). Compute the result, then
/// range-check against `[INT256_MIN, INT256_MAX]`; panic on out-of-range.
/// Works for all three ops because the runtime never wraps the pre-check
/// result. INT256_MIN/MAX are pushed as 32-byte signed-LE ByteArrays so the
/// `less_than` / `greater_than` helpers automatically route through the
/// BigInt comparison path.
pub(crate) fn emit_checked_arith_guard_i256(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    operator: BinaryOperator,
) {
    let tmp_id = ctx.next_label();
    let result_local = ctx.allocate_local(format!("__sarith_res_{tmp_id}"), None);

    // INT256_MAX = 2^255 - 1, signed-LE 32 bytes: [0xFF*31, 0x7F].
    let mut int256_max_bytes: Vec<u8> = vec![0xffu8; 32];
    int256_max_bytes[31] = 0x7f;
    // INT256_MIN = -2^255, signed-LE 32 bytes: [0x00*31, 0x80].
    let mut int256_min_bytes: Vec<u8> = vec![0u8; 32];
    int256_min_bytes[31] = 0x80;

    // Compute result = lhs op rhs using the native signed BigInt path.
    instructions.push(Instruction::BinaryOp(operator));
    instructions.push(Instruction::StoreLocal(result_local));

    // Check: result > INT256_MAX → Panic(0x11).
    // Task #107 — canonical EVM Panic(uint256) envelope for `catch Panic(uint)`.
    let after_max_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        int256_max_bytes,
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
    // JumpIf-on-false: safe when Gt is false.
    instructions.push(Instruction::JumpIf {
        target: after_max_label,
    });
    emit_panic(0x11, instructions);
    instructions.push(Instruction::Label(after_max_label));

    // Check: result < INT256_MIN → Panic(0x11).
    let done_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        int256_min_bytes,
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: done_label });
    emit_panic(0x11, instructions);

    // Done: push result back onto the stack.
    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::LoadLocal(result_local));
}

/// Task #154: emit the narrow signed (intN, N ∈ {8,16,32,64,128})
/// checked-arithmetic range-check guard for Add/Sub/Mul. Consumes `[lhs, rhs]`
/// and pushes the result.
///
/// Strategy mirrors `emit_checked_arith_guard_i256`: compute the op directly
/// (the runtime's wide-BigInt path does not wrap when either operand is a
/// ByteArray > 8 bytes, and the narrow i64 path fits any result of narrow-int
/// arithmetic at widths ≤ 64 bits), then range-check against
/// `[-(2^(bits-1)), 2^(bits-1) - 1]`. Bound literals are pushed as 32-byte
/// signed-LE ByteArrays so the `less_than` / `greater_than` helpers route
/// through the BigInt comparison path (see `cmp_needs_bigint_path`).
///
/// At bit widths ≤ 64 the narrow i64 arithmetic path may itself raise an
/// `ExecutionError` (strict-arithmetic mode) before we reach this guard — that
/// is fine: the caller still surfaces a fault, and the fault shape is
/// upgraded to the canonical Panic(0x11) envelope by this guard whenever the
/// BigInt path is taken (narrow int128, or narrow-int summands whose results
/// happen to overflow i64 without overflowing the Solidity-level width).
pub(crate) fn emit_checked_arith_guard_narrow_i(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    operator: BinaryOperator,
    bits: u16,
) {
    let tmp_id = ctx.next_label();
    let result_local = ctx.allocate_local(format!("__sarith_n_res_{tmp_id}"), None);

    // intN_MAX = 2^(bits-1) - 1, signed-LE 32 bytes.
    // Low `bytes_payload = bits / 8` bytes encode `2^(bits-1) - 1` =
    //   [0xff * (payload-1), 0x7f]; remaining high bytes are sign-extended
    //   zeros (positive).
    let payload = (bits as usize) / 8;
    let mut int_max_bytes: Vec<u8> = vec![0u8; 32];
    for b in int_max_bytes.iter_mut().take(payload - 1) {
        *b = 0xff;
    }
    int_max_bytes[payload - 1] = 0x7f;
    // intN_MIN = -2^(bits-1), signed-LE 32 bytes.
    // Low `payload` bytes encode `-2^(bits-1)` as [0x00 * (payload-1), 0x80];
    // remaining high bytes are sign-extended 0xff (negative).
    let mut int_min_bytes: Vec<u8> = vec![0xffu8; 32];
    for b in int_min_bytes.iter_mut().take(payload - 1) {
        *b = 0x00;
    }
    int_min_bytes[payload - 1] = 0x80;

    // Compute result = lhs op rhs.
    instructions.push(Instruction::BinaryOp(operator));
    instructions.push(Instruction::StoreLocal(result_local));

    // Check: result > intN_MAX → Panic(0x11).
    let after_max_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        int_max_bytes,
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
    instructions.push(Instruction::JumpIf {
        target: after_max_label,
    });
    emit_panic(0x11, instructions);
    instructions.push(Instruction::Label(after_max_label));

    // Check: result < intN_MIN → Panic(0x11).
    let done_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(result_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        int_min_bytes,
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: done_label });
    emit_panic(0x11, instructions);

    // Done: push result back onto the stack.
    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::LoadLocal(result_local));
}

/// Emit the conformant uint256 CHECKED `add`/`sub`/`mul`: compute the unsigned
/// result with the limb routines, and panic (0x11) on unsigned overflow/underflow
/// detected via the carry/borrow signal (NOT the old `GetSize > 32` heuristic,
/// which a two's-complement wrap defeats).
pub(crate) fn emit_checked_arith_guard(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    operator: BinaryOperator,
) {
    if matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        emit_u256_checked_arith(ctx, instructions, operator);
        return;
    }
    let tmp_id = ctx.next_label();
    let lhs_local = ctx.allocate_local(format!("__arith_lhs_{tmp_id}"), None);
    let rhs_local = ctx.allocate_local(format!("__arith_rhs_{tmp_id}"), None);
    let result_local = ctx.allocate_local(format!("__arith_res_{tmp_id}"), None);

    // Store [lhs, rhs] into locals so we can reuse them for the guard check.
    instructions.push(Instruction::StoreLocal(rhs_local));
    instructions.push(Instruction::StoreLocal(lhs_local));

    // Note on JumpIf semantics: in this compiler's IR, `JumpIf { target }`
    // branches when the popped condition is **FALSE** (lowers to NeoVM
    // JMPIFNOT_L). The guards below push the OVERFLOW condition and use
    // JumpIf to skip the THROW when overflow is FALSE (i.e. safe), falling
    // through to THROW when overflow is TRUE.
    //
    // The runtime performs wide ops via BigInt (slice 1 Part C), so arithmetic
    // is effectively arbitrary-precision — guards compare against
    // `UINT256_MAX = 2^256 - 1` to detect out-of-range results rather than
    // looking for a numeric wrap (which doesn't happen).
    let done_label = ctx.next_label();

    match operator {
        BinaryOperator::Add => {
            // Compute result = lhs + rhs.
            instructions.push(Instruction::LoadLocal(lhs_local));
            instructions.push(Instruction::LoadLocal(rhs_local));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::StoreLocal(result_local));

            // Overflow guard: the runtime adds at full BigInt width and stores
            // the raw result as a signed-LE ByteArray via
            // `bigint_to_stack_item`. For values > 2^256-1 the ByteArray exceeds
            // 32 bytes. We detect overflow by checking `GetSize(result) > 32`.
            // This avoids pushing a 33-byte UINT256_MAX literal (which real
            // NeoVM rejects — Integer max is 32 bytes).
            instructions.push(Instruction::LoadLocal(result_local));
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            instructions.push(Instruction::GetSize);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(32u64),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
            // JumpIf-on-false: if overflow is FALSE (safe), skip the THROW.
            instructions.push(Instruction::JumpIf { target: done_label });
            // Fall through: overflow is TRUE → THROW.
            emit_panic(0x11, instructions);
        }
        BinaryOperator::Sub => {
            // Pre-check: underflow iff `rhs > lhs`. Runs BEFORE the Sub so we
            // never observe a signed wrapped value.
            instructions.push(Instruction::LoadLocal(rhs_local));
            instructions.push(Instruction::LoadLocal(lhs_local));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
            // JumpIf-on-false: if underflow is FALSE (safe), skip THROW and
            // compute the Sub below.
            let safe_label = ctx.next_label();
            instructions.push(Instruction::JumpIf { target: safe_label });
            // Task #107 — canonical EVM Panic(uint256) envelope.
            emit_panic(0x11, instructions);
            instructions.push(Instruction::Label(safe_label));

            // Compute result = lhs - rhs.
            instructions.push(Instruction::LoadLocal(lhs_local));
            instructions.push(Instruction::LoadLocal(rhs_local));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
            instructions.push(Instruction::StoreLocal(result_local));
        }
        BinaryOperator::Mul => {
            // Compute result = lhs * rhs.
            instructions.push(Instruction::LoadLocal(lhs_local));
            instructions.push(Instruction::LoadLocal(rhs_local));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
            instructions.push(Instruction::StoreLocal(result_local));

            // Overflow guard: same size-check approach as Add.
            instructions.push(Instruction::LoadLocal(result_local));
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            instructions.push(Instruction::GetSize);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(32u64),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Gt));
            instructions.push(Instruction::JumpIf { target: done_label });
            emit_panic(0x11, instructions);
        }
        _ => {
            // Non-arith ops: this function should not be called; just emit the
            // operator directly to preserve semantics in case of misuse.
            instructions.push(Instruction::LoadLocal(lhs_local));
            instructions.push(Instruction::LoadLocal(rhs_local));
            instructions.push(Instruction::BinaryOp(operator));
            instructions.push(Instruction::StoreLocal(result_local));
        }
    }

    // Done: push result back onto the stack.
    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::LoadLocal(result_local));
}

/// Truncate the top-of-stack integer to `bits` low bits (unsigned): `& (2^bits-1)`.
/// Used for `unchecked` narrow Add/Sub/Mul and narrow `<<` results, which wrap
/// mod 2^bits rather than panicking.
pub(crate) fn emit_truncate_narrow_unsigned(instructions: &mut Vec<Instruction>, bits: u16) {
    let mask = (BigInt::one() << bits as usize) - BigInt::one();
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(mask)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
}

/// Truncate the top-of-stack integer to a signed `bits`-wide value: mask to the
/// low `bits` bits, then two's-complement sign-extend (subtract 2^bits when the
/// sign bit is set). Mirrors the `intN(..)` cast lowering in
/// `src/ir/expressions/calls/type_constructors.rs`.
pub(crate) fn emit_truncate_narrow_signed(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    bits: u16,
) {
    let bits = bits as usize;
    let mask = (BigInt::one() << bits) - BigInt::one();
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(mask)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));

    let tmp_id = ctx.next_label();
    let value_local = ctx.allocate_local(format!("__narrow_trunc_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(value_local));

    let sign_bit = BigInt::one() << (bits.saturating_sub(1));
    let modulus = BigInt::one() << bits;
    let positive_label = ctx.next_label();
    let end_label = ctx.next_label();

    // if value < sign_bit -> already positive (JumpIf branches when false).
    instructions.push(Instruction::LoadLocal(value_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(sign_bit)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf {
        target: positive_label,
    });

    // negative: value - 2^bits
    instructions.push(Instruction::LoadLocal(value_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(modulus)));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
    instructions.push(Instruction::Jump { target: end_label });

    // positive: value
    instructions.push(Instruction::Label(positive_label));
    instructions.push(Instruction::LoadLocal(value_local));

    instructions.push(Instruction::Label(end_label));
}
