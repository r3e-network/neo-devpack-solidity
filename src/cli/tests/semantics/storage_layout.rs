#[test]
fn struct_storage_and_memory_field_access_behaves_consistently() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract StructHarness {
        struct S {
            uint256 a;
            uint256 b;
        }

        mapping(address => S) private m;
        S private s;

        function run() public returns (uint256) {
            S storage ref1 = m[msg.sender];
            ref1.a = 7;
            ref1.b = 9;

            s = S({a: 1, b: 2});

            S memory tmp = m[msg.sender];
            return tmp.a + tmp.b + s.a + s.b;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected struct harness to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, 19i64.to_le_bytes().to_vec());
}

#[test]
fn nested_struct_storage_field_access_behaves_consistently() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NestedStructHarness {
        struct Inner {
            uint256 x;
            bool y;
        }

        struct Outer {
            Inner inner;
            uint256 z;
        }

        mapping(address => Outer) private m;
        Outer private s;

        function run() public returns (uint256) {
            Outer storage o = m[msg.sender];
            o.inner.x = 7;
            o.inner.y = true;
            o.z = 3;

            s.inner.x = 1;
            s.inner.y = false;
            s.z = 2;

            Outer memory tmp = m[msg.sender];
            return tmp.inner.x + (tmp.inner.y ? 1 : 0) + tmp.z
                + s.inner.x + (s.inner.y ? 1 : 0) + s.z;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected nested struct harness to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, 14i64.to_le_bytes().to_vec());
}

#[test]
fn deeply_nested_struct_storage_roundtrip_behaves_consistently() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract DeepNestedStructHarness {
        struct Leaf {
            uint256 a;
        }

        struct Inner {
            Leaf leaf;
            uint256 x;
        }

        struct Outer {
            Inner inner;
            uint256 z;
        }

        mapping(address => Outer) private m;
        Outer private s;

        function run() public returns (uint256) {
            Outer storage o = m[msg.sender];
            o.inner = Inner({ leaf: Leaf({ a: 7 }), x: 9 });
            o.z = 3;

            s = Outer({ inner: Inner({ leaf: Leaf({ a: 1 }), x: 2 }), z: 4 });

            Outer memory tmp = m[msg.sender];
            return tmp.inner.leaf.a + tmp.inner.x + tmp.z
                + s.inner.leaf.a + s.inner.x + s.z;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected deep nested struct harness to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, 26i64.to_le_bytes().to_vec());
}

#[test]
fn delete_resets_storage_values_to_defaults() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract DeleteHarness {
        mapping(bytes32 => bytes32) private mBytes;
        mapping(bytes32 => address) private mAddr;

        struct S {
            address a;
            uint256 b;
            bool c;
            bytes32 d;
        }

        mapping(bytes32 => S) private mStruct;

        function run() public returns (uint256) {
            bytes32 k1 = bytes32(uint256(1));
            bytes32 k2 = bytes32(uint256(2));

            mBytes[k1] = bytes32(uint256(9));
            mAddr[k1] = msg.sender;

            S storage s = mStruct[k1];
            s.a = msg.sender;
            s.b = 7;
            s.c = true;
            s.d = bytes32(uint256(11));

            delete mBytes[k1];
            delete mAddr[k1];
            delete mStruct[k1];

            uint256 ok = 0;

            if (mBytes[k1] == bytes32(0)) ok += 1;
            if (mAddr[k1] == address(0)) ok += 2;
            S memory afterDelete = mStruct[k1];
            if (afterDelete.a == address(0)) ok += 4;
            if (afterDelete.b == 0) ok += 8;
            if (afterDelete.c == false) ok += 16;
            if (afterDelete.d == bytes32(0)) ok += 32;

            // Missing keys should also read as defaults.
            if (mBytes[k2] == bytes32(0)) ok += 64;
            if (mAddr[k2] == address(0)) ok += 128;
            S memory missing = mStruct[k2];
            if (missing.a == address(0)) ok += 256;
            if (missing.b == 0) ok += 512;
            if (missing.c == false) ok += 1024;
            if (missing.d == bytes32(0)) ok += 2048;

            return ok;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected delete harness to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, 4095i64.to_le_bytes().to_vec());
}

#[test]
fn array_pop_throws_on_empty_using_jumpif_false_semantics() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract PopHarness {
        uint256[] public values;

        function popOnce() public returns (uint256) {
            return values.pop();
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let pop_fn = module
        .functions
        .iter()
        .find(|function| function.name == "popOnce")
        .expect("expected popOnce");

    let instrs = &pop_fn.basic_blocks[0].instructions;
    let has_guard = instrs.windows(2).any(|window| {
        matches!(
            window,
            [
                ir::Instruction::BinaryOp(ir::BinaryOperator::Ne),
                ir::Instruction::JumpIf { .. }
            ]
        )
    });

    assert!(has_guard, "expected pop lowering to use Ne + JumpIf guard");
    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::Throw)),
        "expected pop lowering to include Throw on empty array"
    );
}

#[test]
fn new_bytes_strings_and_arrays_are_zero_initialized() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface ITypes {
        struct Pair {
            uint256 a;
            bool b;
        }
    }

    contract NewAlloc is ITypes {
        function run() public pure returns (uint256) {
            bytes memory b = new bytes(4);
            if (b.length != 4) return 0;
            if (uint8(b[2]) != 0) return 0;

            string memory s = new string(3);
            if (bytes(s).length != 3) return 0;

            uint256[] memory arr = new uint256[](3);
            if (arr.length != 3) return 0;
            if (arr[1] != 0) return 0;

            Pair[] memory pairs = new Pair[](1);
            if (pairs.length != 1) return 0;
            if (pairs[0].a != 0) return 0;
            if (pairs[0].b) return 0;

            return 1;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected run() to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, 1i64.to_le_bytes().to_vec());
}
