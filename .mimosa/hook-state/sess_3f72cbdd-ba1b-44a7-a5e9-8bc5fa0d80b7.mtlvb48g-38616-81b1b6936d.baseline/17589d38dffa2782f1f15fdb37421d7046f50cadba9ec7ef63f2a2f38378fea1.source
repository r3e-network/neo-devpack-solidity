// ==================== Helper Function Unit Tests ====================

use crate::opcode::OpCode;

#[test]
fn emit_load_local_uses_short_form_for_small_indices() {
    let mut bytecode = Vec::new();
    emit_load_local(&mut bytecode, 0);
    assert_eq!(bytecode, vec![OpCode::LDLOC0.byte()], "LDLOC0");

    bytecode.clear();
    emit_load_local(&mut bytecode, 6);
    assert_eq!(bytecode, vec![OpCode::LDLOC6.byte()], "LDLOC6");

    bytecode.clear();
    emit_load_local(&mut bytecode, 7);
    assert_eq!(bytecode, vec![OpCode::LDLOC.byte(), 0x07], "LDLOC with index");
}

#[test]
fn emit_store_local_uses_short_form_for_small_indices() {
    let mut bytecode = Vec::new();
    emit_store_local(&mut bytecode, 0);
    assert_eq!(bytecode, vec![OpCode::STLOC0.byte()], "STLOC0");

    bytecode.clear();
    emit_store_local(&mut bytecode, 6);
    assert_eq!(bytecode, vec![OpCode::STLOC6.byte()], "STLOC6");

    bytecode.clear();
    emit_store_local(&mut bytecode, 7);
    assert_eq!(bytecode, vec![OpCode::STLOC.byte(), 0x07], "STLOC with index");
}

#[test]
fn emit_binary_op_emits_correct_opcodes() {
    let test_cases = [
        (ir::BinaryOperator::Add, OpCode::ADD),
        (ir::BinaryOperator::Sub, OpCode::SUB),
        (ir::BinaryOperator::Mul, OpCode::MUL),
        (ir::BinaryOperator::Div, OpCode::DIV),
        (ir::BinaryOperator::Mod, OpCode::MOD),
        (ir::BinaryOperator::BitAnd, OpCode::AND),
        (ir::BinaryOperator::BitOr, OpCode::OR),
        (ir::BinaryOperator::BitXor, OpCode::XOR),
        (ir::BinaryOperator::Lt, OpCode::LT),
        (ir::BinaryOperator::Eq, OpCode::EQUAL),
        (ir::BinaryOperator::Ne, OpCode::NOTEQUAL),
    ];

    for (op, expected_opcode) in test_cases {
        let mut bytecode = Vec::new();
        emit_binary_op(&mut bytecode, op);
        assert_eq!(
            bytecode,
            vec![expected_opcode.byte()],
            "operator {op:?} should emit opcode {}",
            expected_opcode.name()
        );
    }
}

#[test]
fn push_literal_value_handles_all_types() {
    // Boolean true
    let mut bytecode = Vec::new();
    push_literal_value(&mut bytecode, &LiteralValue::Boolean(true));
    assert_eq!(bytecode, vec![OpCode::PUSHT.byte()], "PUSHT for true");

    // Boolean false
    bytecode.clear();
    push_literal_value(&mut bytecode, &LiteralValue::Boolean(false));
    assert_eq!(bytecode, vec![OpCode::PUSHF.byte()], "PUSHF for false");

    // Small integer
    bytecode.clear();
    push_literal_value(&mut bytecode, &LiteralValue::Integer(BigInt::from(5)));
    assert_eq!(bytecode, vec![OpCode::PUSH5.byte()], "PUSH5");

    // Zero
    bytecode.clear();
    push_literal_value(&mut bytecode, &LiteralValue::Integer(BigInt::from(0)));
    assert_eq!(bytecode, vec![OpCode::PUSH0.byte()], "PUSH0");

    // Empty string
    bytecode.clear();
    push_literal_value(&mut bytecode, &LiteralValue::String(vec![]));
    assert_eq!(
        bytecode,
        vec![OpCode::PUSHDATA1.byte(), 0x00],
        "PUSHDATA1 with empty data"
    );

    // Short byte array
    bytecode.clear();
    push_literal_value(&mut bytecode, &LiteralValue::ByteArray(vec![0xAB, 0xCD]));
    assert_eq!(
        bytecode,
        vec![OpCode::PUSHDATA1.byte(), 0x02, 0xAB, 0xCD],
        "PUSHDATA1 with bytes"
    );
}

