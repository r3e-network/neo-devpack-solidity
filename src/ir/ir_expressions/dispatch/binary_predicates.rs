use super::*;

/// True if the expression is a literal number (number/hex). Used to avoid
/// emitting the overflow guard when both operands are compile-time constants —
/// the compiler's constant folder already handles those.
pub(crate) fn is_literal_number(expr: &Expression) -> bool {
    matches!(
        match expr {
            Expression::Parenthesis(_, inner) => inner.as_ref(),
            other => other,
        },
        Expression::NumberLiteral(..)
            | Expression::HexNumberLiteral(..)
            | Expression::RationalNumberLiteral(..)
    )
}

/// True when `expr` is the compile-time integer literal `0`, unwrapping
/// parentheses and integer type casts (`int256(0)`, `int8(0)`, `uint256(0)`).
/// Used to recognise the `0 - x` negation idiom so it can share the unary
/// negate wrap path (bug-hunt #2). A hex/decimal literal whose digits are all
/// zero counts; anything runtime-computed does not.
pub(crate) fn is_zero_literal(expr: &Expression) -> bool {
    match expr {
        Expression::Parenthesis(_, inner) => is_zero_literal(inner),
        Expression::FunctionCall(_, func, args)
            if args.len() == 1
                && matches!(
                    func.as_ref(),
                    Expression::Type(_, PtType::Int(_)) | Expression::Type(_, PtType::Uint(_))
                ) =>
        {
            is_zero_literal(&args[0])
        }
        Expression::NumberLiteral(_, integer, exp, _) => {
            integer.trim_start_matches('0').is_empty()
                && exp.trim_start_matches(['+', '-', '0']).is_empty()
        }
        Expression::HexNumberLiteral(_, value, _) => value
            .trim_start_matches("0x")
            .trim_start_matches("0X")
            .trim_start_matches('0')
            .is_empty(),
        _ => false,
    }
}

/// Infer whether an operand is `uint256` (256-bit unsigned integer). Defaults
/// to `true` for literal numbers because Solidity number literals have type
/// `uint256` by default.
pub(crate) fn is_uint256_operand(expr: &Expression, ctx: &LoweringContext) -> bool {
    match infer_type_from_expression(expr, ctx) {
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }) => true,
        Some(_) => false,
        // Unknown type (complex expressions): assume uint256 for literal
        // numbers, otherwise bail out.
        None => is_literal_number(expr),
    }
}

/// True only when `expr` is a *genuinely-typed* `uint256` operand. A bare
/// integer literal infers as `uint256` by default (see `infer_type_from_expression`
/// in `src/ir/build/inference.rs`), so it is explicitly excluded here: in
/// `narrowVar OP literal` the literal adapts to the narrow operand's type and
/// must NOT count as a real `uint256` that suppresses the narrow guard.
pub(crate) fn is_typed_uint256(expr: &Expression, ctx: &LoweringContext) -> bool {
    !is_literal_number(expr)
        && matches!(
            infer_type_from_expression(expr, ctx),
            Some(ValueType::Integer {
                signed: false,
                bits: 256,
            })
        )
}

/// Infer whether an operand is `int256` (256-bit signed integer).
pub(crate) fn is_int256_operand(expr: &Expression, ctx: &LoweringContext) -> bool {
    matches!(
        infer_type_from_expression(expr, ctx),
        Some(ValueType::Integer {
            signed: true,
            bits: 256,
        })
    )
}

/// True when the arithmetic's RESULT type is a narrow integer (uintN/intN, N<256)
/// — i.e. a narrow-typed operand is present AND no operand is a genuinely-typed
/// `uint256`/`int256`. This distinguishes pure-narrow arithmetic (`uint8 + uint8`,
/// `uint8 + literal`), which the narrow guard / narrow truncation owns, from
/// MIXED-width arithmetic (`uint256 + uint32`), where Solidity widens the narrow
/// operand so the result is `uint256` and the 256-bit path must own it. Untyped
/// literals adapt to the narrow operand's type and never count as wide here.
pub(crate) fn is_narrow_result(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
) -> bool {
    if is_typed_uint256(left, ctx) || is_typed_uint256(right, ctx) {
        return false;
    }
    if is_int256_operand(left, ctx) || is_int256_operand(right, ctx) {
        return false;
    }
    narrow_unsigned_bits(left, right, ctx).is_some()
        || narrow_signed_bits(left, right, ctx).is_some()
}

