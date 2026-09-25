#[test]
fn removes_trivial_jump_to_next_label() {
    let module = Module {
        functions: vec![Function {
            name: "trivialjump".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::Jump { target: 1 },
                    Instruction::Label(1),
                    Instruction::ReturnVoid,
                ],
            }],
            local_count: 0,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    assert!(
        !matches!(instrs[0], Instruction::Jump { .. }),
        "trivial jump should be removed"
    );
    assert!(matches!(instrs[0], Instruction::Label(1)));
    assert!(matches!(instrs[1], Instruction::ReturnVoid));
}

#[test]
fn neovm_peephole_removes_push_drop() {
    use neo_devpack_solidity::ir::LiteralValue;
    use num_bigint::BigInt;

    let module = Module {
        functions: vec![Function {
            name: "peephole".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::PushLiteral(LiteralValue::Integer(BigInt::from(42))),
                    Instruction::Drop(ValueType::Integer {
                        signed: false,
                        bits: 256,
                    }),
                    Instruction::ReturnVoid,
                ],
            }],
            local_count: 0,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // PUSH followed by DROP should be removed
    assert_eq!(instrs.len(), 1);
    assert!(matches!(instrs[0], Instruction::ReturnVoid));
}

#[test]
fn neovm_peephole_removes_load_local_drop() {
    let module = Module {
        functions: vec![Function {
            name: "peephole_local".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::Drop(ValueType::Integer {
                        signed: false,
                        bits: 256,
                    }),
                    Instruction::ReturnVoid,
                ],
            }],
            local_count: 1,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // LoadLocal followed by DROP should be removed
    assert_eq!(instrs.len(), 1);
    assert!(matches!(instrs[0], Instruction::ReturnVoid));
}

#[test]
fn neovm_simplify_removes_add_zero() {
    use neo_devpack_solidity::ir::{BinaryOperator, LiteralValue};
    use num_bigint::BigInt;

    let module = Module {
        functions: vec![Function {
            name: "identity_add".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::PushLiteral(LiteralValue::Integer(BigInt::from(0))),
                    Instruction::BinaryOp(BinaryOperator::Add),
                    Instruction::Return,
                ],
            }],
            local_count: 1,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // x + 0 should be simplified to just x
    assert_eq!(instrs.len(), 2);
    assert!(matches!(instrs[0], Instruction::LoadLocal(0)));
    assert!(matches!(instrs[1], Instruction::Return));
}

#[test]
fn neovm_simplify_removes_mul_one() {
    use neo_devpack_solidity::ir::{BinaryOperator, LiteralValue};
    use num_bigint::BigInt;

    let module = Module {
        functions: vec![Function {
            name: "identity_mul".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::PushLiteral(LiteralValue::Integer(BigInt::from(1))),
                    Instruction::BinaryOp(BinaryOperator::Mul),
                    Instruction::Return,
                ],
            }],
            local_count: 1,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // x * 1 should be simplified to just x
    assert_eq!(instrs.len(), 2);
    assert!(matches!(instrs[0], Instruction::LoadLocal(0)));
    assert!(matches!(instrs[1], Instruction::Return));
}

#[test]
fn neovm_bool_keeps_eq_true_for_safety() {
    // M-BC2 fix regression guard: the `PUSH true, EQ → identity` rewrite was
    // REMOVED because for a non-boolean operand (e.g. NeoVM Integer 5) it
    // left `5` on the stack instead of producing `false`. The optimizer must
    // now KEEP the `PUSH true; EQ` pair so the runtime performs the real
    // equality check.
    use neo_devpack_solidity::ir::{BinaryOperator, LiteralValue};

    let module = Module {
        functions: vec![Function {
            name: "bool_eq_true".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::PushLiteral(LiteralValue::Boolean(true)),
                    Instruction::BinaryOp(BinaryOperator::Eq),
                    Instruction::Return,
                ],
            }],
            local_count: 1,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // x == true must NOT be folded to `x`; the PUSH true; EQ pair survives.
    assert_eq!(instrs.len(), 4, "PUSH true; EQ must be preserved (M-BC2)");
    assert!(matches!(instrs[0], Instruction::LoadLocal(0)));
    assert!(matches!(
        instrs[1],
        Instruction::PushLiteral(LiteralValue::Boolean(true))
    ));
    assert!(matches!(
        instrs[2],
        Instruction::BinaryOp(BinaryOperator::Eq)
    ));
    assert!(matches!(instrs[3], Instruction::Return));
}

#[test]
fn neovm_bool_keeps_ne_false_for_safety() {
    // Mirror of `neovm_bool_keeps_eq_true_for_safety`: the `PUSH false, NE →
    // identity` rewrite was REMOVED because for a non-boolean operand (e.g.
    // NeoVM Integer 5) it left the raw `5` on the stack instead of normalizing
    // `5 != false` to the boolean `true` (1). The optimizer must now KEEP the
    // `PUSH false; NE` pair so the runtime performs the real inequality.
    use neo_devpack_solidity::ir::{BinaryOperator, LiteralValue};

    let module = Module {
        functions: vec![Function {
            name: "bool_ne_false".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::PushLiteral(LiteralValue::Boolean(false)),
                    Instruction::BinaryOp(BinaryOperator::Ne),
                    Instruction::Return,
                ],
            }],
            local_count: 1,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // x != false must NOT be folded to `x`; the PUSH false; NE pair survives.
    assert_eq!(instrs.len(), 4, "PUSH false; NE must be preserved");
    assert!(matches!(instrs[0], Instruction::LoadLocal(0)));
    assert!(matches!(
        instrs[1],
        Instruction::PushLiteral(LiteralValue::Boolean(false))
    ));
    assert!(matches!(
        instrs[2],
        Instruction::BinaryOp(BinaryOperator::Ne)
    ));
    assert!(matches!(instrs[3], Instruction::Return));
}

#[test]
fn neovm_double_bitwise_not_removed() {
    let module = Module {
        functions: vec![Function {
            name: "double_not".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::BitwiseNot,
                    Instruction::BitwiseNot,
                    Instruction::Return,
                ],
            }],
            local_count: 1,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let optimized = optimize_ir(module, 3);
    let instrs = &optimized.functions[0].basic_blocks[0].instructions;
    // ~~x should be simplified to just x
    assert_eq!(instrs.len(), 2);
    assert!(matches!(instrs[0], Instruction::LoadLocal(0)));
    assert!(matches!(instrs[1], Instruction::Return));
}

