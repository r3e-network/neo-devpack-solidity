//! ## ABI Encoding Lowering
//!
//! Lowers Solidity `abi.encode()` / `abi.encodePacked()` / `abi.encodeCall()`
//! builtin calls into NeoVM instructions. Static-slot helpers, dynamic-tail
//! helpers, and packed-mode helpers have been extracted into sibling modules
//! (`abi_encode_static`, `abi_encode_dynamic`, `abi_encode_packed`) to keep
//! this orchestration module under the 800-line limit.
//!
//! * **Static slots** — `abi_encode_static` — 32-byte BE-padded slots for static
//!   types (integers, addresses, bools, bytesN)
//! * **Dynamic tails** — `abi_encode_dynamic` — offset+length+data layout for
//!   strings, bytes, dynamic arrays, and dynamic structs
//! * **Head+tail encoding** — `lower_abi_encode_head_tail_direct` below
//! * **Packed encoding** — `abi_encode_packed` — `abi.encodePacked()` semantics
//! * **Struct flattening** — `try_flatten_struct_arg_for_abi_encode` below
//!
//! ### Related
//!
//! * `abi_decode.rs` — inverse operation
//! * `abi_encode_static.rs` / `abi_encode_dynamic.rs` / `abi_encode_packed.rs`
//! * `stdlib.rs` (runtime) — the `invoke_native_stdlib` handler that produces
//!   the head+tail bytes at runtime

use super::*;

/// Task #124 — `abi.encode(struct)` whole-struct expansion. If `expr` has a
/// struct type, emit per-field extraction instructions so the downstream
/// `AbiEncode` builtin receives N individual stack items (one per direct
/// field) instead of a single `StackItem::Array` that the runtime's
/// `abiencode` handler would misclassify as a DYNAMIC type (Task #121's
/// offset+length+elements shape).
///
/// EVM ABI spec: a struct whose fields are all static types is itself a
/// STATIC type, and `abi.encode(s)` must equal the byte-for-byte concatenation
/// of each field's 32-byte BE-padded slot (no offset/length header). For the
/// `keccak256(abi.encode(Voucher{amount, recipient, expiry}))` case this is
/// the 96-byte buffer `BE(amount) || BE(addr) || BE(expiry)`.
///
/// Emission shape (per field `i`):
///   `lower_expression(expr); push i; ArrayGet`
/// which reads the struct's `StackItem::Array` backing storage and indexes
/// into it. For a bare Variable reference to a struct param this is cheap
/// (each `lower_expression` is a single `LoadParameter`); for storage-
/// backed struct locals it re-runs the storage-load but is still correct.
/// The runtime's `ArrayGet` unwraps the field value (Integer / ByteArray /
/// Boolean) so `abi_pad32_be` in the runtime handler sees a scalar it can
/// classify as STATIC.
///
/// Returns `Some(field_count)` if flattening fired, `None` if the expression
/// is not struct-typed (caller falls through to standard `lower_expression`).
/// `success` is threaded through so the caller can short-circuit when a
/// nested lowering fails.
pub(crate) fn try_flatten_struct_arg_for_abi_encode(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    success: &mut bool,
) -> Option<usize> {
    // Look through parentheses to reach the expression we'll type-check.
    let mut current = expr;
    while let Expression::Parenthesis(_, inner) = current {
        current = inner.as_ref();
    }
    let ValueType::Struct { fields, .. } = infer_type_from_expression(current, ctx)? else {
        return None;
    };
    // Only flatten when EVERY field is a STATIC EVM type. Dynamic fields
    // (string/bytes/T[]) would still need the head+tail layout, which the
    // runtime handler already produces for a top-level Array. Avoid muddying
    // the DD3 fix with nested dynamics — gate the flatten on all-static.
    let all_static = fields.iter().all(|f| is_static_abi_type_value(&f.ty));
    if !all_static {
        return None;
    }
    let field_count = fields.len();
    if field_count == 0 {
        return None;
    }
    for i in 0..field_count {
        if !lower_expression(current, ctx, instructions) {
            *success = false;
            return Some(field_count);
        }
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(i as u64),
        )));
        instructions.push(Instruction::ArrayGet);
    }
    Some(field_count)
}

pub(crate) fn lower_abi_encode_args_direct_from_slice(
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let refs: Vec<&Expression> = args.iter().collect();
    lower_abi_encode_args_direct(&refs, ctx, instructions)
}

