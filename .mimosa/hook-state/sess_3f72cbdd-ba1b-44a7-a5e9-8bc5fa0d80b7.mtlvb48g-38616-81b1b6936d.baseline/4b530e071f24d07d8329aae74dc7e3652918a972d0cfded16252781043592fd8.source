use super::*;

// ============================================================================
// Section 2 — Storage Array Memory ↔ Storage Copy (Task #102)
// ============================================================================

pub(crate) fn lower_storage_array_assign_from_memory(
    state_index: usize,
    rhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let writable = ctx.ensure_state_writable(state_index);

    // Lower RHS into a dedicated local so we can iterate over it twice (length
    // probe + per-index read) without re-evaluating side-effectful expressions.
    let src_local = ctx.allocate_local("__arr_assign_src".to_string(), None);
    if !lower_expression(rhs, ctx, instructions) {
        // RHS failed to lower; nothing useful we can do. `lower_expression`
        // already recorded an error, so just bail out quietly.
        return;
    }
    instructions.push(Instruction::StoreLocal(src_local));

    // If the target state is not writable (e.g. immutable after construction),
    // we've still evaluated the RHS for side effects; skip the mutation half.
    if !writable {
        return;
    }

    let uint256 = ValueType::Integer {
        signed: false,
        bits: 256,
    };

    // new_len = m.length (GetSize on the NeoVM array/buffer).
    let new_len_local =
        ctx.allocate_local("__arr_assign_new_len".to_string(), Some(uint256.clone()));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(new_len_local));

    // old_len = current length slot (the state var itself holds the length).
    let old_len_local =
        ctx.allocate_local("__arr_assign_old_len".to_string(), Some(uint256.clone()));
    instructions.push(Instruction::LoadState(state_index));
    instructions.push(Instruction::StoreLocal(old_len_local));

    // Length slot := new_len. Writing the length before the loops matches the
    // existing `.pop()` helper's ordering and keeps `arr.length` consistent
    // if the copy loop ever panics partway through.
    instructions.push(Instruction::LoadLocal(new_len_local));
    instructions.push(Instruction::StoreState(state_index));

    let element_type = match ctx.state_type(state_index).cloned() {
        Some(ValueType::Array(elem)) => (*elem).clone(),
        _ => ValueType::Any,
    };

    // Copy loop: for (i = 0; i < new_len; i++) { slot[i] = src[i] }
    let copy_cond_label = ctx.next_label();
    let copy_end_label = ctx.next_label();
    let idx_local = ctx.allocate_local("__arr_assign_idx".to_string(), Some(uint256.clone()));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(idx_local));

    instructions.push(Instruction::Label(copy_cond_label));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(new_len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    // IR JumpIf branches when the condition is false, so this exits the loop
    // once idx >= new_len.
    instructions.push(Instruction::JumpIf {
        target: copy_end_label,
    });

    // slot[idx] := src[idx]
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::LoadLocal(idx_local));
    if matches!(element_type, ValueType::Array(_)) {
        instructions.push(Instruction::StoreArrayDeepCopy {
            state_index,
            key_types: vec![uint256.clone()],
        });
    } else {
        instructions.push(Instruction::StoreMappingElement {
            state_index,
            key_types: vec![uint256.clone()],
        });
    }

    // idx += 1
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump {
        target: copy_cond_label,
    });
    instructions.push(Instruction::Label(copy_end_label));

    // Deletion loop: for (i = new_len; i < old_len; i++) { slot[i] = default }
    // Matches the `.pop()` convention of overwriting removed slots with the
    // element type's default value so subsequent reads of the shrunk tail
    // return 0 / "" / etc. instead of stale data.
    let delete_cond_label = ctx.next_label();
    let delete_end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(new_len_local));
    instructions.push(Instruction::StoreLocal(idx_local));

    instructions.push(Instruction::Label(delete_cond_label));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(old_len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf {
        target: delete_end_label,
    });

    push_default_for_value_type(&element_type, ctx, instructions);
    instructions.push(Instruction::LoadLocal(idx_local));
    if matches!(element_type, ValueType::Array(_)) {
        instructions.push(Instruction::StoreArrayDeepCopy {
            state_index,
            key_types: vec![uint256.clone()],
        });
    } else {
        instructions.push(Instruction::StoreMappingElement {
            state_index,
            key_types: vec![uint256.clone()],
        });
    }

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump {
        target: delete_cond_label,
    });
    instructions.push(Instruction::Label(delete_end_label));
}

// ============================================================================
// Section 3 — Storage Array Read to Memory (Bug-hunt #10)
// ============================================================================

