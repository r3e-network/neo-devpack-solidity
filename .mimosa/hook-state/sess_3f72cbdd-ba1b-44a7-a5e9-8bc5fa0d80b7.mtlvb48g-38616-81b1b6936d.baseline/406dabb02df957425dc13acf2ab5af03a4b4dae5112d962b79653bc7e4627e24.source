use super::*;

impl Function {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_metadata_with_warnings(
        metadata: &FunctionMetadata,
        current_contract_name: &str,
        state_variables: &[StateVariableMetadata],
        state_index_map: &HashMap<String, usize>,
        state_types: &[ValueType],
        defined_struct_types: &[ValueType],
        struct_fixed_array_bounds: &HashMap<(String, String), u64>,
        event_index_map: &HashMap<String, usize>,
        event_signature_map: &HashMap<String, Vec<ManifestType>>,
        event_params_map: &HashMap<String, EventSignature>,
        error_signature_map: &HashMap<String, ErrorAbiSignature>,
        enum_variant_map: &HashMap<String, HashMap<String, u64>>,
        contract_types: &HashSet<String>,
        selector_registry: &SelectorRegistry,
        function_names: &HashSet<String>,
        function_overloads: &FunctionOverloadTable,
        function_first_param_types: &HashMap<(String, usize), Vec<ValueType>>,
        function_return_types: &HashMap<(String, usize), ValueType>,
        using_target_types: &[Option<String>],
        using_function_list_targets: &HashMap<String, Vec<Option<String>>>,
        using_function_list_scope_targets: &[Option<String>],
        function_param_names: &HashMap<(String, usize), Vec<String>>,
        void_functions: &HashSet<String>,
        externally_callable_functions: &HashSet<(String, usize)>,
        super_method_map: &HashMap<String, String>,
        library_storage_bodies: &HashMap<(String, usize), LibraryStorageBody>,
        storage_pointer_returning_fns: &HashMap<String, String>,
    ) -> Result<(Self, Vec<crate::solidity::Diagnostic>), Vec<IrDiagnostic>> {
        let parameters: Vec<ValueType> = metadata
            .parameters
            .iter()
            .map(ValueType::from_parameter)
            .collect();

        let returns: Vec<ValueType> = metadata
            .return_parameters
            .iter()
            .map(ValueType::from_parameter)
            .collect();

        let param_index_map = build_parameter_index_map(metadata);
        // Task #64 — Public/External functions are directly callable from
        // the host, so their RET hits the main-frame `stack_item_to_bytes`
        // path. Route multi-value returns through `abiEncode` for those;
        // leave Internal/Private alone so intra-contract callers can still
        // destructure via `ArrayGet`.
        let is_externally_callable = matches!(
            metadata.visibility,
            crate::frontend::VisibilityKind::External | crate::frontend::VisibilityKind::Public
        );
        let mut ctx = LoweringContext::new(
            &metadata.name,
            current_contract_name,
            metadata.selector,
            metadata.state_mutability.is_safe(),
            is_externally_callable,
            param_index_map,
            &parameters,
            state_variables,
            state_index_map,
            state_types,
            defined_struct_types,
            struct_fixed_array_bounds,
            event_index_map,
            event_signature_map,
            event_params_map,
            error_signature_map,
            enum_variant_map,
            contract_types,
            selector_registry,
            function_names,
            function_overloads,
            function_first_param_types,
            function_return_types,
            using_target_types,
            using_function_list_targets,
            using_function_list_scope_targets,
            function_param_names,
            void_functions,
            externally_callable_functions,
            super_method_map,
            library_storage_bodies,
            storage_pointer_returning_fns,
        );

        // Task #186 — register function-pointer bindings for parameters whose
        // declared Solidity type is `function (...) ... returns (...)`. The
        // type string is produced verbatim by `solang-parser::Display`; the
        // shape we recognize is a leading "function" token followed by a
        // parenthesized argument list and an optional `returns (...)` clause.
        // Parsing by string is sufficient because `NeoType` maps function
        // types to `None`, so the ValueType layer cannot carry this info; we
        // avoid plumbing a new ValueType variant through 55+ sites and keep
        // the change local.
        for param in metadata.parameters.iter() {
            if let Some(name) = &param.name {
                if let Some(binding) = parse_function_pointer_type(&param.ty) {
                    ctx.register_function_pointer_binding(name, binding.0, binding.1);
                }
            }
        }

        let mut instructions: Vec<Instruction> = Vec::new();
        let mut return_slots: Vec<Option<usize>> = Vec::new();
        // Task #114 — when modifier epilogue redirect is active, every
        // declared return needs a backing slot (even the unnamed ones) so
        // `Return(expr)` inside the body can store through to it before
        // jumping past the epilogue. We allocate synthetic names so the
        // rest of the lowering continues to see `return_slots[i] == Some`
        // uniformly.
        let needs_synth_return_slots = metadata.had_modifier_epilogue && !returns.is_empty();
        // The modifier-return REDIRECT must be active whenever a modifier
        // epilogue exists — including for VOID functions and constructors (which
        // have no return values, so `needs_synth_return_slots` is false). Without
        // it a bare `return;` in such a body emits `ReturnVoid` directly and
        // SKIPS the epilogue, e.g. a `nonReentrant` guard's `locked = 0;` never
        // runs and the contract is bricked. Slot allocation still gates on
        // `needs_synth_return_slots` (only declared returns need slots).
        let in_modifier_epilogue = metadata.had_modifier_epilogue;
        for (idx, (ret_param, value_type)) in metadata
            .return_parameters
            .iter()
            .zip(returns.iter())
            .enumerate()
        {
            if let Some(name) = &ret_param.name {
                let slot = ctx.allocate_local(name.clone(), Some(value_type.clone()));
                if push_default_for_value_type(value_type, &mut ctx, &mut instructions) {
                    instructions.push(Instruction::StoreLocal(slot));
                }
                return_slots.push(Some(slot));
            } else if needs_synth_return_slots {
                let synth_name = format!("__modret_{idx}");
                let slot = ctx.allocate_local(synth_name, Some(value_type.clone()));
                if push_default_for_value_type(value_type, &mut ctx, &mut instructions) {
                    instructions.push(Instruction::StoreLocal(slot));
                }
                return_slots.push(Some(slot));
            } else {
                return_slots.push(None);
            }
        }
        ctx.set_return_info(return_slots.clone(), returns.clone());
        // Task #185 — plumb the raw Solidity type strings for return
        // parameters (e.g. "uint[3][2]") so `lower_return_statement` can
        // detect nested fixed-size arrays and pick flat EVM-canonical
        // encoding over the dynamic-array offset+length wrapper.
        ctx.set_return_type_strings(
            metadata
                .return_parameters
                .iter()
                .map(|param| param.ty.clone())
                .collect(),
        );

        // Task #114 — activate the modifier-return redirect before lowering
        // the body. `lower_return_statement` will check this and emit
        // "store to slots + jump to modifier-break label" instead of a raw
        // RET, so modifier epilogues still run between the body's `return`
        // and the function's actual exit.
        let modifier_end_label = if in_modifier_epilogue {
            let label = ctx.next_label();
            ctx.set_modifier_return_redirect(return_slots.clone(), label);
            Some(label)
        } else {
            None
        };

        let mut returned = false;

        if let Some(body) = &metadata.body {
            returned = lower_statement(body, &mut ctx, &mut instructions);
        }

        // Clear the redirect before the trailing fall-through emission so
        // the explicit `Return` we emit below isn't itself redirected.
        if in_modifier_epilogue {
            ctx.clear_modifier_return_redirect();
            if let Some(end_label) = modifier_end_label {
                instructions.push(Instruction::Label(end_label));
            }
            // Force the fall-through return emission: the body may have
            // ended with a `Return` that was redirected (so `returned` is
            // `true` from the redirect's `return true;`), but control can
            // fall through the epilogue too. We always emit the final RET
            // sequence when modifier epilogue is active.
            returned = false;
        }

        if !returned {
            match metadata.kind {
                MetadataFunctionKind::Constructor => instructions.push(Instruction::ReturnVoid),
                _ if returns.is_empty() => instructions.push(Instruction::ReturnVoid),
                _ => {
                    if returns.len() == 1 {
                        if let Some(index) = return_slots.first().and_then(|slot| *slot) {
                            instructions.push(Instruction::LoadLocal(index));
                            // Task #64/#137 — keep the fall-off-end shape
                            // identical to explicit `return x;`: externally-
                            // callable ARRAY returns are abi-encoded into
                            // canonical bytes instead of leaking the raw
                            // StackItem::Array as serde_json.
                            let _ = wrap_external_single_array_return_value(
                                &mut ctx,
                                &mut instructions,
                            );
                            instructions.push(Instruction::Return);
                        } else if let Some(ret_ty) = returns.first() {
                            push_default_for_value_type(ret_ty, &mut ctx, &mut instructions);
                            let _ = wrap_external_single_array_return_value(
                                &mut ctx,
                                &mut instructions,
                            );
                            instructions.push(Instruction::Return);
                        } else {
                            instructions.push(Instruction::ReturnVoid);
                        }
                    } else if is_externally_callable {
                        // Task #64 — implicit end-of-function multi-return.
                        // Mirror the explicit `return;` path in
                        // return_revert.rs: load each declared return local
                        // (or a default for unnamed slots) and hand the
                        // sequence to `abiEncode` so the main-frame RET
                        // emits EVM-canonical BE-packed bytes rather than
                        // a serde_json-serialised StackItem::Array.
                        //
                        // Guarded on `is_externally_callable` — internal
                        // functions keep the Array shape so intra-contract
                        // callers can still destructure via `ArrayGet`.
                        let static_slot_return =
                            returns.iter().all(build_is_static_abi_slot_value_type);
                        for (slot, value_type) in return_slots.iter().zip(returns.iter()) {
                            if let Some(local_index) = slot {
                                instructions.push(Instruction::LoadLocal(*local_index));
                            } else {
                                push_default_for_value_type(
                                    value_type,
                                    &mut ctx,
                                    &mut instructions,
                                );
                            }
                            if static_slot_return
                                && !build_emit_static_abi_slot_for_value_type(
                                    value_type,
                                    &mut ctx,
                                    &mut instructions,
                                )
                            {
                                ctx.record_error("failed to encode static ABI return slot");
                                break;
                            }
                        }
                        if static_slot_return {
                            instructions.push(Instruction::CallBuiltin {
                                builtin: BuiltinCall::BytesConcat,
                                arg_count: returns.len(),
                            });
                        } else {
                            instructions.push(Instruction::CallBuiltin {
                                builtin: BuiltinCall::AbiEncode,
                                arg_count: returns.len(),
                            });
                        }
                        instructions.push(Instruction::Return);
                    } else {
                        // Legacy Array-packed shape for internal/private multi-return.
                        let tmp_id = ctx.next_label();
                        let array_local = ctx.allocate_local(
                            format!("__return_tuple_{tmp_id}"),
                            Some(ValueType::Array(Box::new(ValueType::Any))),
                        );

                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(returns.len() as u64),
                        )));
                        instructions.push(Instruction::NewArray {
                            element_type: ValueType::Any,
                        });
                        instructions.push(Instruction::StoreLocal(array_local));

                        for (index, (slot, value_type)) in
                            return_slots.iter().zip(returns.iter()).enumerate()
                        {
                            instructions.push(Instruction::LoadLocal(array_local));
                            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                                BigInt::from(index as u64),
                            )));

