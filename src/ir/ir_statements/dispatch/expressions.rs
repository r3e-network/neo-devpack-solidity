use super::*;

pub(crate) fn lower_expression_statement(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Expression::Assign(_, lhs, rhs) = expr {
        lower_assignment(lhs, rhs, ctx, instructions);
    } else if let Expression::FunctionCall(_, func, args) = expr {
        if let Expression::Variable(identifier) = func.as_ref() {
            if identifier.name == "require" {
                lower_require(args, ctx, instructions);
                return false;
            }
            if identifier.name == "assert" {
                lower_assert(args, ctx, instructions);
                return false;
            }
        }
        if lower_expression(expr, ctx, instructions) {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
    } else if lower_expression(expr, ctx, instructions) {
        instructions.push(Instruction::Drop(ValueType::Any));
    }
    false
}

pub(crate) fn lower_variable_definition_statement(
    decl: &solang_parser::pt::VariableDeclaration,
    init: Option<&Expression>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(ident) = &decl.name {
        if ctx.is_local_in_current_scope(&ident.name) {
            ctx.record_error_with_suggestion(
                format!("local variable '{}' redeclared", ident.name),
                "use a different variable name or assign to the existing variable instead of redeclaring",
            );
        } else {
            let is_storage_reference = matches!(decl.storage, Some(PtStorageLocation::Storage(_)));
            let mut inferred_type = if is_storage_reference {
                None
            } else {
                infer_type_from_expression(&decl.ty, ctx)
            };

            // Best-effort: infer user-defined struct types for locals so that member
            // access (`tmp.field`) can be lowered correctly for memory copies.
            if !is_storage_reference && inferred_type.is_none() {
                if let Expression::Variable(type_ident) = &decl.ty {
                    inferred_type = ctx
                        .defined_struct_types
                        .iter()
                        .chain(ctx.state_types.iter())
                        .chain(ctx.param_types.iter())
                        .chain(ctx.return_types.iter())
                        .chain(ctx.local_types.values())
                        .find_map(|ty| find_named_struct_type(ty, &type_ident.name));
                }
            }

            let slot = ctx.allocate_local(ident.name.clone(), inferred_type.clone());

            // An internal function-pointer local (`function (…) internal … f`)
            // must dispatch through CALLA like an fp PARAMETER does — otherwise
            // a later `f(args)` hits the silent compatibility fallback that
            // drops the arguments and yields 0. Register the binding (arg count
            // + has-return parsed from the declared type) so
            // `try_lower_variable_call` emits a `CallIndirect` instead. Mirrors
            // the parameter path in `ir/build/function.rs`.
            if let Expression::Type(
                _,
                PtType::Function {
                    params, returns, ..
                },
            ) = &decl.ty
            {
                let arg_count = params.len();
                let has_return = returns
                    .as_ref()
                    .map(|(rets, _)| !rets.is_empty())
                    .unwrap_or(false);
                ctx.register_function_pointer_binding(&ident.name, arg_count, has_return);
            }

            if let Some(initializer) = init {
                if is_storage_reference {
                    if let Some(reference) = resolve_storage_reference(initializer, ctx) {
                        ctx.set_storage_alias(ident.name.clone(), reference);
                    } else if lower_expression(initializer, ctx, instructions) {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                } else {
                    match parse_low_level_call_data(initializer, ctx) {
                        Ok(Some((method_name, encode_args))) => {
                            // Support `bytes data = abi.encodeWithSignature/encodeWithSelector(...)`
                            // for subsequent `address.call(data)` lowering.
                            let mut lowered = true;
                            for arg in &encode_args {
                                if !lower_expression(arg, ctx, instructions) {
                                    lowered = false;
                                }
                            }

                            if lowered {
                                if encode_args.is_empty() {
                                    instructions.push(Instruction::PushLiteral(
                                        LiteralValue::Integer(BigInt::zero()),
                                    ));
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
                                instructions.push(Instruction::StoreLocal(slot));
                                ctx.set_call_data_local(slot, method_name);
                            }
                        }
                        Ok(None) => {
                            // Bug-hunt #10 — `T[] memory m = storageArr;` is a
                            // DEEP COPY. A storage dynamic array keeps only its
                            // length in the state slot, so lowering the bare
                            // Variable yields an Integer; `m[i] = v` then faults
                            // `SETITEM: unsupported target Integer`. Materialize a
                            // fresh in-memory array from the slot's elements
                            // instead. Only fires for an unshadowed state-var
                            // array initializer (a shadowing local/param/alias is
                            // a memory array, whose `=` aliasing is EVM-correct).
                            let materialized =
                                if let (Some(ValueType::Array(elem)), Expression::Variable(id)) =
                                    (inferred_type.as_ref(), initializer)
                                {
                                    let shadowed = ctx.storage_alias(&id.name).is_some()
                                        || ctx.param_index_map.contains_key(&id.name)
                                        || ctx.resolve_local(&id.name).is_some();
                                    let sidx = if shadowed {
                                        None
                                    } else {
                                        ctx.state_index_map.get(&id.name).copied()
                                    };
                                    match sidx {
                                        Some(sidx)
                                            if matches!(
                                                ctx.state_type(sidx),
                                                Some(ValueType::Array(_))
                                            ) =>
                                        {
                                            let elem_ty = (**elem).clone();
                                            lower_storage_array_read_to_memory(
                                                sidx,
                                                elem_ty,
                                                slot,
                                                ctx,
                                                instructions,
                                            );
                                            ctx.clear_call_data_local(slot);
                                            true
                                        }
                                        _ => false,
                                    }
                                } else {
                                    false
                                };
                            if materialized {
                                // fully handled (materialize + store into slot)
                                return false;
                            }
                            // Wave-#28 fix: `T[] memory got = this.method();`
                            // (and the interface-cast / address-typed shapes
                            // covered by `is_this_external_tuple_call`) must
                            // run the EVM-canonical dynamic-array decode on
                            // the returned ByteString before storing into
                            // the local. Without this, `got.length` reads
                            // the wire byte count (e.g. 224 for a five-elem
                            // `uint256[]`) instead of the decoded element
                            // count (5). See
                            // `try_lower_this_external_dynamic_assign` in
                            // `ir/statements/assignments/lower_assignment.rs`
                            // for the full rationale.
                            let decoded = if let Some(dst_type) = inferred_type.as_ref() {
                                try_lower_this_external_dynamic_assign(
                                    slot,
                                    initializer,
                                    dst_type,
                                    ctx,
                                    instructions,
                                ) || try_lower_plain_external_dynamic_assign(
                                    slot,
                                    initializer,
                                    dst_type,
                                    ctx,
                                    instructions,
                                )
                            } else {
                                false
                            };
                            // Canonicalize an integer-backed `bytesN` literal to
                            // its big-endian ByteArray when binding to a `bytesN`
                            // local, so the stored value matches the cast/param
                            // representation and downstream consumers (abi.encode,
                            // struct/array slots, indexing) behave.
                            let mut handled = false;
                            if !decoded {
                                if let Some(ty) = inferred_type.as_ref() {
                                    if try_lower_bytesn_literal_canonical(
                                        initializer,
                                        ty,
                                        ctx,
                                        instructions,
                                    ) {
                                        handled = true;
                                    }
                                }
                                if !handled && lower_expression(initializer, ctx, instructions) {
                                    handled = true;
                                }
                            }
                            if handled {
                                instructions.push(Instruction::StoreLocal(slot));
                                ctx.clear_call_data_local(slot);
                            }
                        }
                        Err(message) => {
                            ctx.record_error(message);
                            ctx.clear_call_data_local(slot);
                        }
                    }
                }
            } else if !is_storage_reference {
                // Task #49 fix: `T[N] memory a;` (no initializer) must allocate a real
                // StackItem::Array of length N with zero-initialized elements. Without
                // this, push_default_for_value_type(Array) emits NEWARRAY 0 which fails
                // at runtime (SETITEM "unsupported target Integer(0)", SIZE "unsupported
                // type"). Mirrors the `new T[N]` path in lower_new_array_allocation.
                if let Expression::ArraySubscript(_, array_type_expr, Some(length_expr)) = &decl.ty
                {
                    lower_new_array_allocation(
                        array_type_expr.as_ref(),
                        length_expr.as_ref(),
                        ctx,
                        instructions,
                    );
                } else if let Some(value_type) = inferred_type.as_ref() {
                    push_default_for_value_type(value_type, ctx, instructions);
                } else {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(0u8),
                    )));
                }
                instructions.push(Instruction::StoreLocal(slot));
                ctx.clear_call_data_local(slot);
            }
        }
    } else {
        ctx.record_error_with_suggestion(
            "variable declaration missing identifier",
            "every variable declaration must have a name: e.g. uint256 myVar = 0",
        );
    }
    false
}

