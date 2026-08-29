use super::*;

pub(crate) fn value_type_to_catch_guard(value_type: &ValueType) -> Option<ConvertTarget> {
    match value_type {
        ValueType::Any => None,
        ValueType::Boolean => Some(ConvertTarget::Boolean),
        ValueType::Integer { .. } => Some(ConvertTarget::Integer),
        ValueType::String | ValueType::Address | ValueType::ByteArray { .. } => {
            Some(ConvertTarget::ByteArray)
        }
        ValueType::Array(_) | ValueType::Struct { .. } => Some(ConvertTarget::Array),
        ValueType::Mapping { .. } => Some(ConvertTarget::Map),
    }
}

pub(crate) fn catch_clause_param(
    clause: &solang_parser::pt::CatchClause,
) -> Option<&solang_parser::pt::Parameter> {
    match clause {
        solang_parser::pt::CatchClause::Simple(_, param, _) => param.as_ref(),
        solang_parser::pt::CatchClause::Named(_, _, param, _) => Some(param),
    }
}

pub(crate) fn catch_clause_statement(clause: &solang_parser::pt::CatchClause) -> &Statement {
    match clause {
        solang_parser::pt::CatchClause::Simple(_, _, stmt) => stmt,
        solang_parser::pt::CatchClause::Named(_, _, _, stmt) => stmt,
    }
}

pub(crate) fn is_bare_catch_clause(clause: &solang_parser::pt::CatchClause) -> bool {
    matches!(clause, solang_parser::pt::CatchClause::Simple(_, None, _))
}

pub(crate) fn catch_clause_guard_target(
    clause: &solang_parser::pt::CatchClause,
    ctx: &mut LoweringContext,
) -> Option<ConvertTarget> {
    if let Some(parameter) = catch_clause_param(clause) {
        return infer_type_from_expression(&parameter.ty, ctx)
            .as_ref()
            .and_then(value_type_to_catch_guard);
    }

    if let solang_parser::pt::CatchClause::Named(_, ident, _, _) = clause {
        if ident.name == "Panic" {
            return Some(ConvertTarget::Integer);
        }
    }

    None
}

/// Task #103 — Classify a `catch` clause so the dispatcher can emit the
/// correct guard. EVM-style revert envelopes carry a 4-byte selector
/// prefix (`keccak256("Panic(uint256)")[..4] = 0x4e487b71`,
/// `keccak256("Error(string)")[..4] = 0x08c379a0`) followed by
/// `abi.encode(args)`. We match these known shapes structurally so
/// `catch Panic(uint code)` and `catch Error(string msg)` can decode
/// their payloads directly. Named clauses referencing user-defined
/// errors fall into `UserNamed` and use the legacy (permissive) path.
pub(crate) enum CatchClauseKind {
    /// `catch Panic(uint code)` — match 4-byte selector `0x4e487b71`,
    /// decode code from `abi.encode(uint256)`.
    Panic,
    /// `catch Error(string msg)` — match 4-byte selector `0x08c379a0`,
    /// decode msg from `abi.encode(string)`.
    Error,
    /// `catch (bytes memory reason)` — always match, bind the raw
    /// envelope bytes verbatim.
    Bytes,
    /// Legacy / user-defined named error. Emit a permissive type guard
    /// (ISTYPE ByteArray, which is what the runtime pushes) and bind
    /// the raw payload.
    UserNamed,
    /// Simple catch with a non-bytes parameter (e.g. `catch (uint256 x)`).
    /// Emits the legacy ISTYPE guard on the inferred type. Rare in
    /// practice — Solidity's grammar only accepts `bytes memory` here.
    SimpleTyped,
}

pub(crate) fn classify_catch_clause(
    clause: &solang_parser::pt::CatchClause,
    ctx: &mut LoweringContext,
) -> CatchClauseKind {
    match clause {
        solang_parser::pt::CatchClause::Named(_, ident, param, _) => {
            match ident.name.as_str() {
                "Panic" => {
                    // Spec: single `uint256 code` parameter.
                    if let Some(ty) = infer_type_from_expression(&param.ty, ctx) {
                        if matches!(ty, ValueType::Integer { .. }) {
                            return CatchClauseKind::Panic;
                        }
                    }
                    CatchClauseKind::Panic
                }
                "Error" => {
                    // Spec: single `string` parameter.
                    if let Some(ty) = infer_type_from_expression(&param.ty, ctx) {
                        if matches!(ty, ValueType::String | ValueType::ByteArray { .. }) {
                            return CatchClauseKind::Error;
                        }
                    }
                    CatchClauseKind::Error
                }
                _ => CatchClauseKind::UserNamed,
            }
        }
        solang_parser::pt::CatchClause::Simple(_, Some(param), _) => {
            match infer_type_from_expression(&param.ty, ctx) {
                Some(ValueType::ByteArray { .. }) | Some(ValueType::String) => {
                    CatchClauseKind::Bytes
                }
                _ => CatchClauseKind::SimpleTyped,
            }
        }
        // Bare `catch { }` — handled separately as the fallback.
        solang_parser::pt::CatchClause::Simple(_, None, _) => CatchClauseKind::Bytes,
    }
}

