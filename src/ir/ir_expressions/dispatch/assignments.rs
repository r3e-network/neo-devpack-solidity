use super::*;

pub(crate) fn try_lower_expression_assignments(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::AssignAdd(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Add,
        )),
        Expression::AssignSubtract(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Sub,
        )),
        Expression::AssignShiftLeft(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Shl,
        )),
        Expression::AssignShiftRight(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Shr,
        )),
        Expression::AssignAnd(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::BitAnd,
        )),
        Expression::AssignOr(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::BitOr,
        )),
        Expression::AssignXor(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::BitXor,
        )),
        Expression::AssignMultiply(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Mul,
        )),
        Expression::AssignDivide(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Div,
        )),
        Expression::AssignModulo(_, lhs, rhs) => Some(lower_compound_assignment(
            lhs,
            rhs,
            ctx,
            instructions,
            BinaryOperator::Mod,
        )),
        Expression::Assign(_, lhs, rhs) => {
            lower_assignment(lhs, rhs, ctx, instructions);
            Some(true)
        }
        Expression::PostIncrement(_, inner) => {
            Some(lower_post_inc_dec(inner, ctx, instructions, true))
        }
        Expression::PostDecrement(_, inner) => {
            Some(lower_post_inc_dec(inner, ctx, instructions, false))
        }
        Expression::PreIncrement(_, inner) => {
            Some(lower_pre_inc_dec(inner, ctx, instructions, true))
        }
        Expression::PreDecrement(_, inner) => {
            Some(lower_pre_inc_dec(inner, ctx, instructions, false))
        }
        Expression::Delete(_, target) => Some(lower_delete(target, ctx, instructions)),
        _ => None,
    }
}

pub(crate) fn lower_delete(
    target: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Storage delete: reset to the storage-default value.
    if let Some(reference) = resolve_storage_reference(target, ctx) {
        if !ctx.ensure_state_writable(reference.state_index) {
            return false;
        }

        // Solidity allows `delete` on mappings, but it cannot clear all keys; treat as a no-op.
        if matches!(reference.value_type, ValueType::Mapping { .. }) {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            return true;
        }

        // Whole-array delete (`delete arr;` / `delete m[k];` where the value
        // is an array). Solidity requires every element to be reset to its
        // default — writing only the base (length) slot leaves the per-index
        // slots `keccak256(serialize(i) || base)` fully readable. For
        // fixed-size `T[N]` arrays the bounds guard uses the compile-time
        // constant N (never the base slot), so without the element-clearing
        // loop every `arr[i]` read returns the pre-delete value.
        if matches!(reference.value_type, ValueType::Array(_))
            && reference.field_path.is_empty()
            && reference.trailing_key_expressions.is_empty()
            && lower_delete_storage_array(&reference, ctx, instructions)
        {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            return true;
        }

        push_default_for_storage_value_type(&reference.value_type, ctx, instructions);
        if !emit_storage_store(&reference, ctx, instructions) {
            instructions.push(Instruction::Drop(ValueType::Any));
        }

        // `delete` is a statement-only expression in Solidity; return a placeholder value so
        // expression statements can safely DROP it without underflowing.
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        return true;
    }

    // Local/state variable delete.
    if let Expression::Variable(identifier) = target {
        if let Some(local_index) = ctx.resolve_local(&identifier.name) {
            if let Some(value_type) = ctx.local_type(local_index).cloned() {
                push_default_for_value_type(&value_type, ctx, instructions);
            } else {
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
            }
            ctx.clear_call_data_local(local_index);
            instructions.push(Instruction::StoreLocal(local_index));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            return true;
        }

        if let Some(state_index) = ctx.state_index_map.get(&identifier.name).copied() {
            if !ctx.ensure_state_writable(state_index) {
                return false;
            }

            if let Some(state_type) = ctx.state_type(state_index).cloned() {
                if matches!(state_type, ValueType::Mapping { .. }) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    return true;
                }

                if matches!(state_type, ValueType::Struct { .. }) {
                    let reference = StorageReference {
                        state_index,
                        key_expressions: Vec::new(),
                        key_types: Vec::new(),
                        value_type: state_type.clone(),
                        field_path: Vec::new(),
                        trailing_key_expressions: Vec::new(),
                        trailing_key_types: Vec::new(),
                    };

                    push_default_for_storage_value_type(&reference.value_type, ctx, instructions);
                    if !emit_storage_store(&reference, ctx, instructions) {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    return true;
                }

                push_default_for_storage_value_type(&state_type, ctx, instructions);
                instructions.push(Instruction::StoreState(state_index));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                return true;
            }
        }
    }

    // Memory array element delete: `delete arr[i]`.
    if let Expression::ArraySubscript(_, array, Some(index)) = target {
        if let Some(ValueType::Array(element_type)) = infer_type_from_expression(array, ctx) {
            let tmp_id = ctx.next_label();
            let array_local = ctx.allocate_local(format!("__delete_arr_{tmp_id}"), None);
            let index_local = ctx.allocate_local(format!("__delete_idx_{tmp_id}"), None);

            if !lower_expression(array, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::StoreLocal(array_local));

            if !lower_expression(index, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::StoreLocal(index_local));

            instructions.push(Instruction::LoadLocal(array_local));
            instructions.push(Instruction::LoadLocal(index_local));
            push_default_for_value_type(element_type.as_ref(), ctx, instructions);
            instructions.push(Instruction::ArraySet);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            return true;
        }
    }

    // Memory struct field delete: `delete tmp.field`.
    if let Expression::MemberAccess(_, inner, member) = target {
        if let Expression::Variable(base) = inner.as_ref() {
            if let Some(local_index) = ctx.resolve_local(&base.name) {
                if let Some(ValueType::Struct { fields, .. }) =
                    infer_type_from_expression(inner, ctx)
                {
                    if let Some((field_index, field)) = fields
                        .iter()
                        .enumerate()
                        .find(|(_, field)| field.name == member.name)
                    {
                        instructions.push(Instruction::LoadLocal(local_index));
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(field_index as u64),
                        )));
                        push_default_for_value_type(&field.ty, ctx, instructions);
                        instructions.push(Instruction::ArraySet);
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                        return true;
                    }
                }
            }
        }
    }

    // Compatibility fallback for unsupported `delete` shapes:
    // preserve control flow by treating as a no-op.
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    true
}

