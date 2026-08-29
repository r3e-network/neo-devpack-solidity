use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_deploy_function_with_warnings(
    metadata: &FunctionMetadata,
    current_contract_name: &str,
    constructors: &[&Function],
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
) -> Result<(Function, Vec<crate::solidity::Diagnostic>), Vec<IrDiagnostic>> {
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
    let mut ctx = LoweringContext::new(
        &metadata.name,
        current_contract_name,
        metadata.selector,
        false,
        // The deploy prologue is invoked via the `_deploy` dispatcher path
        // and does not return tuples, so `is_externally_callable=false` is
        // safe here — the abi-encode tuple-return conversion is inert.
        false,
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

    // Lower state variable initializers (non-constant) into a deploy-time prologue.
    let mut init_instructions = Vec::new();
    for (index, state) in state_variables.iter().enumerate() {
        if state.is_constant {
            continue;
        }
        if let Some(initializer) = state.initializer.as_ref() {
            // Task #202 — inline array-literal state-var initializers
            // (`uint256[] public nums = [1, 2, 3, 4, 5];`) must populate
            // the storage-array representation: length slot + one
            // mapping-keyed slot per element. The naive `StoreState` path
            // writes the whole NeoVM Array as a single opaque blob to the
            // length slot, so subsequent `nums.length` reads see a
            // corrupted integer (the array object coerced to an int) and
            // `nums[i]` mapping lookups all miss. Re-use the runtime
            // `storage_arr = memory_arr` helper, which is the same
            // machinery used at runtime for assignment to a storage array.
            if matches!(ctx.state_type(index), Some(ValueType::Array(_))) {
                lower_storage_array_assign_from_memory(
                    index,
                    initializer,
                    &mut ctx,
                    &mut init_instructions,
                );
                continue;
            }
            if lower_expression(initializer, &mut ctx, &mut init_instructions) {
                init_instructions.push(Instruction::StoreState(index));
            }
        }
    }

    if !ctx.errors.is_empty() {
        return Err(ctx.errors);
    }

    let mut instructions = Vec::new();
    let mut local_count = ctx.local_count as usize;

    let has_init = !init_instructions.is_empty();
    if constructors.is_empty() && !has_init {
        instructions.push(Instruction::ReturnVoid);
    } else {
        let run_label = ctx.next_label();
        let end_label = ctx.next_label();

        // `_deploy(data, update)` follows Neo N3 convention:
        // - `update == true` means contract update; skip constructors/initializers.
        // - `update == false` means first deployment; run initializers then constructors.
        //
        // Note: in this compiler's IR, `JumpIf` branches when the condition is false.
        // Therefore, we jump into the deploy prologue when `update == false`.
        instructions.push(Instruction::LoadParameter(1));
        instructions.push(Instruction::JumpIf { target: run_label });
        instructions.push(Instruction::Jump { target: end_label });
        instructions.push(Instruction::Label(run_label));

        if has_init {
            instructions.extend(init_instructions);
        }

        let needs_data_local = constructors.iter().any(|c| !c.parameters.is_empty());
        let data_local = local_count;
        if needs_data_local {
            instructions.push(Instruction::LoadParameter(0));
            instructions.push(Instruction::StoreLocal(data_local));

            // When the contract has a parameterised Solidity constructor, we treat `_deploy(data, update).data`
            // as an array of constructor arguments.
            //
            // Most tooling (including Neo-Express) passes `data` as a ByteString containing JSON (e.g. `[7]`).
            // Some SDKs can pass an Array StackItem directly. Some deploy flows (e.g., contract-to-contract
            // deployment) may pass StdLib.serialize(...) bytes.
            //
            // We support all of these by attempting to parse JSON via StdLib.jsonDeserialize, then falling back
            // to StdLib.deserialize, and finally falling back to the original `data` when native calls throw.
            //
            // StdLib.jsonDeserialize returns a StackItem (Array/Map/etc). Constructor arg extraction then uses
            // PICKITEM/ArrayGet to obtain the arguments.
            //
            // Note: This requires `StdLib.jsonDeserialize` manifest permission.
            let json_catch_label = ctx.next_label();
            let deserialize_label = ctx.next_label();
            let deserialize_catch_label = ctx.next_label();
            let decode_done_label = ctx.next_label();
            instructions.push(Instruction::Try {
                catch_target: json_catch_label,
            });
            instructions.push(Instruction::LoadLocal(data_local));
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::NativeCall {
                    contract: NativeContract::StdLib,
                    method: "jsonDeserialize".to_string(),
                },
                arg_count: 1,
            });
            instructions.push(Instruction::StoreLocal(data_local));
            instructions.push(Instruction::EndTry {
                target: decode_done_label,
            });

            instructions.push(Instruction::Label(json_catch_label));
            // NeoVM pushes the thrown value onto the stack when entering the catch handler.
            // Drop it and attempt a binary `StdLib.deserialize` fallback.
            instructions.push(Instruction::Drop(ValueType::Any));
            instructions.push(Instruction::EndTry {
                target: deserialize_label,
            });

            instructions.push(Instruction::Label(deserialize_label));
            instructions.push(Instruction::Try {
                catch_target: deserialize_catch_label,
            });
            instructions.push(Instruction::LoadLocal(data_local));
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::NativeCall {
                    contract: NativeContract::StdLib,
                    method: "deserialize".to_string(),
                },
                arg_count: 1,
            });
            instructions.push(Instruction::StoreLocal(data_local));
            instructions.push(Instruction::EndTry {
                target: decode_done_label,
            });
            instructions.push(Instruction::Label(deserialize_catch_label));
            // Drop the thrown value and keep the original `data` unchanged.
            instructions.push(Instruction::Drop(ValueType::Any));
            instructions.push(Instruction::EndTry {
                target: decode_done_label,
            });

            instructions.push(Instruction::Label(decode_done_label));
            local_count += 1;
        }

        for constructor in constructors {
            let param_count = constructor.parameters.len();
            if param_count > 0 {
                for index in 0..param_count {
                    instructions.push(Instruction::LoadLocal(data_local));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(index as u64),
                    )));
                    instructions.push(Instruction::ArrayGet);
                }
            }

            instructions.push(Instruction::CallFunction {
                name: constructor.name.clone(),
                arg_count: param_count,
            });
        }

        instructions.push(Instruction::Label(end_label));
        instructions.push(Instruction::ReturnVoid);
    }

    let warnings = std::mem::take(&mut ctx.warnings);
    drop(ctx);

    Ok((
        Function {
            name: metadata.neo_name.clone(),
            kind: FunctionKind::Regular,
            parameters,
            returns,
            basic_blocks: vec![BasicBlock { instructions }],
            local_count: local_count as u16,
        },
        warnings,
    ))
}
