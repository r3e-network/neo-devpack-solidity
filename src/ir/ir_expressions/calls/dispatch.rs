use super::*;

pub(crate) fn lower_function_call_expression(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let mut func = func;

    // Solidity call options `foo{gas: ..., value: ...}()` are represented by solang-parser as a
    // `FunctionCall` where the callee expression is a `FunctionCallBlock`. Neo N3 does not
    // support EVM-style attached value or gas limits, but we can safely ignore `{gas: ...}` to
    // improve compatibility with Solidity sources.
    if let Expression::FunctionCallBlock(_, inner_call, block) = func {
        if let Statement::Args(_, named_args) = block.as_ref() {
            let mut unsupported = Vec::new();
            for arg in named_args {
                match arg.name.name.as_str() {
                    // Neo has no per-invocation gas limit; evaluate for side effects then ignore.
                    "gas" => {
                        if lower_expression(&arg.expr, ctx, instructions) {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }
                    // Neo has no attached value; require explicit NEP-17 transfers instead.
                    "value" => {
                        // Compatibility mode: evaluate and ignore `{value: ...}`.
                        // Neo native value transfer must be explicit via token transfer
                        // calls, but allowing this syntax improves upstream portability.
                        if lower_expression(&arg.expr, ctx, instructions) {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }
                    "salt" => unsupported.push("salt"),
                    other => unsupported.push(other),
                }
            }

            if !unsupported.is_empty() {
                ctx.record_warning_with_suggestion(
                    format!(
                        "function call options (`{{...}}`) for '{}' are ignored on Neo N3. Neo N3 requires explicit NEP-17 transfers instead of attached value.",
                        unsupported.join(", ")
                    ),
                    "Replace {value: x} with explicit NativeCalls.gasTransfer() / NativeCalls.neoTransfer() before the call if value transfer is intended.",
                );
            }

            func = inner_call.as_ref();
        } else {
            ctx.record_error(
                "function call blocks are not supported; only call options (`{...}`) are recognized",
            );
            return false;
        }
    }

    if let Some(result) = try_lower_storage_array_helpers(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_value_transfer_helpers(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_low_level_address_call(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_type_constructor_call(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_struct_constructor_call(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_builtin_call(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_member_call(func, args, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_variable_call(func, args, ctx, instructions) {
        return result;
    }

    for arg in args {
        load_expression(arg, ctx, instructions);
        instructions.push(Instruction::Drop(ValueType::Any));
    }
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    true
}
