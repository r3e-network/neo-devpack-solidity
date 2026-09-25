use super::*;

// Expression lowering.
//
// `stacker::maybe_grow` at the entry point lets the compiler handle
// pathologically nested expressions (e.g. `((((((x))))))` or `!!!!!x`
// 10k+ deep) without stack-overflowing. A fuzz-audit in April 2026
// surfaced six such cases (nested parens / unary / binop chains /
// MemberAccess chains / statement blocks) — see docs/FUZZ.md's
// "Recent bug class: DoS-via-unbounded-recursion / expansion" for
// the full catalogue. The 1-MB grow ceiling is generous: real Solidity
// parsers cap expression depth much lower so legitimate code never
// needs the extra stack.
pub(crate) fn lower_expression(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        lower_expression_inner(expr, ctx, instructions)
    })
}

pub(crate) fn lower_expression_inner(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(result) = try_lower_expression_binary_ops(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_assignments(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_unary(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_comparisons(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_primary(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_calls(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_tuple(expr, ctx, instructions) {
        return result;
    }
    if let Some(result) = try_lower_expression_conditional(expr, ctx, instructions) {
        return result;
    }

    let mut literal_warning = |message: String| ctx.record_warning(message);
    if let Some(literal) = literal_from_expression_with_warning(expr, &mut literal_warning) {
        // Warn when Solidity ether units are used -- the numeric conversion is
        // technically correct (wei=1, gwei=10^9, ether=10^18) but semantically
        // misleading on Neo N3 where the native token is GAS with 10^8 decimals.
        if has_ether_unit(expr) {
            ctx.record_error_with_suggestion(
                "ether units (wei/gwei/ether) are not applicable on Neo N3",
                "use GAS token with 10^8 decimals (1 GAS = 100_000_000 fractions)",
            );
        }
        instructions.push(Instruction::PushLiteral(literal));
        true
    } else {
        ctx.record_error_with_suggestion(
            format!("unsupported expression '{expr:?}'"),
            "Neo N3 supports: int, string, bytes, bool, address, arrays, maps, structs",
        );
        false
    }
}
