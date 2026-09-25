use neo_devpack_solidity::runtime::execution::{ExecutionContext, StackItem};
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

fn syscall_bytes(name: &str) -> [u8; 4] {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    [digest[0], digest[1], digest[2], digest[3]]
}

fn append_syscall(bytecode: &mut Vec<u8>, name: &str) {
    bytecode.push(0x41);
    bytecode.extend_from_slice(&syscall_bytes(name));
}

// Ledger native contract hash in NeoVM stack byte order (UInt160 little-endian).
const LEDGER_HASH_LE: [u8; 20] = [
    0xBE, 0xF2, 0x04, 0x31, 0x40, 0x36, 0x2A, 0x77, 0xC1, 0x50, 0x99, 0xC7, 0xE6, 0x4C, 0x12, 0xF7,
    0x00, 0xB6, 0x65, 0xDA,
];

fn append_ledger_current_index(bytecode: &mut Vec<u8>) {
    // Ledger.currentIndex() via System.Contract.Call
    bytecode.extend_from_slice(&[
        // Stack order for System.Contract.Call: [args, flags, method, hash]
        0x10, // PUSH0
        0xC0, // PACK -> []
        0x1F, // CallFlags.All (0x0F)
        0x0C,
        0x0C,
        b'c',
        b'u',
        b'r',
        b'r',
        b'e',
        b'n',
        b't',
        b'I',
        b'n',
        b'd',
        b'e',
        b'x',
        0x0C,
        0x14,
        LEDGER_HASH_LE[0],
        LEDGER_HASH_LE[1],
        LEDGER_HASH_LE[2],
        LEDGER_HASH_LE[3],
        LEDGER_HASH_LE[4],
        LEDGER_HASH_LE[5],
        LEDGER_HASH_LE[6],
        LEDGER_HASH_LE[7],
        LEDGER_HASH_LE[8],
        LEDGER_HASH_LE[9],
        LEDGER_HASH_LE[10],
        LEDGER_HASH_LE[11],
        LEDGER_HASH_LE[12],
        LEDGER_HASH_LE[13],
        LEDGER_HASH_LE[14],
        LEDGER_HASH_LE[15],
        LEDGER_HASH_LE[16],
        LEDGER_HASH_LE[17],
        LEDGER_HASH_LE[18],
        LEDGER_HASH_LE[19],
    ]);
    append_syscall(bytecode, "System.Contract.Call");
}

#[test]
fn execution_context_reports_default_metadata() {
    let config = RuntimeConfig {
        default_block_height: 1_234,
        default_timestamp: 9_876,
        contract_account: "0x1122334455667788990011223344556677889900".to_string(),
        ..RuntimeConfig::default()
    };

    let mut context = ExecutionContext::new(&config).expect("context creation");
    let mut script = Vec::new();
    append_ledger_current_index(&mut script);
    append_syscall(&mut script, "System.Runtime.GetTime");
    append_syscall(&mut script, "System.Runtime.GetCallingScriptHash");
    append_syscall(&mut script, "System.Runtime.GetInvocationCounter");
    append_syscall(&mut script, "System.Runtime.GetInvocationCounter");
    script.push(0x40); // RET

    context
        .initialize(&script, &[])
        .expect("context initialization");

    loop {
        let step = context.step().expect("execution step");
        if step.halted {
            break;
        }
    }

    match context.pop_stack().expect("second invocation counter") {
        StackItem::UnsignedInteger(value) => assert_eq!(value, 1),
        other => panic!("expected second invocation counter, got {other:?}"),
    }

    match context.pop_stack().expect("first invocation counter") {
        StackItem::UnsignedInteger(value) => assert_eq!(value, 1),
        other => panic!("expected first invocation counter, got {other:?}"),
    }

    match context.pop_stack().expect("calling script hash") {
        StackItem::ByteArray { data: bytes, .. } => {
            // With no caller override, the entry frame's caller is the
            // transaction's entry script — a DISTINCT identity from the
            // executing contract's own hash (see `entry_script_hash`). So
            // GetCallingScriptHash returns `entry_script_hash`, not the
            // configured `contract_account`. This split is what lets a nested
            // call's `msg.sender` (the caller's executing hash) diverge from
            // `tx.origin` (which stays pinned to this entry caller).
            let expected = context.entry_script_hash();
            assert_eq!(bytes.borrow().as_slice(), expected.as_slice());
        }
        other => panic!("expected calling script hash bytes, got {other:?}"),
    }

    match context.pop_stack().expect("timestamp") {
        StackItem::UnsignedInteger(value) => assert_eq!(value, config.default_timestamp),
        other => panic!("expected timestamp, got {other:?}"),
    }

    match context.pop_stack().expect("block height") {
        StackItem::UnsignedInteger(value) => assert_eq!(value, config.default_block_height),
        other => panic!("expected block height, got {other:?}"),
    }

    assert_eq!(context.stack_depth(), 0);
}

#[test]
fn execution_context_applies_metadata_overrides() {
    let mut context = ExecutionContext::new(&RuntimeConfig::default()).expect("context creation");
    context.override_block_height(77);
    context.override_timestamp(1_111);
    context
        .override_caller_account("0x0102030405060708090a0102030405060708090a")
        .expect("caller override");

    let mut script = Vec::new();
    append_ledger_current_index(&mut script);
    append_syscall(&mut script, "System.Runtime.GetTime");
    append_syscall(&mut script, "System.Runtime.GetCallingScriptHash");
    script.push(0x40); // RET

    context
        .initialize(&script, &[])
        .expect("context initialization");

    loop {
        let step = context.step().expect("execution step");
        if step.halted {
            break;
        }
    }

    match context.pop_stack().expect("caller hash") {
        StackItem::ByteArray { data: bytes, .. } => {
            let mut expected =
                hex::decode("0102030405060708090a0102030405060708090a").expect("valid hex");
            expected.reverse();
            assert_eq!(bytes.borrow().as_slice(), expected.as_slice());
        }
        other => panic!("expected caller script hash, got {other:?}"),
    }

    match context.pop_stack().expect("timestamp") {
        StackItem::UnsignedInteger(value) => assert_eq!(value, 1_111),
        other => panic!("expected override timestamp, got {other:?}"),
    }

    match context.pop_stack().expect("block height") {
        StackItem::UnsignedInteger(value) => assert_eq!(value, 77),
        other => panic!("expected override block height, got {other:?}"),
    }

    assert_eq!(context.stack_depth(), 0);
}
