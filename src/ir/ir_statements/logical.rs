use super::*;

pub(crate) fn lower_require(
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    if args.is_empty() {
        ctx.record_error_with_suggestion(
            "require() expects at least one argument",
            "usage: require(condition) or require(condition, \"error message\")",
        );
        return;
    }

    let fail_label = ctx.next_label();
    let ok_label = ctx.next_label();

    // IR JumpIf branches when the condition is false.
    if lower_expression(&args[0], ctx, instructions) {
        instructions.push(Instruction::JumpIf { target: fail_label });
        instructions.push(Instruction::Jump { target: ok_label });
    }

    instructions.push(Instruction::Label(fail_label));
    if args.len() > 1 {
        // Solidity 0.8.x supports three forms:
        //   require(cond)              -> THROW null
        //   require(cond, "message")   -> THROW keccak256("Error(string)")[..4] || abi.encode("message")
        //   require(cond, CustomError(args)) -> THROW keccak256("CustomError(types)")[..4] || abi.encode(args)
        //
        // For the custom error form, the second argument is a FunctionCall whose callee is
        // a Variable naming the error type. We emit the EVM-canonical custom-error envelope
        // so `catch <Name>(...)` / `catch (bytes memory)` selector guards can match and decode
        // the payload verbatim (Task #131 — aligns `require` with `revert` payload shape so
        // `catch Error(string memory r)` absorbs `require(cond, "r")` uniformly across
        // cross-contract and self-call paths).
        if let Expression::FunctionCall(_, callee, error_args) = &args[1] {
            if let Expression::Variable(error_ident) = callee.as_ref() {
                // Resolve the selector from the DECLARED `error` signature
                // when available (same policy as `lower_revert_statement`).
                let arg_types = revert_error_arg_types(&error_ident.name, error_args, ctx);
                let selector = revert_error_selector(&error_ident.name, &arg_types);

                let direct_static_path = error_args
                    .iter()
                    .all(|arg| is_direct_static_revert_arg(arg, ctx));
                if direct_static_path {
                    let pre_len = instructions.len();
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        selector.to_vec(),
                    )));
                    let mut success = true;
                    for error_arg in error_args {
                        if !lower_direct_static_revert_arg_slot(error_arg, ctx, instructions) {
                            success = false;
                            break;
                        }
                    }
                    if success {
                        if !error_args.is_empty() {
                            instructions.push(Instruction::CallBuiltin {
                                builtin: BuiltinCall::BytesConcat,
                                arg_count: error_args.len() + 1,
                            });
                        }
                        instructions.push(Instruction::Throw);
                        instructions.push(Instruction::Label(ok_label));
                        return;
                    }
                    instructions.truncate(pre_len);
                }

                let pre_len = instructions.len();
                instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                    selector.to_vec(),
                )));
                if error_args.is_empty() {
                    instructions.push(Instruction::Throw);
                    instructions.push(Instruction::Label(ok_label));
                    return;
                }
                if let Some(ok) =
                    lower_abi_encode_args_direct_from_slice(error_args, ctx, instructions)
                {
                    if ok {
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::BytesConcat,
                            arg_count: 2,
                        });
                        instructions.push(Instruction::Throw);
                        instructions.push(Instruction::Label(ok_label));
                        return;
                    }
                    instructions.truncate(pre_len);
                } else {
                    instructions.truncate(pre_len);
                }

                let pre_len = instructions.len();
                let mut pushed = 0usize;
                let mut success = true;
                for error_arg in error_args {
                    if lower_expression(error_arg, ctx, instructions) {
                        pushed += 1;
                    } else {
                        success = false;
                        break;
                    }
                }

                if success && pushed == error_args.len() {
                    // Rewind the argument pushes so we can emit selector, then args, in
                    // the correct order for AbiEncode + BytesConcat (mirrors
                    // `lower_revert_statement` shape).
                    let mut arg_instrs = instructions.split_off(pre_len);
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        selector.to_vec(),
                    )));
                    instructions.append(&mut arg_instrs);
                    if pushed == 0 {
                        // `require(cond, CustomError())` — payload is just the 4-byte selector.
                    } else {
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::AbiEncode,
                            arg_count: pushed,
                        });
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::BytesConcat,
                            arg_count: 2,
                        });
                    }
                    instructions.push(Instruction::Throw);
                    instructions.push(Instruction::Label(ok_label));
                    return;
                }

                // Fallback on lowering failure: legacy error-name string so we still surface
                // *something* on the error path.
                instructions.truncate(pre_len);
                let msg = if error_args.is_empty() {
                    error_ident.name.clone()
                } else {
                    format!("{}({} args)", error_ident.name, error_args.len())
                };
                instructions.push(Instruction::PushLiteral(LiteralValue::String(
                    msg.as_bytes().to_vec(),
                )));
                instructions.push(Instruction::Throw);
                instructions.push(Instruction::Label(ok_label));
                return;
            }
        }

        // Task #131 — `require(cond, "msg")` with a string literal emits the
        // EVM-canonical `Error(string)` envelope:
        //   keccak256("Error(string)")[..4] || abi.encode(msg)
        // matching the shape `revert("msg")` already produces (see
        // `lower_revert_statement`). Without this alignment, a `catch Error(string)`
        // clause's 4-byte selector guard misses the bare `"msg"` payload and
        // falls through to a rethrow — breaking `try this.f()` for any callee
        // whose revert path terminates in `require(_, "msg")` (batch60 JJ3).
        if let Expression::StringLiteral(parts) = &args[1] {
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                error_string_literal_envelope(&string_literal_bytes(parts)),
            )));
            instructions.push(Instruction::Throw);
            instructions.push(Instruction::Label(ok_label));
            return;
        }

        // Task #131 follow-up — NON-LITERAL string messages (a string
        // variable, `string.concat(...)`, a propagated constant, ...) must
        // get the same `Error(string)` envelope as the literal case above
        // and as `revert(msg)`. Previously they fell through to the bare
        // `Throw` below, so `catch Error(string)` selector guards missed
        // them and EVM tooling could not decode the revert reason. The
        // helper is shared with `lower_revert_statement` to keep both
        // payload shapes identical.
        if revert_message_arg_is_string(&args[1], ctx)
            && emit_error_string_envelope_throw(&args[1], ctx, instructions)
        {
            instructions.push(Instruction::Label(ok_label));
            return;
        }

        // Preserve diagnostics/type checking for the revert message expression and surface it
        // in the VM fault state when possible (NeoVM THROW).
        if lower_expression(&args[1], ctx, instructions) {
            instructions.push(Instruction::Throw);
            instructions.push(Instruction::Label(ok_label));
            return;
        }
    }

    // NeoVM THROW requires an exception value on the stack. `null` yields an empty message.
    instructions.push(Instruction::PushLiteral(LiteralValue::Null));
    instructions.push(Instruction::Throw);
    instructions.push(Instruction::Label(ok_label));
}

