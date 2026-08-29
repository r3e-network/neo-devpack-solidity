use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_binary_op(bytecode: &mut Vec<u8>, operator: ir::BinaryOperator) {
    let opcode = match operator {
        ir::BinaryOperator::Add => OpCode::ADD,
        ir::BinaryOperator::Sub => OpCode::SUB,
        ir::BinaryOperator::Mul => OpCode::MUL,
        ir::BinaryOperator::Div => OpCode::DIV,
        ir::BinaryOperator::Mod => OpCode::MOD,
        ir::BinaryOperator::BitAnd => OpCode::AND,
        ir::BinaryOperator::BitOr => OpCode::OR,
        ir::BinaryOperator::BitXor => OpCode::XOR,
        ir::BinaryOperator::Shl => OpCode::SHL,
        ir::BinaryOperator::Shr => OpCode::SHR,
        ir::BinaryOperator::Lt => OpCode::LT,
        ir::BinaryOperator::Le => OpCode::LE,
        ir::BinaryOperator::Gt => OpCode::GT,
        ir::BinaryOperator::Ge => OpCode::GE,
        ir::BinaryOperator::Eq => OpCode::EQUAL,
        ir::BinaryOperator::Ne => OpCode::NOTEQUAL,
    };
    bytecode.push(opcode.byte());
}

pub(crate) fn push_literal_value(bytecode: &mut Vec<u8>, literal: &LiteralValue) {
    match literal {
        LiteralValue::Integer(value) => push_integer_bigint(bytecode, value),
        // Use NeoVM boolean opcodes so returned values match the manifest ABI type.
        LiteralValue::Boolean(true) => bytecode.push(OpCode::PUSHT.byte()),
        LiteralValue::Boolean(false) => bytecode.push(OpCode::PUSHF.byte()),
        LiteralValue::String(bytes) => push_data(bytecode, bytes),
        LiteralValue::ByteArray(bytes) => push_data(bytecode, bytes),
        LiteralValue::Address(bytes) => push_data(bytecode, bytes),
        LiteralValue::Null => bytecode.push(OpCode::PUSHNULL.byte()),
    }
}

pub(crate) fn push_integer_bigint(bytecode: &mut Vec<u8>, value: &BigInt) {
    if value.is_zero() {
        bytecode.push(OpCode::PUSH0.byte());
        return;
    }

    if *value == BigInt::from(-1) {
        bytecode.push(OpCode::PUSHM1.byte());
        return;
    }

    if value.is_positive() {
        if let Some(n) = value.to_u8() {
            if let Some(op) = OpCode::push_small(n) {
                bytecode.push(op.byte());
                return;
            }
        }
    }

    // NeoVM integer literals should be pushed as Integer stack items so that operations like
    // `EQUAL` behave correctly for Solidity numeric types. Use the dedicated PUSHINT opcodes
    // whenever the value fits in their width, falling back to raw bytes only when needed.
    if let Some(n) = value.to_i64() {
        if (i8::MIN as i64..=i8::MAX as i64).contains(&n) {
            bytecode.push(OpCode::PUSHINT8.byte());
            bytecode.push(n as i8 as u8);
            return;
        }
        if (i16::MIN as i64..=i16::MAX as i64).contains(&n) {
            bytecode.push(OpCode::PUSHINT16.byte());
            bytecode.extend_from_slice(&(n as i16).to_le_bytes());
            return;
        }
        if (i32::MIN as i64..=i32::MAX as i64).contains(&n) {
            bytecode.push(OpCode::PUSHINT32.byte());
            bytecode.extend_from_slice(&(n as i32).to_le_bytes());
            return;
        }
        bytecode.push(OpCode::PUSHINT64.byte());
        bytecode.extend_from_slice(&n.to_le_bytes());
        return;
    }

    let bytes = value.to_signed_bytes_le();
    if bytes.len() <= 16 {
        bytecode.push(OpCode::PUSHINT128.byte());
        let fill = if value.is_negative() { 0xFF } else { 0x00 };
        let mut buf = [fill; 16];
        buf[..bytes.len()].copy_from_slice(&bytes);
        bytecode.extend_from_slice(&buf);
        return;
    }
    if bytes.len() <= 32 {
        bytecode.push(OpCode::PUSHINT256.byte());
        let fill = if value.is_negative() { 0xFF } else { 0x00 };
        let mut buf = [fill; 32];
        buf[..bytes.len()].copy_from_slice(&bytes);
        bytecode.extend_from_slice(&buf);
        return;
    }

    // uint256 value with the high bit set (`[2^255, 2^256-1]`, e.g.
    // `type(uint256).max`). It is not a positive NeoVM integer (would need 33
    // bytes, which a real node rejects), but IS representable as its 32-byte
    // TWO'S-COMPLEMENT (`value - 2^256`, which "looks negative"). That is the
    // only conformant on-chain form. Any literal in this range must be unsigned
    // (int256 max is 2^255-1), so the reinterpretation is unambiguous. The
    // software uint256 routines (now emitted inline as IR; see
    // ir/expressions/dispatch/binary.rs) and the runtime operate on this
    // representation.
    {
        let two256: BigInt = BigInt::from(1) << 256u32;
        let sign_min: BigInt = BigInt::from(1) << 255u32;
        if value.is_positive() && *value < two256 && *value >= sign_min {
            let twos: BigInt = value - &two256; // negative; fits in 32 signed bytes
            let tb = twos.to_signed_bytes_le();
            bytecode.push(OpCode::PUSHINT256.byte());
            let mut buf = [0xFFu8; 32]; // sign-extend the negative value
            buf[..tb.len()].copy_from_slice(&tb);
            bytecode.extend_from_slice(&buf);
            return;
        }
    }

    // Fallback: genuinely out-of-range literal (magnitude >= 2^256). Push the
    // signed little-endian bytes and coerce them into an Integer via `value + 0`.
    push_data(bytecode, &bytes);
    bytecode.push(OpCode::PUSH0.byte());
    bytecode.push(OpCode::ADD.byte());
}
