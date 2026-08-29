//! Parser unit tests.
//!
//! Tests parsing through compilation.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_simple_contract() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Simple { uint256 x; }
    "#;
    assert!(compile_contracts(source, false, 0).is_ok());
}

#[test]
fn test_function_definition() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Funcs {
        function add(uint256 a, uint256 b) public pure returns (uint256) {
            return a + b;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 0).is_ok());
}
