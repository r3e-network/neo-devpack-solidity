use super::*;

pub(crate) fn try_lower_address_balance(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "balance" {
        return None;
    }

    if matches!(
        infer_type_from_expression(inner, ctx),
        Some(ValueType::Address)
    ) {
        if !lower_expression(inner, ctx, instructions) {
            return Some(false);
        }
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::NativeCall {
                contract: NativeContract::Gas,
                method: "balanceOf".to_string(),
            },
            arg_count: 1,
        });
        return Some(true);
    }

    None
}

pub(crate) fn try_lower_length_property(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "length" {
        return None;
    }

    // Neo doesn't expose EVM bytecode, but many Solidity contracts use
    // `address.code.length > 0` as a contract-existence check. On Neo N3 we can
    // approximate this by calling `ContractManagement.isContract(address)`:
    // - false => non-contract => length 0
    // - true => contract exists => length 1
    if let Expression::MemberAccess(_, code_inner, code_member) = inner {
        if code_member.name == "code"
            && matches!(
                infer_type_from_expression(code_inner.as_ref(), ctx),
                Some(ValueType::Address)
            )
        {
            if !lower_expression(code_inner.as_ref(), ctx, instructions) {
                return Some(false);
            }

            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::NativeCall {
                    contract: NativeContract::ContractManagement,
                    method: "isContract".to_string(),
                },
                arg_count: 1,
            });

            let not_contract_label = ctx.next_label();
            let end_label = ctx.next_label();

            // JumpIf branches on false, so this jumps when isContract == false.
            instructions.push(Instruction::JumpIf {
                target: not_contract_label,
            });
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::one(),
            )));
            instructions.push(Instruction::Jump { target: end_label });
            instructions.push(Instruction::Label(not_contract_label));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::Label(end_label));
            return Some(true);
        }
    }

    if let Expression::Variable(base) = inner {
        if let Some(state_index) = ctx.state_index_map.get(&base.name) {
            if matches!(ctx.state_type(*state_index), Some(ValueType::Array(_))) {
                // FIXED-SIZE storage array `T[N]` (feature audit): the IR
                // collapses `T[N]` and `T[]` into one Array ValueType, and the
                // base slot is only maintained for DYNAMIC arrays (push/pop/
                // assign write it). A fixed array never writes the slot, so
                // `LoadState` read 0 — silently no-op'ing
                // `for (i = 0; i < a.length; ...)` loops and always-reverting
                // `require(a.length == N)`. The declared bound is a compile-
                // time constant: recover it from the state variable's declared
                // type string (`uint256[3]` → outer dim 3) and push it.
                if let Some(meta) = ctx.state_metadata(*state_index) {
                    if let Some(dims) = parse_nested_fixed_array_shape(&meta.ty) {
                        if let Some(outer) = dims.first() {
                            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                                BigInt::from(*outer as u64),
                            )));
                            return Some(true);
                        }
                    }
                }
                // Dynamic storage array: length is stored at the base slot.
                instructions.push(Instruction::LoadState(*state_index));
                return Some(true);
            }
        }
    }

    // Task #196 — `fn_that_returns_storage_pointer().length`. The inner
    // expression is `FunctionCall(Variable(fn), [])`; unwrap it to the
    // backing state-variable and re-run the Array-state-var fast path
    // above. Without this, the legacy `GetSize` fallback at the bottom
    // of this function fires on whatever the call returned (for an
    // array state var, that's the LENGTH integer via `LoadState` →
    // `emit_coerce_storage_value(Array)`), and SIZE on an Integer
    // raises `SIZE: unsupported type`.
    if let Expression::FunctionCall(_, func, args) = inner {
        if args.is_empty() {
            if let Expression::Variable(fn_ident) = func.as_ref() {
                if let Some(state_var_name) = ctx.storage_pointer_returning_fn(&fn_ident.name) {
                    if let Some(state_index) = ctx.state_index_map.get(state_var_name) {
                        if matches!(ctx.state_type(*state_index), Some(ValueType::Array(_))) {
                            instructions.push(Instruction::LoadState(*state_index));
                            return Some(true);
                        }
                    }
                }
            }
        }
    }

    // Task #161 — `.length` on a storage-indirected dynamic array (e.g.
    // `records[msg.sender].length` where `records` is
    // `mapping(address => uint[])`, or `m[k].arr.length` for a struct-array
    // field) must read the length from the same storage slot that the
    // matching `push` / `pop` paths read and write. That slot is derived
    // via `resolve_storage_reference` + `emit_storage_load`, which handles:
    //   - mapping-keyed arrays (`m[k].length` → LoadMappingElement at the
    //     base slot computed from `records.state_index + hash(k)`), and
    //   - struct-field arrays (`m[k].field.length`, nested struct paths).
    //
    // Without this branch we fall through to the generic `GetSize` path
    // below, which emits a bare SIZE opcode against a storage-reference
    // abstraction the runtime doesn't understand — surfacing as the
    // "Execution failed: SIZE: unsupported type" exception the harness
    // catches in batch68_rr4.
    if let Some(reference) = resolve_storage_reference(inner, ctx) {
        if matches!(reference.value_type, ValueType::Array(_)) {
            if !emit_storage_load(&reference, ctx, instructions) {
                return Some(false);
            }
            return Some(true);
        }
    }

    if lower_expression(inner, ctx, instructions) {
        instructions.push(Instruction::GetSize);
        return Some(true);
    }

    Some(false)
}