/// Bug-hunt #10 — materialize a storage array into a FRESH in-memory NeoVM
/// array bound to `dst_slot`, so `T[] memory m = storageArr;` is a DEEP COPY
/// (subsequent `m[i] = v` touches only the copy, and `arr[i]` is unchanged).
/// Storage dynamic arrays keep only the length scalar in the state slot (the
/// elements live in derived `LoadMappingElement` slots), so a bare `LoadState`
/// yields an Integer — writing through it faults `SETITEM: unsupported target
/// Integer`. Mirror-image of `lower_storage_array_assign_from_memory`: read
/// each element via the same slot layout the writer used and SETITEM it into
/// the new array. Leaves nothing on the stack (stores into `dst_slot` itself).
pub(crate) fn lower_storage_array_read_to_memory(
    state_index: usize,
    element_type: ValueType,
    dst_slot: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let uint256 = ValueType::Integer {
        signed: false,
        bits: 256,
    };
    // len = length scalar in the state slot.
    let len_local = ctx.allocate_local("__arr_copy_len".to_string(), Some(uint256.clone()));
    instructions.push(Instruction::LoadState(state_index));
    instructions.push(Instruction::StoreLocal(len_local));

    // dst = new array[len] (zero-initialized; overwritten below).
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::NewArray {
        element_type: element_type.clone(),
    });
    instructions.push(Instruction::StoreLocal(dst_slot));

    // for (i = 0; i < len; i++) { dst[i] = storage[i] }
    let cond_label = ctx.next_label();
    let end_label = ctx.next_label();
    let idx_local = ctx.allocate_local("__arr_copy_idx".to_string(), Some(uint256.clone()));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(idx_local));

    instructions.push(Instruction::Label(cond_label));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    // IR JumpIf branches when the condition is FALSE → exit once idx >= len.
    instructions.push(Instruction::JumpIf { target: end_label });

    // dst[idx] = storage[idx]  (SETITEM pops value, index, array).
    instructions.push(Instruction::LoadLocal(dst_slot));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    if matches!(element_type, ValueType::Struct { .. }) {
        instructions.push(Instruction::LoadStructArrayElement {
            state_index,
            key_types: Vec::new(),
            field_keys: Vec::new(),
            element_type: element_type.clone(),
        });
    } else {
        instructions.push(Instruction::LoadMappingElement {
            state_index,
            key_types: vec![uint256.clone()],
        });
    }
    instructions.push(Instruction::ArraySet);

    // idx += 1
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump { target: cond_label });
    instructions.push(Instruction::Label(end_label));
}

// ============================================================================
// Section 4 — External Dynamic Assignment (`this.f()` / `addr.f()` = value)
// ============================================================================

/// True if `rhs` is an external-target member call that, at the lowered
/// bytecode level, goes through `System.Contract.Call` and therefore
/// leaves an EVM-canonical ABI-encoded ByteString on the stack for
/// multi-return methods. Covers three shapes:
///
///   1. `this.<method>(...)`            — Variable("this") inner
///   2. `IContract(addr).<method>(...)` — `FunctionCall(ContractType, [addr])` inner
///   3. `<addr-typed var>.<method>(...)` — inner infers to `ValueType::Address`
///
/// Without this broader match, (2) and (3) skipped the compile-time
/// ABI-decode and the destructure PICKITEM chain indexed the raw
/// encoded bytes — producing zero-valued per-byte reads.
pub(crate) fn is_this_external_tuple_call(rhs: &Expression, ctx: &mut LoweringContext) -> bool {
    let Expression::FunctionCall(_, func, _) = rhs else {
        return false;
    };
    let Expression::MemberAccess(_, inner, member) = func.as_ref() else {
        return false;
    };

    // Bug-hunt #21 — a low-level `addr.call/.staticcall/.delegatecall/.callcode`
    // already produces a NeoVM `[bool ok, bytes data]` Array (see
    // try_lower_low_level_address_call). It must NEVER be re-interpreted as a
    // `this.method()` ABI-return buffer: doing so ran the encoded-bytes decode
    // over the tuple and mis-read `ok` (reported ok=true for a reverting call).
    // Reject these members up front.
    if matches!(
        member.name.as_str(),
        "call" | "staticcall" | "delegatecall" | "callcode"
    ) {
        return false;
    }

    // Shape 1 — `this.method()`.
    if matches!(inner.as_ref(), Expression::Variable(id) if id.name == "this") {
        return true;
    }

    // Shape 2 — `IContract(addr).method()` / `IContract(other).method()`.
    // The inner is a single-arg FunctionCall whose callee resolves to a
    // contract / interface type name.
    if let Expression::FunctionCall(_, cast_func, cast_args) = inner.as_ref() {
        if cast_args.len() == 1 {
            let is_contract_type = match cast_func.as_ref() {
                Expression::Variable(id) => ctx.is_contract_type_name(&id.name),
                Expression::MemberAccess(_, _, id) => ctx.is_contract_type_name(&id.name),
                _ => false,
            };
            if is_contract_type {
                return true;
            }
        }
    }

    // Shape 3 — any other inner that type-infers to `Address`. This catches
    // stored state vars / parameters of contract/interface types that flow
    // through type inference as `Address`.
    matches!(
        infer_type_from_expression(inner.as_ref(), ctx),
        Some(ValueType::Address)
    )
}

