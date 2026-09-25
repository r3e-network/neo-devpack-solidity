use super::*;

pub(crate) fn try_lower_storage_reference_array_helpers(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let Expression::MemberAccess(_, inner, member) = func else {
        return None;
    };

    if !matches!(member.name.as_str(), "push" | "pop") {
        return None;
    }

    // Task #117 regression guard: fix-117 widened `resolve_storage_reference`
    // so a bare `Expression::Variable` naming an Array-typed state variable
    // returns a no-key/no-field StorageReference. Routing a direct
    // `ps.push(P(a,b))` through `lower_storage_reference_push` below would
    // collapse struct-element arrays onto the empty-field_path branch, which
    // emits a flat `StoreMappingElement` — but struct-array reads
    // (`ps[i].a`) expect the per-field slot layout that the Task #104 fix in
    // `state_var.rs::lower_state_array_push` derives via
    // `StoreStructArrayElement`. For direct state-var arrays we yield to
    // `try_lower_state_array_helpers` instead.
    if let Expression::Variable(identifier) = inner.as_ref() {
        if ctx.storage_alias(&identifier.name).is_none()
            && ctx.state_index_map.contains_key(&identifier.name)
        {
            return None;
        }
    }

    let reference = resolve_storage_reference(inner, ctx)?;

    // We support pushing/popping on:
    // - state arrays (`arr.push(x)`)
    // - storage references to arrays in mappings (`m[key].arr.push(x)`)
    // - storage references to array-typed struct fields (`m[key].struct.arr.push(x)`)
    // - nested struct paths (`m[key].outer.inner.arr.push(x)`)

    let ValueType::Array(element_type) = reference.value_type.clone() else {
        return None;
    };

    match member.name.as_str() {
        "push" => Some(lower_storage_reference_push(
            &reference,
            args,
            ctx,
            instructions,
        )),
        "pop" => Some(lower_storage_reference_pop(
            &reference,
            element_type.as_ref(),
            args,
            ctx,
            instructions,
        )),
        _ => None,
    }
}

pub(crate) fn lower_storage_reference_push(
    reference: &StorageReference,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Task #203 — Zero-arg `.push()` on array-of-mappings (even when
    // reached via a storage reference such as
    // `m[k].grids.push()` where `grids` is `mapping(K=>V)[]`) is the
    // ONLY valid push shape. Mapping elements are pure storage-slot-
    // derivation, so we only need to bump the length slot — no
    // element write is required.
    if args.is_empty() {
        if let ValueType::Array(element_type) = &reference.value_type {
            if matches!(element_type.as_ref(), ValueType::Mapping { .. }) {
                let len_local = ctx.allocate_local("__array_len".to_string(), None);
                if !emit_storage_load(reference, ctx, instructions) {
                    return false;
                }
                instructions.push(Instruction::StoreLocal(len_local));

                // Increment length.
                instructions.push(Instruction::LoadLocal(len_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::one(),
                )));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Add));

                if !emit_storage_store(reference, ctx, instructions) {
                    return false;
                }

                instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
                return true;
            }
        }
    }

    if args.len() != 1 {
        ctx.record_error("array push expects exactly one argument");
        return false;
    }

    if !lower_expression(&args[0], ctx, instructions) {
        return false;
    }
    let value_local = ctx.allocate_local("__array_push_value".to_string(), None);
    instructions.push(Instruction::StoreLocal(value_local));

    let len_local = ctx.allocate_local("__array_len".to_string(), None);
    if !emit_storage_load(reference, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(len_local));

    // Store element at index `len`.
    instructions.push(Instruction::LoadLocal(value_local));
    instructions.push(Instruction::LoadLocal(len_local));
    for expr in reference.key_expressions.iter().rev() {
        load_expression(expr, ctx, instructions);
    }

    let ValueType::Array(element_type) = &reference.value_type else {
        ctx.record_error("array push target is not an array");
        return false;
    };

    if !reference.field_path.is_empty() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        instructions.push(Instruction::StoreStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            element_type: (**element_type).clone(),
        });
    } else if matches!(element_type.as_ref(), ValueType::Struct { .. }) {
        // Task #170 — symmetric to the Task #104 fix for direct state
        // arrays (`P[] ps; ps.push(P(a,b))`): when a storage-reference
        // push targets a struct-element array (e.g.
        // `mapping(address => Checkpoint[]) history; history[acct].push(...)`),
        // `StoreMappingElement` writes the whole struct as a single
        // serde-JSON blob at `keccak256(serialize(i) || mapping_slot)`,
        // while the matching read path (`history[acct][i].field`) goes
        // through `emit_storage_load` → `LoadStructField` which derives
        // `keccak256(field_key || keccak256(serialize(i) || mapping_slot))`.
        // That slot mismatch surfaced as the TT2 harness reading `(0, 0)`
        // after a successful `record(alice, 100)` — the checkpoint field
        // values were written to the blob slot but the reader queried the
        // per-field slots. Route struct elements through
        // `StoreStructArrayElement` with empty `field_keys` so the write
        // hits the same per-field layout the read side expects.
        instructions.push(Instruction::StoreStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys: Vec::new(),
            element_type: (**element_type).clone(),
        });
    } else if matches!(element_type.as_ref(), ValueType::Array(_)) {
        let mut key_types = reference.key_types.clone();
        key_types.push(ValueType::Integer {
            signed: false,
            bits: 256,
        });
        instructions.push(Instruction::StoreArrayDeepCopy {
            state_index: reference.state_index,
            key_types,
        });
    } else {
        let mut key_types = reference.key_types.clone();
        key_types.push(ValueType::Integer {
            signed: false,
            bits: 256,
        });

        instructions.push(Instruction::StoreMappingElement {
            state_index: reference.state_index,
            key_types,
        });
    }

    // Increment length.
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));

    if !emit_storage_store(reference, ctx, instructions) {
        return false;
    }

    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
    true
}

