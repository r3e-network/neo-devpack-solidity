use super::*;

pub(crate) fn lower_variable_expression(
    identifier: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if identifier.name == "this" {
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::Syscall("System.Runtime.GetExecutingScriptHash".to_string()),
            arg_count: 0,
        });
        return true;
    }
    if identifier.name == "block" || identifier.name == "msg" {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        return true;
    }
    if identifier.name == "super" {
        // `super` is only meaningful as `super.method()`. When used as a bare
        // identifier (e.g., assigned to a variable), emit a targeted diagnostic
        // instead of the generic "unknown identifier" error.
        ctx.record_error_with_suggestion(
            "the 'super' keyword can only be used in member-call expressions (super.method())",
            "use super.methodName() to call a parent contract's function, or inline the parent logic directly",
        );
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        return false;
    }
    if let Some(alias) = ctx.storage_alias(&identifier.name).cloned() {
        return emit_storage_load(&alias, ctx, instructions);
    }
    if let Some(index) = ctx.param_index_map.get(&identifier.name) {
        instructions.push(Instruction::LoadParameter(*index));
        true
    } else if let Some(index) = ctx.resolve_local(&identifier.name) {
        instructions.push(Instruction::LoadLocal(index));
        true
    } else if let Some(index) = ctx.state_index_map.get(&identifier.name) {
        let state_index = *index;
        let state_type = ctx.state_type(state_index).cloned();
        let const_initializer = ctx.state_metadata(*index).and_then(|meta| {
            if meta.is_constant {
                meta.initializer.clone()
            } else {
                None
            }
        });
        if let Some(initializer) = const_initializer {
            // Preserve hash160 constants as address literals so downstream manifest
            // permission inference can recover exact contract hashes for
            // `Syscalls.contractCall(CONSTANT_HASH, ...)` patterns.
            if matches!(state_type.as_ref(), Some(ValueType::Address)) {
                if let Some(bytes) = address_bytes_le_from_expression(&initializer) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                    return true;
                }
            }

            // Canonicalize a `bytesN` constant whose initializer is an
            // integer-backed literal (e.g. `bytes2 constant B = 0x1234`) to its
            // big-endian ByteString, so `uint16(B)` / byte indexing read the
            // face value — not the byte-reversed little-endian Integer the raw
            // hex literal would lower to (same class as the scalar / struct /
            // array / mapping-key bytesN canonicalization fixes).
            if let Some(ty @ ValueType::ByteArray { fixed_len: Some(_) }) = state_type.as_ref() {
                if super::dispatch::try_lower_bytesn_literal_canonical(
                    &initializer,
                    ty,
                    ctx,
                    instructions,
                ) {
                    return true;
                }
            }

            return lower_expression(&initializer, ctx, instructions);
        }

        // Struct-typed state variables are stored field-by-field in derived slots. Loading the
        // base slot does not materialize the struct value, so build it by reading each field.
        if let Some(vt @ ValueType::Struct { .. }) = state_type {
            let reference = StorageReference {
                state_index,
                key_expressions: Vec::new(),
                key_types: Vec::new(),
                value_type: vt,
                field_path: Vec::new(),
                trailing_key_expressions: Vec::new(),
                trailing_key_types: Vec::new(),
            };
            return emit_storage_load(&reference, ctx, instructions);
        }

        instructions.push(Instruction::LoadState(state_index));
        true
    } else if ctx.is_contract_type_name(&identifier.name) {
        // Contract/interface/library type names can appear as namespaces in
        // member expressions (e.g. `Enum.Operation.Call`). As a bare value they
        // are not runtime variables; emit a neutral placeholder instead of a
        // hard unknown-identifier error.
        instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
            0u8;
            20
        ])));
        true
    } else if ctx.function_names.contains(&identifier.name) {
        // Task #186 — a bare reference to an internal function name in value
        // position (e.g. passed as an argument of function-pointer type such as
        // `apply_(add, 2, 3)`) materializes the function's bytecode offset as a
        // PUSHINT32 literal. The bytecode emitter patches the placeholder with
        // `method.offset` once all methods have been laid out; the runtime
        // later consumes it via `CALLA` through a `CallIndirect` instruction.
        //
        // We resolve the overload-neutral `neo_name` for the zero-arg overload
        // key first; if that fails we fall back to the raw identifier, which
        // matches the `neo_name` used as a CALL_L target for same-name,
        // same-arity overloads in `try_lower_variable_call`.
        let neo_name = ctx
            .function_overloads
            .iter()
            .find_map(|((n, _), bucket)| {
                if n == &identifier.name {
                    bucket.first().map(|(_, neo_name)| neo_name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| identifier.name.clone());
        instructions.push(Instruction::PushFunctionOffset { name: neo_name });
        true
    } else {
        // Compatibility fallback for unresolved constants/symbols from foreign
        // environments (e.g., chain-specific helper contracts).
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        true
    }
}
