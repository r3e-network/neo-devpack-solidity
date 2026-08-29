//! ## Low-Level Call Support Helpers
//!
//! Helpers extracted from low_level.rs to keep the main dispatch module under
//! the 800-line limit. This module covers signature/argument parsing, call data
//! resolution, target resolution (precompile + pseudo-runtime), VM instruction
//! emit helpers, precompile support, and return-data re-encoding.
//!
//! Functions are pub(crate) and re-exported from calls so the main
//! low_level module can access them through a single namespace.

use super::*;

pub(crate) fn resolve_signature_string(expr: &Expression, ctx: &LoweringContext) -> Option<String> {
    match expr {
        Expression::Parenthesis(_, inner) => resolve_signature_string(inner, ctx),
        Expression::StringLiteral(parts) => {
            Some(String::from_utf8_lossy(&string_literal_bytes(parts)).to_string())
        }
        Expression::Variable(identifier) => {
            let state_index = ctx.state_index_map.get(&identifier.name).copied()?;
            let meta = ctx.state_metadata(state_index)?;
            if !meta.is_constant {
                return None;
            }
            let initializer = meta.initializer.as_ref()?;
            resolve_signature_string(initializer, ctx)
        }
        Expression::FunctionCall(_, func, args) => {
            if args.len() == 1 {
                match func.as_ref() {
                    Expression::Type(_, _) => resolve_signature_string(&args[0], ctx),
                    Expression::Variable(id) if id.name == "bytes" || id.name == "string" => {
                        resolve_signature_string(&args[0], ctx)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn is_single_argument_bytes_or_type_wrapper(
    func: &Expression,
    args: &[Expression],
) -> bool {
    if args.len() != 1 {
        return false;
    }

    match func {
        Expression::Type(_, _) => true,
        Expression::Variable(id) => id.name == "bytes" || id.name == "string",
        _ => false,
    }
}

pub(crate) fn is_empty_low_level_payload(expr: &Expression) -> bool {
    match expr {
        Expression::Parenthesis(_, inner) => is_empty_low_level_payload(inner),
        Expression::FunctionCall(_, func, args)
            if is_single_argument_bytes_or_type_wrapper(func.as_ref(), args.as_slice()) =>
        {
            is_empty_low_level_payload(&args[0])
        }
        Expression::StringLiteral(parts) => string_literal_bytes(parts).is_empty(),
        _ => match literal_from_expression(expr) {
            Some(LiteralValue::ByteArray(bytes)) => bytes.is_empty(),
            Some(LiteralValue::String(bytes)) => bytes.is_empty(),
            _ => false,
        },
    }
}

pub(crate) fn is_contract_type_reference(expr: &Expression, ctx: &LoweringContext) -> bool {
    match expr {
        Expression::Variable(type_id) => ctx.is_contract_type_name(&type_id.name),
        Expression::MemberAccess(_, namespace_expr, type_id) => {
            matches!(
                namespace_expr.as_ref(),
                Expression::Variable(namespace_id)
                    if !ctx.param_index_map.contains_key(&namespace_id.name)
                        && ctx.resolve_local(&namespace_id.name).is_none()
                        && !ctx.state_index_map.contains_key(&namespace_id.name)
                        && !ctx.is_contract_type_name(&namespace_id.name)
            ) && ctx.is_contract_type_name(&type_id.name)
        }
        _ => false,
    }
}

pub(crate) fn resolve_encode_call_method_name(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<String> {
    if let Some(name) = resolve_selector_method_name(expr, ctx) {
        if !name.trim().is_empty() {
            return Some(name);
        }
    }

    match expr {
        Expression::Parenthesis(_, inner) => resolve_encode_call_method_name(inner, ctx),
        Expression::FunctionCall(_, func, args)
            if is_single_argument_bytes_or_type_wrapper(func.as_ref(), args.as_slice()) =>
        {
            resolve_encode_call_method_name(&args[0], ctx)
        }
        Expression::MemberAccess(_, inner, member) => {
            if member.name == "selector" {
                if let Expression::MemberAccess(_, type_expr, function_member) = inner.as_ref() {
                    if is_contract_type_reference(type_expr.as_ref(), ctx) {
                        let function_name = function_member.name.trim();
                        if !function_name.is_empty() {
                            return Some(function_name.to_string());
                        }
                    }
                }
                return None;
            }

            let name = member.name.trim();
            if name.is_empty() {
                return None;
            }

            if is_contract_type_reference(inner.as_ref(), ctx) {
                return Some(name.to_string());
            }

            // Compatibility fallback for instance-style references like
            // `token.transfer` in `abi.encodeCall(token.transfer, (...))`.
            if matches!(
                inner.as_ref(),
                Expression::Variable(_)
                    | Expression::MemberAccess(_, _, _)
                    | Expression::FunctionCall(_, _, _)
            ) {
                return Some(name.to_string());
            }

            None
        }
        _ => None,
    }
}

pub(crate) fn extract_encode_call_arguments(expr: &Expression) -> Option<Vec<&Expression>> {
    match expr {
        Expression::Parenthesis(_, inner) => extract_encode_call_arguments(inner),
        Expression::FunctionCall(_, func, args)
            if is_single_argument_bytes_or_type_wrapper(func.as_ref(), args.as_slice()) =>
        {
            extract_encode_call_arguments(&args[0])
        }
        Expression::List(_, params) => {
            let mut arguments = Vec::with_capacity(params.len());
            for (_, param) in params {
                let param = param.as_ref()?;
                arguments.push(&param.ty);
            }
            Some(arguments)
        }
        _ => Some(vec![expr]),
    }
}

// ============================================================================
// Section 2 — Call Data Resolution
// ============================================================================

pub(crate) fn parse_low_level_call_data<'a>(
    expr: &'a Expression,
    ctx: &LoweringContext,
) -> Result<Option<(String, Vec<&'a Expression>)>, String> {
    match expr {
        Expression::Parenthesis(_, inner) => parse_low_level_call_data(inner, ctx),
        Expression::FunctionCall(_, func, args)
            if is_single_argument_bytes_or_type_wrapper(func.as_ref(), args.as_slice()) =>
        {
            parse_low_level_call_data(&args[0], ctx)
        }
        Expression::FunctionCall(_, func, args) => {
            let Expression::MemberAccess(_, inner, member) = func.as_ref() else {
                return Ok(None);
            };

            if !matches!(inner.as_ref(), Expression::Variable(id) if id.name == "abi") {
                return Ok(None);
            }

            match member.name.as_str() {
                "encodeWithSignature" => {
                    let Some((first, rest)) = args.split_first() else {
                        return Err(
                            "abi.encodeWithSignature requires a signature argument".to_string()
                        );
                    };

                    let signature = resolve_signature_string(first, ctx).ok_or_else(|| {
						"abi.encodeWithSignature signature must be a string literal or a constant string"
							.to_string()
					})?;

                    let Some(name) = crate::utils::method_name_from_signature(&signature) else {
                        return Err(
                            "abi.encodeWithSignature signature must include a function name"
                                .to_string(),
                        );
                    };
                    Ok(Some((name, rest.iter().collect())))
                }
                "encodeWithSelector" => {
                    let Some((first, rest)) = args.split_first() else {
                        return Err(
                            "abi.encodeWithSelector requires a selector argument".to_string()
                        );
                    };

                    let Some(name) = resolve_selector_method_name(first, ctx) else {
                        // Compatibility fallback: unresolved runtime selectors cannot be
                        // rewritten into Neo method-name calls. Let the caller emit the
                        // opaque-payload diagnostic.
                        return Ok(None);
                    };
                    if name.trim().is_empty() {
                        return Err(
                            "abi.encodeWithSelector selector resolves to an empty name".to_string()
                        );
                    }
                    Ok(Some((name, rest.iter().collect())))
                }
                "encodeCall" => {
                    if args.len() != 2 {
                        return Err(
                            "abi.encodeCall requires function selector and tuple argument list"
                                .to_string(),
                        );
                    }

                    let method_name =
                        resolve_encode_call_method_name(&args[0], ctx).ok_or_else(|| {
                            "abi.encodeCall has an unsupported function reference".to_string()
                        })?;

                    let call_args = extract_encode_call_arguments(&args[1]).ok_or_else(|| {
                        "abi.encodeCall tuple argument list must contain positional expressions"
                            .to_string()
                    })?;

                    Ok(Some((method_name, call_args)))
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

pub(crate) fn resolve_call_data_local(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<(usize, String)> {
    match expr {
        Expression::Parenthesis(_, inner) => resolve_call_data_local(inner, ctx),
        Expression::FunctionCall(_, func, args)
            if is_single_argument_bytes_or_type_wrapper(func.as_ref(), args.as_slice()) =>
        {
            resolve_call_data_local(&args[0], ctx)
        }
        Expression::Variable(identifier) => {
            let slot = ctx.resolve_local(&identifier.name)?;
            let method = ctx.call_data_method_for_local(slot)?.to_string();
            Some((slot, method))
        }
        _ => None,
    }
}
/// Task #H5: detect `address(0xNN)` targets where NN is in the EVM
/// precompile range 0x01..=0x09. Returns the precompile index if the
/// target is a compile-time constant within that range.
///
/// The `inner` in `inner.staticcall(...)` for `address(0xNN).staticcall(...)`
/// is `FunctionCall(Type(Address), [NumberLiteral(0xNN)])`. We unwrap the
/// `address(...)` cast and then delegate to `address_bytes_le_from_expression`
/// for the underlying literal parsing (same codepath used by the address
/// type constructor at `type_constructors.rs:88`). Runtime-computed
/// addresses never match.
// ============================================================================
// Section 3 — Target Resolution (precompile + pseudo-runtime)
// ============================================================================
pub(crate) fn precompile_index_from_target(expr: &Expression) -> Option<u8> {
    let bytes = match expr {
        Expression::Parenthesis(_, inner) => return precompile_index_from_target(inner),
        Expression::FunctionCall(_, func, args) if args.len() == 1 => match func.as_ref() {
            Expression::Type(_, PtType::Address) | Expression::Type(_, PtType::AddressPayable) => {
                address_bytes_le_from_expression(&args[0])?
            }
            _ => return None,
        },
        _ => address_bytes_le_from_expression(expr)?,
    };
    if bytes.len() != 20 {
        return None;
    }
    // UInt160 LE: byte[0] holds the low byte. 0x01..=0x09 precompiles have
    // all other bytes zero.
    if bytes.iter().skip(1).any(|b| *b != 0) {
        return None;
    }
    match bytes[0] {
        idx @ 0x01..=0x09 => Some(idx),
        _ => None,
    }
}

pub(crate) fn pseudo_runtime_index_from_target(expr: &Expression) -> Option<u8> {
    let bytes = match expr {
        Expression::Parenthesis(_, inner) => return pseudo_runtime_index_from_target(inner),
        Expression::FunctionCall(_, func, args) if args.len() == 1 => match func.as_ref() {
            Expression::Type(_, PtType::Address) | Expression::Type(_, PtType::AddressPayable) => {
                address_bytes_le_from_expression(&args[0])?
            }
            _ => return None,
        },
        _ => address_bytes_le_from_expression(expr)?,
    };
    if bytes.len() != 20 || bytes.iter().skip(1).any(|b| *b != 0) {
        return None;
    }
    match bytes[0] {
        idx @ (0xfe | 0xff) => Some(idx),
        _ => None,
    }
}

pub(crate) fn runtime_syscall_for_low_level_method(method: &str) -> Option<&'static str> {
    match method {
        "getNetwork" | "getNetworkMagic" => Some("System.Runtime.GetNetwork"),
        "getPlatform" => Some("System.Runtime.Platform"),
        "getEntryScriptHash" => Some("System.Runtime.GetEntryScriptHash"),
        "getExecutingScriptHash" => Some("System.Runtime.GetExecutingScriptHash"),
        "getCallingScriptHash" => Some("System.Runtime.GetCallingScriptHash"),
        "getAddressVersion" => Some("System.Runtime.GetAddressVersion"),
        "gasLeft" => Some("System.Runtime.GasLeft"),
        _ => None,
    }
}

// ============================================================================
// Section 4 — VM Instruction Emit Helpers
// ============================================================================

pub(crate) fn emit_low_level_success_tuple_from_stack(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let data_local = ctx.allocate_local("__runtime_call_data".to_string(), None);
    let tuple_local = ctx.allocate_local("__runtime_call_tuple".to_string(), None);
    instructions.push(Instruction::StoreLocal(data_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(tuple_local));

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

    instructions.push(Instruction::LoadLocal(tuple_local));
}

pub(crate) fn emit_pseudo_runtime_staticcall(
    index: u8,
    payload: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match index {
        // Compatibility shim used by fuzz/devpack probes:
        // address(0xfe).staticcall(abi.encodePacked(account)) -> Runtime.CheckWitness(account).
        0xfe => {
            if let Ok(Some((method, args))) = parse_low_level_call_data(payload, ctx) {
                if method != "checkWitness" || args.len() != 1 {
                    return None;
                }
                if !lower_expression(args[0], ctx, instructions) {
                    return Some(false);
                }
            } else if !lower_expression(payload, ctx, instructions) {
                return Some(false);
            }
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::RuntimeCheckWitness,
                arg_count: 1,
            });
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_low_level_success_tuple_from_stack(ctx, instructions);
            Some(true)
        }
        // address(0xff).staticcall(abi.encodeWithSignature("getPlatform()")) and friends.
        0xff => {
            let (method, args) = match parse_low_level_call_data(payload, ctx) {
                Ok(Some(parsed)) => parsed,
                Ok(None) => return None,
                Err(message) => {
                    ctx.record_error(message);
                    return Some(false);
                }
            };
            if !args.is_empty() {
                return None;
            }
            let syscall = runtime_syscall_for_low_level_method(&method)?;
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::Syscall(syscall.to_string()),
                arg_count: 0,
            });
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_low_level_success_tuple_from_stack(ctx, instructions);
            Some(true)
        }
        _ => None,
    }
}

// ============================================================================
// Section 5 — Precompile Support
// ============================================================================

pub(crate) fn unsupported_precompile_name(index: u8) -> &'static str {
    match index {
        0x06 => "bn256Add",
        0x07 => "bn256ScalarMul",
        0x08 => "bn256Pairing",
        0x09 => "blake2f",
        _ => "unknown",
    }
}

pub(crate) fn record_unsupported_precompile_error(
    index: u8,
    member_name: &str,
    ctx: &mut LoweringContext,
) {
    let name = unsupported_precompile_name(index);
    ctx.record_error_with_suggestion(
		format!(
			"address(0x{index:02x}).{member_name}(...) targets unsupported EVM precompile {name}; Neo N3 has no equivalent lowering, so compiling it as a generic contract call would be a semantic miscompile",
		),
		"remove this precompile call, guard it behind an EVM-only code path, or add a dedicated Neo implementation whose input/output semantics match the EVM precompile",
	);
}

/// Emit an `(ok, data)` tuple where `data` is the result of routing a
/// precompile-address staticcall to its CryptoLib / StdLib native
/// equivalent. Covers 0x02 sha256, 0x03 ripemd160, 0x04 identity (Task #H5),
/// 0x01 ecrecover (Task #H6a), and 0x05 modexp (Task #H6b, 1-byte-operand
/// subset). Returns `true` if the index was handled and `false` if lowering
/// the payload failed. Unsupported precompiles must be rejected by the caller
/// before this helper is reached.
pub(crate) fn emit_precompile_staticcall(
    index: u8,
    payload: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    // Task #H6a / #H6b: 0x01 ecrecover and 0x05 modexp route to dedicated
    // bytecode builtins that decode the ABI payload inline (SUBSTR-based).
    // Emit the `(true, result32)` tuple directly here and short-circuit.
    if matches!(index, 0x01 | 0x05) {
        let builtin = match index {
            0x01 => BuiltinCall::PrecompileEcrecover,
            _ => BuiltinCall::PrecompileModexp,
        };
        let tuple_local = ctx.allocate_local("__call_tuple".to_string(), None);
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(2u8),
        )));
        instructions.push(Instruction::NewArray {
            element_type: ValueType::Any,
        });
        instructions.push(Instruction::StoreLocal(tuple_local));

        let data_local = ctx.allocate_local("__call_data".to_string(), None);
        if !lower_expression(payload, ctx, instructions) {
            return Some(false);
        }
        instructions.push(Instruction::CallBuiltin {
            builtin,
            arg_count: 1,
        });
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

        instructions.push(Instruction::LoadLocal(tuple_local));
        return Some(true);
    }

    // Resolve the native method first so unsupported indices skip local
    // allocation / side-effect emission entirely.
    let (contract, method) = match index {
        0x02 => (NativeContract::CryptoLib, "sha256".to_string()),
        0x03 => (NativeContract::CryptoLib, "ripemd160".to_string()),
        0x04 => {
            // Identity: out = input. Short-circuit without invoking a native.
            let tuple_local = ctx.allocate_local("__call_tuple".to_string(), None);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(2u8),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            instructions.push(Instruction::StoreLocal(tuple_local));
            let data_local = ctx.allocate_local("__call_data".to_string(), None);
            if !lower_expression(payload, ctx, instructions) {
                return Some(false);
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
            instructions.push(Instruction::LoadLocal(tuple_local));
            return Some(true);
        }
        _ => return None,
    };

    // Build `(ok, data)` tuple and emit: data = CryptoLib.<method>(payload).
    let tuple_local = ctx.allocate_local("__call_tuple".to_string(), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(tuple_local));

    let data_local = ctx.allocate_local("__call_data".to_string(), None);
    if !lower_expression(payload, ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall { contract, method },
        arg_count: 1,
    });
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

    instructions.push(Instruction::LoadLocal(tuple_local));
    Some(true)
}

pub(crate) fn emit_reencoded_low_level_return_data(
    data_local: usize,
    return_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    instructions.push(Instruction::LoadLocal(data_local));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::StdLib,
            method: "deserialize".to_string(),
        },
        arg_count: 1,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::ArrayGet);
    if emit_abi_encode_single_stack_value_for_type(return_type, ctx, instructions).is_some() {
        instructions.push(Instruction::StoreLocal(data_local));
    } else {
        instructions.push(Instruction::Drop(ValueType::Any));
    }
}
