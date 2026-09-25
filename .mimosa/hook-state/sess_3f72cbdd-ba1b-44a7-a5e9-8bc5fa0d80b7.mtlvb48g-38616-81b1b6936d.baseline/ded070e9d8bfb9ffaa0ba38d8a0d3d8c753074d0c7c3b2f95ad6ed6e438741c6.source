//! Common test utilities for CLI tests
//!
//! This module provides shared helper functions and utilities for testing
//! the neo-devpack-solidity compiler. It reduces code duplication across test files
//! and provides a consistent testing interface.

#![allow(dead_code)]

use neo_devpack_solidity::ir;
use neo_devpack_solidity::runtime::{ExecutionResult, NeoRuntime, RuntimeConfig};

/// Execute bytecode and return the execution result.
///
/// # Arguments
/// * `bytecode` - The compiled bytecode to execute
///
/// # Returns
/// The execution result containing return data, gas used, and any exceptions
pub fn execute_bytecode(bytecode: &[u8]) -> ExecutionResult {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    runtime.execute(bytecode, &[]).expect("execution")
}

/// Execute bytecode with method tokens for CALLT instruction support.
///
/// # Arguments
/// * `bytecode` - The compiled bytecode to execute
/// * `tokens` - Method tokens for native contract calls
///
/// # Returns
/// The execution result
pub fn execute_bytecode_with_tokens(
    bytecode: &[u8],
    tokens: &[neo_devpack_solidity::neo::MethodToken],
) -> ExecutionResult {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    runtime
        .execute_with_tokens(bytecode, &[], tokens)
        .expect("execution")
}

/// Execute bytecode with input data.
///
/// # Arguments
/// * `bytecode` - The compiled bytecode to execute
/// * `input` - Input data to pass to the contract
///
/// # Returns
/// The execution result
pub fn execute_bytecode_with_input(bytecode: &[u8], input: &[u8]) -> ExecutionResult {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    runtime.execute(bytecode, input).expect("execution")
}

/// Find a function in an IR module by name.
///
/// # Arguments
/// * `module` - The IR module to search
/// * `name` - The function name to find
///
/// # Returns
/// Reference to the matching function
///
/// # Panics
/// Panics if no function with the given name is found
pub fn find_function<'a>(module: &'a ir::Module, name: &str) -> &'a ir::Function {
    module
        .functions
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("function '{name}' not found"))
}

/// Get all instructions from a function's basic blocks.
///
/// # Arguments
/// * `function` - The IR function
///
/// # Returns
/// Vector of references to all instructions
pub fn get_all_instructions(function: &ir::Function) -> Vec<&ir::Instruction> {
    function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect()
}

/// Check if any instruction matches a predicate.
///
/// # Arguments
/// * `function` - The IR function to check
/// * `predicate` - A closure that returns true for matching instructions
///
/// # Returns
/// true if any instruction matches the predicate
pub fn has_instruction<F>(function: &ir::Function, predicate: F) -> bool
where
    F: Fn(&ir::Instruction) -> bool,
{
    get_all_instructions(function).iter().any(|i| predicate(i))
}

/// Assert that execution succeeded and return data matches expected value.
///
/// # Arguments
/// * `result` - The execution result
/// * `expected` - Expected return data as i64
/// * `message` - Error message if assertion fails
pub fn assert_execution_returns(result: &ExecutionResult, expected: i64, message: &str) {
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(result.is_success(), "{message}: got {failure}");
    assert_eq!(
        result.return_data,
        expected.to_le_bytes().to_vec(),
        "{message}"
    );
}

/// Assert that execution succeeded.
///
/// # Arguments
/// * `result` - The execution result
/// * `message` - Error message if assertion fails
pub fn assert_execution_success(result: &ExecutionResult, message: &str) {
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(result.is_success(), "{message}: got {failure}");
}

/// Assert that execution failed with an exception.
///
/// # Arguments
/// * `result` - The execution result
/// * `message` - Error message if assertion fails
pub fn assert_execution_failed(result: &ExecutionResult, message: &str) {
    assert!(
        !result.is_success(),
        "{message}: expected failure but got success"
    );
}
