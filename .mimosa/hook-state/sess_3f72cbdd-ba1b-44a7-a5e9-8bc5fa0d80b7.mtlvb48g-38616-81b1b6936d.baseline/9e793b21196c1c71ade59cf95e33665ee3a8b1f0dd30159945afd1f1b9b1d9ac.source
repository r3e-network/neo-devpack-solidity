use super::*;
use crate::opcode::OpCode;

/// Output of [`emit_ir_function`]: the emitted bytes plus the call / token
/// patches the caller must still resolve against the full bytecode buffer.
type EmitIrOutput = Result<(Vec<u8>, Vec<CallPatch>, Vec<MethodTokenPatch>), String>;

pub(crate) fn emit_ir_function(
    function: &ir::Function,
    module: &ir::Module,
    method: &FunctionMetadata,
    use_callt: bool,
) -> EmitIrOutput {
    use std::{collections::HashMap, convert::TryFrom};

    let mut local = Vec::new();
    // INITSLOT's `args` operand is the number of evaluation-stack items the
    // method pops into its argument slots, which must equal the number of
    // arguments a conformant caller pushes — i.e. the NOMINAL parameter count.
    // On Neo N3 a struct argument is passed as a single `Array` StackItem (the
    // manifest declares it as one `Array` parameter) and the body reads its
    // fields with `LDARG; PICKITEM`, so it occupies exactly one arg slot.
    //
    // (Earlier, Task #106 set this to the FLATTENED struct-field count on the
    // theory that EVM-style callers push tuple fields individually. That is the
    // shape of the `abi.encodeCall` *byte payload*, not Neo's calling
    // convention — see `member_access.rs`/`abi_structs_notify.rs`, which build
    // that payload independently of these arg slots. Over-counting made
    // INITSLOT pop more items than the caller pushes: every external/public
    // function taking a struct parameter then underflowed the eval stack and
    // FAULTed on a real node. The in-tree simulator masked it by substituting
    // `Null` on an empty pop.)
    let nominal_arg_count = method.parameters.len();
    let arg_count = u8::try_from(nominal_arg_count).unwrap_or(u8::MAX);
    let local_count = u8::try_from(function.local_count).unwrap_or(u8::MAX);
    if local_count > 0 || arg_count > 0 {
        local.push(OpCode::INITSLOT.byte());
        local.push(local_count);
        local.push(arg_count);
    }
    let mut label_offsets: HashMap<usize, u32> = HashMap::new();
    let mut jump_patches: Vec<(usize, usize)> = Vec::new();
    let mut call_patches: Vec<CallPatch> = Vec::new();
    let mut token_patches: Vec<MethodTokenPatch> = Vec::new();

    for block in &function.basic_blocks {
        for instruction in &block.instructions {
            match instruction {
                ir::Instruction::Drop(_) => local.push(OpCode::DROP.byte()),
                ir::Instruction::LoadParameter(index) => {
                    emit_load_parameter(&mut local, method, *index)
                }
                ir::Instruction::StoreParameter(index) => emit_store_parameter(&mut local, *index),
                ir::Instruction::PushLiteral(literal) => {
                    push_literal_value(&mut local, literal);
                }
                ir::Instruction::BinaryOp(operator) => emit_binary_op(&mut local, *operator),
                ir::Instruction::Return | ir::Instruction::ReturnVoid => {
                    local.push(OpCode::RET.byte())
                }
                ir::Instruction::ReturnDefault(value_type) => {
                    append_default_value(&mut local, value_type);
                    local.push(OpCode::RET.byte());
                }
                ir::Instruction::LoadState(index) => emit_load_state(&mut local, module, *index),
                ir::Instruction::StoreState(index) => emit_store_state(&mut local, module, *index),
                ir::Instruction::LoadStorageDynamic => emit_load_storage_dynamic(&mut local),
                ir::Instruction::LoadLocal(index) => emit_load_local(&mut local, *index),
                ir::Instruction::StoreLocal(index) => emit_store_local(&mut local, *index),
                ir::Instruction::LoadMappingElement {
                    state_index,
                    key_types,
                } => emit_load_mapping(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::StoreMappingElement {
                    state_index,
                    key_types,
                } => emit_store_mapping(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::StoreArrayDeepCopy {
                    state_index,
                    key_types,
                } => emit_store_mapping_array_deep_copy(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::LoadStructField {
                    state_index,
                    key_types,
                    field_keys,
                    field_type,
                    ..
                } => emit_load_struct_field(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    StructFieldAccess {
                        field_keys: field_keys.as_slice(),
                        ty: field_type,
                    },
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::StoreStructField {
                    state_index,
                    key_types,
                    field_keys,
                    field_type,
                    ..
                } => emit_store_struct_field(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    StructFieldAccess {
                        field_keys: field_keys.as_slice(),
                        ty: field_type,
                    },
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::LoadStructArrayElement {
                    state_index,
                    key_types,
                    field_keys,
                    element_type,
                } => emit_load_struct_array_element(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    StructArrayElementAccess {
                        field_keys: field_keys.as_slice(),
                        element_type,
                    },
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::StoreStructArrayElement {
                    state_index,
                    key_types,
                    field_keys,
                    element_type,
                } => emit_store_struct_array_element(
                    &mut local,
                    module,
                    *state_index,
                    key_types,
                    StructArrayElementAccess {
                        field_keys: field_keys.as_slice(),
                        element_type,
                    },
                    use_callt,
                    &mut token_patches,
                ),
                ir::Instruction::LoadStructFieldMappingElement {
                    state_index,
                    key_types,
                    field_keys,
                    trailing_key_types,
                    value_type,
                } => {
                    let slot = StructFieldMappingSlot {
                        module,
                        state_index: *state_index,
                        key_types,
                        field_keys: field_keys.as_slice(),
                        trailing_key_types,
                        use_callt,
                    };
                    emit_load_struct_field_mapping_element(
                        &mut local,
                        &slot,
                        value_type,
                        &mut token_patches,
                    )
                }
                ir::Instruction::StoreStructFieldMappingElement {
                    state_index,
                    key_types,
                    field_keys,
                    trailing_key_types,
                } => {
                    let slot = StructFieldMappingSlot {
                        module,
                        state_index: *state_index,
                        key_types,
                        field_keys: field_keys.as_slice(),
                        trailing_key_types,
                        use_callt,
                    };
                    emit_store_struct_field_mapping_element(&mut local, &slot, &mut token_patches)
                }
                ir::Instruction::LoadRuntimeValue(value) => {
                    emit_load_runtime_value(&mut local, value, use_callt, &mut token_patches)
                }
                ir::Instruction::GetSize => local.push(OpCode::SIZE.byte()),
                ir::Instruction::CallBuiltin { builtin, arg_count } => {
                    emit_builtin_call(
                        &mut local,
                        builtin,
                        *arg_count,
                        use_callt,
                        &mut token_patches,
                    );
                }
                ir::Instruction::CallFunction { name, arg_count } => {
                    // NeoVM's `INITSLOT` assigns argument slots by popping values from the
                    // evaluation stack in order, meaning the callee sees the last-pushed value
                    // as `arg0`. Solidity evaluates arguments left-to-right, so call sites push
                    // `arg0, arg1, ...` onto the stack. Before calling, reverse the argument
                    // segment so that `arg0` is on top and parameters arrive in the expected
                    // order inside the callee.
                    if *arg_count > 1 {
                        push_integer_bigint(&mut local, &BigInt::from(*arg_count));
                        local.push(OpCode::REVERSEN.byte());
                    }
                    // NeoVM CALL_L uses a 4-byte signed offset relative to the
                    // beginning of the CALL_L instruction. We always use the
                    // wide form for simplicity.
                    local.push(OpCode::CALL_L.byte());
                    let patch_pos = local.len();
                    local.extend_from_slice(&[0, 0, 0, 0]);
                    call_patches.push(CallPatch {
                        position: patch_pos,
                        target: name.clone(),
                        kind: CallPatchKind::CallRelative,
                    });
                }
                ir::Instruction::PushFunctionOffset { name } => {
                    // Task #186 — push the target function as a code POINTER via
                    // PUSHA (a signed offset relative to the PUSHA opcode, fixed
                    // up after all methods are emitted), which `CALLA` consumes.
                    // Real NeoVM `CALLA` requires a `Pointer` produced by `PUSHA`;
                    // the previous `PUSHINT32` pushed a bare Integer that faults
                    // on-chain ("not a Pointer"), even though the local runtime —
                    // which models CALLA as popping an integer position — masked it.
                    local.push(OpCode::PUSHA.byte());
                    let patch_pos = local.len();
                    local.extend_from_slice(&[0, 0, 0, 0]);
                    call_patches.push(CallPatch {
                        position: patch_pos,
                        target: name.clone(),
                        kind: CallPatchKind::AbsoluteOffset,
                    });
                }
                ir::Instruction::CallIndirect { arg_count, .. } => {
                    // Task #186 — indirect call through an internal function-
                    // pointer value.
                    //
                    // IR-layer convention: emit `PushFunctionOffset` FIRST,
                    // then the N argument expressions left-to-right. The stack
                    // going into this instruction is therefore:
                    //     [..., target, arg0, arg1, ..., argN-1]
                    // (argN-1 on top).
                    //
                    // NeoVM `INITSLOT` binds arg0 = first popped = top-of-stack,
                    // so the callee needs `arg0` on top after CALLA consumes
                    // `target`. REVERSEN(arg_count + 1) flips the window to:
                    //     [..., argN-1, ..., arg1, arg0, target]
                    // CALLA then pops `target` and jumps, leaving:
                    //     [..., argN-1, ..., arg1, arg0]
                    // which INITSLOT pops into slot 0 = arg0, slot 1 = arg1, ...
                    if *arg_count >= 1 {
                        push_integer_bigint(&mut local, &BigInt::from(*arg_count + 1));
                        local.push(OpCode::REVERSEN.byte());
                    }
                    local.push(OpCode::CALLA.byte());
                }
                ir::Instruction::EmitEvent {
                    event_index,
                    arg_count,
                } => emit_event(&mut local, module, *event_index, *arg_count),
                ir::Instruction::EmitEventByName { name, arg_count } => {
                    emit_event_by_name(&mut local, name, *arg_count)
                }
                ir::Instruction::Convert { target } => emit_convert(&mut local, *target),
                ir::Instruction::IsType { target } => emit_is_type(&mut local, *target),
                ir::Instruction::NewBuffer => emit_new_buffer(&mut local),
                ir::Instruction::NewArray { .. } => emit_new_array(&mut local),
                ir::Instruction::NewMap => local.push(OpCode::NEWMAP.byte()),
                ir::Instruction::ArrayGet => emit_array_get(&mut local),
                ir::Instruction::ArraySet => emit_array_set(&mut local),
                ir::Instruction::HasKey => local.push(OpCode::HASKEY.byte()),
                ir::Instruction::MemCpy => {
                    local.push(OpCode::MEMCPY.byte());
                }
                ir::Instruction::Substr => {
                    local.push(OpCode::SUBSTR.byte());
                }
                ir::Instruction::ReverseItems => {
                    local.push(OpCode::REVERSEITEMS.byte());
                }
                ir::Instruction::BitwiseNot => {
                    local.push(OpCode::INVERT.byte());
                }
                ir::Instruction::LogicalNot => {
                    local.push(OpCode::NOT.byte()); // NOT (logical boolean negation)
                }
                ir::Instruction::Try { catch_target } => {
                    // NeoVM TRY_L uses 4-byte signed offsets (catch, finally) relative to
                    // the beginning of the TRY_L instruction. We always emit the wide form
                    // and omit `finally` (offset = 0).
                    local.push(OpCode::TRY_L.byte());
                    let position = local.len();
                    local.extend_from_slice(&[0, 0, 0, 0]); // catch offset placeholder
                    jump_patches.push((position, *catch_target));
                    local.extend_from_slice(&[0, 0, 0, 0]); // finally offset (absent)
                }
                ir::Instruction::EndTry { target } => {
                    // NeoVM ENDTRY_L uses a 4-byte signed offset relative to the beginning
                    // of the ENDTRY_L instruction. We always emit the wide form.
                    local.push(OpCode::ENDTRY_L.byte());
                    let position = local.len();
                    local.extend_from_slice(&[0, 0, 0, 0]);
                    jump_patches.push((position, *target));
                }
                ir::Instruction::Jump { target } => {
                    // NeoVM JMP_L uses a 4-byte signed offset relative to the
                    // beginning of the JMP_L instruction. We always use the
                    // wide form for simplicity.
                    local.push(OpCode::JMP_L.byte());
                    let position = local.len();
                    local.extend_from_slice(&[0, 0, 0, 0]);
                    jump_patches.push((position, *target));
                }
                ir::Instruction::JumpIf { target } => {
                    // IR JumpIf branches when the condition is false.
                    // NeoVM JMPIFNOT_L uses a 4-byte signed offset relative to the
                    // beginning of the JMPIFNOT_L instruction.
                    local.push(OpCode::JMPIFNOT_L.byte());
                    let position = local.len();
                    local.extend_from_slice(&[0, 0, 0, 0]);
                    jump_patches.push((position, *target));
                }
                ir::Instruction::Label(label) => {
                    label_offsets.insert(*label, local.len() as u32);
                }
                ir::Instruction::AbortMsg => {
                    local.push(OpCode::ABORTMSG.byte());
                }
                ir::Instruction::Abort => {
                    local.push(OpCode::ABORT.byte());
                }
                ir::Instruction::Throw => {
                    local.push(OpCode::THROW.byte());
                }
                ir::Instruction::Dup => {
                    local.push(OpCode::DUP.byte());
                }
                ir::Instruction::Swap => {
                    local.push(OpCode::SWAP.byte());
                }
            }
        }
    }

    for (position, label) in jump_patches {
        // M-BC1 fix — guard against an unresolved label or an out-of-range
        // patch position. An unresolved label previously fell through to
        // `local.len()` silently (turning the jump into a jump-to-RET); an
        // out-of-range `position` (possible if the dead-code pruner removed a
        // label but left its jump operand) would panic the slice write. Both
        // are compiler-internal bugs; surface them as a hard error instead.
        let Some(&target) = label_offsets.get(&label) else {
            return Err(format!(
                "bytecode emission: jump target label {label} unresolved                  (dead-code pruner likely removed it)"
            ));
        };
        if position + 4 > local.len() {
            return Err(format!(
                "bytecode emission: jump patch position {position} out of                  range (local len {})",
                local.len()
            ));
        }
        let target_offset = target as i32;
        // `position` points at the beginning of the 4-byte operand; the opcode
        // is immediately before it.
        let opcode_pos = (position - 1) as i32;
        let relative = target_offset.checked_sub(opcode_pos).unwrap_or(0);
        local[position..position + 4].copy_from_slice(&relative.to_le_bytes());
    }

    Ok((local, call_patches, token_patches))
}

pub(crate) fn append_default_value(bytecode: &mut Vec<u8>, value_type: &ValueType) {
    match value_type {
        ValueType::Integer { .. } => bytecode.push(OpCode::PUSH0.byte()),
        // Solidity booleans are represented as 0/1 values on the stack. Use PUSH0 for the
        // default `false` to match numeric/ABI semantics.
        ValueType::Boolean => bytecode.push(OpCode::PUSH0.byte()),
        ValueType::String => push_data(bytecode, &[]),
        ValueType::Address => push_data(bytecode, &[0u8; 20]),
        ValueType::ByteArray { fixed_len } => {
            if let Some(len) = fixed_len {
                let zeros = vec![0u8; *len as usize];
                push_data(bytecode, &zeros);
            } else {
                push_data(bytecode, &[]);
            }
        }
        ValueType::Array(_) => bytecode.push(OpCode::NEWARRAY0.byte()),
        ValueType::Mapping { .. } => bytecode.push(OpCode::NEWMAP.byte()),
        ValueType::Struct { .. } => bytecode.push(OpCode::NEWSTRUCT0.byte()),
        ValueType::Any => bytecode.push(OpCode::PUSHNULL.byte()),
    }
}
