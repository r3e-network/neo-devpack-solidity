//! ## ABI Encoding — Packed Mode
//!
//! Helpers extracted from `abi_encode.rs` to keep the orchestration module
//! under the 800-line limit. This module covers Solidity `abi.encodePacked()`
//! lowering, which concatenates each argument's minimal encoding (no padding
//! to 32-byte slots, no head/tail indirection).
//!
//! Functions are `pub(crate)` and re-exported from `builtins` so the main
//! `abi_encode` module can access them through a single namespace.

use super::*;

pub(crate) fn lower_abi_encode_packed_args_direct_from_slice(
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if args.is_empty() {
        instructions.push(Instruction::PushLiteral(
            LiteralValue::ByteArray(Vec::new()),
        ));
        return Some(true);
    }

    let pre_len = instructions.len();
    for arg in args {
        if !lower_packed_abi_bytes_for_expr(arg, ctx, instructions)? {
            instructions.truncate(pre_len);
            return Some(false);
        }
    }

    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: args.len(),
    });
    Some(true)
}

pub(crate) fn lower_packed_abi_bytes_for_expr(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let value_type = infer_type_from_expression(expr, ctx)?;
    // An integer-backed `bytesN` literal/constant packs as exactly its N
    // big-endian bytes — not the little-endian integer backing (which would be
    // reversed and the wrong width). Canonicalize before the generic path.
    if try_lower_bytesn_literal_canonical(expr, &value_type, ctx, instructions) {
        return Some(true);
    }
    let pre_len = instructions.len();
    if !lower_expression(expr, ctx, instructions) {
        return Some(false);
    }

    let lowered = match value_type {
        // Bug #23 (packed variant): negative signed integers must be
        // SIGN-EXTENDED (0xff fill) to their declared width, not zero-padded
        // — `abi.encodePacked(int16(-1))` is 0xffff, not 0x00ff. Route signed
        // through the sign-aware buffer at the packed width.
        ValueType::Integer { bits, signed: true } => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_abi_fixed_buffer_signed(ctx, instructions, (bits / 8) as usize);
            Some(true)
        }
        ValueType::Integer {
            bits,
            signed: false,
        } => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_abi_fixed_buffer(ctx, instructions, (bits / 8) as usize, true);
            Some(true)
        }
        ValueType::Boolean => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_abi_fixed_buffer(ctx, instructions, 1, true);
            Some(true)
        }
        ValueType::Address => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_abi_fixed_buffer(ctx, instructions, 20, true);
            Some(true)
        }
        ValueType::ByteArray { fixed_len: Some(_) }
        | ValueType::ByteArray { fixed_len: None }
        | ValueType::String => Some(true),
        ValueType::Array(element_type) if is_static_abi_type_value(&element_type) => {
            emit_abi_packed_static_array(&element_type, ctx, instructions)?;
            Some(true)
        }
        _ => None,
    };

    if lowered.is_none() {
        instructions.truncate(pre_len);
    }
    lowered
}
