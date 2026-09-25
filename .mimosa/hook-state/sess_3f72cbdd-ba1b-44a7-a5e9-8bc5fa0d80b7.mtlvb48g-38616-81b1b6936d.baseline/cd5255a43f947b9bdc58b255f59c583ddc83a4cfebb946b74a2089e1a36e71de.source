use super::*;

pub(crate) fn try_lower_member_builtin(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let Expression::MemberAccess(_, inner, member) = func else {
        return None;
    };
    let Expression::Variable(base) = inner.as_ref() else {
        return None;
    };

    // Task #65 — `abi.encodeCall(Target.foo, (args))` resolves the function
    // pointer to its 4-byte selector via the type-method selector registry
    // (same mechanism Task #54 wired up for `this.method.selector`). The
    // resulting calldata mirrors `abi.encodeWithSelector(selector, args)`:
    // `selector ++ abi.encode(args)`.
    //
    // Task #44 (this re-apply) complements the selector by routing the
    // trailing args through the EVM-canonical `abiEncode` handler; together
    // they produce the 36-byte payload the fuzz harness
    // `abi_encode_call_same_bug_as_encode` pins.
    //
    // Task #106 — canonicalise struct params into their EVM tuple form
    // (`(field1,field2,...)`) both in the selector signature AND in the
    // encoded payload (by expanding each struct arg into per-field 32-byte
    // slots). Without this, `abi.encodeCall(this.f, (p))` where
    // `p = P{uint256 a; bool b}` under-sizes to 36 bytes (selector+BE(p.a))
    // and computes the wrong selector.
    if base.name == "abi" && member.name == "encodeCall" {
        if args.len() != 2 {
            ctx.record_error(
                "abi.encodeCall requires a function selector and a tuple argument list",
            );
            return Some(false);
        }

        let selector_bytes = match &args[0] {
            Expression::MemberAccess(_, target_inner, target_method) => {
                // Task #106 — also recognise `this.method` by resolving
                // against the current contract's method table (mirrors the
                // `.selector` path in member_access/selectors.rs). Without
                // this, `abi.encodeCall(this.f, ...)` falls through every
                // contract-type check and lands in the `keccak256(name())`
                // fallback — producing the parameterless-method selector
                // regardless of the real signature.
                let type_name = match target_inner.as_ref() {
                    Expression::Variable(type_id) if ctx.is_contract_type_name(&type_id.name) => {
                        Some(type_id.name.clone())
                    }
                    Expression::Variable(type_id) if type_id.name == "this" => {
                        Some(ctx.current_contract_name().to_string())
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
                };

                let resolved = type_name
                    .and_then(|name| {
                        ctx.type_method_selectors(&name, &target_method.name)
                            .and_then(|selectors| {
                                if selectors.len() == 1 {
                                    Some(selectors[0].to_vec())
                                } else {
                                    None
                                }
                            })
                    })
                    .unwrap_or_else(|| {
                        // Fallback: keccak256(methodName()) matches the
                        // Ethereum-spec convention when the method has no
                        // params and is the default for unresolved refs.
                        let mut hasher = Keccak256::new();
                        hasher.update(format!("{}()", target_method.name).as_bytes());
                        let digest = hasher.finalize();
                        digest[..4].to_vec()
                    });
                resolved
            }
            _ => {
                ctx.record_error(
                    "abi.encodeCall function reference must be `Type.method` or an interface member",
                );
                return Some(false);
            }
        };

        // Push the 4-byte selector.
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            selector_bytes,
        )));

        // Extract the tuple argument list: `(arg0, arg1, ...)` or a single
        // non-tuple expression. The AST shape is `Expression::List` for
        // multi-arg tuples and a plain expression for single-arg tuples.
        let payload_args: Vec<&Expression> = match &args[1] {
            Expression::List(_, params) => params
                .iter()
                .filter_map(|(_, p)| p.as_ref().map(|param| &param.ty))
                .collect(),
            Expression::Parenthesis(_, inner) => vec![inner.as_ref()],
            other => vec![other],
        };

        if let Some(result) =
            lower_abi_encode_args_direct_for_encode_call(&payload_args, ctx, instructions)
        {
            if !result {
                return Some(false);
            }
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
            return Some(true);
        }

        // Task #89 — `bytesN(..)` / `address(..)` casts route through
        // `coerce_to_fixed_bytes` which leaves the MEMCPY-returned dst
        // buffer on the stack BENEATH the canonical ByteString result.
        // The subsequent `AbiEncode` only pops `arg_count` canonical items
        // from the top, so each leaked buffer would persist under the
        // selector pushed above. The final `BytesConcat(2)` would then
        // CAT a stray leak with the encoded payload instead of the real
        // selector — producing a 0x00000000 selector slot that duplicates
        // the arg bytes (see fuzz harness
        // batch35_k5c_encode_call_selector_bytes32_mismatch). Mirror the
        // Task #66 fix in resolved.rs: follow each leaky cast with
        // `Swap; Drop` to discard the leak before AbiEncode.
        //
        // Task #106 — when an arg's inferred type is a struct, expand it into
        // its fields for the payload so `abi.encode(p.a, p.b, ...)` is produced.
        // The struct param arrives as a single `Array` (one arg slot), so each
        // field is read by index from that Array. If the arg isn't a struct,
        // lower it normally.
        let mut success = true;
        let mut flat_arg_count: usize = 0;
        for arg in &payload_args {
            // Detect struct-typed arg whose expression is a bare Variable —
            // then read each field by index from its loaded Array.
            if let Some((param_slot, field_count)) = resolve_struct_param_flat_slots(arg, ctx) {
                emit_struct_param_field_loads(instructions, param_slot, field_count);
                flat_arg_count += field_count;
                continue;
            }

            if !lower_expression(arg, ctx, instructions) {
                success = false;
                continue;
            }
            // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
            flat_arg_count += 1;
        }
        if !success {
            return Some(false);
        }

        if flat_arg_count > 0 {
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::AbiEncode,
                arg_count: flat_arg_count,
            });
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
        }

        return Some(true);
    }

    if base.name == "abi"
        && matches!(
            member.name.as_str(),
            "encodeWithSignature" | "encodeWithSelector"
        )
    {
        ctx.record_warning_with_suggestion(
            format!(
                "abi.{}(...) is approximated on Neo N3 as selector bytes concatenated with abi.encode(args). This differs from raw EVM calldata semantics.",
                member.name
            ),
            "Prefer typed contract calls when possible, or use the returned bytes as a Neo-side calldata approximation only.",
        );

        if member.name == "encodeWithSignature" {
            let Some((signature_expr, payload_args)) = args.split_first() else {
                ctx.record_error("abi.encodeWithSignature requires a signature argument");
                return Some(false);
            };

            let Some(signature) = resolve_signature_string(signature_expr, ctx) else {
                ctx.record_error(
                    "abi.encodeWithSignature signature must be a string literal or a constant string",
                );
                return Some(false);
            };

            let mut hasher = Keccak256::new();
            hasher.update(signature.as_bytes());
            let digest = hasher.finalize();
            let selector = digest[..4].to_vec();

            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(selector)));

            if payload_args.is_empty() {
                return Some(true);
            }

            if let Some(result) =
                lower_abi_encode_args_direct_from_slice(payload_args, ctx, instructions)
            {
                if !result {
                    return Some(false);
                }
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: 2,
                });
                return Some(true);
            }

            let mut success = true;
            for arg in payload_args {
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                }
            }
            if !success {
                return Some(false);
            }

            if !payload_args.is_empty() {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::AbiEncode,
                    arg_count: payload_args.len(),
                });
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: 2,
                });
            }

            return Some(true);
        }

        let Some((selector_expr, payload_args)) = args.split_first() else {
            ctx.record_error("abi.encodeWithSelector requires a selector argument");
            return Some(false);
        };

        if !lower_expression(selector_expr, ctx, instructions) {
            return Some(false);
        }

        if payload_args.is_empty() {
            return Some(true);
        }

        if let Some(result) =
            lower_abi_encode_args_direct_from_slice(payload_args, ctx, instructions)
        {
            if !result {
                return Some(false);
            }
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
            return Some(true);
        }

        let mut success = true;
        for arg in payload_args {
            if !lower_expression(arg, ctx, instructions) {
                success = false;
            }
        }
        if !success {
            return Some(false);
        }

        if !payload_args.is_empty() {
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::AbiEncode,
                arg_count: payload_args.len(),
            });
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
        }

        return Some(true);
    }

    if let Some(result) = try_lower_runtime_member_builtin(base, member, args, ctx, instructions) {
        return Some(result);
    }

    if let Some(result) = try_lower_syscalls_member_builtin(base, member, args, ctx, instructions) {
        return Some(result);
    }

    if let Some(result) = try_lower_storage_member_builtin(base, member, args, ctx, instructions) {
        return Some(result);
    }

    if let Some(result) = try_lower_neo_member_builtin(base, member, args, ctx, instructions) {
        return Some(result);
    }

    if let Some(result) =
        try_lower_nativecalls_member_builtin(base, member, args, ctx, instructions)
    {
        return Some(result);
    }

    None
}
