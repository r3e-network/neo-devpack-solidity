use super::*;

pub(crate) fn resolve_storage_reference(
    expression: &Expression,
    ctx: &LoweringContext,
) -> Option<StorageReference> {
    if let Some(mapping) = resolve_mapping_access(expression, ctx) {
        return Some(mapping.to_storage_reference());
    }

    match expression {
        // Task #196 — unwrap a zero-arg internal function that trivially
        // forwards a state variable as `T storage`. At the IR layer the
        // call returns `LoadState(state_index)` which, for an Array state
        // variable, is the LENGTH integer (not the backing Array) — so
        // `foo().length` crashes at `SIZE: unsupported type` and
        // `foo().push(v)` silently no-ops. Resolving the call back into
        // the underlying `Variable(state_var)` lets the existing
        // storage-reference machinery alias the actual slot: subsequent
        // `.push(v)`, `.length`, `[i] = v`, etc. walk through the same
        // `emit_storage_load` / `StoreMappingElement` path that
        // direct state-var access uses. Extends Task #117 across the
        // function-return boundary.
        Expression::FunctionCall(_, func, args) if args.is_empty() => {
            if let Expression::Variable(fn_ident) = func.as_ref() {
                if let Some(state_var_name) = ctx.storage_pointer_returning_fn(&fn_ident.name) {
                    let state_index = *ctx.state_index_map.get(state_var_name)?;
                    let state_type = ctx.state_type(state_index)?;
                    if matches!(
                        state_type,
                        ValueType::Struct { .. } | ValueType::Array(_) | ValueType::Mapping { .. }
                    ) {
                        return Some(StorageReference {
                            state_index,
                            key_expressions: Vec::new(),
                            key_types: Vec::new(),
                            value_type: state_type.clone(),
                            field_path: Vec::new(),
                            trailing_key_expressions: Vec::new(),
                            trailing_key_types: Vec::new(),
                        });
                    }
                }
            }
            None
        }
        Expression::Variable(identifier) => {
            if let Some(alias) = ctx.storage_alias(&identifier.name).cloned() {
                return Some(alias);
            }

            // Treat struct-typed state variables as storage references so that:
            // - field accesses (`stateStruct.field`) load from storage slots; and
            // - whole-struct assignments (`stateStruct = StructName({...})`) can be lowered
            //   into per-field stores.
            //
            // Task #117: also recognize Array- and Mapping-typed state variables as
            // storage reference bases so that `T[] storage a = arr;` can register a
            // symbolic handle to `arr`. Subsequent `a[idx] = v` / `a[idx]` reads then
            // walk the `ArraySubscript` branch below and land on the same
            // `StoreMappingElement` / `LoadMappingElement` path that `arr[idx] = v`
            // uses — the storage pointer aliases the backing slot instead of
            // copying the array contents.
            let state_index = *ctx.state_index_map.get(&identifier.name)?;
            let state_type = ctx.state_type(state_index)?;
            if matches!(
                state_type,
                ValueType::Struct { .. } | ValueType::Array(_) | ValueType::Mapping { .. }
            ) {
                Some(StorageReference {
                    state_index,
                    key_expressions: Vec::new(),
                    key_types: Vec::new(),
                    value_type: state_type.clone(),
                    field_path: Vec::new(),
                    trailing_key_expressions: Vec::new(),
                    trailing_key_types: Vec::new(),
                })
            } else {
                None
            }
        }
        Expression::MemberAccess(_, inner, member) => {
            let mut base = resolve_storage_reference(inner, ctx)?;
            // Field access only makes sense on a struct-typed current value (not on a
            // mapping/array tail — those must be subscripted first).
            if !base.trailing_key_expressions.is_empty() {
                return None;
            }
            let field = find_struct_field(&base.value_type, &member.name)?;
            base.field_path.push(StorageReferenceField {
                key: field.key,
                ty: field.ty.clone(),
            });
            base.value_type = field.ty.clone();
            Some(base)
        }
        Expression::ArraySubscript(_, inner, Some(index_expr)) => {
            // Task #82: `slots[k].balances[a]` and similar nested-mapping-in-struct chains.
            // The inner must already resolve into a struct field (or nested mapping) whose
            // current value_type is a mapping or array. Extend the trailing-key chain.
            //
            // Task #117: when the base has no field_path yet (e.g. a storage-pointer
            // alias to a top-level array or mapping, `T[] storage a = arr; a[idx]`),
            // the index belongs in the PRIMARY key chain — it is semantically the
            // same slot key that a direct `arr[idx]` access produces. Putting it in
            // `trailing_key_expressions` would only be honoured by the
            // `StoreStructFieldMappingElement` branch, which never fires without a
            // field_path, so the index would be silently dropped.
            let mut base = resolve_storage_reference(inner, ctx)?;
            let (key_type, value_type) = match &base.value_type {
                ValueType::Mapping { key, value } => ((**key).clone(), (**value).clone()),
                ValueType::Array(element) => (
                    ValueType::Integer {
                        signed: false,
                        bits: 256,
                    },
                    (**element).clone(),
                ),
                _ => return None,
            };
            if base.field_path.is_empty() {
                base.key_expressions.push((**index_expr).clone());
                base.key_types.push(key_type);
            } else {
                base.trailing_key_expressions.push((**index_expr).clone());
                base.trailing_key_types.push(key_type);
            }
            base.value_type = value_type;
            Some(base)
        }
        _ => None,
    }
}