/// Wave-#28 fix — same-contract `T[] memory got = this.method();` (and the
/// interface-cast / address-typed call shapes covered by
/// `is_this_external_tuple_call`) where `method` returns a single
/// dynamic-ABI-shaped value (`T[]` / `bytes` / `string`).
///
/// Background. Bug #25 wired up `abi.decode(buf, (T[]))` correctly in the
/// builtins helpers (`emit_abi_decode_dynamic_top_level`). The sibling
/// fix (above, ~line 347) extends the same compile-time decode to the
/// tuple-destructure RHS — but only for **static-1-slot** members. A
/// single-LHS dynamic return (`uint256[] memory got = this.makeArray()`)
/// dropped through to the generic `Variable` branch which `StoreLocal`'d
/// the raw ABI ByteString. Subsequent `got.length` then read the byte
/// length of the encoded payload (e.g. 224 = 32 head + 32 length-prefix +
/// 5*32 elements for a five-element `uint256[]`) instead of the element
/// count (5).
///
/// This helper, called from both the plain-assignment branch and the
/// `lower_variable_definition_statement` initializer path, detects the
/// pattern, runs the dynamic-decode chain on the buffer, and stores the
/// resulting NeoVM Array (or ByteString for string/bytes) into the
/// destination slot. Callers must have already verified the LHS slot
/// type — passed via `dst_type` — and resolved the slot index. Returns
/// `true` if the decode-and-store path fired (the caller must skip its
/// own lowering of `rhs`); `false` otherwise (caller falls back).
///
/// Static-slot single-LHS returns (`uint256 x = this.foo()`) are NOT
/// routed through here — they are handled by the existing single-result
/// dispatch lowering elsewhere in the call chain. Dynamic types are the
/// novel case: their wire shape (head-offset + length + payload) needs
/// the dedicated walker.
/// Bug-hunt #9 — a PLAIN same-contract call `f(args)` to a Public/External
/// function returning a single dynamic array / `bytes` / `string`. The callee
/// body ABI-encodes that return (the external calling convention), so binding
/// the raw call result would store the EVM `offset||length||elements` blob
/// (e.g. `m.length` reads 160 instead of 3). Decode the blob back into the
/// value — the same fix the `this.method()` path already applies, but keyed on
/// `is_externally_callable_fn` since a plain call has no `this.`/cast receiver.
/// (An `internal` callee does NOT ABI-encode and already binds correctly.)
pub(crate) fn try_lower_plain_external_dynamic_assign(
    slot: usize,
    rhs: &Expression,
    dst_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if !abi_dynamic_decode_value_type_is_supported(dst_type) {
        return false;
    }
    // `rhs` must be a plain call `f(args)` to an externally-callable function
    // of THIS contract (not a member/`this.`/cast call — those go through
    // try_lower_this_external_dynamic_assign).
    let Expression::FunctionCall(_, func, args) = rhs else {
        return false;
    };
    let Expression::Variable(id) = func.as_ref() else {
        return false;
    };
    if !ctx.is_externally_callable_fn(&id.name, args.len()) {
        return false;
    }

    let pre_len = instructions.len();
    if !lower_expression(rhs, ctx, instructions) {
        instructions.truncate(pre_len);
        return false;
    }
    let buffer_local = ctx.allocate_local(
        "__plain_dyn_abi_buf".to_string(),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    instructions.push(Instruction::StoreLocal(buffer_local));
    emit_abi_decode_dynamic_top_level(buffer_local, dst_type, ctx, instructions);
    instructions.push(Instruction::StoreLocal(slot));
    ctx.clear_call_data_local(slot);
    true
}

pub(crate) fn try_lower_this_external_dynamic_assign(
    slot: usize,
    rhs: &Expression,
    dst_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if !abi_dynamic_decode_value_type_is_supported(dst_type) {
        return false;
    }
    if !is_this_external_tuple_call(rhs, ctx) {
        return false;
    }

    // Lower the call into a temp buffer; on failure restore the caller's
    // instruction list to avoid half-built stack frames (matches the
    // tuple-destructure branch's `pre_len`/`truncate` discipline).
    let pre_len = instructions.len();
    if !lower_expression(rhs, ctx, instructions) {
        instructions.truncate(pre_len);
        return false;
    }

    let buffer_local = ctx.allocate_local(
        "__this_dyn_abi_buf".to_string(),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    instructions.push(Instruction::StoreLocal(buffer_local));

    // Decode the top-level dynamic value (`T[]` => Array of decoded
    // elements; `string`/`bytes` => ByteString). Stack on exit:
    // `[decoded_value]`.
    emit_abi_decode_dynamic_top_level(buffer_local, dst_type, ctx, instructions);

    instructions.push(Instruction::StoreLocal(slot));
    ctx.clear_call_data_local(slot);
    true
}
