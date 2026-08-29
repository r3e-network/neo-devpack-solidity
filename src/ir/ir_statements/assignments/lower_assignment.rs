//! ## Assignment Lowering (`lhs = rhs`)
//!
//! Lowers Solidity assignment statements into NeoVM instructions. Handles:
//!
//! * **Storage aliases** — `S[] storage a = arr;` creates an alias without copying
//! * **Tuple destructuring** — `(a, b) = expr` with task #156 parameter swap
//! * **Storage↔memory array copies** — element-wise `STORAGE_PUT` loops (Task #102)
//! * **Memory struct field assignment** — `s.field = rhs` (Task #191)
//! * **External dynamic assign** — `this.dynVar() = value` patterns
//!
//! ### Dispatch flow (within `lower_assignment`)
//!
//! 1. Storage alias check → early return if alias created
//! 2. Direct state-var array detection → skip reference path
//! 3. LHS resolution → match on expression shape (Identifier, Subscript, MemberAccess, Tuple)
//! 4. RHS lowering → emit storage/memory writes based on resolved LHS

use super::*;

// ============================================================================
// Section 1 — Main Assignment Lowering
// ============================================================================

pub(crate) fn lower_assignment(
    lhs: &Expression,
    rhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    if let Expression::Variable(identifier) = lhs {
        if ctx.storage_alias(&identifier.name).is_some() {
            if let Some(source_reference) = resolve_storage_reference(rhs, ctx) {
                ctx.set_storage_alias(identifier.name.clone(), source_reference);
                return;
            }
        }
    }

    // Task #117 regression guard: fix-117 widened `resolve_storage_reference`
    // so a bare `Expression::Variable` that names an Array- or Mapping-typed
    // state variable now returns a no-key/no-field StorageReference (needed so
    // `T[] storage a = arr;` alias declarations can anchor to `arr`). If we
    // routed `arr = m;` through that path here, the `emit_storage_store`
    // fallback would fire a single `StoreMappingElement` with empty
    // `key_types`, which writes the whole memory blob instead of the per-index
    // STORAGE_PUT loop that `arr[i]` reads expect. For direct state-var
    // assignments (no alias shadowing the name) we skip the reference path and
    // fall through to the Task #102 per-element copy below.
    let direct_state_var_array = if let Expression::Variable(identifier) = lhs {
        ctx.storage_alias(&identifier.name).is_none()
            && ctx
                .state_index_map
                .get(&identifier.name)
                .and_then(|idx| ctx.state_type(*idx))
                .map(|ty| matches!(ty, ValueType::Array(_) | ValueType::Mapping { .. }))
                .unwrap_or(false)
    } else {
        false
    };

    let resolved = if direct_state_var_array {
        None
    } else {
        resolve_storage_reference(lhs, ctx)
    };

    if let Some(reference) = resolved {
        if !ctx.ensure_state_writable(reference.state_index) {
            if lower_expression(rhs, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
            return;
        }

        if let ValueType::Struct { name, fields } = &reference.value_type {
            // Support struct construction on the RHS for storage assignments, both
            // named and positional: `S({a:1,b:2})` or `S(1,2)`.
            let mut ctor_args_by_name: Option<&[solang_parser::pt::NamedArgument]> = None;
            let mut ctor_args_positional: Option<&[Expression]> = None;
            let mut ctor_name_matches = false;

            match rhs {
                Expression::NamedFunctionCall(_, func, args) => {
                    if let Expression::Variable(identifier) = func.as_ref() {
                        ctor_name_matches = identifier.name.eq_ignore_ascii_case(name);
                        if ctor_name_matches {
                            ctor_args_by_name = Some(args.as_slice());
                        }
                    }
                }
                Expression::FunctionCall(_, func, args) => {
                    if let Expression::Variable(identifier) = func.as_ref() {
                        ctor_name_matches = identifier.name.eq_ignore_ascii_case(name);
                        if ctor_name_matches {
                            ctor_args_positional = Some(args.as_slice());
                        }
                    }
                }
                _ => {}
            }

            if ctor_name_matches {
                for (index, field) in fields.iter().enumerate() {
                    let mut field_reference = reference.clone();
                    field_reference.field_path.push(StorageReferenceField {
                        key: field.key,
                        ty: field.ty.clone(),
                    });
                    field_reference.value_type = field.ty.clone();

                    let success = if let Some(named_args) = ctor_args_by_name {
                        if let Some(arg) = named_args.iter().find(|arg| arg.name.name == field.name)
                        {
                            lower_expression(&arg.expr, ctx, instructions)
                        } else {
                            push_default_for_storage_value_type(&field.ty, ctx, instructions)
                        }
                    } else if let Some(pos_args) = ctor_args_positional {
                        if let Some(arg) = pos_args.get(index) {
                            lower_expression(arg, ctx, instructions)
                        } else {
                            push_default_for_storage_value_type(&field.ty, ctx, instructions)
                        }
                    } else {
                        push_default_for_storage_value_type(&field.ty, ctx, instructions)
                    };

                    if success && !emit_storage_store(&field_reference, ctx, instructions) {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                }

                return;
            }
        }

        // Canonicalize an integer-backed `bytesN` literal RHS to its big-endian
        // ByteString when the storage target (struct field, or mapping/array
        // element) is a `bytesN` — otherwise the hex literal is stored
        // little-endian and reads back byte-reversed. The target type is the
        // deepest struct field when present, else the reference's value type.
        let target_ty = reference
            .field_path
            .last()
            .map(|f| f.ty.clone())
            .unwrap_or_else(|| reference.value_type.clone());
        let success = if matches!(&target_ty, ValueType::ByteArray { fixed_len: Some(_) }) {
            try_lower_bytesn_literal_canonical(rhs, &target_ty, ctx, instructions)
                || lower_expression(rhs, ctx, instructions)
        } else {
            lower_expression(rhs, ctx, instructions)
        };
        if success {
            if !emit_storage_store(&reference, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
        } else {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
        return;
    }

    if let Expression::List(_, params) = lhs {
        // Tuple destructuring assignment: `(a, b) = expr;`
        // We represent tuples as arrays and assign by index.

        #[derive(Clone)]
        enum TupleTarget {
            Ignore,
            DeclaredLocal {
                local_index: usize,
                inferred_type: Option<ValueType>,
            },
            ExistingLocal(usize),
            // Task #156 — `(a, b) = (b, a)` where `a`/`b` are function parameters.
            // `resolve_local` only walks `local_index_map`; parameters live in
            // `param_index_map` and are written via `StoreParameter` / NeoVM
            // STARG. Prior to this target kind, the resolver returned `Invalid`
            // and the tuple element was silently dropped (visible effect: the
            // swap became a no-op on its own parameter slots).
            ExistingParameter(usize),
            ExistingState(usize),
            Storage(StorageReference),
            Nested(Vec<TupleTarget>),
            Invalid,
        }

        fn resolve_optional_tuple_target(
            parameter: &Option<solang_parser::pt::Parameter>,
            ctx: &mut LoweringContext,
        ) -> TupleTarget {
            let Some(parameter) = parameter else {
                return TupleTarget::Ignore;
            };

            if let Some(name) = parameter.name.as_ref() {
                // Declaration: `(bool ok, uint v) = ...`
                if ctx.is_local_in_current_scope(&name.name) {
                    ctx.record_error_with_suggestion(
                        format!("local variable '{}' redeclared", name.name),
                        "use a different variable name or assign to the existing variable instead of redeclaring",
                    );
                }

                let inferred_type = infer_type_from_expression(&parameter.ty, ctx);
                let local_index = ctx.allocate_local(name.name.clone(), inferred_type.clone());
                return TupleTarget::DeclaredLocal {
                    local_index,
                    inferred_type,
                };
            }

            if let Expression::List(_, nested_params) = &parameter.ty {
                let children = nested_params
                    .iter()
                    .map(|(_, param)| resolve_optional_tuple_target(param, ctx))
                    .collect();
                return TupleTarget::Nested(children);
            }

            // Assignment to an existing lvalue: `(ok, v) = ...`
            if let Some(reference) = resolve_storage_reference(&parameter.ty, ctx) {
                return TupleTarget::Storage(reference);
            }

            if let Expression::Variable(identifier) = &parameter.ty {
                if let Some(local_index) = ctx.resolve_local(&identifier.name) {
                    return TupleTarget::ExistingLocal(local_index);
                }

                // Task #156 — an identifier that doesn't match a local may still
                // be a function parameter. `lower_variable_expression` already
                // reads parameters via `LoadParameter`; we mirror that on the
                // write side with `StoreParameter` so a tuple-LVALUE `(a, b)`
                // on function params lands in the right slots. The RHS `(b, a)`
                // is fully stashed in `__tuple_assign` before any `StoreParameter`
                // fires, so the evaluate-RHS-first semantic holds.
                if let Some(param_index) = ctx.param_index_map.get(&identifier.name).copied() {
                    return TupleTarget::ExistingParameter(param_index);
                }

                if let Some(state_index) = ctx.state_index_map.get(&identifier.name).copied() {
                    return TupleTarget::ExistingState(state_index);
                }

                return TupleTarget::Invalid;
            }

            TupleTarget::Invalid
        }

        fn initialize_declared_tuple_targets(
            target: &TupleTarget,
            ctx: &mut LoweringContext,
            instructions: &mut Vec<Instruction>,
        ) {
            match target {
                TupleTarget::DeclaredLocal {
                    local_index,
                    inferred_type,
                } => {
                    if let Some(ty) = inferred_type.as_ref() {
                        push_default_for_value_type(ty, ctx, instructions);
                    } else {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                    }
                    instructions.push(Instruction::StoreLocal(*local_index));
                }
                TupleTarget::Nested(children) => {
                    for child in children {
                        initialize_declared_tuple_targets(child, ctx, instructions);
                    }
                }
                _ => {}
            }
        }

        fn emit_tuple_element_load(
            tuple_local: usize,
            path: &[usize],
            instructions: &mut Vec<Instruction>,
        ) {
            instructions.push(Instruction::LoadLocal(tuple_local));
            for index in path {
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(*index as u64),
                )));
                instructions.push(Instruction::ArrayGet);
            }
        }

        fn assign_tuple_target(
            tuple_local: usize,
            path: &mut Vec<usize>,
            target: &TupleTarget,
            ctx: &mut LoweringContext,
            instructions: &mut Vec<Instruction>,
        ) {
            match target {
                TupleTarget::Ignore => {}
                TupleTarget::Nested(children) => {
                    for (index, child) in children.iter().enumerate() {
                        path.push(index);
                        assign_tuple_target(tuple_local, path, child, ctx, instructions);
                        path.pop();
                    }
                }
                TupleTarget::DeclaredLocal { local_index, .. }
                | TupleTarget::ExistingLocal(local_index) => {
                    emit_tuple_element_load(tuple_local, path, instructions);
                    ctx.clear_call_data_local(*local_index);
                    instructions.push(Instruction::StoreLocal(*local_index));
                }
                TupleTarget::ExistingParameter(param_index) => {
                    // Task #156 — same shape as StoreLocal but writes to the NeoVM
                    // arg slot (STARG) so the caller-observed parameter is
                    // updated. The tuple RHS has already been materialised into
                    // `__tuple_assign`, so `(a, b) = (b, a)` reads pre-swap
                    // values before any STARG overwrites the originals.
                    emit_tuple_element_load(tuple_local, path, instructions);
                    instructions.push(Instruction::StoreParameter(*param_index));
                }
                TupleTarget::ExistingState(state_index) => {
                    emit_tuple_element_load(tuple_local, path, instructions);
                    if ctx.ensure_state_writable(*state_index) {
                        instructions.push(Instruction::StoreState(*state_index));
                    } else {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                }
                TupleTarget::Storage(reference) => {
                    emit_tuple_element_load(tuple_local, path, instructions);
                    if ctx.ensure_state_writable(reference.state_index) {
                        if !emit_storage_store(reference, ctx, instructions) {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    } else {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                }
                TupleTarget::Invalid => {
                    emit_tuple_element_load(tuple_local, path, instructions);
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
            }
        }

        let targets: Vec<TupleTarget> = params
            .iter()
            .map(|(_, parameter)| resolve_optional_tuple_target(parameter, ctx))
            .collect();

        // Lower RHS into a temporary buffer so failures don't leave partial stack state.
        let mut rhs_instrs = Vec::new();
        if !lower_expression(rhs, ctx, &mut rhs_instrs) {
            // Ensure declared locals exist with default values to avoid cascading errors.
            for target in &targets {
                initialize_declared_tuple_targets(target, ctx, instructions);
            }
            return;
        }
        instructions.append(&mut rhs_instrs);

        // Detect `(a, b, ...) = this.<method>()` where `<method>` is an
        // externally-callable function in the current contract returning
        // multiple values. The external-call lowering routes `this.<method>`
        // through `System.Contract.Call` (which for self-dispatch runs the
        // callee's body, including its ABI-encoding epilogue) — so after
        // `lower_expression(rhs)` the stack carries an EVM-canonical
        // 32-byte-per-slot ABI-encoded ByteString, NOT a NeoVM `Array`.
        //
        // The standard destructure path below (STLOC → PICKITEM[i]) indexes
        // the top-of-stack item as an Array, which for a ByteString yields
        // individual bytes (each 0x00 for a well-aligned BE slot). Mirror
        // Solidity's `(a, b) = abi.decode(this.f(), (T1, T2, ...))` sugar by
        // ABI-decoding the buffer into an Array of native values, matching
        // the arity and static types of the LHS tuple targets. The existing
        // PICKITEM chain then operates on the Array correctly.
        //
        // Only static-slot types (uint*/int*/bool/address/bytesN) are
        // handled here; dynamic types (string/bytes/T[]) fall through to the
        // legacy path.
        if is_this_external_tuple_call(rhs, ctx) {
            // Walk the declared LHS parameter types (not the `TupleTarget`
            // enum, which is function-local) to collect static-slot types
            // suitable for the compile-time EVM ABI-decode path. Any miss
            // (Ignore/Invalid/nested/dynamic-type) falls back to the legacy
            // destructure.
            // Per-slot static types. A SKIPPED tuple slot — `(a, ) = f()` or
            // `( , b) = f()` — has no LHS parameter; represent it as an inner
            // `None` so it is left undecoded (and ignored by the destructure
            // below). Previously `param.as_ref()?` short-circuited the entire
            // `collect()` to `None` on the first skipped slot, so a partial
            // destructure fell through to the legacy PICKITEM path — which
            // indexes the ABI ByteString as an Array and reads back garbage
            // (each well-aligned BE slot looks like 0x00 bytes). Each static
            // slot is decoded at head offset `index * 32` independently (see
            // `emit_abi_decode_slot_slice`), so skipped slots never shift the
            // offsets of the slots we keep. A KEPT slot that is dynamic or
            // un-inferable still bails to the legacy path (outer `None`).
            let slot_types: Option<Vec<Option<ValueType>>> = params
                .iter()
                .map(|(_, param)| match param.as_ref() {
                    None => Some(None),
                    Some(parameter) => {
                        let ty = infer_type_from_expression(&parameter.ty, ctx)?;
                        if abi_static_slot_count(&ty) == Some(1) {
                            Some(Some(ty))
                        } else {
                            None
                        }
                    }
                })
                .collect();
            if let Some(slot_types) = slot_types {
                // Stack top: ABI-encoded ByteString of `slot_types.len() * 32` bytes.
                let buffer_local = ctx.allocate_local(
                    "__this_tuple_abi_buf".to_string(),
                    Some(ValueType::ByteArray { fixed_len: None }),
                );
                instructions.push(Instruction::StoreLocal(buffer_local));

                // Materialise a fresh Array of the same arity as the LHS tuple.
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(slot_types.len() as u64),
                )));
                instructions.push(Instruction::NewArray {
                    element_type: ValueType::Any,
                });
                let array_local = ctx.allocate_local(
                    "__this_tuple_array".to_string(),
                    Some(ValueType::Array(Box::new(ValueType::Any))),
                );
                instructions.push(Instruction::StoreLocal(array_local));

                // Decode each KEPT static slot in-place from the buffer; skipped
                // slots stay `Null` (the destructure below ignores them).
                for (index, slot) in slot_types.iter().enumerate() {
                    if let Some(value_type) = slot {
                        instructions.push(Instruction::LoadLocal(array_local));
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(index as u64),
                        )));
                        emit_abi_decode_static_slot(
                            buffer_local,
                            index,
                            value_type,
                            ctx,
                            instructions,
                        );
                        instructions.push(Instruction::ArraySet);
                    }
                }

                // Hand the populated Array to the legacy tuple-destructure below.
                instructions.push(Instruction::LoadLocal(array_local));
            }
        }

        let tuple_local = ctx.allocate_local("__tuple_assign".to_string(), None);
        instructions.push(Instruction::StoreLocal(tuple_local));

        for (index, target) in targets.iter().enumerate() {
            let mut path = vec![index];
            assign_tuple_target(tuple_local, &mut path, target, ctx, instructions);
        }

        return;
    }

    if matches!(lhs, Expression::ArraySubscript(_, _, Some(_))) {
        lower_array_store(lhs, rhs, ctx, instructions);
        return;
    }

    if let Expression::Variable(identifier) = lhs {
        if let Some(index) = ctx.resolve_local(&identifier.name) {
            match parse_low_level_call_data(rhs, ctx) {
                Ok(Some((method_name, encode_args))) => {
                    let mut lowered = true;
                    for arg in &encode_args {
                        if !lower_expression(arg, ctx, instructions) {
                            lowered = false;
                        }
                    }

                    if lowered {
                        if encode_args.is_empty() {
                            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                                BigInt::zero(),
                            )));
                            instructions.push(Instruction::NewArray {
                                element_type: ValueType::Any,
                            });
                            instructions.push(Instruction::CallBuiltin {
                                builtin: BuiltinCall::NativeCall {
                                    contract: NativeContract::StdLib,
                                    method: "serialize".to_string(),
                                },
                                arg_count: 1,
                            });
                        } else {
                            instructions.push(Instruction::CallBuiltin {
                                builtin: BuiltinCall::AbiEncode,
                                arg_count: encode_args.len(),
                            });
                        }
                        instructions.push(Instruction::StoreLocal(index));
                        ctx.set_call_data_local(index, method_name);
                    }
                }
                Ok(None) => {
                    // Wave-#28: same-contract dynamic-return path. See
                    // `try_lower_this_external_dynamic_assign` for details.
                    let dst_type = ctx.local_type(index).cloned();
                    let decoded = match dst_type.as_ref() {
                        Some(ty) => try_lower_this_external_dynamic_assign(
                            index,
                            rhs,
                            ty,
                            ctx,
                            instructions,
                        ),
                        None => false,
                    };
                    if !decoded && lower_expression(rhs, ctx, instructions) {
                        instructions.push(Instruction::StoreLocal(index));
                        ctx.clear_call_data_local(index);
                    }
                }
                Err(message) => {
                    ctx.record_error(message);
                    ctx.clear_call_data_local(index);
                }
            }
            return;
        }
        // Task #156 — plain `a = expr;` on a function parameter. Before this
        // branch, `ensure_local` further below would allocate a fresh local
        // (shadowing the name in `local_index_map` but NOT in
        // `param_index_map`), so subsequent reads — which check params first
        // in `lower_variable_expression` — would keep returning the original
        // parameter value. The write was silently lost. Emit `StoreParameter`
        // so the caller-visible slot is updated and later reads stay coherent.
        if let Some(index) = ctx.param_index_map.get(&identifier.name).copied() {
            if lower_expression(rhs, ctx, instructions) {
                instructions.push(Instruction::StoreParameter(index));
            }
            return;
        }
        if let Some(index) = ctx.state_index_map.get(&identifier.name).copied() {
            // Task #102: `storage_arr = memory_arr` must lower to element-wise
            // STORAGE_PUT plus a length-slot update (and delete any trailing
            // slots left over from a longer prior value). The default
            // StoreState path only writes the length slot — element slots
            // are keyed mapping-style via StoreMappingElement.
            if matches!(ctx.state_type(index), Some(ValueType::Array(_))) {
                lower_storage_array_assign_from_memory(index, rhs, ctx, instructions);
                return;
            }
            // Canonicalize an integer-backed `bytesN` literal RHS to its
            // big-endian ByteString for a `bytesN` state var. Without this the
            // hex literal lowers to a little-endian Integer and StoreState
            // persists it byte-reversed, so `uint256(top)` / byte indexing read
            // back a corrupted value (the same class of bug already fixed for
            // call args, struct fields, and mapping keys).
            let state_ty = ctx.state_type(index).cloned();
            let lowered = match state_ty.as_ref() {
                Some(ty @ ValueType::ByteArray { fixed_len: Some(_) }) => {
                    try_lower_bytesn_literal_canonical(rhs, ty, ctx, instructions)
                        || lower_expression(rhs, ctx, instructions)
                }
                _ => lower_expression(rhs, ctx, instructions),
            };
            if lowered {
                if ctx.ensure_state_writable(index) {
                    instructions.push(Instruction::StoreState(index));
                } else {
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
            }
            return;
        }

        let index = ctx.ensure_local(&identifier.name);
        match parse_low_level_call_data(rhs, ctx) {
            Ok(Some((method_name, encode_args))) => {
                let mut lowered = true;
                for arg in &encode_args {
                    if !lower_expression(arg, ctx, instructions) {
                        lowered = false;
                    }
                }

                if lowered {
                    if encode_args.is_empty() {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                        instructions.push(Instruction::NewArray {
                            element_type: ValueType::Any,
                        });
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::NativeCall {
                                contract: NativeContract::StdLib,
                                method: "serialize".to_string(),
                            },
                            arg_count: 1,
                        });
                    } else {
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::AbiEncode,
                            arg_count: encode_args.len(),
                        });
                    }
                    instructions.push(Instruction::StoreLocal(index));
                    ctx.set_call_data_local(index, method_name);
                }
            }
            Ok(None) => {
                if lower_expression(rhs, ctx, instructions) {
                    instructions.push(Instruction::StoreLocal(index));
                    ctx.clear_call_data_local(index);
                }
            }
            Err(message) => {
                ctx.record_error(message);
                ctx.clear_call_data_local(index);
            }
        }
        return;
    }

    // Task #191 — memory struct field assignment: `s.field = rhs` where `s` is a
    // local or function parameter whose type is a struct represented as an
    // array of fields. The compound-assignment path (`s.field += v`,
    // `s.field++`) already handles the local-base case via ArrayGet/ArraySet,
    // but plain assignment had no MemberAccess-LHS branch and fell through to
    // the silent-drop fallback below. This made every `s.field = v` a no-op,
    // which in turn made free-function-attach chains like
    // `c.inc().inc().inc()` (where `inc(Counter memory c)` writes `c.value`)
    // appear to flatten to identity — the returned copy was always the
    // untouched initial struct.
    //
    // NeoVM Array slots hold an Rc<RefCell<Vec<StackItem>>>, so loading the
    // parameter/local and then ArraySet-ing a field mutates the underlying
    // storage in place. No write-back to the parameter/local slot is needed
    // because the slot still holds the same reference.
    if let Expression::MemberAccess(_, inner, member) = lhs {
        if let Expression::Variable(base) = inner.as_ref() {
            if let Some(ValueType::Struct { fields, .. }) = infer_type_from_expression(inner, ctx) {
                if let Some((field_index, field)) = fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| field.name == member.name)
                {
                    let field_ty = field.ty.clone();
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
                        // Emit: load base → push field_index → load rhs → ArraySet.
                        // ArraySet pops (array, index, value) and mutates the
                        // array in place.
                        instructions.push(load_base);
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(field_index as u64),
                        )));
                        // Canonicalize an integer-backed `bytesN` literal RHS to
                        // its big-endian ByteString for a `bytesN` struct field
                        // (otherwise the hex literal is stored little-endian and
                        // reads back byte-reversed) — same class as the scalar
                        // state-var, call-arg, and mapping-key fixes.
                        let lowered =
                            if matches!(&field_ty, ValueType::ByteArray { fixed_len: Some(_) }) {
                                try_lower_bytesn_literal_canonical(
                                    rhs,
                                    &field_ty,
                                    ctx,
                                    instructions,
                                ) || lower_expression(rhs, ctx, instructions)
                            } else {
                                lower_expression(rhs, ctx, instructions)
                            };
                        if !lowered {
                            // Leave the half-built frame balanced: we've pushed
                            // base + index; drop them to keep the stack clean.
                            instructions.push(Instruction::Drop(ValueType::Any));
                            instructions.push(Instruction::Drop(ValueType::Any));
                            return;
                        }
                        instructions.push(Instruction::ArraySet);
                        return;
                    }
                }
            }
        }
    }

    // Fallback: evaluate RHS (if possible) and drop to allow compilation to continue.
    if lower_expression(rhs, ctx, instructions) {
        instructions.push(Instruction::Drop(ValueType::Any));
    }
}

// Task #102 — `storage_arr = memory_arr` bulk copy.
//
// Storage dynamic arrays in this IR lower their length to a scalar state slot
// (accessed via `LoadState`/`StoreState`) and their elements to mapping-style
// slots keyed by uint256 index (accessed via `LoadMappingElement`/
// `StoreMappingElement` with `key_types = [uint256]`). A plain `arr = m`
// therefore needs three things: (1) update the length slot, (2) copy each
// `m[i]` into the per-index slot, and (3) clear any trailing slots that were
// populated by a longer prior value so the state matches Solidity semantics.
//
// Slot-key scheme: identical to the existing `.push()` / `.pop()` / subscript
// paths — `state_index` plus a single uint256 key. Re-using those paths keeps
// the in-storage representation compatible with subsequent reads via
// `arr[i]` and with length-aware helpers like `arr.length`.
