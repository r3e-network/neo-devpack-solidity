#[test]
fn try_catch_lowers_to_try_and_endtry_in_ir() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFoo {
        function foo() external;
    }

    contract TryHarness {
        function run(address target) public {
            try IFoo(target).foo() {
                // success
            } catch {
                // catch
            }
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let run_fn = module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("expected run function");
    let instrs = &run_fn.basic_blocks[0].instructions;

    let tries: Vec<_> = instrs
        .iter()
        .filter_map(|instr| match instr {
            ir::Instruction::Try { catch_target } => Some(*catch_target),
            _ => None,
        })
        .collect();
    assert_eq!(tries.len(), 1, "expected exactly one TRY in lowered IR");

    let endtrys: Vec<_> = instrs
        .iter()
        .filter_map(|instr| match instr {
            ir::Instruction::EndTry { target } => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(
        endtrys.len(),
        2,
        "expected success+catch ENDTRY in lowered IR"
    );

    let catch_label = tries[0];
    let catch_index = instrs
        .iter()
        .position(|instr| matches!(instr, ir::Instruction::Label(id) if *id == catch_label))
        .expect("catch label should exist");

    assert!(
        matches!(instrs.get(catch_index + 1), Some(ir::Instruction::Drop(_))),
        "expected simple catch handler to drop NeoVM exception value"
    );
}

#[test]
fn try_catch_multiple_clauses_emit_runtime_type_guards() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFoo {
        function foo() external;
    }

    contract TryHarness {
        function run(address target) public {
            try IFoo(target).foo() {
                // success
            } catch Panic(uint256 code) {
                code;
            } catch (bytes memory lowLevelData) {
                lowLevelData;
            } catch {
                // fallback
            }
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let run_fn = module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("expected run function");
    let instrs = &run_fn.basic_blocks[0].instructions;

    // Task #103 — `catch Panic(uint256)` now emits a 4-byte selector
    // match against `keccak256("Panic(uint256)")[..4] = 0x4e487b71`
    // instead of an ISTYPE Integer guard. The guard sequence is
    //   IsType ByteArray → JumpIf (not-bytes)
    //   GetSize → Push(36) → Lt → JumpIf (long-enough) → Jump (short)
    //   Substr(0, 4) → Push(selector) → Eq → JumpIf (mismatch)
    // so we pin the selector byte literal as the landmark instead of
    // the old ISTYPE Integer.
    let panic_selector = [0x4eu8, 0x48, 0x7b, 0x71];
    assert!(
        instrs.iter().any(|instr| matches!(instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes))
            if bytes.as_slice() == panic_selector
        )),
        "expected catch Panic(uint256) to emit a selector literal 0x4e487b71"
    );

    // The bytes catch still runs via the legacy ISTYPE ByteArray guard
    // (for the bare `catch` fallback compatibility path) or binds
    // directly. We now use the specialized `Bytes` arm that always
    // matches, so we instead pin that at least one `IsType ByteArray`
    // exists — it's still emitted by the Panic selector guard.
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::IsType {
                target: ir::ConvertTarget::ByteArray
            }
        )),
        "expected an ISTYPE ByteArray guard to precede the Panic selector match"
    );
}
