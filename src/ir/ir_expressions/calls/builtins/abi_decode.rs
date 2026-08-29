//! ## ABI Decoding Lowering
//!
//! Lowers Solidity `abi.decode()` builtin calls into NeoVM instructions.
//! Decodes an EVM-canonical head+tail byte buffer back into typed values
//! matching the caller's declared return types.
//!
//! The helpers for slot reading, dynamic-tail decoding, and static-slot
//! decoding have been extracted into sibling modules to keep this
//! orchestration module under the 800-line limit.
//!
//! ### Dispatch flow
//!
//! 1. Direct decode → `lower_abi_decode_direct` (static-only or single-return)
//! 2. Dynamic top-level → `emit_abi_decode_dynamic_top_level` (strings, bytes, arrays)
//! 3. Per-element read → `emit_abi_decode_u256_at`, `emit_abi_decode_static_slot_at_runtime_offset`
//! 4. Value-type conversion → `abi_decode_value_types`
//!
//! ### Related
//!
//! * `abi_encode.rs` — inverse operation
//! * `abi_decode_slot.rs` / `abi_decode_dynamic.rs` / `abi_decode_static.rs`
//! * `stdlib.rs` (runtime) — the `abidecode` dispatch arm in `invoke_native_stdlib`

use super::*;

pub(crate) fn lower_abi_decode_direct(
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if args.len() != 2 {
        return None;
    }

    let types = abi_decode_value_types(&args[1], ctx)?;
    if types.is_empty() {
        return None;
    }

    // Each declared decode type must be either static (single- or multi-slot,
    // mirroring the canonical ENCODE side which emits per-field 32-byte slots
    // for all-static structs) or one of the dynamic types we know how to walk
    // (string / bytes / T[] for static T). Bug #24/#25 fix: previously we only
    // accepted static-1-slot types here and fell through to
    // `StdLib.deserialize` (which is Neo-native, NOT EVM-ABI-compatible) for
    // any dynamic type, producing an opaque StackItem callers couldn't
    // `.length` / re-decode. The multi-slot extension closes the same
    // asymmetry for all-static structs: `abi.encode(S(7,9))` produces 64
    // canonical bytes, so `abi.decode(buf, (S))` must walk those slots
    // instead of handing raw ABI bytes to `StdLib.deserialize`.
    let supported = types.iter().all(|value_type| {
        abi_static_slot_count(value_type).is_some()
            || abi_dynamic_decode_value_type_is_supported(value_type)
    });
    if !supported {
        return None;
    }

    let pre_len = instructions.len();
    if !lower_expression(&args[0], ctx, instructions) {
        instructions.truncate(pre_len);
        return Some(false);
    }

    let buffer_local = ctx.allocate_local("__abi_decode_buf".to_string(), None);
    instructions.push(Instruction::StoreLocal(buffer_local));

    let any_dynamic = types.iter().any(abi_dynamic_decode_value_type_is_supported);

    if !any_dynamic {
        // All declared types are static; total payload size is the SUM of
        // each type's slot count × 32 (multi-slot structs occupy one slot
        // per field, mirroring the encode side). Mismatches panic 0x41.
        let expected_bytes: usize = types
            .iter()
            .map(|value_type| abi_static_slot_count(value_type).unwrap_or(1) * 32)
            .sum();
        let decode_ok_label = ctx.next_label();
        instructions.push(Instruction::LoadLocal(buffer_local));
        instructions.push(Instruction::GetSize);
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(expected_bytes as u64),
        )));
        // Only an UNDER-length buffer reverts; `abi.decode` ignores trailing
        // bytes (matches solc/ethers). Use `<` rather than `!=`.
        instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
        instructions.push(Instruction::JumpIf {
            target: decode_ok_label,
        });
        emit_panic(0x41, instructions);
        instructions.push(Instruction::Label(decode_ok_label));
    }
    // For the dynamic case we cannot statically know the total size; the
    // per-slot decoders bound-check the length / offset reads themselves.

    if types.len() == 1 {
        let value_type = &types[0];
        if abi_dynamic_decode_value_type_is_supported(value_type) {
            emit_abi_decode_dynamic_top_level(buffer_local, value_type, ctx, instructions);
        } else {
            emit_abi_decode_static_slot(buffer_local, 0, value_type, ctx, instructions);
        }
        return Some(true);
    }

    let tuple_local = ctx.allocate_local(
        "__abi_decode_tuple".to_string(),
        Some(ValueType::Array(Box::new(ValueType::Any))),
    );
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(types.len() as u64),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(tuple_local));

    // Track the running HEAD slot separately from the tuple index: a
    // multi-slot static member (all-static struct) consumes one head slot
    // per field, shifting every subsequent member's slot position.
    let mut head_slot = 0usize;
    for (index, value_type) in types.iter().enumerate() {
        instructions.push(Instruction::LoadLocal(tuple_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(index as u64),
        )));
        if abi_dynamic_decode_value_type_is_supported(value_type) {
            // Tuple member is dynamic: head slot at index `head_slot` is a
            // u256 byte-offset (relative to the start of the encoded
            // buffer) pointing at the value's tail. Walk that offset to
            // reach the length+payload (string/bytes) or length+elements
            // (static-element array).
            emit_abi_decode_dynamic_tuple_member(
                buffer_local,
                head_slot,
                value_type,
                ctx,
                instructions,
            );
            head_slot += 1;
        } else {
            emit_abi_decode_static_slot(buffer_local, head_slot, value_type, ctx, instructions);
            head_slot += abi_static_slot_count(value_type).unwrap_or(1);
        }
        instructions.push(Instruction::ArraySet);
    }

    instructions.push(Instruction::LoadLocal(tuple_local));
    Some(true)
}

pub(crate) fn abi_decode_value_types(
    types_expr: &Expression,
    ctx: &LoweringContext,
) -> Option<Vec<ValueType>> {
    match types_expr {
        Expression::List(_, params) => params
            .iter()
            .map(|(_, param)| {
                param
                    .as_ref()
                    .and_then(|parameter| infer_type_from_expression(&parameter.ty, ctx))
            })
            .collect(),
        Expression::Parenthesis(_, inner) => {
            infer_type_from_expression(inner, ctx).map(|ty| vec![ty])
        }
        other => infer_type_from_expression(other, ctx).map(|ty| vec![ty]),
    }
}