                            if let Some(local_index) = slot {
                                instructions.push(Instruction::LoadLocal(*local_index));
                            } else {
                                push_default_for_value_type(
                                    value_type,
                                    &mut ctx,
                                    &mut instructions,
                                );
                            }

                            instructions.push(Instruction::ArraySet);
                        }

                        instructions.push(Instruction::LoadLocal(array_local));
                        instructions.push(Instruction::Return);
                    }
                }
            };
        }

        if !ctx.errors.is_empty() {
            return Err(ctx.errors);
        }

        let local_count = ctx.local_count;
        let warnings = std::mem::take(&mut ctx.warnings);
        drop(ctx);

        Ok((
            Self {
                name: metadata.neo_name.clone(),
                kind: match metadata.kind {
                    MetadataFunctionKind::Constructor => FunctionKind::Constructor,
                    MetadataFunctionKind::Regular => FunctionKind::Regular,
                },
                parameters,
                returns,
                basic_blocks: vec![BasicBlock { instructions }],
                local_count,
            },
            warnings,
        ))
    }
}

pub(crate) fn build_is_static_abi_slot_value_type(value_type: &ValueType) -> bool {
    matches!(
        value_type,
        ValueType::Integer { .. }
            | ValueType::Boolean
            | ValueType::Address
            | ValueType::ByteArray {
                fixed_len: Some(1..=32)
            }
    )
}

