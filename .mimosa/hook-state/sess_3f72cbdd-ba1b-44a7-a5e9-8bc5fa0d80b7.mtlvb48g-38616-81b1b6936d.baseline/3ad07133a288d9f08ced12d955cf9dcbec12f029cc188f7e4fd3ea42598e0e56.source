use super::*;

pub(crate) fn validate_runtime_notify_call(args: &[Expression], ctx: &mut LoweringContext) {
    if args.len() != 2 {
        return;
    }

    let Some(event_name) = extract_string_literal(&args[0]) else {
        return;
    };

    let Some(expected_sig) = ctx.event_signature(&event_name).map(|sig| sig.to_vec()) else {
        ctx.record_error(format!(
            "Runtime.notify refers to event '{event_name}' which is not declared; declare a matching Solidity `event` so it is included in the contract manifest ABI"
        ));
        return;
    };

    let Some(encoded_args) = extract_abi_encode_args(&args[1]) else {
        return;
    };

    if expected_sig.len() != encoded_args.len() {
        ctx.record_error(format!(
            "Runtime.notify event '{event_name}' expects {} argument(s), but abi.encode(...) provides {}",
            expected_sig.len(),
            encoded_args.len()
        ));
        return;
    }

    for (index, (expected_ty, arg_expr)) in expected_sig
        .iter()
        .copied()
        .zip(encoded_args.iter())
        .enumerate()
    {
        if expected_ty == ManifestType::Any {
            continue;
        }

        let Some(actual_ty) = infer_type_from_expression(arg_expr, ctx) else {
            continue;
        };

        if value_type_satisfies_manifest_type(&actual_ty, expected_ty) == Some(false) {
            ctx.record_error(format!(
                "Runtime.notify event '{event_name}' argument #{} has incompatible type (expected {}, got {:?})",
                index,
                manifest_type_name(expected_ty),
                actual_ty
            ));
        }
    }
}

/// Task #106 — when an expression passed to `abi.encodeCall` / `abi.encode` is
/// a bare Variable reference to a struct-typed function parameter, return the
/// parameter's arg slot and its direct field count `(param_slot, field_count)`
/// so the caller can expand the struct into its fields for the encoded payload.
/// On Neo a struct argument arrives as a single `Array` StackItem (the manifest
/// declares it as one `Array` parameter), so field `i` is read with
/// `LoadParameter(param_slot); PushLiteral(i); ArrayGet` (PICKITEM). This lets
/// `abi.encodeCall(this.f, (p))` for `p = P{uint256 a; bool b}` lower to
/// `sel || abiEncode(p.a, p.b)` instead of `sel || abiEncode(p)` (which would
/// produce zero-slot output for the struct-as-Array shape).
///
/// Returns `None` if `expr` is not a bare Variable reference, the variable
/// is not a function parameter, or the parameter is not a struct type. The
/// caller should then fall through to the standard `lower_expression`
/// path.
pub(crate) fn resolve_struct_param_flat_slots(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<(usize, usize)> {
    // Look through parentheses.
    let mut current = expr;
    while let Expression::Parenthesis(_, inner) = current {
        current = inner.as_ref();
    }
    let Expression::Variable(id) = current else {
        return None;
    };
    let &param_index = ctx.param_index_map.get(&id.name)?;
    let param_ty = ctx.param_types.get(param_index)?;
    // Only expand struct-typed params. Single-level expansion: a struct's
    // direct fields are read by index from its Array; nested structs would
    // require recursive expansion, which is a follow-up.
    let ValueType::Struct { fields, .. } = param_ty else {
        return None;
    };
    // The struct param occupies exactly ONE arg slot (its nominal index); the
    // caller indexes into the loaded Array for each field.
    Some((param_index, fields.len()))
}

/// Emit the field-expansion of a struct parameter loaded as a single `Array`:
/// for each of `field_count` direct fields, push `param[i]` via
/// `LoadParameter(param_slot); PushLiteral(i); ArrayGet` (PICKITEM). Shared by
/// the `abi.encodeCall` / `abi.encode` struct-arg paths so they stay consistent
/// with the single-`Array` calling convention (the manifest declares the struct
/// as one `Array` parameter and INITSLOT reserves one slot for it).
pub(crate) fn emit_struct_param_field_loads(
    instructions: &mut Vec<Instruction>,
    param_slot: usize,
    field_count: usize,
) {
    for i in 0..field_count {
        instructions.push(Instruction::LoadParameter(param_slot));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(i),
        )));
        instructions.push(Instruction::ArrayGet);
    }
}