pub(crate) fn try_lower_current_key(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "currentKey" {
        return None;
    }

    // Neo storage iterators yield [key, value] pairs. Devpacks sometimes model this as
    // a struct field; extract key via `System.Iterator.Value` + PICKITEM(0).
    if !lower_expression(inner, ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::Syscall("System.Iterator.Value".to_string()),
        arg_count: 1,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::ArrayGet);
    Some(true)
}

pub(crate) fn try_lower_current_value(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "currentValue" {
        return None;
    }

    // Neo storage iterators yield [key, value] pairs. Devpacks sometimes model this as
    // a struct field; extract value via `System.Iterator.Value` + PICKITEM(1).
    if !lower_expression(inner, ctx, instructions) {
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
    Some(true)
}

pub(crate) fn try_lower_code_property(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name == "codehash" {
        if matches!(
            infer_type_from_expression(inner, ctx),
            Some(ValueType::Address)
        ) {
            // Neo N3 auto-compat: address.codehash → the address itself (script hash)
            // On Neo, a contract's "code hash" IS its script hash (UInt160), which is
            // the same as the address. For non-contract addresses, return bytes32(0).
            ctx.record_warning_with_suggestion(
                "address.codehash auto-mapped to the contract script hash on Neo N3. For contract addresses this returns the address itself; for non-contract addresses it returns bytes32(0).",
                "Prefer explicit ContractManagement.isContract checks when you need Neo-native contract detection semantics.",
            );
            if !lower_expression(inner, ctx, instructions) {
                return Some(false);
            }
            // Duplicate the address for the isContract check
            instructions.push(Instruction::Dup);
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::NativeCall {
                    contract: NativeContract::ContractManagement,
                    method: "isContract".to_string(),
                },
                arg_count: 1,
            });
            let not_contract_label = ctx.next_label();
            let end_label = ctx.next_label();
            instructions.push(Instruction::JumpIf {
                target: not_contract_label,
            });
            // Is a contract: address IS the script hash — leave it on stack
            instructions.push(Instruction::Jump { target: end_label });
            instructions.push(Instruction::Label(not_contract_label));
            // Not a contract: drop address, push zero bytes
            instructions.push(Instruction::Drop(ValueType::Any));
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![
                0u8;
                32
            ])));
            instructions.push(Instruction::Label(end_label));
            return Some(true);
        }
        return None;
    }

    if member.name != "code" {
        return None;
    }

    if lower_expression(inner, ctx, instructions) {
        ctx.record_warning_with_suggestion(
            "address.code auto-mapped to the Neo contract script bytes via ContractManagement.getContract(). This differs from EVM runtime bytecode semantics and returns empty bytes for non-contract addresses.",
            "Use ContractManagement.isContract(address) or address.code.length when you only need existence checks.",
        );
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::GetContractScript,
            arg_count: 1,
        });
        return Some(true);
    }

    Some(false)
}