pub(crate) fn lower_abi_encode_args_direct(
    args: &[&Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    lower_abi_encode_args_direct_impl(args, ctx, instructions, false)
}

pub(crate) fn lower_abi_encode_args_direct_for_encode_call(
    args: &[&Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    lower_abi_encode_args_direct_impl(args, ctx, instructions, true)
}

pub(crate) fn lower_abi_encode_args_direct_impl(
    args: &[&Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    flatten_struct_params: bool,
) -> Option<bool> {
    if args.is_empty() {
        instructions.push(Instruction::PushLiteral(
            LiteralValue::ByteArray(Vec::new()),
        ));
        return Some(true);
    }

    if let Some(result) = lower_abi_encode_head_tail_direct(args, ctx, instructions) {
        return Some(result);
    }

    let pre_len = instructions.len();
    let mut slot_count = 0usize;
    for arg in args {
        match lower_static_abi_slots_for_expr(arg, ctx, instructions, flatten_struct_params) {
            Some(slots) => slot_count += slots,
            None => {
                instructions.truncate(pre_len);
                return None;
            }
        }
    }

    if slot_count == 0 {
        instructions.truncate(pre_len);
        return None;
    }

    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: slot_count,
    });
    Some(true)
}

pub(crate) fn lower_abi_encode_head_tail_direct(
    args: &[&Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let arg_types: Vec<ValueType> = args
        .iter()
        .map(|arg| infer_type_from_expression(arg, ctx))
        .collect::<Option<Vec<_>>>()?;

    if !arg_types.iter().any(abi_value_type_is_dynamic) {
        return None;
    }

    let mut head_slot_count = 0usize;
    for value_type in &arg_types {
        if abi_value_type_is_dynamic(value_type) {
            if !abi_dynamic_value_type_is_supported(value_type) {
                return None;
            }
            head_slot_count += 1;
        } else if let Some(slots) = abi_static_slot_count(value_type) {
            head_slot_count += slots;
        } else {
            return None;
        }
    }

    let pre_len = instructions.len();
    let mut value_locals = Vec::with_capacity(args.len());
    for (index, (arg, value_type)) in args.iter().zip(arg_types.iter()).enumerate() {
        if !lower_expression(arg, ctx, instructions) {
            instructions.truncate(pre_len);
            return Some(false);
        }
        let tmp_id = ctx.next_label();
        let local = ctx.allocate_local(
            format!("__abi_arg_{index}_{tmp_id}"),
            Some(value_type.clone()),
        );
        instructions.push(Instruction::StoreLocal(local));
        value_locals.push(local);
    }

    let offset_local = ctx.allocate_local("__abi_tail_offset".to_string(), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from((head_slot_count * 32) as u64),
    )));
    instructions.push(Instruction::StoreLocal(offset_local));

    let mut tail_locals = Vec::new();
    let mut part_count = 0usize;

    for (local, value_type) in value_locals.iter().zip(arg_types.iter()) {
        if abi_value_type_is_dynamic(value_type) {
            instructions.push(Instruction::LoadLocal(offset_local));
            emit_abi_u256_slot(ctx, instructions);
            part_count += 1;

            instructions.push(Instruction::LoadLocal(*local));
            emit_abi_dynamic_tail_for_value_type(value_type, 0, ctx, instructions)?;
            let tail_local = ctx.allocate_local("__abi_tail".to_string(), None);
            instructions.push(Instruction::StoreLocal(tail_local));

            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::LoadLocal(tail_local));
            instructions.push(Instruction::GetSize);
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::StoreLocal(offset_local));
            tail_locals.push(tail_local);
        } else {
            let slots = emit_abi_static_slots_from_local(*local, value_type, ctx, instructions)?;
            part_count += slots;
        }
    }

    for tail_local in tail_locals {
        instructions.push(Instruction::LoadLocal(tail_local));
        part_count += 1;
    }

    if part_count == 0 {
        instructions.truncate(pre_len);
        return None;
    }

    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: part_count,
    });
    Some(true)
}

pub(crate) fn emit_abi_encode_single_stack_value_for_type(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    if abi_value_type_is_dynamic(value_type) {
        if !abi_dynamic_value_type_is_supported(value_type) {
            return None;
        }

        let value_local = ctx.allocate_local("__abi_single_value".to_string(), None);
        instructions.push(Instruction::StoreLocal(value_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(32u8),
        )));
        emit_abi_u256_slot(ctx, instructions);
        instructions.push(Instruction::LoadLocal(value_local));
        emit_abi_dynamic_tail_for_value_type(value_type, 0, ctx, instructions)?;
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::BytesConcat,
            arg_count: 2,
        });
        return Some(());
    }

    if abi_static_slot_count(value_type) == Some(1) {
        emit_expr_static_abi_slot_for_value_type(value_type, ctx, instructions)?;
        return Some(());
    }

    None
}
