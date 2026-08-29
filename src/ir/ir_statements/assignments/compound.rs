use super::*;

/// Dispatch compound-assignment arithmetic (`x <op>= y`, `x++`, `--x`, `x <<= y`)
/// through the SAME overflow-guard / width-truncation ladder as regular binary
/// expressions, so `x <op>= y` is byte-for-byte identical to `x = x <op> y`.
/// Both operands are already on the stack as `[.., lhs_value, rhs_value]`.
///
/// The ladder's gate predicates only need the *result type*, which for a
/// compound op is the LHS's type, so `lhs` is passed as both type operands
/// (`rhs` adapts to `lhs` in Solidity; passing `lhs` twice cannot be misread as
/// a both-literal constant-fold since an l-value location is never a literal).
/// This previously emitted only the uint256/int256 guards, silently skipping the
/// narrow-width overflow check so `uint8 x; x += 1` / `x++` overflowed instead of
/// reverting Panic(0x11), and narrow `unchecked`/`<<=` results were not truncated.
pub(crate) fn emit_compound_binary_op_for_lhs(
    lhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    op: BinaryOperator,
) {
    // `true`: route `unchecked` uint256 compound ops through the software limb
    // routines so operands >= 2^255 wrap mod 2^256 instead of faulting on a
    // native 33-byte intermediate. The historical `false` workaround existed
    // because the OLD widen+truncate path left a 32-byte Buffer that broke an
    // l-value reused as an integer index (`for (...; i++) a[i]`); the limb
    // routines now produce a canonical Integer/two's-complement value, so that
    // concern no longer applies.
    emit_arith_with_overflow_ladder(lhs, lhs, ctx, instructions, op, true);
}

/// Task #118: lower the RHS of a compound assignment. Older runtime shims
/// treated MEMCPY as returning a destination buffer and needed stack cleanup
/// here; real NeoVM MEMCPY pushes no value, so the lowered RHS is already the
/// canonical result.
pub(crate) fn lower_compound_rhs(
    rhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if !lower_expression(rhs, ctx, instructions) {
        return false;
    }
    // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
    true
}