/// Compute the 4-byte keccak selector for an EVM-style revert shape.
pub(crate) fn revert_envelope_selector(signature: &[u8]) -> [u8; 4] {
    let mut hasher = Keccak256::new();
    hasher.update(signature);
    let digest = hasher.finalize();
    [digest[0], digest[1], digest[2], digest[3]]
}

/// Emit a guard that jumps to `fail_label` unless the payload in
/// `catch_local` is a ByteArray whose first 4 bytes match `selector`.
/// Leaves the stack empty on fall-through (match) and on jump (mismatch).
pub(crate) fn emit_selector_guard(
    catch_local: usize,
    selector: [u8; 4],
    min_len: u64,
    fail_label: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    // 1. Payload must be a ByteArray (runtime always pushes ByteArray per
    //    src/runtime/execution/instruction/flow/try_frames.rs — but a
    //    hand-rolled `throw 42;` could push an Integer, so we guard).
    //    IsType pushes true when matching. JumpIf branches when the popped
    //    value is false, so we negate via a NOT using the PUSHF+EQUAL combo
    //    isn't available — instead we invert by jumping on the match and
    //    falling through when it fails. Strategy: push IsType, then use a
    //    match_label to keep control flow explicit.
    let match_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(catch_local));
    instructions.push(Instruction::IsType {
        target: ConvertTarget::ByteArray,
    });
    // JumpIf branches when popped value is FALSE. If IsType returned false
    // (payload is not a ByteArray), jump to fail_label. We want to branch
    // to fail when NOT ByteArray, so: PUSH IsType result; JumpIf-false →
    // fail_label; fall through when ByteArray. But IR JumpIf = JMPIFNOT,
    // so it already branches on false, exactly what we want.
    instructions.push(Instruction::JumpIf { target: fail_label });

    // 2. Payload length must be >= min_len (4 for selector-only, 36 for
    //    selector + 32-byte arg, etc). GetSize pushes the byte length.
    instructions.push(Instruction::LoadLocal(catch_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(min_len),
    )));
    // Comparator: size >= min_len. Use Lt(size, min_len) then JumpIf-false
    // to fall through when NOT less (i.e. size >= min_len). Inverting: we
    // need to JumpIf to fail when size < min_len. So compute `size < min`
    // and JumpIf-false (branches on not-less) jumps to match_label.
    // Simpler: compute `size < min`, JumpIf jumps on false (i.e. safe —
    // size is not less than min) to match_label; fall through to
    // fail_label when size < min (JumpIf does not branch).
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf {
        target: match_label,
    });
    instructions.push(Instruction::Jump { target: fail_label });
    instructions.push(Instruction::Label(match_label));

    // 3. Extract the first 4 bytes and compare to `selector`. SUBSTR
    //    consumes [bytes, index, count] and pushes the substring.
    instructions.push(Instruction::LoadLocal(catch_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(4u8),
    )));
    instructions.push(Instruction::Substr);
    // `SUBSTR` yields a Buffer; `EQUAL` against the ByteString selector below
    // would be a REFERENCE comparison on a real Neo node (Buffer is not a
    // primitive), always false — so `catch Panic`/`catch Error` would never
    // match on-chain. Normalize the slice to a ByteString for value equality.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        selector.to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    // JumpIf branches when comparison is false (selector mismatch) — fall
    // through on match.
    instructions.push(Instruction::JumpIf { target: fail_label });
}

pub(crate) fn emit_decode_be_u256_low_u64(
    source_local: usize,
    slot_offset: u64,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let tmp = ctx.allocate_local("__catch_u256_low_u64".to_string(), None);
    instructions.push(Instruction::LoadLocal(source_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(slot_offset + 24),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(8u8),
    )));
    instructions.push(Instruction::Substr);
    instructions.push(Instruction::StoreLocal(tmp));
    instructions.push(Instruction::LoadLocal(tmp));
    instructions.push(Instruction::LoadLocal(tmp));
    instructions.push(Instruction::ReverseItems);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
}

