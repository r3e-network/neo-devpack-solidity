#[test]
fn address_code_length_uses_contract_management_iscontract() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CodeLenHarness {
        function len(address account) public view returns (uint256) {
            return account.code.length;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let len_fn = module
        .functions
        .iter()
        .find(|function| function.name == "len")
        .expect("expected len function");

    let instrs = &len_fn.basic_blocks[0].instructions;
    let mut saw_is_contract = false;

    for instr in instrs {
        match instr {
            ir::Instruction::CallBuiltin {
                builtin:
                    ir::BuiltinCall::NativeCall {
                        contract: ir::NativeContract::ContractManagement,
                        method,
                    },
                ..
            } if method == "isContract" => saw_is_contract = true,
            _ => {}
        }
    }

    assert!(
        saw_is_contract,
        "expected code.length to call ContractManagement.isContract"
    );
}

#[test]
fn address_code_uses_contract_management_get_contract_script() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CodeHarness {
        function codeOf(address account) public view returns (bytes memory) {
            return account.code;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let code_fn = module
        .functions
        .iter()
        .find(|function| function.name == "codeOf")
        .expect("expected codeOf function");

    let instrs = &code_fn.basic_blocks[0].instructions;
    let mut saw_get_contract_script = false;

    for instr in instrs {
        if let ir::Instruction::CallBuiltin {
            builtin: ir::BuiltinCall::GetContractScript,
            ..
        } = instr
        {
            saw_get_contract_script = true;
        }
    }

    assert!(
        saw_get_contract_script,
        "expected address.code to use ContractManagement.getContract via GetContractScript"
    );
}

#[test]
fn iterator_currentkey_and_value_extract_storage_pair_elements() {
    use num_bigint::BigInt;
    use num_traits::{One, Zero};

    let source = r#"
    pragma solidity ^0.8.19;

    contract IteratorHarness {
        function read(bytes memory prefix) public returns (bytes memory key, bytes memory value) {
            bytes memory it = Storage.find(prefix);
            if (it.next()) {
                key = it.currentKey;
                value = it.value();
            }
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let read_fn = module
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("expected read function");
    let instrs = &read_fn.basic_blocks[0].instructions;

    let iterator_value_calls = instrs
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::CallBuiltin { builtin: ir::BuiltinCall::Syscall(name), .. }
                    if name == "System.Iterator.Value"
            )
        })
        .count();
    assert!(
        iterator_value_calls >= 2,
        "expected iterator.currentKey and iterator.value() to use System.Iterator.Value"
    );

    let mut saw_key_extract = false;
    let mut saw_value_extract = false;
    for window in instrs.windows(2) {
        match window {
            [ir::Instruction::PushLiteral(ir::LiteralValue::Integer(n)), ir::Instruction::ArrayGet]
                if n.is_zero() =>
            {
                saw_key_extract = true
            }
            [ir::Instruction::PushLiteral(ir::LiteralValue::Integer(n)), ir::Instruction::ArrayGet]
                if n == &BigInt::one() =>
            {
                saw_value_extract = true
            }
            _ => {}
        }
    }

    assert!(saw_key_extract, "expected PICKITEM(0) for currentKey");
    assert!(saw_value_extract, "expected PICKITEM(1) for value()");
}