pub(crate) fn lower_storage_reference_pop(
    reference: &StorageReference,
    element_type: &ValueType,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if !args.is_empty() {
        ctx.record_error("array pop expects no arguments");
        return false;
    }

    let len_local = ctx.allocate_local("__array_len".to_string(), None);
    if !emit_storage_load(reference, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(len_local));

    let empty_label = ctx.next_label();
    let end_label = ctx.next_label();

    // Abort on empty array.
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ne));
    instructions.push(Instruction::JumpIf {
        target: empty_label,
    });

    let new_len_local = ctx.allocate_local("__array_new_len".to_string(), None);
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
    instructions.push(Instruction::StoreLocal(new_len_local));

    // Update length before returning element.
    instructions.push(Instruction::LoadLocal(new_len_local));
    if !emit_storage_store(reference, ctx, instructions) {
        return false;
    }

    // Load element at new_len.
    instructions.push(Instruction::LoadLocal(new_len_local));
    for expr in reference.key_expressions.iter().rev() {
        load_expression(expr, ctx, instructions);
    }

    let mut key_types = reference.key_types.clone();
    key_types.push(ValueType::Integer {
        signed: false,
        bits: 256,
    });

    if !reference.field_path.is_empty() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        instructions.push(Instruction::LoadStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            element_type: element_type.clone(),
        });
    } else if matches!(element_type, ValueType::Struct { .. }) {
        // Task #170 — symmetric read-side of the push fix above. Must
        // match the per-field `StoreStructArrayElement` layout that
        // `lower_storage_reference_push` now writes for struct-element
        // storage-reference arrays (e.g. `history[acct].pop()` where
        // `history` is `mapping(address => Checkpoint[])`).
        instructions.push(Instruction::LoadStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys: Vec::new(),
            element_type: element_type.clone(),
        });
    } else {
        instructions.push(Instruction::LoadMappingElement {
            state_index: reference.state_index,
            key_types: key_types.clone(),
        });
    }

    let popped_local = ctx.allocate_local("__array_popped".to_string(), Some(element_type.clone()));
    instructions.push(Instruction::StoreLocal(popped_local));

    // Overwrite removed slot with default value.
    push_default_for_value_type(element_type, ctx, instructions);
    instructions.push(Instruction::LoadLocal(new_len_local));
    for expr in reference.key_expressions.iter().rev() {
        load_expression(expr, ctx, instructions);
    }
    if !reference.field_path.is_empty() {
        let field_keys: Vec<[u8; 32]> =
            reference.field_path.iter().map(|field| field.key).collect();
        instructions.push(Instruction::StoreStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys,
            element_type: element_type.clone(),
        });
    } else if matches!(element_type, ValueType::Struct { .. }) {
        // Task #170 — symmetric default-write to match the per-field
        // layout `lower_storage_reference_push` uses above.
        instructions.push(Instruction::StoreStructArrayElement {
            state_index: reference.state_index,
            key_types: reference.key_types.clone(),
            field_keys: Vec::new(),
            element_type: element_type.clone(),
        });
    } else if matches!(element_type, ValueType::Array(_)) {
        instructions.push(Instruction::StoreArrayDeepCopy {
            state_index: reference.state_index,
            key_types: key_types.clone(),
        });
    } else {
        instructions.push(Instruction::StoreMappingElement {
            state_index: reference.state_index,
            key_types,
        });
    }

    instructions.push(Instruction::LoadLocal(popped_local));
    instructions.push(Instruction::Jump { target: end_label });

    // Task #98 / Task #107 — Solidity 0.8.x specifies that `.pop()` on an
    // empty array reverts with Panic(0x31) (array-pop-underflow). Route
    // through the shared `emit_panic` helper which emits the canonical
    //   keccak256("Panic(uint256)")[0..4] || abi.encode(0x31)
    // payload so `ExecutionResult.return_data` begins with the EVM-canonical
    // Panic(uint256) envelope — matches the shape used by assert(false)=0x01,
    // enum-cast=0x21, arith-overflow=0x11, div-zero=0x12, abi.decode=0x41.
    instructions.push(Instruction::Label(empty_label));
    emit_panic(0x31, instructions);

    instructions.push(Instruction::Label(end_label));
    true
}
