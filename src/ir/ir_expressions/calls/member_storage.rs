use super::*;

/// Task #91 — inline a member call where the library function's first
/// parameter is `T storage`. Binds `param_names[0]` as a storage alias to
/// `receiver`, the remaining params as locals populated from call-site args,
/// then lowers the body in place. Any `return expr;` inside the body is
/// redirected via `inline_return_stack` to the synthesised `__ret` local.
pub(crate) fn inline_library_storage_call(
    body_info: LibraryStorageBody,
    receiver: StorageReference,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Step 1: evaluate args in the caller's scope (so `v` resolves to the
    // caller's parameter) and stash each value in a fresh temp local.
    let mut ok = true;
    let mut arg_temp_slots: Vec<usize> = Vec::with_capacity(args.len());
    for (index, arg) in args.iter().enumerate() {
        let param_type = body_info.value_param_types.get(index).cloned().flatten();
        let tmp = ctx.allocate_local(format!("__libarg_{index}"), param_type);
        if !lower_expression(arg, ctx, instructions) {
            ok = false;
        }
        instructions.push(Instruction::StoreLocal(tmp));
        arg_temp_slots.push(tmp);
    }

    // Step 2: hide any caller-parameter names that collide with the library
    // parameter names, so the body's `v` resolves to our local, not the
    // caller's `LoadParameter(…)`.
    let hidden_params: Vec<(String, Option<usize>)> = body_info
        .param_names
        .iter()
        .filter(|name| !name.is_empty())
        .map(|name| (name.clone(), ctx.hide_param_binding(name)))
        .collect();

    ctx.enter_scope();
    let recv_name = body_info.param_names.first().cloned().unwrap_or_default();
    if !recv_name.is_empty() {
        ctx.set_storage_alias(recv_name, receiver);
    }

    // Step 3: rebind each library value-parameter as a local in the inlined
    // scope, copying the value from its stashed temp.
    for (index, tmp_slot) in arg_temp_slots.iter().enumerate() {
        let param_name = body_info
            .param_names
            .get(index + 1)
            .cloned()
            .unwrap_or_default();
        if param_name.is_empty() {
            continue;
        }
        let param_type = body_info.value_param_types.get(index).cloned().flatten();
        let slot = ctx.allocate_local(param_name, param_type);
        instructions.push(Instruction::LoadLocal(*tmp_slot));
        instructions.push(Instruction::StoreLocal(slot));
    }

    // Step 4: install the inline-return redirect and lower the body.
    let ret_slot = if body_info.return_type.is_some() {
        Some(ctx.allocate_local("__ret".to_string(), body_info.return_type.clone()))
    } else {
        None
    };
    let end_label = ctx.next_label();
    ctx.push_inline_return(ret_slot, end_label);

    lower_statement(&body_info.body, ctx, instructions);

    ctx.pop_inline_return();
    instructions.push(Instruction::Label(end_label));
    if let Some(slot) = ret_slot {
        instructions.push(Instruction::LoadLocal(slot));
    }
    ctx.exit_scope();
    for (name, index) in hidden_params {
        ctx.restore_param_binding(name, index);
    }
    ok
}
