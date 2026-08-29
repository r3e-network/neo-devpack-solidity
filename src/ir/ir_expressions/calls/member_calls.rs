//! ## Member-Style Call Lowering (`obj.method(args)`)
//!
//! Lowers Solidity member-style calls — covering struct methods, library
//! `using for` member calls, contract method calls, and inline library
//! storage calls — into NeoVM instructions.
//!
//! ### Dispatch flow (within `try_lower_member_call`)
//!
//! 1. Struct method → forward to struct call lowering
//! 2. `using for` library call → resolve target type, inline library code
//! 3. Contract method call → emit `System.Contract.Call` or inline
//! 4. Library storage call → `inline_library_storage_call`
//!
//! ### Related
//!
//! * `abi_encode.rs` — ABI argument encoding for cross-contract calls
//! * `low_level.rs` — Low-level `addr.call{}()` lowering

use super::*;

/// Lower a single call argument, canonicalizing an integer-backed `bytesN`
/// literal to its big-endian ByteString when the callee's `index`-th parameter
/// is a known `bytesN`. A `bytesN` literal otherwise reaches the callee as a
/// little-endian Integer (NeoVM is type-strict — there is no implicit coercion
/// at the call boundary), so any byte-level operation on it inside the callee —
/// indexing, comparison, hashing — is wrong on a real node (the simulator's
/// lenient typing masks it). Falls back to a plain `lower_expression` when the
/// parameter type is unknown (e.g. an external method on a contract not compiled
/// in this unit) or is not a `bytesN`.
pub(crate) fn lower_call_arg_canonical(
    name: &str,
    arg_count: usize,
    index: usize,
    arg: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(ty) = ctx.callee_param_type(name, arg_count, index) {
        if try_lower_bytesn_literal_canonical(arg, &ty, ctx, instructions) {
            return true;
        }
    }
    lower_expression(arg, ctx, instructions)
}

