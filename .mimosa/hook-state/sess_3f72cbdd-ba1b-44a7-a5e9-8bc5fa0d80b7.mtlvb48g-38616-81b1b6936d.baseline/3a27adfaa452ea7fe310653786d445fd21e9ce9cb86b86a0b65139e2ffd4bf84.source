//! ## ABI Decoding — Slot Reading Helpers
//!
//! Low-level helpers extracted from `abi_decode.rs` to keep the orchestration
//! module under the 800-line limit. This module reads the big-endian 32-byte
//! length/offset slots used by the EVM ABI head section and guards against
//! non-canonical high bytes.
//!
//! Functions are `pub(crate)` and re-exported from `builtins` so the dynamic
//! and static decoding modules can access them through a single namespace.

use super::*;

/// Read the (length-/offset-shaped) u256 stored at byte offset
/// `byte_offset` inside `buffer_local` (BE-encoded 32-byte slot) and
/// push it as a NeoVM `Integer`.
///
/// The runtime's `CONVERT → Integer` of a >8-byte ByteArray returns a
/// signed-LE `ByteArray`, NOT an `Integer`, to preserve precision (see
/// `convert_item` in `runtime/execution/execution_impl_part3_conversion.rs`
/// and the Task #111 history). That breaks downstream callers (SUBSTR
/// `pop_usize`) which require a real Integer.
///
/// EVM-canonical length/offset slots are bounded by `usize::MAX` in
/// practice (calldata cannot exceed a few MB on any real chain, let alone
/// 2^64 bytes), so it is safe to read only the low 8 bytes of the slot
/// and rely on the ≤8-byte fast path of `convert_item` which produces an
/// `Integer(i64)`. The high 24 bytes of every well-formed length/offset
/// slot are zero — if a malicious / corrupted payload claims a length
/// exceeding `i64::MAX`, the subsequent `SUBSTR` will fault on bounds
/// anyway.
/// Reject a non-canonical length/offset slot whose HIGH 24 bytes are nonzero.
///
/// The slot readers trust only the low 8 bytes (calldata lengths/offsets never
/// exceed a few MB), but a crafted payload could set bytes [0..24) to encode a
/// value in `[2^64, 2^192)` whose low 64 bits happen to be small and in-bounds.
/// Reading only the low 8 bytes would then silently use the truncated value and
/// decode the wrong region instead of reverting. `offset_push` pushes the byte
/// offset of the slot start; Panic(0x41) fires when any high byte is set. A
/// conformant slot always has zero high bytes, so this never faults valid input.
pub(crate) fn emit_abi_decode_slot_high_bits_guard(
    buffer_local: usize,
    offset_push: Instruction,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let ok = ctx.next_label();
    instructions.push(Instruction::LoadLocal(buffer_local));
    instructions.push(offset_push);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(24u8),
    )));
    instructions.push(Instruction::Substr);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![
        0u8;
        24
    ])));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::LogicalNot); // true when the high 24 bytes are NONZERO
                                                // JumpIf -> JMPIFNOT: branch to `ok` when the nonzero flag is FALSE (i.e.
                                                // the high bytes are all zero); otherwise fall through to the panic.
    instructions.push(Instruction::JumpIf { target: ok });
    emit_panic(0x41, instructions);
    instructions.push(Instruction::Label(ok));
}

pub(crate) fn emit_abi_decode_u256_at(
    buffer_local: usize,
    byte_offset: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    emit_abi_decode_slot_high_bits_guard(
        buffer_local,
        Instruction::PushLiteral(LiteralValue::Integer(BigInt::from(byte_offset as u64))),
        ctx,
        instructions,
    );
    instructions.push(Instruction::LoadLocal(buffer_local));
    // The low 8 bytes of the 32-byte BE slot live at `byte_offset + 24`.
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from((byte_offset + 24) as u64),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(8u8),
    )));
    instructions.push(Instruction::Substr);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    // Reverse BE→LE so the ≤8-byte CONVERT→Integer path interprets the
    // bytes correctly.
    materialize_byte_array_buffer(ctx, instructions, true);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
}

/// Read the u256 stored at the runtime byte offset held in `offset_local`.
/// Same low-8-bytes shortcut as [`emit_abi_decode_u256_at`].
pub(crate) fn emit_abi_decode_u256_at_runtime(
    buffer_local: usize,
    offset_local: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    emit_abi_decode_slot_high_bits_guard(
        buffer_local,
        Instruction::LoadLocal(offset_local),
        ctx,
        instructions,
    );
    instructions.push(Instruction::LoadLocal(buffer_local));
    // adj_offset = offset_local + 24 (skip the 24 BE high-zero bytes).
    instructions.push(Instruction::LoadLocal(offset_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(24u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(8u8),
    )));
    instructions.push(Instruction::Substr);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    materialize_byte_array_buffer(ctx, instructions, true);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
}
