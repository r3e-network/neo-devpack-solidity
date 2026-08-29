//! ## Low-Level Call Lowering (`address.call`, `address.staticcall`, `address.delegatecall`)
//!
//! This module lowers EVM-style low-level calls into Neo N3 runtime operations.
//! It handles signature resolution, precompile dispatch, pseudo-runtime
//! emulation, and re-encoding of return data.
//!
//! ### Pipeline
//!
//! | Section | Purpose |
//! |---------|---------|
//! | Signature parsing | Resolve `abi.encodeWithSignature` / `abi.encodeCall` payloads to method names |
//! | Call data resolution | Parse low-level call payload into (method, arguments) |
//! | Target resolution | Map target addresses to precompile indices or pseudo-runtime indices |
//! | Emit helpers | Build VM instructions for success tuples and staticcall dispatch |
//! | Precompile support | Precompile index naming, error recording, staticcall emission |
//! | Main dispatch | `try_lower_low_level_address_call` — the main entry point |
//! | Return data | Re-encode return data after a call completes |

use super::*;
pub(crate) fn try_lower_low_level_address_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    // Limited low-level call support:
    // `address.call(abi.encodeWithSignature("foo(T1,T2)", a, b))`
    // `address.staticcall(abi.encodeWithSignature("foo(T1,T2)", a, b))`
    // `address.delegatecall(abi.encodeWithSignature("foo(T1,T2)", a, b))`
    // `bytes data = abi.encodeWithSignature(...); address.call(data)`
    // `bytes data = abi.encodeWithSelector(...); address.staticcall(data)`
    //
    // These are lowered into Neo `System.Contract.Call` invocations and return
    // `(success, Serialize(ret))`, mirroring Solidity's `(bool, bytes)` low-level calls.
    // We wrap only the contract call itself in NeoVM TRY/ENDTRY so that callee faults
    // become `success=false` with empty return data, while local evaluation errors still
    // abort execution.
    if let Expression::MemberAccess(_, inner, member) = func {
        let member_name = member.name.as_str();
        let is_staticcall = member_name == "staticcall";
        let is_delegatecall = member_name == "delegatecall";
        let is_callcode = member_name == "callcode";
        // Task #101 — delegatecall is not supported on Neo N3; there is no
        // semantic equivalent. Previously the compiler emitted a warning and
        // silently lowered `target.delegatecall(data)` to `System.Contract.Call`,
        // which runs the callee's bytecode in the CALLEE's storage context —
        // the inverse of EVM semantics. That silent miscompile broke EIP-1967 /
        // OpenZeppelin TransparentProxy / UUPS / Beacon proxy patterns at
        // runtime with no user-visible warning. We now reject the construct at
        // compile time so users are forced to rewrite using Neo's upgrade
        // primitive (ContractManagement.update) or inheritance. `.callcode` is
        // the deprecated EVM symmetric form and receives the same treatment.
        if (is_delegatecall || is_callcode) && args.len() == 1 {
            let which = if is_delegatecall {
                "delegatecall"
            } else {
                "callcode"
            };
            // Neo N3 has no semantic equivalent for delegatecall — you cannot
            // execute another contract's code against the caller's storage.
            // Previously the compiler rejected this at compile time, which
            // is correct *if* the user is actually trying to use delegatecall
            // semantics. But many real-world contracts only include
            // delegatecall through dead inheritance paths (every contract
            // that imports OZ Address.sol gets `functionDelegateCall` in its
            // function table even if it never calls it). Reject at runtime
            // instead, via ABORTMSG — the contract compiles, deploys, and
            // only fails if execution actually reaches the delegatecall.
            //
            // Compile-time guards are still emitted as warnings so users
            // know they need to rewrite proxy/upgrade patterns. Note that
            // this still doesn't silently miscompile to a wrong-storage
            // System.Contract.Call (which was the original Task #101 bug);
            // it's a hard runtime trap.
            ctx.record_warning_with_suggestion(
                format!(
                    "{which} is not supported on Neo N3; the compiler emitted \
                     a runtime trap at this call site — invoking this code \
                     path will revert. Use ContractManagement.update for \
                     upgradeability or inherit the target contract instead."
                ),
                "Neo N3 has no semantic equivalent for delegatecall/callcode: there \
				 is no way to execute another contract's code against the caller's \
				 storage. Rewrite proxy/upgrade patterns using ContractManagement.update, \
				 or replace delegation with inheritance (library calls / abstract \
				 contracts) or explicit cross-contract calls via address.call().",
            );
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                format!("{which} is not supported on Neo N3").into_bytes(),
            )));
            instructions.push(Instruction::AbortMsg);
            // Push a `(bool, bytes)` tuple shape so downstream stack
            // consumers see a well-typed value. The abort traps before
            // these literals are observed at runtime.
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(2u8),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            return Some(true);
        }
        if (member_name == "call" || is_staticcall) && args.len() == 1 {
            if ctx.is_safe && member_name == "call" {
                ctx.record_error(
						"address.call(...) / address.delegatecall(...) is not allowed in view/pure functions; use address.staticcall(...) or an external view/pure interface call",
					);
                return Some(false);
            }

            if is_staticcall {
                if let Some(index) = pseudo_runtime_index_from_target(inner.as_ref()) {
                    if let Some(ok) =
                        emit_pseudo_runtime_staticcall(index, &args[0], ctx, instructions)
                    {
                        return Some(ok);
                    }
                }
            }

            // Task #H5: compile-time `address(0x01..=0x09).staticcall(input)`
            // routes to Neo N3 native equivalents (CryptoLib.sha256 / ripemd160
            // / identity pass-through) instead of the opaque-bytes no-op path
            // that returned `(true, bytes(""))`. Only applies to staticcall —
            // call/delegatecall to a precompile is EVM-idiomatic but meaningless
            // on Neo N3 (no storage mutation at those addresses). Unsupported
            // indices (0x06..0x09) fail here instead of falling through to a
            // generic call or fake success tuple.
            if let Some(idx) = precompile_index_from_target(inner.as_ref()) {
                if matches!(idx, 0x06..=0x09) {
                    record_unsupported_precompile_error(idx, member_name, ctx);
                    return Some(false);
                }

                if is_staticcall {
                    if let Some(ok) = emit_precompile_staticcall(idx, &args[0], ctx, instructions) {
                        return Some(ok);
                    }
                }
            }

            match parse_low_level_call_data(&args[0], ctx) {
                Ok(Some((method_name, encode_args))) => {
                    let data_local = ctx.allocate_local("__call_data".to_string(), None);

                    // Build tuple `(success, data)` as an array.
                    let tuple_local = ctx.allocate_local("__call_tuple".to_string(), None);
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(2u8),
                    )));
                    instructions.push(Instruction::NewArray {
                        element_type: ValueType::Any,
                    });
                    instructions.push(Instruction::StoreLocal(tuple_local));

                    if !lower_expression(inner.as_ref(), ctx, instructions) {
                        return Some(false);
                    }

                    instructions.push(Instruction::PushLiteral(LiteralValue::String(
                        method_name.as_bytes().to_vec(),
                    )));

                    let mut lowered = true;
                    for call_arg in &encode_args {
                        if !lower_expression(call_arg, ctx, instructions) {
                            lowered = false;
                        }
                    }

                    if !lowered {
                        return Some(false);
                    }

                    if encode_args.is_empty() {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
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

                    let catch_label = ctx.next_label();
                    let end_label = ctx.next_label();
                    instructions.push(Instruction::Try {
                        catch_target: catch_label,
                    });

                    if is_staticcall {
                        // CallFlags.ReadOnly (ReadStates | AllowCall).
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(0x05u8),
                        )));
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::ContractCallWithFlags,
                            arg_count: 4,
                        });
                    } else {
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::ContractCall,
                            arg_count: 3,
                        });
                    }

                    instructions.push(Instruction::StoreLocal(data_local));
                    if let Some(return_type) = ctx
                        .get_function_return_type(&method_name, encode_args.len())
                        .cloned()
                    {
                        emit_reencoded_low_level_return_data(
                            data_local,
                            &return_type,
                            ctx,
                            instructions,
                        );
                    }

                    // success at index 0
                    instructions.push(Instruction::LoadLocal(tuple_local));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
                    instructions.push(Instruction::ArraySet);

                    // data at index 1
                    instructions.push(Instruction::LoadLocal(tuple_local));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::one(),
                    )));
                    instructions.push(Instruction::LoadLocal(data_local));
                    instructions.push(Instruction::ArraySet);

                    instructions.push(Instruction::EndTry { target: end_label });

                    instructions.push(Instruction::Label(catch_label));
                    // The catch stack top is ALREADY the raw revert payload — the
                    // EVM-canonical ABI envelope (`Error(string)` / `Panic(uint)` /
                    // custom-error selector||abi.encode) pushed by the runtime
                    // (try_frames.rs). Store it directly as the `bytes returndata`.
                    // The old `StdLib.serialize` wrapped it in Neo binary framing
                    // (`[0x28, varint(len), …]`) so `(bool ok, bytes data)` saw a
                    // corrupted, prefixed blob (bug #28) and a no-reason revert came
                    // back non-empty (bug #29). Coerce to ByteArray to pin the type
                    // (matches the normal `catch (bytes)` binding in try_catch.rs).
                    instructions.push(Instruction::Convert {
                        target: ConvertTarget::ByteArray,
                    });
                    instructions.push(Instruction::StoreLocal(data_local));

                    // success=false
                    instructions.push(Instruction::LoadLocal(tuple_local));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
                    instructions.push(Instruction::ArraySet);

                    // data=serialized exception
                    instructions.push(Instruction::LoadLocal(tuple_local));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::one(),
                    )));
                    instructions.push(Instruction::LoadLocal(data_local));
                    instructions.push(Instruction::ArraySet);

                    instructions.push(Instruction::EndTry { target: end_label });
                    instructions.push(Instruction::Label(end_label));
                    instructions.push(Instruction::LoadLocal(tuple_local));
                    return Some(true);
                }
                Ok(None) => {}
                Err(message) => {
                    ctx.record_error(message);
                    return Some(false);
                }
            }

            if let Some((call_data_slot, method_name)) = resolve_call_data_local(&args[0], ctx) {
                let data_local = ctx.allocate_local("__call_data".to_string(), None);

                let tuple_local = ctx.allocate_local("__call_tuple".to_string(), None);
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(2u8),
                )));
                instructions.push(Instruction::NewArray {
                    element_type: ValueType::Any,
                });
                instructions.push(Instruction::StoreLocal(tuple_local));

                if !lower_expression(inner.as_ref(), ctx, instructions) {
                    return Some(false);
                }

                instructions.push(Instruction::PushLiteral(LiteralValue::String(
                    method_name.as_bytes().to_vec(),
                )));
                instructions.push(Instruction::LoadLocal(call_data_slot));

                let catch_label = ctx.next_label();
                let end_label = ctx.next_label();
                instructions.push(Instruction::Try {
                    catch_target: catch_label,
                });

                if is_staticcall {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(0x05u8),
                    )));
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::ContractCallWithFlags,
                        arg_count: 4,
                    });
                } else {
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::ContractCall,
                        arg_count: 3,
                    });
                }

                instructions.push(Instruction::StoreLocal(data_local));

                instructions.push(Instruction::LoadLocal(tuple_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
                instructions.push(Instruction::ArraySet);

                instructions.push(Instruction::LoadLocal(tuple_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::one(),
                )));
                instructions.push(Instruction::LoadLocal(data_local));
                instructions.push(Instruction::ArraySet);

                instructions.push(Instruction::EndTry { target: end_label });

                instructions.push(Instruction::Label(catch_label));
                // Store the raw revert payload directly (see the mirror handler
                // above): the runtime already pushes the EVM ABI envelope, so
                // `StdLib.serialize` only added spurious Neo framing (bug #28/#29).
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::ByteArray,
                });
                instructions.push(Instruction::StoreLocal(data_local));

                instructions.push(Instruction::LoadLocal(tuple_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
                instructions.push(Instruction::ArraySet);

                instructions.push(Instruction::LoadLocal(tuple_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::one(),
                )));
                instructions.push(Instruction::LoadLocal(data_local));
                instructions.push(Instruction::ArraySet);

                instructions.push(Instruction::EndTry { target: end_label });
                instructions.push(Instruction::Label(end_label));
                instructions.push(Instruction::LoadLocal(tuple_local));
                return Some(true);
            }

            // Compatibility fallback for `addr.call("")` patterns (e.g. OpenZeppelin
            // `Address.sendValue`). Neo has no native ETH transfer semantics, so we
            // model this as a successful no-op low-level call with empty returndata.
            if is_empty_low_level_payload(&args[0]) {
                let tuple_local = ctx.allocate_local("__call_tuple".to_string(), None);
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(2u8),
                )));
                instructions.push(Instruction::NewArray {
                    element_type: ValueType::Any,
                });
                instructions.push(Instruction::StoreLocal(tuple_local));

                if !lower_expression(inner.as_ref(), ctx, instructions) {
                    return Some(false);
                }
                instructions.push(Instruction::Drop(ValueType::Any));

                instructions.push(Instruction::LoadLocal(tuple_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
                instructions.push(Instruction::ArraySet);

                instructions.push(Instruction::LoadLocal(tuple_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::one(),
                )));
                instructions.push(Instruction::PushLiteral(
                    LiteralValue::ByteArray(Vec::new()),
                ));
                instructions.push(Instruction::ArraySet);

                instructions.push(Instruction::LoadLocal(tuple_local));
                return Some(true);
            }

            // Opaque `bytes memory` payloads prevent static method-name
            // inference. We can't emit a real `System.Contract.Call`, but we
            // also don't want to refuse compilation outright — many real-world
            // contracts (every OZ contract that imports `Address.sol`, every
            // Safe handler that relays generic calldata) just transitively
            // include opaque-call helpers as dead code that's never reached
            // from the entry points the user actually deploys.
            //
            // Compromise: emit a warning at compile time, then lower the call
            // to a runtime trap (`ABORTMSG`) that surfaces a clear diagnostic
            // if execution ever reaches this point. The contract compiles,
            // unrelated functions work, and only the specific opaque-call
            // path fails — at runtime, not at compile time. Manifest
            // permission analysis won't see a `System.Contract.Call` here,
            // which is fine because no call is actually emitted.
            ctx.record_warning_with_suggestion(
                format!(
                    "address.{member_name}(<opaque bytes>) cannot be statically \
                     lowered to a Neo N3 contract call because the method name \
                     is not known at compile time. The compiler emitted a \
                     runtime trap at this call site — invoking it will revert.",
                ),
                "rewrite the payload as a literal `abi.encodeWithSignature(\"method(T1,T2)\", a, b)` \
                 or `abi.encodeCall(Iface.method, (a, b))` at the call site so the compiler can \
                 lower a real System.Contract.Call and emit the correct permission entry.",
            );
            // Push a static error message and ABORTMSG. ABORTMSG halts the
            // current invocation with that message — equivalent to an EVM
            // revert with reason.
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                format!("opaque address.{member_name}(<bytes>) is not lowerable on Neo N3")
                    .into_bytes(),
            )));
            instructions.push(Instruction::AbortMsg);
            // Per the (bool success, bytes returndata) shape that callers
            // expect, push a default tuple so any pending stack consumers see
            // a well-typed value before the abort traps execution. The
            // abort happens BEFORE these literals would actually be read,
            // but the IR-level type checker (and any downstream peephole
            // optimizer that walks past the abort) wants a typed result.
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(2u8),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            return Some(true);
        }
    }

    None
}