pub(crate) fn try_lower_member_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    pub(crate) fn format_builtin_member_list(members: &[&str]) -> String {
        const MAX_SHOWN: usize = 12;
        if members.len() <= MAX_SHOWN {
            return members.join(", ");
        }

        format!(
            "{} … (+{} more)",
            members[..MAX_SHOWN].join(", "),
            members.len() - MAX_SHOWN
        )
    }

    pub(crate) fn resolve_static_library_base(
        inner: &Expression,
        ctx: &LoweringContext,
    ) -> Option<String> {
        match inner {
            Expression::Variable(lib_id)
                if !ctx.param_index_map.contains_key(&lib_id.name)
                    && ctx.resolve_local(&lib_id.name).is_none()
                    && !ctx.state_index_map.contains_key(&lib_id.name) =>
            {
                Some(lib_id.name.clone())
            }
            Expression::MemberAccess(_, namespace_expr, imported_symbol)
                if matches!(
                    namespace_expr.as_ref(),
                    Expression::Variable(namespace_id)
                        if !ctx.param_index_map.contains_key(&namespace_id.name)
                            && ctx.resolve_local(&namespace_id.name).is_none()
                            && !ctx.state_index_map.contains_key(&namespace_id.name)
                            && !ctx.is_contract_type_name(&namespace_id.name)
                ) && ctx.is_contract_type_name(&imported_symbol.name) =>
            {
                Some(imported_symbol.name.clone())
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_contract_type_name(
        inner: &Expression,
        ctx: &LoweringContext,
    ) -> Option<String> {
        match inner {
            Expression::Variable(type_id) if ctx.is_contract_type_name(&type_id.name) => {
                Some(type_id.name.clone())
            }
            Expression::MemberAccess(_, namespace_expr, type_id)
                if matches!(
                    namespace_expr.as_ref(),
                    Expression::Variable(namespace_id)
                        if !ctx.param_index_map.contains_key(&namespace_id.name)
                            && ctx.resolve_local(&namespace_id.name).is_none()
                            && !ctx.state_index_map.contains_key(&namespace_id.name)
                            && !ctx.is_contract_type_name(&namespace_id.name)
                ) && ctx.is_contract_type_name(&type_id.name) =>
            {
                Some(type_id.name.clone())
            }
            _ => None,
        }
    }

    pub(crate) fn native_contract_from_constant(
        base: &str,
        constant: &str,
    ) -> Option<NativeContract> {
        if !matches!(base, "NativeCalls" | "NativeContracts") {
            return None;
        }

        match constant {
            "NEO_CONTRACT" => Some(NativeContract::Neo),
            "GAS_CONTRACT" => Some(NativeContract::Gas),
            "CONTRACT_MANAGEMENT" => Some(NativeContract::ContractManagement),
            "POLICY_CONTRACT" => Some(NativeContract::Policy),
            "ORACLE_CONTRACT" => Some(NativeContract::Oracle),
            "ROLE_MANAGEMENT" => Some(NativeContract::RoleManagement),
            "NOTARY_CONTRACT" => Some(NativeContract::Notary),
            "TREASURY_CONTRACT" => Some(NativeContract::Treasury),
            "LEDGER_CONTRACT" => Some(NativeContract::Ledger),
            "CRYPTO_LIB" => Some(NativeContract::CryptoLib),
            "STD_LIB" => Some(NativeContract::StdLib),
            _ => None,
        }
    }

    if let Expression::MemberAccess(_, inner, member) = func {
        // `super.method()` — resolve to the renamed base method preserved during
        // inheritance flattening. The flattener stores overridden base methods as
        // `__super_{methodName}` and records the mapping in `super_method_map`.
        if matches!(inner.as_ref(), Expression::Variable(id) if id.name == "super") {
            if let Some(super_name) = ctx.super_method_name(&member.name) {
                let super_name = super_name.to_string();
                let mut success = true;
                for (index, arg) in args.iter().enumerate() {
                    if !lower_call_arg_canonical(
                        &member.name,
                        args.len(),
                        index,
                        arg,
                        ctx,
                        instructions,
                    ) {
                        success = false;
                    }
                }
                if success {
                    let neo_name = ctx
                        .neo_function_name(&super_name, args.len())
                        .unwrap_or_else(|| super_name.clone());
                    instructions.push(Instruction::CallFunction {
                        name: neo_name,
                        arg_count: args.len(),
                    });
                }
                if ctx.is_void_function(&super_name) {
                    return Some(false);
                }
                return Some(success);
            }

            if ctx.function_names.contains(&member.name) {
                let mut success = true;
                for (index, arg) in args.iter().enumerate() {
                    if !lower_call_arg_canonical(
                        &member.name,
                        args.len(),
                        index,
                        arg,
                        ctx,
                        instructions,
                    ) {
                        success = false;
                    }
                }
                if success {
                    if let Some(neo_name) = ctx.neo_function_name(&member.name, args.len()) {
                        instructions.push(Instruction::CallFunction {
                            name: neo_name,
                            arg_count: args.len(),
                        });
                    } else {
                        success = false;
                    }
                }
                if ctx.is_void_function(&member.name) {
                    return Some(false);
                }
                return Some(success);
            }

            // No super method found — the method was not overridden or has no body.
            ctx.record_error_with_suggestion(
                format!(
                    "super.{}() cannot be resolved; no overridden base method with a body was found",
                    member.name
                ),
                "ensure the base contract defines this function with a body and marks it 'virtual'",
            );
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            return Some(false);
        }

        // User-defined value type `wrap`/`unwrap` — compile as no-ops.
        // `TypeName.wrap(value)` and `TypeName.unwrap(value)` are identity operations
        // on NeoVM since user-defined value types are transparent type aliases.
        if (member.name == "wrap" || member.name == "unwrap") && args.len() == 1 {
            if let Expression::Variable(type_id) = inner.as_ref() {
                let is_type_alias = !ctx.param_index_map.contains_key(&type_id.name)
                    && ctx.resolve_local(&type_id.name).is_none()
                    && !ctx.state_index_map.contains_key(&type_id.name);
                if is_type_alias {
                    // No-op: just lower the single argument — the value passes through.
                    let ok = lower_expression(&args[0], ctx, instructions);
                    return Some(ok);
                }
            }
        }

        // Fallback: treat unresolved member calls on address-like values as external
        // contract calls. Lower to System.Contract.Call with default flags.
        //
        // Task #87 — `using L1 for uint; using L2 for uint; x.f1().f2();` arrives
        // here with `inner = x.f1()` (an `Expression::FunctionCall`). The generic
        // heuristic below treats *any* FunctionCall receiver as an external
        // contract handle, which miscompiles the method chain into a
        // `System.Contract.Call` using the intermediate value as a target hash
        // (runtime returns empty). When the outer member binds to a resolved
        // using-for library function whose first parameter can accept the
        // inner call's concrete return type, prefer the library-dispatch
        // branch below instead of the external-call fallback. We restrict
        // this carve-out to FunctionCall receivers so unrelated shadow cases
        // (e.g. a parameter named identically to an in-scope contract type)
        // still fall through to the existing address-receiver checks.
        let inner_is_function_call = matches!(inner.as_ref(), Expression::FunctionCall(_, _, _));
        let chained_call_binds_to_using_library = inner_is_function_call
            && ctx.has_using_directives()
            && ctx.function_names.contains(&member.name)
            && ctx
                .neo_function_name(&member.name, args.len() + 1)
                .is_some();
        // Task #93 — `Lib.val()` where `Lib` names a non-builtin library
        // whose body was merged into this contract must lower as an internal
        // call. Without this carve-out, `Variable("Lib")` infers to
        // `ValueType::Address` (libraries land in `contract_types`) and the
        // external-target fallback below emits
        // `System.Contract.Call([0;20], "val", …)` → empty return-data.
        //
        // The exclusion list below is critical: `this` / `super` resolve
        // through `resolve_static_library_base` (they're Variable expressions
        // that match `!param && !local && !state`), so without explicit
        // exclusion the path would miscompile `this.externalFn()` as an
        // internal CALL_L, bypassing the external ABI-return contract and
        // returning StackItems unsuitable for tuple destructuring. Leaves the
        // genuine merged-library case (`Lib.val()` where Lib isn't a builtin
        // namespace and isn't `this`/`super`) on the internal-call path.
        let merged_static_library_call = resolve_static_library_base(inner.as_ref(), ctx)
            .as_deref()
            .is_some_and(|base| {
                !matches!(
                    base,
                    "Runtime"
                        | "abi"
                        | "Storage"
                        | "Syscalls"
                        | "Neo"
                        | "NativeCalls"
                        | "StdLib"
                        | "CryptoLib"
                        | "this"
                        | "super"
                )
            })
            && ctx.function_names.contains(&member.name)
            && ctx.neo_function_name(&member.name, args.len()).is_some();
        let is_external_target = !chained_call_binds_to_using_library
            && !merged_static_library_call
            && (matches!(
                infer_type_from_expression(inner.as_ref(), ctx),
                Some(ValueType::Address)
            ) || matches!(
                inner.as_ref(),
                Expression::FunctionCall(_, cast_func, cast_args)
                    if cast_args.len() == 1
                        && resolve_contract_type_name(cast_func.as_ref(), ctx).is_some()
                        && (matches!(
                            infer_type_from_expression(&cast_args[0], ctx),
                            Some(ValueType::Address)
                        ) || address_bytes_le_from_expression(&cast_args[0]).is_some())
            ) || matches!(
                inner.as_ref(),
                // Heuristic: chained member calls on a function-call result often
                // represent interface/contract handles (e.g. `_getRouter().foo()`).
                // If type inference cannot prove an internal target, treat it as
                // an external call target for compatibility.
                Expression::FunctionCall(_, _, _)
            ));

        if is_external_target {
            if is_low_level_evm_member(&member.name) {
                let suggestion = match member.name.as_str() {
                    "delegatecall" => "delegatecall is not available on Neo N3; Neo contracts have isolated storage. Use Syscalls.contractCall() for cross-contract calls",
                    "staticcall" => "staticcall is not available on Neo N3; use view/pure functions or Syscalls.contractCallWithFlags() with ReadOnly flags",
                    _ => "Neo N3 does not support low-level EVM calls; use NativeCalls.sol for contract-to-contract interactions",
                };
                ctx.record_warning_with_suggestion(
                    format!(
                        "unsupported low-level EVM call '{}' ignored (returns false)",
                        member.name
                    ),
                    suggestion,
                );
                instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
                return Some(true);
            }

            // NativeCalls/NativeContracts expose native contract hashes as constants. When
            // users write `NativeCalls.GAS_CONTRACT.totalSupply()` or
            // `NativeContracts.GAS_CONTRACT.totalSupply()` we resolve the target at compile
            // time and emit a `NativeCall` builtin. This avoids wildcard manifest permissions
            // and unlocks CALLT/method-token optimizations.
            let native_contract = match inner.as_ref() {
                Expression::MemberAccess(_, base, constant) => match base.as_ref() {
                    Expression::Variable(id) => {
                        native_contract_from_constant(&id.name, &constant.name)
                    }
                    _ => None,
                },
                _ => None,
            };

            if let Some(contract) = native_contract {
                let mut success = true;
                for arg in args {
                    if !lower_expression(arg, ctx, instructions) {
                        success = false;
                    }
                }

                if success {
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::NativeCall {
                            contract,
                            method: member.name.clone(),
                        },
                        arg_count: args.len(),
                    });
                }

                return Some(success);
            }

            // Neo N3 syscall convention: the first argument is at the top of the stack.
            // `System.Contract.Call(hash, method, flags, args)` therefore expects stack order:
            // `[args, flags, method, hash]` (top-of-stack is `hash`).
            //
            // Preserve Solidity evaluation order by evaluating the target expression first,
            // then the arguments, and only then arranging the stack for the syscall.
            let tmp_id = ctx.next_label();
            let target_slot = ctx.allocate_local(format!("__neo_extcall_target_{tmp_id}"), None);

            if !lower_expression(inner.as_ref(), ctx, instructions) {
                return Some(false);
            }
            instructions.push(Instruction::StoreLocal(target_slot));

            // Build the argument array directly (more efficient than serialize+deserialize).
            let args_slot = ctx.allocate_local(
                format!("__neo_extcall_args_{tmp_id}"),
                Some(ValueType::Array(Box::new(ValueType::Any))),
            );

            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(args.len()),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            instructions.push(Instruction::StoreLocal(args_slot));

            for (index, arg) in args.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(args_slot));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));

                // Canonicalize a bytesN literal arg to its BE ByteString using
                // the callee's parameter type, so a `bytesN` constant passed to
                // an external/interface method (`ISelf(x).f(0x01..)`) reaches the
                // callee as the correct bytes, not a little-endian Integer.
                if !lower_call_arg_canonical(
                    &member.name,
                    args.len(),
                    index,
                    arg,
                    ctx,
                    instructions,
                ) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                }

                instructions.push(Instruction::ArraySet);
            }

            instructions.push(Instruction::LoadLocal(args_slot));

            // Use read-only call flags in `view`/`pure` contexts to align with Solidity's
            // static-call behavior and Neo N3 `safe` method expectations.
            let flags = if ctx.is_safe { 0x05u8 } else { 0x0Fu8 };
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(flags),
            )));
            instructions.push(Instruction::PushLiteral(LiteralValue::String(
                member.name.as_bytes().to_vec(),
            )));
            instructions.push(Instruction::LoadLocal(target_slot));

            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::Syscall("System.Contract.Call".to_string()),
                arg_count: 4,
            });
            return Some(true);
        }

        // Attempt to lower member calls as internal/library calls.
        if ctx.function_names.contains(&member.name) {
            let static_library_base = resolve_static_library_base(inner.as_ref(), ctx);
            let is_library_static = static_library_base.is_some();

            let mut success = true;
            if is_library_static {
                // Built-in libraries are lowered directly as compiler intrinsics. If builtin
                // resolution didn't match the member, do not fall back to calling an internal
                // function with the same name (which can silently miscompile into recursion).
                if static_library_base.as_deref().is_some_and(|base| {
                    crate::ir::ir_context::BUILTIN_LIBRARY_BASES.contains(&base)
                }) {
                    let base = static_library_base.as_deref().unwrap_or("<library>");
                    let mut message =
                        format!("unsupported builtin library call '{base}.{}'", member.name);
                    if let Some(supported) = builtin_library_supported_members(base) {
                        message.push_str(&format!(
                            "; supported {base} intrinsics: {}",
                            format_builtin_member_list(supported)
                        ));
                    }
                    message.push_str(
                        ". (Builtin devpack libraries are compiler intrinsics; their Solidity bodies are not compiled.)",
                    );
                    ctx.record_error(message);
                    return Some(false);
                }

                // Namespaced library call `Lib.f(0x01..)`: args are parameters
                // 0.. (no receiver). Canonicalize a bytesN literal arg to its BE
                // ByteString so the library callee gets the right bytes.
                for (index, arg) in args.iter().enumerate() {
                    if !lower_call_arg_canonical(
                        &member.name,
                        args.len(),
                        index,
                        arg,
                        ctx,
                        instructions,
                    ) {
                        success = false;
                    }
                }

                if success {
                    if let Some(neo_name) = ctx.neo_function_name(&member.name, args.len()) {
                        instructions.push(Instruction::CallFunction {
                            name: neo_name,
                            arg_count: args.len(),
                        });
                    } else {
                        for _ in args {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                    }
                }
                // Void functions don't push a value onto the stack, so return
                // false to prevent the caller from emitting a spurious DROP.
                if ctx.is_void_function(&member.name) {
                    return Some(false);
                }
                return Some(success);
            }

            let with_receiver = ctx.neo_function_name(&member.name, args.len() + 1);
            let without_receiver = ctx.neo_function_name(&member.name, args.len());
            let receiver_type = infer_type_from_expression(inner.as_ref(), ctx);

            let (neo_name, arg_count, use_receiver) = if let Some(name) = with_receiver {
                if !ctx.has_using_directives() {
                    ctx.record_error_with_suggestion(
                        format!(
                            "member-style call '{}(...)' requires an explicit `using` directive",
                            member.name
                        ),
                        "add `using <Library> for <Type>;` (or `for *`) in the contract",
                    );
                    let mut success = true;
                    if !lower_expression(inner.as_ref(), ctx, instructions) {
                        success = false;
                    } else {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                    for arg in args {
                        if !lower_expression(arg, ctx, instructions) {
                            success = false;
                        } else {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    return Some(success);
                }

                if let Some(receiver_type) = receiver_type.as_ref() {
                    if !ctx.using_target_allows_receiver(receiver_type) {
                        ctx.record_error_with_suggestion(
                            format!(
                                "member-style call '{}(...)' is not available for receiver type '{receiver_type:?}' under the current `using` directives",
                                member.name
                            ),
                            "add/adjust `using <Library> for <Type>` so the receiver type is covered",
                        );
                        let mut success = true;
                        if !lower_expression(inner.as_ref(), ctx, instructions) {
                            success = false;
                        } else {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                        for arg in args {
                            if !lower_expression(arg, ctx, instructions) {
                                success = false;
                            } else {
                                instructions.push(Instruction::Drop(ValueType::Any));
                            }
                        }
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                        return Some(success);
                    }
                }

                if !ctx.using_function_list_allows_receiver(&member.name, receiver_type.as_ref()) {
                    ctx.record_error_with_suggestion(
                        format!(
                            "member-style call '{}(...)' is not allowed by `using {{...}} for ...` function lists",
                            member.name
                        ),
                        "add the function name to the corresponding `using { ... } for <Type>` directive",
                    );
                    let mut success = true;
                    if !lower_expression(inner.as_ref(), ctx, instructions) {
                        success = false;
                    } else {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                    for arg in args {
                        if !lower_expression(arg, ctx, instructions) {
                            success = false;
                        } else {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    return Some(success);
                }

                if let Some(receiver_type) = receiver_type.as_ref() {
                    if !ctx.receiver_matches_function_overload(
                        &member.name,
                        args.len() + 1,
                        receiver_type,
                    ) {
                        ctx.record_error_with_suggestion(
                            format!(
                                "member-style call '{}(...)' cannot bind receiver type '{receiver_type:?}' to the library function first parameter",
                                member.name
                            ),
                            "ensure the library function first parameter type is implicitly compatible with the receiver",
                        );
                        let mut success = true;
                        if !lower_expression(inner.as_ref(), ctx, instructions) {
                            success = false;
                        } else {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                        for arg in args {
                            if !lower_expression(arg, ctx, instructions) {
                                success = false;
                            } else {
                                instructions.push(Instruction::Drop(ValueType::Any));
                            }
                        }
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                        return Some(success);
                    }
                }

                // Task #91 — storage-pointer receiver inlining. When the
                // library function declares `T storage d` as its first
                // parameter, a regular `CallFunction` would materialise the
                // receiver into a struct copy and `d.x = v` would mutate a
                // local. Inline the body with `d` aliased to the caller's
                // `StorageReference` instead. See
                // `inline_library_storage_call` below.
                if let Some(body_info) = ctx
                    .library_storage_body(&member.name, args.len() + 1)
                    .cloned()
                {
                    if let Some(reference) = resolve_storage_reference(inner.as_ref(), ctx) {
                        let produces_value = body_info.return_type.is_some();
                        let _ = inline_library_storage_call(
                            body_info,
                            reference,
                            args,
                            ctx,
                            instructions,
                        );
                        // Void library calls must report `false` so the outer
                        // expression-statement lowering does not emit a
                        // spurious `Drop` against an empty stack.
                        return Some(produces_value);
                    }
                }

                (name, args.len() + 1, true)
            } else if let Some(name) = without_receiver {
                // Compatibility fallback for merged library helpers where the
                // receiver parameter is erased in the lowered signature.
                (name, args.len(), false)
            } else {
                let mut success = true;
                if !lower_expression(inner.as_ref(), ctx, instructions) {
                    success = false;
                } else {
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
                for arg in args {
                    if !lower_expression(arg, ctx, instructions) {
                        success = false;
                    } else {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                }
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                return Some(success);
            };

            // The receiver `inner` is the library function's FIRST parameter
            // when `use_receiver` (e.g. `(0x01..).f()` via `using ... for`); the
            // explicit args then follow at parameter indices 1.. . When
            // `!use_receiver` (a namespaced `Lib.f(...)`) the receiver is just
            // the library name — lowered then dropped — and the args are
            // parameters 0.. . Canonicalize a bytesN literal receiver/arg to its
            // big-endian ByteString so the library callee receives the correct
            // bytes, not a little-endian Integer (NeoVM is type-strict).
            let recv_canon = use_receiver
                && ctx
                    .callee_param_type(&member.name, arg_count, 0)
                    .is_some_and(|ty| {
                        try_lower_bytesn_literal_canonical(inner.as_ref(), &ty, ctx, instructions)
                    });
            if !recv_canon && !lower_expression(inner.as_ref(), ctx, instructions) {
                success = false;
            } else if !use_receiver {
                // Preserve side effects of receiver evaluation.
                instructions.push(Instruction::Drop(ValueType::Any));
            }

            let arg_param_offset = if use_receiver { 1 } else { 0 };
            for (index, arg) in args.iter().enumerate() {
                if !lower_call_arg_canonical(
                    &member.name,
                    arg_count,
                    index + arg_param_offset,
                    arg,
                    ctx,
                    instructions,
                ) {
                    success = false;
                }
            }

            if success {
                instructions.push(Instruction::CallFunction {
                    name: neo_name,
                    arg_count,
                });
            }

            // A VOID library/receiver function (e.g. `x.assertEq(4)` via a
            // `using ... for` directive, where `assertEq` returns nothing)
            // pushes no value. Report `false` so the statement-position lowering
            // does not emit a trailing DROP against an empty stack — a
            // "Stack underflow" VM fault. Mirrors the namespaced `Lib.f(...)`
            // void path above.
            if ctx.is_void_function(&member.name) {
                return Some(false);
            }
            return Some(success);
        }

        // NeoVM iterator handles are advanced via syscalls. Some devpacks express this as
        // `iterator.next()` / `iterator.value()` on a handle-like type; treat those as
        // `System.Iterator.Next` / `System.Iterator.Value` when no user-defined overload exists.
        if !ctx.function_names.contains(&member.name) && args.is_empty() {
            if member.name == "next" {
                if !lower_expression(inner.as_ref(), ctx, instructions) {
                    return Some(false);
                }
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::Syscall("System.Iterator.Next".to_string()),
                    arg_count: 1,
                });
                return Some(true);
            }

            if member.name == "value" {
                if !lower_expression(inner.as_ref(), ctx, instructions) {
                    return Some(false);
                }
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::Syscall("System.Iterator.Value".to_string()),
                    arg_count: 1,
                });
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::one(),
                )));
                instructions.push(Instruction::ArrayGet);
                return Some(true);
            }
        }

        // Builtin helper libraries (Runtime/Storage/Syscalls/Neo/NativeCalls/abi) are lowered
        // directly by the compiler. When a call targets one of these libraries but does not map
        // to a supported intrinsic, emit a targeted diagnostic rather than a generic
        // "unsupported external/library call" error.
        if let Some(base) = resolve_static_library_base(inner.as_ref(), ctx) {
            if let Some(supported) = builtin_library_supported_members(base.as_str()) {
                ctx.record_error(format!(
                    "unsupported builtin library call '{}.{}'; supported {} intrinsics: {}. (Builtin devpack libraries are compiler intrinsics; their Solidity bodies are not compiled.)",
                    base,
                    member.name,
                    base,
                    format_builtin_member_list(supported)
                ));
                return Some(false);
            }
        }

        // NOTE: `push`/`pop` used to have a deliberate "compatibility fallback"
        // here that evaluated the receiver and arguments, DROPPED them, and
        // pushed `true`/`0` — so `uint[] memory a = new uint[](1); a.push(7)`
        // silently no-op'd (a.length stayed 1) and `require(a.push(x), ...)`
        // was vacuously true. solc rejects `push`/`pop` on memory arrays at
        // compile time, and unresolved receivers are always a user error, so
        // the fallback now falls through to the same fail-loud path as every
        // other unresolved member (the storage-array push/pop intrinsics are
        // resolved long before this point).

        // A member call that matched none of the resolution branches above is
        // a genuine error (typo'd method, missing/misspelled library function,
        // member access on a type that doesn't support it). Previously this
        // fell through to a silent fallback that dropped the arguments and
        // pushed 0 — masking the error and shipping a function that returns 0
        // (and loses any side-effecting argument). Fail loud instead, matching
        // solc's "member not found or not visible".
        ctx.record_error(format!(
            "unresolved member call '{}(...)': no matching function, library, \
             using-directive method, or builtin found for the receiver",
            member.name
        ));
        return Some(false);
    }

    None
}

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