/// Lower `delete arr;` for a storage-backed array (top-level state variable
/// or `m[k]` mapping value) by clearing every element slot, mirroring the
/// shrink loop in `lower_storage_array_assign_from_memory`
/// (src/ir/statements/assignments/lower_assignment.rs):
///
///   * fixed-size `T[N]` state arrays — the bound is the compile-time `N`
///     parsed from the declared type string (same parse as Task #199's
///     `StorageArrayBound::FixedSizeKnown`); these arrays have no length
///     slot, so no base-slot write is needed (or possible).
///   * dynamic `T[]` arrays — the old length is read from the base slot
///     BEFORE it is zeroed, each element slot is reset to the element
///     default, then the base (length) slot is set to 0.
///
/// Returns `false` (without emitting the clear loop) for element types whose
/// defaults are not directly storable via a raw `Storage.Put` (structs,
/// mappings, `Any`); the caller then falls back to the legacy base-slot-only
/// write so behaviour for those shapes is unchanged.
pub(crate) fn lower_delete_storage_array(
    reference: &StorageReference,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let element_type = match &reference.value_type {
        ValueType::Array(element) => (**element).clone(),
        _ => return false,
    };
    // Mapping/Any element defaults are `Null` — a raw Put would fault on real
    // Neo N3 — so leave those on the legacy base-slot-only path. Struct elements
    // ARE cleared (below) via `StoreStructArrayElement` so the per-field slots a
    // `push` wrote are actually zeroed; without it `delete ps` left every
    // `ps[i].field` readable (length-0 array but live field slots), because
    // `ps[i].field` reads through `LoadStructField` and skips the array bounds
    // guard.
    if matches!(element_type, ValueType::Mapping { .. } | ValueType::Any) {
        return false;
    }

    let uint256 = ValueType::Integer {
        signed: false,
        bits: 256,
    };
    let tmp_id = ctx.next_label();

    // Evaluate any outer mapping/array keys once (left-to-right, Solidity
    // order) so the per-element loop can re-push them without re-running
    // side effects.
    let mut key_locals: Vec<usize> = Vec::new();
    for (index, expr) in reference.key_expressions.iter().enumerate() {
        let local = ctx.allocate_local(
            format!("__delete_sarr_key_{tmp_id}_{index}"),
            reference.key_types.get(index).cloned(),
        );
        if !lower_expression(expr, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(local));
        key_locals.push(local);
    }

    // Bound: compile-time N for a fixed-size `T[N]` state array (only
    // detectable on the direct state-variable shape — the declared type
    // string ends with `[N]`), otherwise the dynamic length read from the
    // base slot. The read MUST happen before the base slot is zeroed.
    let is_fixed_size = reference.key_expressions.is_empty()
        && ctx
            .state_metadata(reference.state_index)
            .map(|meta| meta.ty.clone())
            .and_then(|ty| extract_fixed_array_bound_at_depth(&ty, 0))
            .is_some();
    let len_local =
        ctx.allocate_local(format!("__delete_sarr_len_{tmp_id}"), Some(uint256.clone()));
    if is_fixed_size {
        let bound = ctx
            .state_metadata(reference.state_index)
            .map(|meta| meta.ty.clone())
            .and_then(|ty| extract_fixed_array_bound_at_depth(&ty, 0))
            .unwrap_or(0);
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(bound),
        )));
    } else if reference.key_expressions.is_empty() {
        // Dynamic state-var array: length lives at LoadState(state_index).
        instructions.push(Instruction::LoadState(reference.state_index));
    } else {
        // Mapping-of-dynamic-array: length lives at the mapping head slot.
        for local in key_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
        instructions.push(Instruction::LoadMappingElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
        });
    }
    instructions.push(Instruction::StoreLocal(len_local));

    // Clear loop: for (i = 0; i < len; i++) { arr[i] = default; }
    let mut element_key_types = reference.key_types.clone();
    element_key_types.push(uint256.clone());
    let cond_label = ctx.next_label();
    let end_label = ctx.next_label();
    let idx_local =
        ctx.allocate_local(format!("__delete_sarr_idx_{tmp_id}"), Some(uint256.clone()));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(idx_local));

    instructions.push(Instruction::Label(cond_label));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    // IR JumpIf branches when the condition is FALSE: exit once idx >= len.
    instructions.push(Instruction::JumpIf { target: end_label });

    push_default_for_value_type(&element_type, ctx, instructions);
    // Keys are pushed deepest-first (the bytecode slot-hash helper expects
    // the outermost key on top once the base slot is pushed).
    instructions.push(Instruction::LoadLocal(idx_local));
    for local in key_locals.iter().rev() {
        instructions.push(Instruction::LoadLocal(*local));
    }
    if matches!(element_type, ValueType::Struct { .. }) {
        // Struct elements use the per-field layout `push` wrote (the array index
        // is consumed by the instruction, so `key_types` is the OUTER keys only
        // and `field_keys` is empty — matching the push path in
        // `storage_reference.rs` / `state_var.rs`).
        instructions.push(Instruction::StoreStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys: Vec::new(),
            element_type: element_type.clone(),
        });
    } else if matches!(element_type, ValueType::Array(_)) {
        instructions.push(Instruction::StoreArrayDeepCopy {
            state_index: reference.state_index,
            key_types: element_key_types.clone(),
        });
    } else {
        instructions.push(Instruction::StoreMappingElement {
            state_index: reference.state_index,
            key_types: element_key_types.clone(),
        });
    }

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump { target: cond_label });
    instructions.push(Instruction::Label(end_label));

    // Dynamic arrays: zero the base (length) slot. Fixed-size arrays have no
    // length slot — leave the never-read base slot untouched.
    if !is_fixed_size {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        for local in key_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
        instructions.push(Instruction::StoreMappingElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
        });
    }

    true
}