#[test]
fn push_integer_bigint_handles_special_values() {
    // Zero
    let mut bytecode = Vec::new();
    push_integer_bigint(&mut bytecode, &BigInt::from(0));
    assert_eq!(bytecode, vec![OpCode::PUSH0.byte()], "PUSH0");

    // -1
    bytecode.clear();
    push_integer_bigint(&mut bytecode, &BigInt::from(-1));
    assert_eq!(bytecode, vec![OpCode::PUSHM1.byte()], "PUSHM1");

    // Small positive (1-16)
    for i in 1..=16u8 {
        bytecode.clear();
        push_integer_bigint(&mut bytecode, &BigInt::from(i));
        let expected_opcode = OpCode::push_small(i).expect("0..=16");
        assert_eq!(
            bytecode,
            vec![expected_opcode.byte()],
            "PUSH{i} should emit {}",
            expected_opcode.name()
        );
    }

    // Larger positive
    bytecode.clear();
    push_integer_bigint(&mut bytecode, &BigInt::from(255));
    assert!(bytecode.len() > 1, "larger integers use PUSHDATA");
}

#[test]
fn push_integer_bigint_coerces_out_of_range_unsigned_literals() {
    use num_traits::One;

    // 2^256 - 1 (`type(uint256).max`) is not a positive NeoVM integer, but IS
    // its conformant 32-byte TWO'S-COMPLEMENT (`-1` => `0xFF` * 32), pushed via
    // PUSHINT256. A real Neo node rejects the old 33-byte PUSH0+ADD form.
    let mut value = BigInt::one();
    value <<= 256usize;
    value -= BigInt::one();

    let mut bytecode = Vec::new();
    push_integer_bigint(&mut bytecode, &value);

    let mut expected = vec![OpCode::PUSHINT256.byte()];
    expected.extend_from_slice(&[0xFFu8; 32]); // two's-complement of -1
    assert_eq!(
        bytecode, expected,
        "expected type(uint256).max to lower to PUSHINT256 of the 32-byte two's-complement"
    );
}

#[test]
fn append_default_value_handles_all_types() {
    // Integer
    let mut bytecode = Vec::new();
    append_default_value(
        &mut bytecode,
        &ValueType::Integer {
            signed: false,
            bits: 256,
        },
    );
    assert_eq!(bytecode, vec![OpCode::PUSH0.byte()], "default integer is 0");

    // Boolean
    bytecode.clear();
    append_default_value(&mut bytecode, &ValueType::Boolean);
    assert_eq!(
        bytecode,
        vec![OpCode::PUSH0.byte()],
        "default boolean is false (0)"
    );

    // Array
    bytecode.clear();
    append_default_value(
        &mut bytecode,
        &ValueType::Array(Box::new(ValueType::Boolean)),
    );
    assert_eq!(bytecode, vec![OpCode::NEWARRAY0.byte()], "NEWARRAY0");

    // Mapping
    bytecode.clear();
    append_default_value(
        &mut bytecode,
        &ValueType::Mapping {
            key: Box::new(ValueType::Address),
            value: Box::new(ValueType::Integer {
                signed: false,
                bits: 256,
            }),
        },
    );
    assert_eq!(bytecode, vec![OpCode::NEWMAP.byte()], "NEWMAP");

    // Any
    bytecode.clear();
    append_default_value(&mut bytecode, &ValueType::Any);
    assert_eq!(bytecode, vec![OpCode::PUSHNULL.byte()], "PUSHNULL");
}

#[test]
fn interop_id_bytes_produces_consistent_hashes() {
    let id1 = interop_id_bytes("System.Storage.Get");
    let id2 = interop_id_bytes("System.Storage.Get");
    assert_eq!(id1, id2, "same syscall should produce same ID");

    let id3 = interop_id_bytes("System.Storage.Put");
    assert_ne!(id1, id3, "different syscalls should produce different IDs");

    assert_eq!(id1.len(), 4, "interop ID should be 4 bytes");
}
