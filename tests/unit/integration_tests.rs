//! Integration tests for full compilation pipeline.
//!
//! Tests end-to-end compilation from Solidity source.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_erc20_like_contract() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Token {
        mapping(address => uint256) public balances;
        
        function transfer(address to, uint256 amount) public returns (bool) {
            require(balances[msg.sender] >= amount, "Insufficient");
            balances[msg.sender] -= amount;
            balances[to] += amount;
            return true;
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    assert!(result.is_ok(), "ERC20-like contract should compile");
}

#[test]
fn test_complex_control_flow() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Fibonacci {
        function fib(uint256 n) public pure returns (uint256) {
            if (n < 2) return n;
            uint256 a = 0;
            uint256 b = 1;
            for (uint256 i = 2; i <= n; i++) {
                uint256 temp = a + b;
                a = b;
                b = temp;
            }
            return b;
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    assert!(result.is_ok(), "Control flow contract should compile");
}

#[test]
fn test_optimization_levels() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Simple {
        uint256 public value;
        function set(uint256 v) public { value = v; }
    }
    "#;

    // Test different optimization levels
    for level in 0..=3 {
        let result = compile_contracts(source, false, level);
        assert!(result.is_ok(), "Should compile at O{level}");
    }
}
