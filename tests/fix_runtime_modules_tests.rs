//! Regression tests for the include!()-to-modules refactor of src/runtime.
//!
//! The refactor replaced 156 include!() fragments with proper Rust modules.
//! It must be a pure structural change: every public API path that existed
//! before must still resolve, and runtime behavior must be unchanged.

// Pin the public re-export paths (compile-time check: failure = broken API).
use neo_devpack_solidity::runtime::bridge::{VMBridge, VMBridgeError};
use neo_devpack_solidity::runtime::execution::types::StackItem as TypesStackItem;
use neo_devpack_solidity::runtime::execution::{ExecutionContext, GasTracker, StackItem};
use neo_devpack_solidity::runtime::spec::{
    native_contract_name, opcode_gas, opcode_name, syscall_gas_table, syscall_name,
    NativeContractSpec, OpcodeSpec, SyscallSpec, NATIVE_CONTRACTS, OPCODES, SYSCALLS,
};
use neo_devpack_solidity::runtime::state::{StateManager, StateSnapshot};
use neo_devpack_solidity::runtime::storage::{StorageManager, StorageQuery};
use neo_devpack_solidity::runtime::types::RuntimeValue;
use neo_devpack_solidity::runtime::{
    ExceptionType, ExecutionResult, LogEntry, NeoRuntime, RuntimeConfig, RuntimeError,
    RuntimeException, StackFrame, StateChange, StateChangeType,
};

#[test]
fn spec_registry_paths_resolve_and_behave() {
    // Items formerly defined via include!("spec/*.rs") in the spec module.
    assert!(OPCODES.len() >= 120);
    assert!(SYSCALLS.len() >= 30);
    assert!(!NATIVE_CONTRACTS.is_empty());
    assert_eq!(opcode_name(0x34), Some("CALL"));
    assert!(opcode_gas(0x34).is_some());
    assert!(!syscall_gas_table().is_empty());
    let spec: &OpcodeSpec = OPCODES.get(&0x34).unwrap();
    assert!(spec.is_call());
    let any_syscall: &SyscallSpec = SYSCALLS.values().next().unwrap();
    assert_eq!(syscall_name(&any_syscall.id), Some(any_syscall.name));
    let any_native: &NativeContractSpec = NATIVE_CONTRACTS.values().next().unwrap();
    assert_eq!(
        native_contract_name(&any_native.hash),
        Some(any_native.name)
    );
}

#[test]
fn runtime_unit_constructors_resolve() {
    // Types formerly assembled from runtime_parts/, state/, storage/, bridge/
    // and execution/ include fragments.
    let config = RuntimeConfig::default();
    let runtime: NeoRuntime = NeoRuntime::new(config.clone()).unwrap();
    let _snapshot: StateSnapshot = runtime.get_state_snapshot();
    let _ctx: ExecutionContext = ExecutionContext::new(&config).unwrap();
    let _gas: GasTracker = GasTracker::new(config.gas_limit);
    let mut state: StateManager = StateManager::new(&config).unwrap();
    let _snapshot_id: u64 = state.create_snapshot("refactor smoke".to_string());
    let _storage: StorageManager = StorageManager::new(&config).unwrap();
    let _bridge: VMBridge = VMBridge::new(&config).unwrap();
    let _query: StorageQuery = StorageQuery::default();
    let _err: Option<VMBridgeError> = None;
    let _item: StackItem = StackItem::Null;
    let _item2: TypesStackItem = TypesStackItem::Null;
    let _value = RuntimeValue::UnsignedInteger(42);
}

#[test]
fn runtime_value_and_exception_types_behave() {
    assert!(ExceptionType::Halt.is_recoverable());
    assert!(!ExceptionType::OutOfGas.is_recoverable());
    let value = RuntimeValue::UnsignedInteger(7);
    assert_eq!(RuntimeValue::from_bytes(&value.to_bytes()), value);
    // Exercise types that flow through ExecutionResult.
    let _: Option<RuntimeException> = None;
    let _: Vec<StateChange> = Vec::new();
    let _: Vec<LogEntry> = Vec::new();
    let _: Option<Vec<StackFrame>> = None;
    let _: Option<StateChangeType> = None;
    let _: Option<RuntimeError> = None;
    let _: Option<ExecutionResult> = None;
}

#[test]
fn neo_runtime_executes_after_refactor() {
    // End-to-end smoke test across the formerly include!()-built units:
    // NeoRuntime (runtime_parts) -> ExecutionContext (execution) ->
    // StorageManager (storage) -> StateManager (state).
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    // PUSH1 (0x11), RET (0x40): minimal well-formed NeoVM script.
    let result = runtime.execute(&[0x11, 0x40], &[]).unwrap();
    assert!(result.success, "trivial script should execute: {result:?}");
    assert!(result.gas_used > 0, "gas accounting should still charge");
}