/// Task #30 slice 2: gate for `uint256` Add/Sub/Mul overflow-guard emission.
/// Returns `true` when the binary op needs a Solidity-0.8.x checked-arithmetic
/// panic guard emitted around it:
///   - operator is Add/Sub/Mul
///   - at least one operand is `uint256`
///   - not inside an `unchecked { ... }` block
///   - not both operands are compile-time literals (constant-folded)
pub(crate) fn should_emit_u256_arith_guard(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> bool {
    if ctx.in_unchecked_block() {
        return false;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return false;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return false;
    }
    // Task #67: when either operand is typed `int256`, Solidity's type-coercion
    // rules widen the other operand (typically a literal) to `int256`. The
    // int256 guard path owns the lowering in that case.
    if is_int256_operand(left, ctx) || is_int256_operand(right, ctx) {
        return false;
    }
    // `narrowVar OP literal` (and `uintN OP uintN`) is narrow arithmetic — the
    // untyped literal adapts to the narrow operand's type. Without this, the
    // literal's `uint256` default would route `uint8 x; x + 1` through the
    // 256-bit guard (which only checks `result > 2^256-1` and never trips for a
    // narrow overflow like 255+1=256), silently skipping the required
    // Panic(0x11). The narrow guard owns it. NOTE: mixed `uint256 OP uintN` is
    // NOT narrow (the narrow operand widens to uint256) and stays on this path.
    if is_narrow_result(left, right, ctx) {
        return false;
    }
    is_uint256_operand(left, ctx) || is_uint256_operand(right, ctx)
}

/// Batch-#30 H1: detect narrow unsigned integer operands (uint8/16/32/64/128)
/// so we can emit a width-aware post-op overflow check. The runtime performs
/// arithmetic through the i64/BigInt stack-item paths which DON'T wrap at the
/// narrow Solidity width; `uint8(255) + uint8(1)` evaluates to `256` on the
/// stack instead of Panic(0x11). Returns the bit width when BOTH operand
/// types are the same narrow unsigned width (Solidity rejects mixed-width
/// arithmetic at parse time unless one side is an untyped literal, in which
/// case we rely on the typed operand's width).
pub(crate) fn narrow_unsigned_bits(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
) -> Option<u16> {
    pub(crate) fn narrow_bits(expr: &Expression, ctx: &LoweringContext) -> Option<u16> {
        match infer_type_from_expression(expr, ctx) {
            Some(ValueType::Integer {
                signed: false,
                bits,
            }) if bits < 256 && bits % 8 == 0 => Some(bits),
            _ => None,
        }
    }
    narrow_bits(left, ctx).or_else(|| narrow_bits(right, ctx))
}

/// Batch-#30 H1: gate for narrow unsigned (uintN, N in {8,16,32,64,128})
/// Add/Sub/Mul overflow-guard emission. Same preconditions as
/// `should_emit_u256_arith_guard` except it fires for narrow widths instead
/// of 256-bit. Returns the target bit width when a guard should be emitted.
pub(crate) fn should_emit_narrow_u_arith_guard(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> Option<u16> {
    if ctx.in_unchecked_block() {
        return None;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return None;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return None;
    }
    // Don't clash with the uint256 or int256 guards. Use the *typed* uint256
    // check (not `is_uint256_operand`) so a literal partner — which defaults to
    // uint256 — to a narrow-typed operand does NOT suppress the narrow guard.
    if is_typed_uint256(left, ctx) || is_typed_uint256(right, ctx) {
        return None;
    }
    if is_int256_operand(left, ctx) || is_int256_operand(right, ctx) {
        return None;
    }
    narrow_unsigned_bits(left, right, ctx)
}

/// Task #154: detect narrow signed integer operands (int8/16/32/64/128) so we
/// can emit a width-aware post-op overflow check. The runtime performs
/// arithmetic through the i64/BigInt stack-item paths which DON'T wrap at the
/// narrow Solidity width; `int128(type(int128).max) + int128(1)` evaluates to
/// `2^127` on the stack instead of Panic(0x11). Returns the bit width when
/// BOTH operand types are the same narrow signed width (Solidity rejects
/// mixed-width arithmetic at parse time unless one side is an untyped literal,
/// in which case we rely on the typed operand's width).
pub(crate) fn narrow_signed_bits(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
) -> Option<u16> {
    pub(crate) fn narrow_bits(expr: &Expression, ctx: &LoweringContext) -> Option<u16> {
        match infer_type_from_expression(expr, ctx) {
            Some(ValueType::Integer { signed: true, bits }) if bits < 256 && bits % 8 == 0 => {
                Some(bits)
            }
            _ => None,
        }
    }
    narrow_bits(left, ctx).or_else(|| narrow_bits(right, ctx))
}

/// Task #154: gate for narrow signed (intN, N in {8,16,32,64,128}) Add/Sub/Mul
/// overflow-guard emission. Mirrors `should_emit_narrow_u_arith_guard` for the
/// signed domain. Returns the target bit width when a guard should be emitted.
pub(crate) fn should_emit_narrow_i_arith_guard(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> Option<u16> {
    if ctx.in_unchecked_block() {
        return None;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return None;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return None;
    }
    // Don't clash with the uint256 or int256 guards. Use the *typed* uint256
    // check so a literal partner (uint256 by default) to a narrow-typed operand
    // doesn't suppress the narrow signed guard.
    if is_typed_uint256(left, ctx) || is_typed_uint256(right, ctx) {
        return None;
    }
    if is_int256_operand(left, ctx) || is_int256_operand(right, ctx) {
        return None;
    }
    narrow_signed_bits(left, right, ctx)
}

/// Task #30 slice 4 / Task #67: gate for `int256` Add/Sub/Mul overflow-guard
/// emission. The runtime performs wide arithmetic via `BigInt` at true signed
/// precision (`to_signed_bytes_le`/`from_signed_bytes_le`), so post-op range
/// checks against `INT256_MIN..=INT256_MAX` observe the un-wrapped result and
/// can detect overflow/underflow directly. Accepted when either operand is
/// typed `int256` (literal numbers on one side inherit the typed operand's
/// width via Solidity's implicit conversion rules).
pub(crate) fn should_emit_i256_arith_guard(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> bool {
    if ctx.in_unchecked_block() {
        return false;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return false;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return false;
    }
    is_int256_operand(left, ctx) || is_int256_operand(right, ctx)
}

/// Bug #16: gate for unchecked uint256 Add/Sub/Mul. Returns `true` when:
///   - in an `unchecked { ... }` block (so the checked guard does NOT fire);
///   - operator is Add/Sub/Mul;
///   - at least one operand is `uint256`;
///   - not both operands are compile-time literals (constant-folded);
///   - not an int256 op (those flow through their own guard).
///
/// When this fires, the lowering widens both operands to a >8-byte unsigned
/// representation so the runtime's wide-BigInt arithmetic path takes over —
/// avoiding the i64 narrow-path strict-overflow fault that diverges from
/// constant-folded results at higher optimizer levels.
pub(crate) fn should_widen_unchecked_u256(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> bool {
    if !ctx.in_unchecked_block() {
        return false;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return false;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return false;
    }
    // Pure-narrow `unchecked` arithmetic truncates mod 2^N (N<256), not mod
    // 2^256; the narrow unchecked-truncation path owns it. Without this, the
    // literal's uint256 default would widen `unchecked { uint8 x; x + 1 }` to
    // 256-bit and truncate mod 2^256 (yielding 256 instead of the correct 0).
    // Mixed `uint256 OP uintN` is NOT narrow (the narrow operand widens) and
    // stays on this 256-bit widen+truncate path.
    if is_narrow_result(left, right, ctx) {
        return false;
    }
    // int256 is accepted too (feature audit): `unchecked { int256 }` Add/Sub/
    // Mul must wrap two's-complement mod 2^256, but previously fell through to
    // the bare native op, whose unbounded-BigInt result never wraps (max+1
    // yielded the un-representable +2^255 instead of int256.min). The limb
    // routines mask operands mod 2^256 (a negative input is its unsigned
    // alias) and emit the canonical sign-extended 32-byte result via
    // `emit_u256_combine_limbs`, so the same emitters produce the correct
    // signed wrap for free.
    is_uint256_operand(left, ctx)
        || is_uint256_operand(right, ctx)
        || is_int256_operand(left, ctx)
        || is_int256_operand(right, ctx)
}

/// `unchecked` narrow unsigned (uintN, N<256) Add/Sub/Mul truncation gate. Same
/// preconditions as the checked narrow guard but for `unchecked` blocks, where
/// the result wraps mod 2^N instead of panicking. Returns the bit width.
pub(crate) fn should_truncate_unchecked_narrow_u(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> Option<u16> {
    if !ctx.in_unchecked_block() {
        return None;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return None;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return None;
    }
    if is_typed_uint256(left, ctx) || is_typed_uint256(right, ctx) {
        return None;
    }
    if is_int256_operand(left, ctx) || is_int256_operand(right, ctx) {
        return None;
    }
    narrow_unsigned_bits(left, right, ctx)
}

/// `unchecked` narrow signed (intN, N<256) Add/Sub/Mul truncation gate.
pub(crate) fn should_truncate_unchecked_narrow_i(
    left: &Expression,
    right: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> Option<u16> {
    if !ctx.in_unchecked_block() {
        return None;
    }
    if !matches!(
        operator,
        BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul
    ) {
        return None;
    }
    if is_literal_number(left) && is_literal_number(right) {
        return None;
    }
    if is_typed_uint256(left, ctx) || is_typed_uint256(right, ctx) {
        return None;
    }
    if is_int256_operand(left, ctx) || is_int256_operand(right, ctx) {
        return None;
    }
    narrow_signed_bits(left, right, ctx)
}

/// In Solidity the result of `a << b` has the type of the left operand and is
/// truncated to that width (shift overflow never panics, in checked OR unchecked
/// mode). The runtime/optimizer don't wrap at narrow widths, so `uint8(200) << 1`
/// leaves `400` on the stack instead of `144`. Returns `(bits, signed)` of the
/// left operand when it is a narrow integer (N<256); uint256/int256 are already
/// clamped to 256 bits by the runtime SHL path.
pub(crate) fn shl_narrow_truncation(
    left: &Expression,
    ctx: &LoweringContext,
    operator: BinaryOperator,
) -> Option<(u16, bool)> {
    if !matches!(operator, BinaryOperator::Shl) {
        return None;
    }
    match infer_type_from_expression(left, ctx) {
        Some(ValueType::Integer { signed, bits }) if bits < 256 && bits % 8 == 0 => {
            Some((bits, signed))
        }
        _ => None,
    }
}
