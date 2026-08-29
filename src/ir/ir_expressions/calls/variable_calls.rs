use super::*;

pub(crate) fn try_lower_variable_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Expression::Variable(identifier) = func {
        if identifier.name == "require" || identifier.name == "assert" {
            ctx.record_error(format!(
                "{}() cannot be used as an expression",
                identifier.name
            ));
            return Some(false);
        }

        if identifier.name == "selfdestruct" {
            // Neo N3 auto-compat: selfdestruct(addr) → ContractManagement.destroy()
            // Note: Neo destroy does NOT transfer remaining funds to addr.
            // The addr argument is evaluated (for side effects) then dropped.
            ctx.record_warning_with_suggestion(
                "selfdestruct() auto-mapped to ContractManagement.destroy() on Neo N3. The recipient address argument is ignored because Neo does not transfer remaining funds on destroy.",
                "Use NativeCalls.gasTransfer() to move funds before destroying the contract.",
            );
            if args.len() == 1 {
                // Evaluate the address argument for side effects, then drop it.
                if !lower_expression(&args[0], ctx, instructions) {
                    return Some(false);
                }
                instructions.push(Instruction::Drop(ValueType::Any));
            }
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::NativeCall {
                    contract: NativeContract::ContractManagement,
                    method: "destroy".to_string(),
                },
                arg_count: 0,
            });
            // `destroy` is a VOID native method, so the native-call emitter
            // pushes a PUSH1 placeholder to balance the statement-position DROP
            // that other void calls get. `selfdestruct` reports no value
            // (`Some(false)`) and so receives no such DROP — drop the
            // placeholder here, otherwise it leaks onto the stack and becomes a
            // spurious return value (e.g. `kill()` returning `1`).
            instructions.push(Instruction::Drop(ValueType::Any));
            return Some(false); // void — no return value
        }

        if identifier.name == "blockhash" {
            // Neo N3 auto-compat: blockhash(n) → Ledger.getBlock(n).hash.
            //
            // The live LedgerContract native ABI has NO `getBlockHash` method
            // (only getBlock/getTransaction/... — verified against mainnet
            // getnativecontracts), so the previous lowering to a
            // `getBlockHash` native call compiled clean but FAULTED
            // method-not-found on a real node for every blockhash() use.
            // Instead call the real `getBlock(index)` (returns the Block array,
            // whose field 0 is the block hash) and read index 0, matching
            // EVM's "0 for an unavailable block" by returning bytes32(0) when
            // getBlock yields Null (out-of-range / future index).
            ctx.record_warning_with_suggestion(
                "blockhash() maps to Ledger.getBlock(n).hash on Neo N3.",
                "Use Ledger.getBlock(index) explicitly in Neo-native Solidity.",
            );
            if args.len() == 1 {
                if !lower_expression(&args[0], ctx, instructions) {
                    return Some(false);
                }
            } else {
                ctx.record_error("blockhash() requires exactly 1 argument");
                return Some(false);
            }
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::NativeCall {
                    contract: NativeContract::Ledger,
                    method: "getBlock".to_string(),
                },
                arg_count: 1,
            });
            // Null-guard the Block array: [block_or_null].
            let notnull_label = ctx.next_label();
            let end_label = ctx.next_label();
            instructions.push(Instruction::Dup);
            instructions.push(Instruction::PushLiteral(LiteralValue::Null));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
            // JumpIf branches when the condition is FALSE (block != null) ->
            // extract the hash; otherwise fall through to the null case.
            instructions.push(Instruction::JumpIf {
                target: notnull_label,
            });
            // Null: drop it and yield bytes32(0).
            instructions.push(Instruction::Drop(ValueType::Any));
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![
                0u8;
                32
            ])));
            instructions.push(Instruction::Jump { target: end_label });
            // Not null: Block field 0 is the hash.
            instructions.push(Instruction::Label(notnull_label));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::ArrayGet);
            instructions.push(Instruction::Label(end_label));
            return Some(true);
        }

        if identifier.name == "gasleft" {
            // Neo N3 auto-compat: gasleft() → System.Runtime.GasLeft
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::Syscall("System.Runtime.GasLeft".to_string()),
                arg_count: 0,
            });
            return Some(true);
        }

        if identifier.name == "mulmod" || identifier.name == "addmod" {
            // EVM builtin compatibility:
            // - mulmod(a, b, m): m == 0 ? 0 : (a * b) % m
            // - addmod(a, b, m): m == 0 ? 0 : (a + b) % m
            if args.len() != 3 {
                ctx.record_error(format!(
                    "{}() requires exactly 3 arguments",
                    identifier.name
                ));
                return Some(false);
            }

            let tmp_id = ctx.next_label();
            let lhs_slot = ctx.allocate_local(format!("__{}_lhs_{tmp_id}", identifier.name), None);
            let rhs_slot = ctx.allocate_local(format!("__{}_rhs_{tmp_id}", identifier.name), None);
            let modulus_slot =
                ctx.allocate_local(format!("__{}_modulus_{tmp_id}", identifier.name), None);

            if !lower_expression(&args[0], ctx, instructions) {
                return Some(false);
            }
            instructions.push(Instruction::StoreLocal(lhs_slot));

            if !lower_expression(&args[1], ctx, instructions) {
                return Some(false);
            }
            instructions.push(Instruction::StoreLocal(rhs_slot));

            if !lower_expression(&args[2], ctx, instructions) {
                return Some(false);
            }
            instructions.push(Instruction::StoreLocal(modulus_slot));

            let ok_label = ctx.next_label();

            instructions.push(Instruction::LoadLocal(modulus_slot));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
            // JumpIf branches when the condition is FALSE (modulus != 0) -> skip
            // the panic and compute normally.
            instructions.push(Instruction::JumpIf { target: ok_label });
            // Solidity >= 0.5.0: zero modulus reverts Panic(0x12) (not the raw
            // EVM opcode's 0). `emit_panic` throws and diverges.
            emit_panic(0x12, instructions);
            instructions.push(Instruction::Label(ok_label));
            // Compute `(a OP b) % m` with FULL-WIDTH intermediates. Both routines
            // must avoid the native-op truncation at 2^256:
            //   - `mulmod`'s product can be up to 512 bits; a native MUL would
            //     truncate it and give a wrong residue whenever `a*b >= 2^256`,
            //     so it uses the 512-bit schoolbook routine.
            //   - `addmod`'s sum can be 257 bits; a native ADD adds the two
            //     32-byte words as SIGNED and discards the carry out of bit 255,
            //     so it uses `emit_u256_addmod_ir` (reduce-then-carry-correct).
            // Both keep the modulus reduction UNSIGNED (native MOD is signed and
            // wrong for moduli >= 2^255).
            if identifier.name == "mulmod" {
                instructions.push(Instruction::LoadLocal(lhs_slot));
                instructions.push(Instruction::LoadLocal(rhs_slot));
                instructions.push(Instruction::LoadLocal(modulus_slot));
                emit_u256_mulmod_512bit_ir(ctx, instructions);
            } else {
                emit_u256_addmod_ir(ctx, instructions, lhs_slot, rhs_slot, modulus_slot);
            }
            return Some(true);
        }

        // Task #92 — `E(v)` enum value cast. Parser shape is `Variable(E)`
        // + one arg; previously this fell through to the unresolved-call
        // compatibility path that dropped `v` and pushed 0. Preserve the
        // discriminant and emit the Solidity-spec range guard: Panic(0x21)
        // when `v >= variant_count`.
        if args.len() == 1 {
            if let Some(variants) = ctx.enum_variant_map.get(&identifier.name) {
                let variant_count = variants.len() as u64;
                let fail_label = ctx.next_label();
                let ok_label = ctx.next_label();
                if !lower_expression(&args[0], ctx, instructions) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    return Some(false);
                }
                let tmp_id = ctx.next_label();
                let value_slot = ctx.allocate_local(
                    format!("__enum_cast_{tmp_id}"),
                    Some(ValueType::Integer {
                        signed: false,
                        bits: 8,
                    }),
                );
                instructions.push(Instruction::StoreLocal(value_slot));
                // IR `JumpIf` branches when the operand is false. Compute
                // `v < variant_count`: when true, fall through and Jump to
                // `ok_label`; when false (v out-of-range), JumpIf branches
                // to `fail_label` and emits the Panic(0x21) payload.
                instructions.push(Instruction::LoadLocal(value_slot));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(variant_count),
                )));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
                instructions.push(Instruction::JumpIf { target: fail_label });
                instructions.push(Instruction::Jump { target: ok_label });
                instructions.push(Instruction::Label(fail_label));
                // Task #107 — route through the shared `emit_panic` helper
                // which emits the canonical EVM Panic(uint256) envelope so
                // `catch Panic(uint code)` can bind code = 0x21.
                emit_panic(0x21, instructions);
                instructions.push(Instruction::Label(ok_label));
                instructions.push(Instruction::LoadLocal(value_slot));
                return Some(true);
            }
        }

        // Treat `ContractType(addressExpr)` as a no-op cast for known contract/interface
        // types when the argument is already address-like (including 20-byte hex literals).
        if args.len() == 1 && ctx.is_contract_type_name(&identifier.name) {
            if matches!(
                infer_type_from_expression(&args[0], ctx),
                Some(ValueType::Address)
            ) {
                return Some(lower_expression(&args[0], ctx, instructions));
            }

            if let Some(bytes) = address_bytes_le_from_expression(&args[0]) {
                instructions.push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                return Some(true);
            }
        }

        if ctx.function_names.contains(&identifier.name) {
            // Task #8 — storage-pointer receiver inlining for a PLAIN positional
            // call `f(accts[a])` where `f`'s first parameter is `T storage`. A
            // regular `CallFunction` materialises the storage struct into a
            // detached NeoVM copy and the callee's `x.nonce += 1` mutates that
            // copy, which is discarded on return (the field write is lost).
            // Mirror the member-call storage-pointer path (member_calls.rs:615)
            // by inlining the body with the first param aliased to the caller's
            // `StorageReference`, so field stores write through to the real slot.
            // Keyed by (name, args.len()) because the storage pointer is the
            // FIRST positional arg here (vs. the implicit receiver in `x.f(..)`).
            if !args.is_empty() {
                if let Some(body_info) = ctx
                    .library_storage_body(&identifier.name, args.len())
                    .cloned()
                {
                    if let Some(reference) = resolve_storage_reference(&args[0], ctx) {
                        let produces_value = body_info.return_type.is_some();
                        let _ = inline_library_storage_call(
                            body_info,
                            reference,
                            &args[1..],
                            ctx,
                            instructions,
                        );
                        return Some(produces_value);
                    }
                }
            }
            // Infer argument types BEFORE lowering (lowering consumes the
            // expressions onto the stack) so a same-arity overload can be
            // resolved by type.
            let arg_types: Vec<Option<ValueType>> = args
                .iter()
                .map(|arg| infer_type_from_expression(arg, ctx))
                .collect();
            let mut success = true;
            for (index, arg) in args.iter().enumerate() {
                // Canonicalize a bytesN literal arg to its BE ByteString using
                // the callee's parameter type (see lower_call_arg_canonical) so
                // an internal call like `f(0x01..)` delivers the correct bytes,
                // not a little-endian Integer.
                if !lower_call_arg_canonical(
                    &identifier.name,
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
                // A positive integer literal arg is convertible to any integer
                // param (Solidity), so mark it for the overload resolver.
                let arg_is_int_literal: Vec<bool> = args
                    .iter()
                    .map(|a| matches!(a, Expression::NumberLiteral(..)))
                    .collect();
                if let Some(neo_name) = ctx.resolve_overload(
                    &identifier.name,
                    args.len(),
                    &arg_types,
                    &arg_is_int_literal,
                ) {
                    instructions.push(Instruction::CallFunction {
                        name: neo_name,
                        arg_count: args.len(),
                    });
                } else {
                    // The function name is in scope (`ctx.function_names`)
                    // but no overload with this arity exists in the host's
                    // function table. This typically happens for abstract
                    // declarations imported via sibling-merge: e.g. when
                    // VRFCoordinatorV2_5 sibling-merges VRFConsumerBaseV2's
                    // `rawFulfillRandomWords`, the body calls
                    // `fulfillRandomWords(uint256, uint256[])` — an abstract
                    // declaration whose body lives in a different contract
                    // we don't compile alongside.
                    //
                    // Instead of failing compilation outright (which
                    // forbids deploying ANY part of the contract), emit a
                    // runtime ABORTMSG so the well-formed pieces still
                    // deploy and only the unimplemented path traps. Drop
                    // the already-lowered args first so the stack ends up
                    // in a sensible state for downstream consumers.
                    ctx.record_warning_with_suggestion(
                        format!(
                            "no overload of '{}' with {} argument(s) is reachable; emitting runtime trap at this call site",
                            identifier.name,
                            args.len()
                        ),
                        "this often comes from a sibling-merged body referencing an abstract function defined in another contract; provide a concrete override or remove the call path",
                    );
                    for _ in 0..args.len() {
                        instructions.push(Instruction::Drop(ValueType::Any));
                    }
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        format!("'{}'/{} has no compiled body", identifier.name, args.len())
                            .into_bytes(),
                    )));
                    instructions.push(Instruction::AbortMsg);
                    // Push a default return value so any caller that expects
                    // a value sees something well-typed. The abort traps
                    // before this is read at runtime.
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        num_bigint::BigInt::zero(),
                    )));
                }
            }

            // Void functions don't push a value onto the stack, so return
            // false to prevent the caller from emitting a spurious DROP.
            if ctx.is_void_function(&identifier.name) {
                return Some(false);
            }
            return Some(success);
        }

        // Task #186 — function-pointer invocation. If the identifier names a
        // parameter or local that has been recorded as an internal-function-
        // pointer binding, lower the call as:
        //     PushFunctionOffset (via LoadParameter/LoadLocal of the slot)
        //     arg0, arg1, ..., argN-1
        //     CallIndirect { arg_count, has_return }
        // The bytecode emitter turns this into REVERSEN(N+1) + CALLA.
        if let Some(binding) = ctx.function_pointer_binding(&identifier.name).cloned() {
            // Load the function-offset value first so that `CallIndirect` sees
            // the stack as `[target, arg0, arg1, ..., argN-1]`.
            if let Some(param_index) = ctx.param_index_map.get(&identifier.name).copied() {
                instructions.push(Instruction::LoadParameter(param_index));
            } else if let Some(local_index) = ctx.resolve_local(&identifier.name) {
                instructions.push(Instruction::LoadLocal(local_index));
            } else {
                // Defensive: binding tracked but no backing slot — fall through
                // to the legacy compatibility path.
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
            }
            let mut success = true;
            for arg in args {
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                }
            }
            if args.len() != binding.arg_count {
                ctx.record_error(format!(
                    "internal function pointer '{}' expects {} argument(s), got {}",
                    identifier.name,
                    binding.arg_count,
                    args.len()
                ));
                return Some(false);
            }
            instructions.push(Instruction::CallIndirect {
                arg_count: args.len(),
                has_return: binding.has_return,
            });
            // When the target function is void, CallIndirect leaves no value on
            // the stack; mirror `CallFunction`'s void-return convention.
            if !binding.has_return {
                return Some(false);
            }
            return Some(success);
        }

        if ctx.resolve_local(&identifier.name).is_some() {
            // Compatibility fallback for function-typed locals and unresolved callables.
            // Preserve argument side effects and materialize a default return value.
            let mut success = true;
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

        // Compatibility fallback for unresolved free-function calls.
        let mut success = true;
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

    None
}
