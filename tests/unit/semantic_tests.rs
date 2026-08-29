//! Semantic analyzer unit tests.
//!
//! Tests semantic analysis through compilation.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_variable_scoping() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Scoping {
        function test() public pure {
            uint256 x = 1;
            { uint256 y = x; }
        }
    }
    "#;
    assert!(compile_contracts(source, false, 0).is_ok());
}

#[test]
fn test_function_validation() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Funcs {
        function f(uint256 a, uint256 b) public pure returns (uint256, uint256) {
            return (a, b);
        }
    }
    "#;
    assert!(compile_contracts(source, false, 0).is_ok());
}