/// Bind the `uint code` parameter for a `catch Panic(uint code)` clause.
/// The payload is `0x4e487b71 || abi.encode(uint256 code)`; extract bytes
/// [4..36] via SUBSTR and Convert to Integer.
pub(crate) fn bind_panic_code_parameter(
    clause: &solang_parser::pt::CatchClause,
    catch_local: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let Some(parameter) = catch_clause_param(clause) else {
        return;
    };
    let Some(name) = parameter.name.as_ref().map(|id| id.name.clone()) else {
        return;
    };

    let inferred = infer_type_from_expression(&parameter.ty, ctx).unwrap_or(ValueType::Integer {
        signed: false,
        bits: 256,
    });
    let slot = ctx.allocate_local(name, Some(inferred));

    // Decode the low u64 directly from the big-endian ABI word. Do not route
    // through the AbiDecode fallback: on production NeoVM that maps to
    // StdLib.deserialize, which expects Neo JSON serialization rather than raw
    // EVM ABI bytes.
    emit_decode_be_u256_low_u64(catch_local, 4, ctx, instructions);
    instructions.push(Instruction::StoreLocal(slot));
}

/// Bind the `string msg` parameter for a `catch Error(string)` clause.
/// The payload is `0x08c379a0 || abi.encode(string msg)`. `abi.encode`
/// for a single `string` argument produces:
///   0x20 (offset, 32 bytes BE) || length (32 bytes BE) || string bytes
///   (padded to 32-byte boundary)
/// Decode: read length from bytes [36..68], then extract bytes
/// [68..68+length] as the raw string.
pub(crate) fn bind_error_message_parameter(
    clause: &solang_parser::pt::CatchClause,
    catch_local: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let Some(parameter) = catch_clause_param(clause) else {
        return;
    };
    let Some(name) = parameter.name.as_ref().map(|id| id.name.clone()) else {
        return;
    };

    let inferred = infer_type_from_expression(&parameter.ty, ctx).unwrap_or(ValueType::String);
    let slot = ctx.allocate_local(name, Some(inferred));

    // Read string length from bytes [36..68] (BE uint256). Offset 68 in
    // the payload is where the actual string bytes start (4-byte selector
    // + 32-byte head offset + 32-byte length = 68). Decode the low u64
    // directly from raw ABI bytes for the same reason as Panic(uint).
    let len_tmp = ctx.allocate_local("__catch_err_len".to_string(), None);
    emit_decode_be_u256_low_u64(catch_local, 36, ctx, instructions);
    instructions.push(Instruction::StoreLocal(len_tmp));

    // SUBSTR(payload, 68, len_tmp) → raw string bytes.
    instructions.push(Instruction::LoadLocal(catch_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(68u8),
    )));
    instructions.push(Instruction::LoadLocal(len_tmp));
    instructions.push(Instruction::Substr);
    instructions.push(Instruction::StoreLocal(slot));
}

pub(crate) fn bind_catch_clause_parameter(
    clause: &solang_parser::pt::CatchClause,
    catch_local: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let Some(parameter) = catch_clause_param(clause) else {
        return;
    };

    let Some(name) = parameter.name.as_ref().map(|id| id.name.clone()) else {
        return;
    };

    let inferred = infer_type_from_expression(&parameter.ty, ctx).unwrap_or(ValueType::Any);
    let slot = ctx.allocate_local(name, Some(inferred));
    instructions.push(Instruction::LoadLocal(catch_local));
    instructions.push(Instruction::StoreLocal(slot));
}