pub(crate) fn lower_emit_statement(
    call: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    lower_emit(call, ctx, instructions);
    false
}

pub(crate) fn lower_assembly_statement(
    block: &solang_parser::pt::YulBlock,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Task #99 — narrow yul support: lower mstore/mload/return + let/:= and a
    // handful of arithmetic opcodes into NeoVM IR. Specialized handlers
    // (e.g., extsload/exttload from the param-name sniffer) still take
    // precedence when they recognise the enclosing function shape.
    if lower_special_assembly(ctx, instructions) {
        return false;
    }

    // Attempt to lower the yul block. If any statement is unsupported, we
    // fall back to the legacy no-op compatibility warning so contracts that
    // use more exotic yul (for/switch/sload/sstore/...) continue to compile.
    if lower_yul_block(block, ctx, instructions) {
        return false;
    }

    // Compatibility mode: preserve compilation for contracts that use
    // inline assembly by treating unrecognized assembly blocks as no-ops.
    ctx.record_warning_with_suggestion(
        "inline assembly block compiled as no-op: NeoVM does not support EVM \
         assembly instructions. Any logic inside this assembly block will be silently \
         skipped at runtime.",
        "replace inline assembly with equivalent Solidity code, or use Neo-specific \
         builtins for low-level operations",
    );
    false
}