pub(crate) fn lower_compound_assignment(
    lhs: &Expression,
    rhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    op: BinaryOperator,
) -> bool {
    // Solidity compound assignment semantics:
    // - Evaluate the LHS location once (including any array/mapping index expressions).
    // - Compute `lhs_value op rhs_value`.
    // - Store the result back to the same LHS.
    // - The expression result is the stored value (so statement lowering can safely DROP it).

    if let Some(mapping) = resolve_mapping_access(lhs, ctx) {
        if !ctx.ensure_state_writable(mapping.state_index) {
            return false;
        }

        // Evaluate keys left-to-right (Solidity semantics) and store them so we can reuse them
        // for both the load and the store without re-evaluating side effects.
        let tmp_id = ctx.next_label();
        let mut key_locals: Vec<usize> = Vec::new();
        for (index, key_expr) in mapping.key_expressions.iter().enumerate() {
            let local = ctx.allocate_local(
                format!("__compound_key_{tmp_id}_{index}"),
                mapping.key_types.get(index).cloned(),
            );
            if !lower_expression(key_expr, ctx, instructions) {
                ctx.record_error_with_suggestion(
                    "failed to lower mapping key in compound assignment",
                    "ensure the mapping key expression is a supported type (integer, string, bytes, or address)",
                );
                return false;
            }
            instructions.push(Instruction::StoreLocal(local));
            key_locals.push(local);
        }

        // Load current value (keys must be pushed in reverse order for slot hashing).
        for local in key_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
        instructions.push(Instruction::LoadMappingElement {
            state_index: mapping.state_index,
            key_types: mapping.key_types.clone(),
        });

        // Evaluate RHS after LHS value (left-to-right semantics for `lhs op rhs`).
        if !lower_compound_rhs(rhs, ctx, instructions) {
            return false;
        }

        emit_compound_binary_op_for_lhs(lhs, ctx, instructions, op);

        // Store result while preserving it as the expression value.
        let result_local = ctx.allocate_local(format!("__compound_value_{tmp_id}"), None);
        instructions.push(Instruction::StoreLocal(result_local));

        instructions.push(Instruction::LoadLocal(result_local));
        for local in key_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
        instructions.push(Instruction::StoreMappingElement {
            state_index: mapping.state_index,
            key_types: mapping.key_types.clone(),
        });

        instructions.push(Instruction::LoadLocal(result_local));
        return true;
    }

    if let Some(reference) = resolve_storage_reference(lhs, ctx) {
        if !ctx.ensure_state_writable(reference.state_index) {
            return false;
        }

        // Task #82: trailing-key storage references (e.g. `slots[k].balances[a] += v`)
        // must route through the general load/store helpers so the nested-mapping
        // slot derivation is applied.
        if !reference.trailing_key_expressions.is_empty() {
            let tmp_id = ctx.next_label();
            let result_local = ctx.allocate_local(format!("__compound_value_{tmp_id}"), None);
            if !emit_storage_load(&reference, ctx, instructions) {
                return false;
            }
            if !lower_compound_rhs(rhs, ctx, instructions) {
                return false;
            }
            emit_compound_binary_op_for_lhs(lhs, ctx, instructions, op);
            instructions.push(Instruction::StoreLocal(result_local));
            instructions.push(Instruction::LoadLocal(result_local));
            if !emit_storage_store(&reference, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::LoadLocal(result_local));
            return true;
        }

        if let Some(field) = reference.field_path.last() {
            let field_keys: Vec<[u8; 32]> =
                reference.field_path.iter().map(|field| field.key).collect();
            let tmp_id = ctx.next_label();
            let mut key_locals: Vec<usize> = Vec::new();

            for (index, key_expr) in reference.key_expressions.iter().enumerate() {
                let local = ctx.allocate_local(
                    format!("__compound_key_{tmp_id}_{index}"),
                    reference.key_types.get(index).cloned(),
                );
                if !lower_expression(key_expr, ctx, instructions) {
                    ctx.record_error("failed to lower storage key in compound assignment");
                    return false;
                }
                instructions.push(Instruction::StoreLocal(local));
                key_locals.push(local);
            }

            let push_keys_for_slot = |instructions: &mut Vec<Instruction>| {
                for local in key_locals.iter().rev() {
                    instructions.push(Instruction::LoadLocal(*local));
                }
            };

            // Load current field value.
            push_keys_for_slot(instructions);
            instructions.push(Instruction::LoadStructField {
                state_index: reference.state_index,
                key_types: reference.key_types.clone(),
                field_keys: field_keys.clone(),
                field_type: field.ty.clone(),
            });

            // Evaluate RHS after LHS value (left-to-right semantics for `lhs op rhs`).
            if !lower_compound_rhs(rhs, ctx, instructions) {
                return false;
            }

            emit_compound_binary_op_for_lhs(lhs, ctx, instructions, op);

            // Store result while preserving it as the expression value.
            let result_local = ctx.allocate_local(format!("__compound_value_{tmp_id}"), None);
            instructions.push(Instruction::StoreLocal(result_local));

            instructions.push(Instruction::LoadLocal(result_local));
            push_keys_for_slot(instructions);
            instructions.push(Instruction::StoreStructField {
                state_index: reference.state_index,
                key_types: reference.key_types.clone(),
                field_keys,
                field_type: field.ty.clone(),
            });

            instructions.push(Instruction::LoadLocal(result_local));
            return true;
        }
    }

    if let Expression::ArraySubscript(_, array, Some(index)) = lhs {
        // Memory array element compound assignment: `arr[i] op= rhs`.
        // Preserve evaluation order and avoid evaluating `array` / `index` twice.
        let tmp_id = ctx.next_label();
        let array_local = ctx.allocate_local(format!("__compound_arr_{tmp_id}"), None);
        let index_local = ctx.allocate_local(format!("__compound_idx_{tmp_id}"), None);

        if !lower_expression(array, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(array_local));

        if !lower_expression(index, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(index_local));

        // Load current element.
        instructions.push(Instruction::LoadLocal(array_local));
        instructions.push(Instruction::LoadLocal(index_local));
        instructions.push(Instruction::ArrayGet);

        if !lower_compound_rhs(rhs, ctx, instructions) {
            return false;
        }

        emit_compound_binary_op_for_lhs(lhs, ctx, instructions, op);

        // Store and return the updated value.
        let result_local = ctx.allocate_local(format!("__compound_value_{tmp_id}"), None);
        instructions.push(Instruction::StoreLocal(result_local));

        instructions.push(Instruction::LoadLocal(array_local));
        instructions.push(Instruction::LoadLocal(index_local));
        instructions.push(Instruction::LoadLocal(result_local));
        instructions.push(Instruction::ArraySet);

        instructions.push(Instruction::LoadLocal(result_local));
        return true;
    }

    if let Expression::MemberAccess(_, inner, member) = lhs {
        // Memory struct field compound assignment: `tmp.field op= rhs`.
        // The base may be either a local or a function parameter — both hold
        // the struct as an NeoVM Array slot (Rc<RefCell<Vec<StackItem>>>), so
        // LoadLocal/LoadParameter yields a reference whose ArraySet mutates in
        // place. Task #191 widened this branch from local-only to also accept
        // parameter bases so that free-function-attach chains (`using {inc}
        // for Counter; c.inc().inc()...`) where `inc(Counter memory c)` does
        // `c.value++` actually mutate the passed-in copy.
        if let Expression::Variable(base) = inner.as_ref() {
            let load_base = ctx
                .resolve_local(&base.name)
                .map(Instruction::LoadLocal)
                .or_else(|| {
                    ctx.param_index_map
                        .get(&base.name)
                        .copied()
                        .map(Instruction::LoadParameter)
                });

            if let Some(load_base) = load_base {
                if let Some(ValueType::Struct { fields, .. }) =
                    infer_type_from_expression(inner, ctx)
                {
                    if let Some((field_index, _field)) = fields
                        .iter()
                        .enumerate()
                        .find(|(_, field)| field.name == member.name)
                    {
                        let tmp_id = ctx.next_label();
                        let result_local =
                            ctx.allocate_local(format!("__compound_value_{tmp_id}"), None);

                        // Load current field value.
                        instructions.push(load_base.clone());
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(field_index as u64),
                        )));
                        instructions.push(Instruction::ArrayGet);

                        // Evaluate RHS and apply operation.
                        if !lower_compound_rhs(rhs, ctx, instructions) {
                            return false;
                        }
                        emit_compound_binary_op_for_lhs(lhs, ctx, instructions, op);
                        instructions.push(Instruction::StoreLocal(result_local));

                        // Store updated field value back into the struct ref.
                        instructions.push(load_base);
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(field_index as u64),
                        )));
                        instructions.push(Instruction::LoadLocal(result_local));
                        instructions.push(Instruction::ArraySet);

                        // Compound assignment expression result is the stored value.
                        instructions.push(Instruction::LoadLocal(result_local));
                        return true;
                    }
                }
            }
        }
    }

    if let Expression::Variable(identifier) = lhs {
        let store_instr = if let Some(local) = ctx.resolve_local(&identifier.name) {
            Instruction::StoreLocal(local)
        } else if let Some(param_index) = ctx.param_index_map.get(&identifier.name).copied() {
            // Task #156 — compound-assign on a function parameter (`a += 1;`).
            // Same rationale as the plain-assign path: `ensure_local` below
            // would silently divert writes to a shadow local that reads never
            // see (reads check `param_index_map` first). Emit `StoreParameter`
            // to update the caller-visible slot.
            Instruction::StoreParameter(param_index)
        } else if let Some(state_index) = ctx.state_index_map.get(&identifier.name).copied() {
            if !ctx.ensure_state_writable(state_index) {
                return false;
            }
            Instruction::StoreState(state_index)
        } else {
            let local = ctx.ensure_local(&identifier.name);
            Instruction::StoreLocal(local)
        };

        let tmp_id = ctx.next_label();
        let result_local = ctx.allocate_local(format!("__compound_value_{tmp_id}"), None);

        // Evaluate lhs then rhs (left-to-right) so BinaryOp sees [lhs, rhs].
        if !lower_expression(lhs, ctx, instructions) {
            return false;
        }
        if !lower_compound_rhs(rhs, ctx, instructions) {
            return false;
        }

        emit_compound_binary_op_for_lhs(lhs, ctx, instructions, op);
        instructions.push(Instruction::StoreLocal(result_local));

        instructions.push(Instruction::LoadLocal(result_local));
        instructions.push(store_instr);

        instructions.push(Instruction::LoadLocal(result_local));
        return true;
    }

    // Compatibility fallback: preserve side effects and produce a placeholder result.
    let mut success = true;
    if lower_expression(lhs, ctx, instructions) {
        instructions.push(Instruction::Drop(ValueType::Any));
    } else {
        success = false;
    }
    if lower_expression(rhs, ctx, instructions) {
        instructions.push(Instruction::Drop(ValueType::Any));
    } else {
        success = false;
    }
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    success
}