pub(crate) fn build_emit_static_abi_slot_for_value_type(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    match value_type {
        ValueType::Integer { signed: true, .. } => {
            // Negative signed integers must sign-extend to the full 32-byte ABI
            // slot; mask to 2^256 bits so the zero-pad encoder sees the
            // canonical two's-complement (mirrors
            // return_revert.rs::emit_static_abi_slot_for_value_type).
            let mask: BigInt = (BigInt::one() << 256usize) - BigInt::one();
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(mask)));
            instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            build_emit_static_slot_32(ctx, instructions, true);
            true
        }
        ValueType::Integer { .. } | ValueType::Boolean | ValueType::Address => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            build_emit_static_slot_32(ctx, instructions, true);
            true
        }
        ValueType::ByteArray {
            fixed_len: Some(len),
        } if *len == 32 => true,
        ValueType::ByteArray {
            fixed_len: Some(len),
        } if *len < 32 => {
            build_emit_pad_bytesn_to_32(ctx, instructions, *len as usize);
            true
        }
        _ => false,
    }
}

pub(crate) fn build_emit_static_slot_32(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    reverse: bool,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__abi_ret_slot_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__abi_ret_slot_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__abi_ret_slot_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__abi_ret_slot_count_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);

    if reverse {
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::ReverseItems);
    } else {
        instructions.push(Instruction::LoadLocal(dst_local));
    }
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

