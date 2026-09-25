//! Runtime execution unit tests.
//!
//! Tests memory management, calldata, and returndata operations.

use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

#[test]
fn runtime_captures_return_payload() {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    // Push "foo" and return
    let script = vec![0x0C, 0x03, b'f', b'o', b'o', 0x40];
    let result = runtime.execute(&script, &[]).expect("execution");
    assert!(result.is_success());
    assert_eq!(result.return_data, b"foo");
}

#[test]
fn runtime_handles_empty_script() {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let script = vec![0x40]; // Just RET
    let result = runtime.execute(&script, &[]).expect("execution");
    assert!(result.is_success());
}

#[test]
fn runtime_config_default_values() {
    let config = RuntimeConfig::default();
    assert!(config.gas_limit > 0);
}