pub(crate) fn emit_storage_load(
    reference: &StorageReference,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let tmp_id = ctx.next_label();

    // Evaluate mapping/array keys left-to-right (Solidity semantics) and store them so we can
    // push them in the stack order required by the storage slot hashing routine.
    let mut key_locals: Vec<usize> = Vec::new();
    for (index, expr) in reference.key_expressions.iter().enumerate() {
        let key_ty = reference.key_types.get(index).cloned();
        let local = ctx.allocate_local(format!("__storage_key_{tmp_id}_{index}"), key_ty.clone());
        // Canonicalize an integer-backed bytesN literal key to its full-width
        // big-endian ByteString so it hashes to the SAME storage slot as a
        // runtime bytesN key of the same value. Without this, `m[0x..01]` pushes
        // an Integer that serializes to a short little-endian ByteString and
        // keccaks to a DIFFERENT slot than a `bytes32` param key, silently
        // reading/writing the wrong entry on a real node (the simulator masks
        // it). Address keys are not integer-backed, so they are unaffected.
        let canon = key_ty
            .as_ref()
            .is_some_and(|ty| try_lower_bytesn_literal_canonical(expr, ty, ctx, instructions));
        if !canon && !lower_expression(expr, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(local));
        key_locals.push(local);
    }

    // Task #82: evaluate trailing (inner-mapping) keys also left-to-right.
    let mut trailing_locals: Vec<usize> = Vec::new();
    for (index, expr) in reference.trailing_key_expressions.iter().enumerate() {
        let trail_ty = reference.trailing_key_types.get(index).cloned();
        let local = ctx.allocate_local(
            format!("__storage_trail_{tmp_id}_{index}"),
            trail_ty.clone(),
        );
        // Same bytesN-literal-key canonicalization as the outer keys above, so an
        // inner-mapping literal bytesN key hashes to the runtime key's slot.
        let canon = trail_ty
            .as_ref()
            .is_some_and(|ty| try_lower_bytesn_literal_canonical(expr, ty, ctx, instructions));
        if !canon && !lower_expression(expr, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(local));
        trailing_locals.push(local);
    }

    let push_keys_for_slot = |instructions: &mut Vec<Instruction>| {
        // The bytecode emission expects keys in reverse order (deepest key first), with the
        // outer-most key closest to the top of the stack when the base slot is pushed.
        for local in key_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
    };
    let push_trailing_keys_for_slot = |instructions: &mut Vec<Instruction>| {
        for local in trailing_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
    };

    // Task #82: nested mapping inside struct field — `slots[k].balances[a]`.
    if !reference.trailing_key_expressions.is_empty() && !reference.field_path.is_empty() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        push_trailing_keys_for_slot(instructions);
        push_keys_for_slot(instructions);
        instructions.push(Instruction::LoadStructFieldMappingElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            trailing_key_types: reference.trailing_key_types.clone(),
            value_type: reference.value_type.clone(),
        });
        return true;
    }

    if let Some(field) = reference.field_path.last() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        push_keys_for_slot(instructions);
        instructions.push(Instruction::LoadStructField {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            field_type: field.ty.clone(),
        });
        return true;
    }

    // Loading a struct value from storage requires fetching each field individually.
    if let ValueType::Struct { fields, .. } = &reference.value_type {
        let out_local = ctx.allocate_local(
            format!("__storage_struct_{tmp_id}"),
            Some(reference.value_type.clone()),
        );

        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(fields.len() as u64),
        )));
        instructions.push(Instruction::NewArray {
            element_type: ValueType::Any,
        });
        instructions.push(Instruction::StoreLocal(out_local));

        for (index, field) in fields.iter().enumerate() {
            instructions.push(Instruction::LoadLocal(out_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(index as u64),
            )));
            // Mapping members have no loadable storage value (their entries
            // live at per-key derived slots) — keep the struct array at
            // `fields.len()` entries so PICKITEM field indices stay aligned,
            // but write a Null placeholder instead of issuing a useless
            // `Storage.Get` against the mapping's base field slot.
            if matches!(field.ty, ValueType::Mapping { .. }) {
                instructions.push(Instruction::PushLiteral(LiteralValue::Null));
                instructions.push(Instruction::ArraySet);
                continue;
            }
            push_keys_for_slot(instructions);
            instructions.push(Instruction::LoadStructField {
                state_index: reference.state_index,
                key_types: reference.key_types.clone(),
                field_keys: vec![field.key],
                field_type: field.ty.clone(),
            });
            instructions.push(Instruction::ArraySet);
        }

        instructions.push(Instruction::LoadLocal(out_local));
        return true;
    }

    push_keys_for_slot(instructions);
    instructions.push(Instruction::LoadMappingElement {
        state_index: reference.state_index,
        key_types: reference.key_types.clone(),
    });
    true
}