pub(crate) fn build_emit_pad_bytesn_to_32(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    n: usize,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__abi_ret_bytesn_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__abi_ret_bytesn_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__abi_ret_bytesn_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__abi_ret_bytesn_count_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(n as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(n as u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

/// Task #186 — parse a Solidity type string of the form
/// `"function (T1, T2, ...) [mutability] [returns (R1, R2, ...)]"` and extract
/// the argument count and whether a return clause is present. Returns `None`
/// when the string is not a function type. Used to detect function-pointer
/// parameters that need `CallIndirect` dispatch at call sites, since the
/// `NeoType` layer maps function types to `None` and the ValueType erases
/// this info.
pub(crate) fn parse_function_pointer_type(ty: &str) -> Option<(usize, bool)> {
    let trimmed = ty.trim();
    if !trimmed.starts_with("function") {
        return None;
    }
    let rest = trimmed[..].strip_prefix("function")?.trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    // Scan to matching `)` respecting nested parens (tuple types).
    let bytes = rest.as_bytes();
    let mut depth: i32 = 0;
    let mut end: Option<usize> = None;
    let mut comma_count: usize = 0;
    let mut any_token = false;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => {
                depth += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
            }
            b',' if depth == 1 => {
                comma_count += 1;
            }
            c if !c.is_ascii_whitespace() && depth >= 1 => {
                any_token = true;
            }
            _ => {}
        }
    }
    let end_idx = end?;
    let arg_count = if any_token { comma_count + 1 } else { 0 };
    let tail = rest[end_idx + 1..].trim_start();
    let has_return = tail.contains("returns");
    Some((arg_count, has_return))
}

#[cfg(test)]
mod function_pointer_type_tests {
    use super::parse_function_pointer_type;

    #[test]
    pub(crate) fn parses_two_arg_with_return() {
        let s = "function (uint256, uint256) pure returns (uint256)";
        assert_eq!(parse_function_pointer_type(s), Some((2, true)));
    }

    #[test]
    pub(crate) fn parses_void_function_type() {
        assert_eq!(parse_function_pointer_type("function ()"), Some((0, false)));
    }

    #[test]
    pub(crate) fn rejects_non_function_types() {
        assert_eq!(parse_function_pointer_type("uint256"), None);
        assert_eq!(parse_function_pointer_type("mapping(uint => uint)"), None);
    }

    #[test]
    pub(crate) fn parses_nested_tuple_args() {
        // Defensive: tuple-shaped args should not mis-split on inner commas.
        let s = "function ((uint256, bool), address) returns (uint256)";
        assert_eq!(parse_function_pointer_type(s), Some((2, true)));
    }
}