pub(crate) fn lower_assert(
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    if args.len() != 1 {
        ctx.record_error_with_suggestion(
            "assert() expects exactly one argument",
            "usage: assert(condition)",
        );
        return;
    }

    let fail_label = ctx.next_label();
    let ok_label = ctx.next_label();

    // IR JumpIf branches when the condition is false.
    if lower_expression(&args[0], ctx, instructions) {
        instructions.push(Instruction::JumpIf { target: fail_label });
        instructions.push(Instruction::Jump { target: ok_label });
    }

    instructions.push(Instruction::Label(fail_label));
    // Task #27 (compiler slice) / Task #107 — Solidity `assert(false)`
    // compiles to an EVM Panic with code 0x01 (assertion failed). Route
    // through the shared `emit_panic` helper which emits the canonical
    //   keccak256("Panic(uint256)")[0..4] || abi.encode(0x01)
    // payload so `ExecutionResult.return_data` matches what Ethereum
    // tooling expects and `try { ... } catch Panic(uint code)` clauses can
    // decode the code verbatim (per try_catch.rs).
    emit_panic(0x01, instructions);

    instructions.push(Instruction::Label(ok_label));
}

pub(crate) fn lower_logical_or(
    left: &Expression,
    right: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let false_label = ctx.next_label();
    let end_label = ctx.next_label();

    if !lower_expression(left, ctx, instructions) {
        return false;
    }

    instructions.push(Instruction::JumpIf {
        target: false_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(false_label));

    if !lower_expression(right, ctx, instructions) {
        return false;
    }

    // M-IR2: normalize the right operand to a canonical Boolean so a non-bool
    // value (e.g. injected by inline assembly or a frontend type-inference
    // miss) cannot leak through the short-circuit as the raw operand. Solidity
    // guarantees `bool` operands, so this CONVERT (0xDB 0x20) is a no-op for
    // well-typed input and a defensive guard for the rest — matching NeoVM's
    // own truthiness rule used by the JMPIF that drives the short-circuit.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Boolean,
    });

    instructions.push(Instruction::Label(end_label));
    true
}

pub(crate) fn lower_logical_and(
    left: &Expression,
    right: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let false_label = ctx.next_label();
    let end_label = ctx.next_label();

    if !lower_expression(left, ctx, instructions) {
        return false;
    }

    instructions.push(Instruction::JumpIf {
        target: false_label,
    });

    if !lower_expression(right, ctx, instructions) {
        return false;
    }

    // M-IR2: bool-normalize the right operand (see lower_logical_or).
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Boolean,
    });

    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(false_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
    instructions.push(Instruction::Label(end_label));
    true
}
