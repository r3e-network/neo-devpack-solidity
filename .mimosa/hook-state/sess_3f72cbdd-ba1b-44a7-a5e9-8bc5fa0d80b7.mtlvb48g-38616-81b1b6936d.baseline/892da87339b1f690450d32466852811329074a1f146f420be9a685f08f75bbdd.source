use super::*;

pub(crate) fn try_lower_expression_primary(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::Variable(identifier) => {
            Some(lower_variable_expression(identifier, ctx, instructions))
        }
        Expression::ArraySubscript(_, _, None) => {
            instructions.push(Instruction::PushLiteral(
                LiteralValue::ByteArray(Vec::new()),
            ));
            Some(true)
        }
        Expression::ArraySubscript(_, array, Some(index)) => {
            let ok = lower_array_subscript_expression(
                expr,
                array.as_ref(),
                index.as_ref(),
                ctx,
                instructions,
            );
            // A byte index of a `bytes`/`bytesN` value (`b[i]`) lowers to
            // PICKITEM, which yields a NeoVM Integer (the byte value 0-255). In
            // Solidity `b[i]` is a `bytes1`, so its canonical NeoVM form is a
            // 1-byte ByteString. NeoVM `EQUAL` is TYPE-STRICT (Integer != Byte-
            // String), so `b[i] == bytes1(x)` / `!=`, assigning `b[i]` to a
            // `bytes1`, or passing it as a `bytes1` argument all mis-compare on a
            // real node when `b[i]` is left as an Integer (the simulator's
            // lenient EQUAL masks it). Coerce to a 1-byte ByteString HERE, at the
            // source, so every consumer (comparison, assignment, argument,
            // return) sees the correct type. `Convert ByteArray` first so the
            // Integer (or already-ByteString) is size-able, then
            // `coerce_to_fixed_bytes(1, reverse=true)` — the same conversion the
            // `bytes1(uintN)` cast uses, correct for all byte values incl. 0x00
            // and >=0x80. Gated on the base being a contiguous bytes value
            // (`ByteArray`); array element access (base is `Array`) is untouched,
            // and `uintN(b[i])` / bitwise re-coerce the ByteString as needed.
            if ok
                && matches!(
                    infer_type_from_expression(array.as_ref(), ctx),
                    Some(ValueType::ByteArray { .. })
                )
            {
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::ByteArray,
                });
                coerce_to_fixed_bytes(1, true, ctx, instructions);
            }
            Some(ok)
        }
        Expression::ArraySlice(_, array, start, end) => Some(lower_array_slice_expression(
            array.as_ref(),
            start.as_deref(),
            end.as_deref(),
            ctx,
            instructions,
        )),
        Expression::ArrayLiteral(_, elements) => {
            Some(lower_array_literal_expression(elements, ctx, instructions))
        }
        Expression::Or(_, left, right) => Some(lower_logical_or(left, right, ctx, instructions)),
        Expression::And(_, left, right) => Some(lower_logical_and(left, right, ctx, instructions)),
        _ => None,
    }
}