pub(crate) fn emit_storage_store(
    reference: &StorageReference,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let tmp_id = ctx.next_label();

    // Evaluate mapping/array keys left-to-right and store them in locals so we can re-push them
    // in the correct order for slot hashing.
    let mut key_locals: Vec<usize> = Vec::new();
    for (index, expr) in reference.key_expressions.iter().enumerate() {
        let key_ty = reference.key_types.get(index).cloned();
        let local = ctx.allocate_local(format!("__storage_key_{tmp_id}_{index}"), key_ty.clone());
        // Canonicalize an integer-backed bytesN literal key to its full-width
        // big-endian ByteString so it hashes to the SAME storage slot as a
        // runtime bytesN key of the same value. Without this, `m[0x..01]` pushes
        // an Integer that serializes to a short little-endian ByteString and
        // keccaks to a DIFFERENT slot than a `bytes32` param key, silently
        // reading/writing the wrong entry on a real node (the simulator masks
        // it). Address keys are not integer-backed, so they are unaffected.
        let canon = key_ty
            .as_ref()
            .is_some_and(|ty| try_lower_bytesn_literal_canonical(expr, ty, ctx, instructions));
        if !canon && !lower_expression(expr, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(local));
        key_locals.push(local);
    }

    // Task #82: evaluate trailing (inner-mapping) keys also left-to-right.
    let mut trailing_locals: Vec<usize> = Vec::new();
    for (index, expr) in reference.trailing_key_expressions.iter().enumerate() {
        let trail_ty = reference.trailing_key_types.get(index).cloned();
        let local = ctx.allocate_local(
            format!("__storage_trail_{tmp_id}_{index}"),
            trail_ty.clone(),
        );
        // Same bytesN-literal-key canonicalization as the outer keys above, so an
        // inner-mapping literal bytesN key hashes to the runtime key's slot.
        let canon = trail_ty
            .as_ref()
            .is_some_and(|ty| try_lower_bytesn_literal_canonical(expr, ty, ctx, instructions));
        if !canon && !lower_expression(expr, ctx, instructions) {
            return false;
        }
        instructions.push(Instruction::StoreLocal(local));
        trailing_locals.push(local);
    }

    let push_keys_for_slot = |instructions: &mut Vec<Instruction>| {
        for local in key_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
    };
    let push_trailing_keys_for_slot = |instructions: &mut Vec<Instruction>| {
        for local in trailing_locals.iter().rev() {
            instructions.push(Instruction::LoadLocal(*local));
        }
    };

    // Task #82: nested mapping inside struct field — `slots[k].balances[a] = v`.
    if !reference.trailing_key_expressions.is_empty() && !reference.field_path.is_empty() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        push_trailing_keys_for_slot(instructions);
        push_keys_for_slot(instructions);
        instructions.push(Instruction::StoreStructFieldMappingElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            trailing_key_types: reference.trailing_key_types.clone(),
        });
        return true;
    }

    if let Some(field) = reference.field_path.last() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        push_keys_for_slot(instructions);
        instructions.push(Instruction::StoreStructField {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            field_type: field.ty.clone(),
        });
        return true;
    }

    // Storing an entire struct value writes each field into its own derived slot.
    if let ValueType::Struct { fields, .. } = &reference.value_type {
        let value_local = ctx.allocate_local(
            format!("__storage_struct_value_{tmp_id}"),
            Some(reference.value_type.clone()),
        );
        instructions.push(Instruction::StoreLocal(value_local));

        for (index, field) in fields.iter().enumerate() {
            // Skip Mapping members: Solidity `delete s` / whole-struct
            // assignment leaves mapping members untouched, and the memory
            // representation holds Null at the mapping's field index — a
            // bare `Storage.Put` of a Null value faults on real Neo N3
            // (NullReferenceException in the C# core, Null.TryBytes() error
            // in neo-go). Each loop iteration is stack-self-contained, so
            // skipping is safe.
            if matches!(field.ty, ValueType::Mapping { .. }) {
                continue;
            }
            instructions.push(Instruction::LoadLocal(value_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(index as u64),
            )));
            instructions.push(Instruction::ArrayGet);
            push_keys_for_slot(instructions);
            instructions.push(Instruction::StoreStructField {
                state_index: reference.state_index,
                key_types: reference.key_types.clone(),
                field_keys: vec![field.key],
                field_type: field.ty.clone(),
            });
        }

        return true;
    }

    push_keys_for_slot(instructions);
    instructions.push(Instruction::StoreMappingElement {
        state_index: reference.state_index,
        key_types: reference.key_types.clone(),
    });
    true
}

