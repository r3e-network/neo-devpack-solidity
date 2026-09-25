use super::*;

mod abi_predicates;
pub(crate) use abi_predicates::*;
mod abi_buffers;
pub(crate) use abi_buffers::*;
mod abi_encode;
pub(crate) use abi_encode::*;
mod abi_encode_static;
pub(crate) use abi_encode_static::*;
mod abi_encode_dynamic;
pub(crate) use abi_encode_dynamic::*;
mod abi_encode_packed;
pub(crate) use abi_encode_packed::*;
mod abi_decode;
pub(crate) use abi_decode::*;
mod abi_decode_slot;
pub(crate) use abi_decode_slot::*;
mod abi_decode_dynamic;
pub(crate) use abi_decode_dynamic::*;
mod abi_decode_static;
pub(crate) use abi_decode_static::*;
mod abi_structs_notify;
pub(crate) use abi_structs_notify::*;
mod native_serialize;
pub(crate) use native_serialize::*;
mod member_access;
pub(crate) use member_access::*;
mod member_runtime;
pub(crate) use member_runtime::*;
mod member_syscalls;
pub(crate) use member_syscalls::*;
mod member_storage;
pub(crate) use member_storage::*;
mod member_neo;
pub(crate) use member_neo::*;
mod member_nativecalls;
pub(crate) use member_nativecalls::*;
mod resolved;
pub(crate) use resolved::*;

pub(crate) fn try_lower_builtin_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    // Solidity 0.8.x type-level concat: `bytes.concat(a, b, ...)` / `string.concat(a, b, ...)`
    // solang-parser represents these as MemberAccess(Type(DynamicBytes|String), "concat").
    if let Some(result) = try_lower_type_concat(func, args, ctx, instructions) {
        return Some(result);
    }

    if let Some(result) = try_lower_member_builtin(func, args, ctx, instructions) {
        return Some(result);
    }

    try_lower_resolved_builtin_call(func, args, ctx, instructions)
}

/// Handle `bytes.concat(a, b, ...)` and `string.concat(a, b, ...)`.
///
/// These are Solidity 0.8.x type-level functions. Each argument is lowered onto
/// the stack, then chained with NeoVM CAT opcodes. Zero arguments produce an
/// empty byte string; one argument is a pass-through.
pub(crate) fn try_lower_type_concat(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let Expression::MemberAccess(_, inner, member) = func else {
        return None;
    };
    if member.name != "concat" {
        return None;
    }
    // Match `bytes.concat(...)` or `string.concat(...)` where inner is a Type node.
    let is_bytes_or_string = matches!(
        inner.as_ref(),
        Expression::Type(_, PtType::DynamicBytes | PtType::String)
    );
    if !is_bytes_or_string {
        return None;
    }

    // Zero args: push empty byte array.
    if args.is_empty() {
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![])));
        return Some(true);
    }

    // Lower all arguments, then emit BytesConcat builtin (CAT chain).
    //
    // Task #76 — `bytesN(..)` / `address(..)` casts route through
    // `coerce_to_fixed_bytes`, which leaves the MEMCPY-returned destination
    // buffer on the stack BENEATH the canonical ByteString (MEMCPY pushes
    // `dst` back per C-style memcpy semantics — see
    // src/runtime/execution/execution_impl_part3_bytes.rs::memcpy_bytes).
    // Chaining CAT across `N` such args would splice the leaked buffer of
    // arg `k+1` between `canonical_k` and `canonical_{k+1}`, producing
    // wrong content (e.g. `bytes.concat(bytes32(1), bytes32(2))` yields
    // `bytes32(2) || bytes32(2)` instead of `bytes32(1) || bytes32(2)`).
    // Mirror the Task #66 packed-encoding fix in resolved.rs: follow each
    // leaky arg with `Swap; Drop` to discard the leaked buffer.
    let mut success = true;
    for arg in args {
        if !lower_expression(arg, ctx, instructions) {
            success = false;
            continue;
        }
        // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
    }

    if success {
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::BytesConcat,
            arg_count: args.len(),
        });
        // On real NeoVM, CAT yields a Buffer (0x30) while ABI parameters,
        // hex literals, keccak256 results, and storage reads are ByteStrings
        // (0x28). StdLib.serialize embeds the stack-item type byte, so an
        // un-canonicalized concat result used as a mapping key would derive
        // a different storage slot than the same bytes presented as a
        // ByteString. Coerce to ByteString so equality / mapping-key /
        // ABI-return layers all see a canonical contiguous bytes value
        // (mirrors the SUBSTR slice coercion in
        // `src/ir/expressions/arrays.rs`).
        instructions.push(Instruction::Convert {
            target: ConvertTarget::ByteArray,
        });
    }

    Some(success)
}