pub(crate) fn lower_try_statement(
    expr: &Expression,
    handler: &Option<(solang_parser::pt::ParameterList, Box<Statement>)>,
    catches: &[solang_parser::pt::CatchClause],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Solidity try/catch is only defined for external calls and contract creation.
    // NeoVM supports structured exception handling via TRY/ENDTRY/ENDFINALLY, so we can
    // map it directly.

    let catch_label = ctx.next_label();
    let success_label = ctx.next_label();
    let end_label = ctx.next_label();

    let (call_expr, inline_success) = match expr {
        Expression::FunctionCallBlock(_, call, block) => (call.as_ref(), Some(block.as_ref())),
        _ => (expr, None),
    };

    let (handler_params, handler_stmt) = handler
        .as_ref()
        .map(|(params, stmt)| (params.as_slice(), Some(stmt.as_ref())))
        .unwrap_or((&[][..], None));

    let success_stmt = handler_stmt.or(inline_success);

    if catches.is_empty() {
        ctx.record_error_with_suggestion(
            "try statement without catch clause is not supported",
            "add a catch clause: try expr { ... } catch { ... }",
        );
        if lower_expression(call_expr, ctx, instructions) {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
        if let Some(success_stmt) = success_stmt {
            let _ = lower_statement(success_stmt, ctx, instructions);
        }
        return false;
    }

    // Lower the call expression inside a TRY, then ENDTRY to the success label.
    instructions.push(Instruction::Try {
        catch_target: catch_label,
    });

    let mut try_return_slots: Vec<(usize, ValueType)> = Vec::new();
    if lower_expression(call_expr, ctx, instructions) {
        if handler_params.len() == 1 {
            if let Some(param) = handler_params[0].1.as_ref() {
                let inferred_type =
                    infer_type_from_expression(&param.ty, ctx).unwrap_or(ValueType::Any);
                let tmp = ctx.allocate_local("__try_ret".to_string(), Some(inferred_type.clone()));
                instructions.push(Instruction::StoreLocal(tmp));
                try_return_slots.push((tmp, inferred_type));
            } else {
                ctx.record_error_with_suggestion(
                    "try returns(...) parameter is missing",
                    "specify a return parameter: try func() returns (uint256 result) { ... }",
                );
                instructions.push(Instruction::Drop(ValueType::Any));
            }
        } else if handler_params.len() > 1 {
            let array_tmp = ctx.allocate_local(
                "__try_ret_array".to_string(),
                Some(ValueType::Array(Box::new(ValueType::Any))),
            );
            instructions.push(Instruction::StoreLocal(array_tmp));

            for (i, param_tuple) in handler_params.iter().enumerate() {
                if let Some(param) = param_tuple.1.as_ref() {
                    let inferred_type =
                        infer_type_from_expression(&param.ty, ctx).unwrap_or(ValueType::Any);
                    let tmp =
                        ctx.allocate_local(format!("__try_ret_{i}"), Some(inferred_type.clone()));

                    instructions.push(Instruction::LoadLocal(array_tmp));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(i),
                    )));
                    instructions.push(Instruction::ArrayGet);
                    instructions.push(Instruction::StoreLocal(tmp));

                    try_return_slots.push((tmp, inferred_type));
                }
            }
        } else {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
    } else if !handler_params.is_empty() {
        ctx.record_error("try returns(...) expects a return value");
    }

    instructions.push(Instruction::EndTry {
        target: success_label,
    });

    // Catch handler: bind or drop the thrown value, then execute the matching catch body.
    instructions.push(Instruction::Label(catch_label));
    ctx.enter_scope();
    // The TRY frame is live (state = Catch) throughout the catch bodies, so a
    // `return` inside any of them must pop it (one ENDTRY) before the RET. The
    // success handler runs after the success-path ENDTRY (frame already popped),
    // so it is intentionally left out of this region.
    ctx.enter_catch_frame();

    if catches.len() == 1 && is_bare_catch_clause(&catches[0]) {
        // Preserve existing compact lowering for a lone `catch { ... }`.
        instructions.push(Instruction::Drop(ValueType::Any));
        let _ = lower_statement(catch_clause_statement(&catches[0]), ctx, instructions);
    } else {
        let catch_local = ctx.allocate_local("__catch_exception".to_string(), None);
        instructions.push(Instruction::StoreLocal(catch_local));

        let mut fallback_clause: Option<&solang_parser::pt::CatchClause> = None;

        for clause in catches {
            if is_bare_catch_clause(clause) {
                if fallback_clause.is_none() {
                    fallback_clause = Some(clause);
                }
                continue;
            }

            let next_clause_label = ctx.next_label();
            let kind = classify_catch_clause(clause, ctx);

            // Task #103 — Each catch kind uses a shape-specific guard:
            //   * `Panic(uint)` / `Error(string)` match the 4-byte EVM
            //     selector prefix so `catch Panic(uint code)` sees
            //     `code == 0x12` on div-by-zero (not the stringified
            //     `"Panic: 0x12"` byte-prefix that the ISTYPE Integer
            //     guard used to miss entirely).
            //   * `catch (bytes memory)` matches any payload (it binds
            //     the raw envelope verbatim for low-level decoding).
            //   * Legacy user-named and simple-typed clauses retain the
            //     ISTYPE guard from the pre-Task-#103 lowering so
            //     existing custom-error tests keep passing.
            match kind {
                CatchClauseKind::Panic => {
                    emit_selector_guard(
                        catch_local,
                        revert_envelope_selector(b"Panic(uint256)"),
                        36,
                        next_clause_label,
                        ctx,
                        instructions,
                    );
                    ctx.enter_scope();
                    bind_panic_code_parameter(clause, catch_local, ctx, instructions);
                    let _ = lower_statement(catch_clause_statement(clause), ctx, instructions);
                    ctx.exit_scope();
                }
                CatchClauseKind::Error => {
                    emit_selector_guard(
                        catch_local,
                        revert_envelope_selector(b"Error(string)"),
                        68,
                        next_clause_label,
                        ctx,
                        instructions,
                    );
                    ctx.enter_scope();
                    bind_error_message_parameter(clause, catch_local, ctx, instructions);
                    let _ = lower_statement(catch_clause_statement(clause), ctx, instructions);
                    ctx.exit_scope();
                }
                CatchClauseKind::Bytes => {
                    // `catch (bytes memory data)` — always matches. Bind
                    // the raw envelope.
                    ctx.enter_scope();
                    bind_catch_clause_parameter(clause, catch_local, ctx, instructions);
                    let _ = lower_statement(catch_clause_statement(clause), ctx, instructions);
                    ctx.exit_scope();
                }
                CatchClauseKind::UserNamed | CatchClauseKind::SimpleTyped => {
                    // Legacy path — ISTYPE guard on the inferred parameter
                    // type. The payload is always a ByteArray, so named
                    // catches for user-defined errors just bind the raw
                    // bytes.
                    if let Some(guard_target) = catch_clause_guard_target(clause, ctx) {
                        instructions.push(Instruction::LoadLocal(catch_local));
                        instructions.push(Instruction::IsType {
                            target: guard_target,
                        });
                        instructions.push(Instruction::JumpIf {
                            target: next_clause_label,
                        });
                    }
                    ctx.enter_scope();
                    bind_catch_clause_parameter(clause, catch_local, ctx, instructions);
                    let _ = lower_statement(catch_clause_statement(clause), ctx, instructions);
                    ctx.exit_scope();
                }
            }

            // A MATCHED non-fallback catch must exit via ENDTRY, not a plain
            // JMP: ENDTRY is the only op that pops the NeoVM TRY frame (no
            // finally in Solidity, so it pops + jumps to `end_label`). A bare
            // `Jump` here would branch PAST the lone `EndTry` below, leaving the
            // frame on the try-stack — which the reference C# NeoVM faults on at
            // RET ("There is still try in the stack"). The fallback/rethrow path
            // still reaches the `EndTry` at the end of the catch region.
            instructions.push(Instruction::EndTry { target: end_label });
            instructions.push(Instruction::Label(next_clause_label));
        }

        if let Some(clause) = fallback_clause {
            ctx.enter_scope();
            let _ = lower_statement(catch_clause_statement(clause), ctx, instructions);
            ctx.exit_scope();
        } else {
            // No fallback clause and no type guard matched: rethrow the original exception.
            instructions.push(Instruction::LoadLocal(catch_local));
            instructions.push(Instruction::Throw);
        }
    }

    ctx.exit_catch_frame();
    ctx.exit_scope();
    instructions.push(Instruction::EndTry { target: end_label });

    // Success handler: bind return values (if present) and run handler block.
    instructions.push(Instruction::Label(success_label));
    if let Some(success_stmt) = success_stmt {
        ctx.enter_scope();

        for (i, param_tuple) in handler_params.iter().enumerate() {
            if let Some(param) = param_tuple.1.as_ref() {
                if let Some(name) = param.name.as_ref().map(|id| id.name.clone()) {
                    let (tmp, inferred) = try_return_slots
                        .get(i)
                        .cloned()
                        .unwrap_or((0, ValueType::Any));
                    let slot = ctx.allocate_local(name, Some(inferred.clone()));
                    if try_return_slots.get(i).is_some() {
                        instructions.push(Instruction::LoadLocal(tmp));
                        instructions.push(Instruction::StoreLocal(slot));
                    } else {
                        push_default_for_value_type(&inferred, ctx, instructions);
                        instructions.push(Instruction::StoreLocal(slot));
                    }
                }
            }
        }

        let _ = lower_statement(success_stmt, ctx, instructions);
        ctx.exit_scope();
    }

    instructions.push(Instruction::Label(end_label));
    false
}