pub(crate) fn resolve_mapping_access<'a>(
    expression: &'a Expression,
    ctx: &LoweringContext,
) -> Option<MappingAccess<'a>> {
    let mut keys: Vec<&'a Expression> = Vec::new();
    let mut current = expression;

    loop {
        match current {
            Expression::ArraySubscript(_, inner, maybe_index) => {
                let index_expr = maybe_index.as_ref()?.as_ref();
                keys.insert(0, index_expr);
                current = inner;
            }
            Expression::Variable(identifier) => {
                // A bare variable without subscript keys is NOT a mapping access;
                // it should be handled as a plain state-variable load so that
                // `emit_load_state` (with proper null-coercion) is used.
                if keys.is_empty() {
                    return None;
                }

                let state_index = *ctx.state_index_map.get(&identifier.name)?;
                let mut current_type = ctx.state_type(state_index)?.clone();
                let mut key_types = Vec::with_capacity(keys.len());

                for _key_expr in &keys {
                    match current_type {
                        ValueType::Mapping { ref key, ref value } => {
                            key_types.push((**key).clone());
                            current_type = (**value).clone();
                        }
                        ValueType::Array(ref element) => {
                            // Storage arrays are lowered like mappings keyed by uint256 index.
                            key_types.push(ValueType::Integer {
                                signed: false,
                                bits: 256,
                            });
                            current_type = (**element).clone();
                        }
                        _ => return None,
                    }
                }

                return Some(MappingAccess {
                    state_index,
                    key_expressions: keys,
                    key_types,
                    value_type: current_type,
                });
            }
            _ => return None,
        }
    }
}
