use super::*;

pub(crate) fn emit_abi_decode_slot_slice(
    buffer_local: usize,
    index: usize,
    slot_offset: usize,
    len: usize,
    instructions: &mut Vec<Instruction>,
) {
    instructions.push(Instruction::LoadLocal(buffer_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from((index * 32 + slot_offset) as u64),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::Substr);
}

pub(crate) fn emit_abi_u256_slot(ctx: &mut LoweringContext, instructions: &mut Vec<Instruction>) {
    let _ = emit_expr_static_abi_slot_for_value_type(
        &ValueType::Integer {
            signed: false,
            bits: 256,
        },
        ctx,
        instructions,
    );
}

pub(crate) fn emit_abi_fixed_buffer(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    len: usize,
    reverse: bool,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__abi_fixed_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__abi_fixed_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__abi_fixed_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__abi_fixed_count_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);

    if reverse {
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::ReverseItems);
    } else {
        instructions.push(Instruction::LoadLocal(dst_local));
    }
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

/// Bug #23 fix: emit a `len`-byte big-endian buffer for a signed integer
/// whose signed-LE byte representation is on top of the stack. The 32-byte
/// slot encoder passes `len = 32`; `abi.encodePacked(intN)` passes
/// `len = N / 8`.
///
/// Differs from `emit_abi_fixed_buffer(.., len, true)` in one key way: when
/// the source value is negative (high bit of its highest LE byte is set), the
/// destination buffer is initialised to all `0xff` bytes (sign-extension)
/// rather than zeros. The low `count` bytes (= min(size, len)) of the source
/// are then copied in, and the result is reversed to big-endian — matching
/// EVM canonical ABI sign-extension for `intN` (N ∈ {8, 16, 32, 64, 128}).
///
/// Stack on entry: `[src_signed_le_bytearray]`.
/// Stack on exit:  `[buffer_bytearray]` (`len`-byte big-endian, sign-extended).
pub(crate) fn emit_abi_fixed_buffer_signed(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    len: usize,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__abi_sfixed_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__abi_sfixed_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__abi_sfixed_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__abi_sfixed_count_{tmp_id}"), None);
    let fill_local = ctx.allocate_local(format!("__abi_sfixed_fill_{tmp_id}"), None);

    // Save the source ByteArray.
    instructions.push(Instruction::StoreLocal(src_local));

    // size = src.size().
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    // count = min(size, len). Same shape as `emit_abi_fixed_buffer`.
    let ge_label = ctx.next_label();
    let count_done_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump {
        target: count_done_label,
    });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(count_done_label));

    // Detect negative: convert src (signed-LE ByteArray) to Integer and test < 0.
    // NeoVM's CONVERT to Integer treats the operand as signed little-endian, so
    // this matches the original Solidity-level sign of the value.
    //
    // CRITICAL: in this codebase `Instruction::JumpIf` lowers to NeoVM
    // `JMPIFNOT_L` (see `bytecode_emit_ir.rs` ~line 340) — i.e. jumps when the
    // condition is FALSE. So `JumpIf { target }` after `Lt` jumps when
    // `val < 0` is FALSE → fall-through is the negative case.
    let pos_label = ctx.next_label();
    let init_done_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    // Jump to pos_label when `val < 0` is FALSE (positive/zero).
    instructions.push(Instruction::JumpIf { target: pos_label });

    // Negative path (fall-through when `val < 0` is TRUE): 0xff-filled
    // `len`-byte buffer. NewBuffer only zero-fills, so we allocate a fresh
    // zero buffer and MemCpy a literal `[0xff; len]` ByteArray over it.
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));
    // Stash the literal ByteArray into a local so MemCpy's [src, src_offset]
    // operands resolve to a stable ByteString (mirrors the dynamic-bytes-tail
    // pattern at line 832+).
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![
        0xffu8;
        len
    ])));
    instructions.push(Instruction::StoreLocal(fill_local));
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(fill_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::MemCpy);
    instructions.push(Instruction::Jump {
        target: init_done_label,
    });

    // Positive (or zero) path: zero-filled buffer.
    instructions.push(Instruction::Label(pos_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::Label(init_done_label));

    // MemCpy the low `count` bytes of src into dst at offset 0. The remaining
    // `len - count` high bytes of dst keep their fill value (0x00 or 0xff).
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);

    // Reverse to big-endian and convert Buffer → ByteArray.
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::ReverseItems);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

pub(crate) fn emit_abi_bytesn_slot(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    len: usize,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__abi_bytesn_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__abi_bytesn_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__abi_bytesn_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__abi_bytesn_count_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}
