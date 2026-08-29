//! Code generator unit tests.
//!
//! Tests code generation through compilation.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_simple_arithmetic() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Arith {
        function add(uint256 a, uint256 b) public pure returns (uint256) {
            return a + b;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_function_compilation() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Funcs {
        function addOne(uint256 x) public pure returns (uint256) {
            return x + 1;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_for_loop_compiles() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Loop {
        function sum(uint256 n) public pure returns (uint256) {
            uint256 total = 0;
            for (uint256 i = 0; i < n; i++) {
                total = total + i;
            }
            return total;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_while_loop_compiles() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Loop {
        function countdown(uint256 n) public pure returns (uint256) {
            uint256 count = 0;
            while (n > 0) {
                n--;
                count++;
            }
            return count;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_break_statement_compiles() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Loop {
        function findFirst(uint256[] memory arr, uint256 target) public pure returns (uint256) {
            for (uint256 i = 0; i < arr.length; i++) {
                if (arr[i] == target) {
                    break;
                }
            }
            return 0;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_continue_statement_compiles() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Loop {
        function countEven(uint256[] memory arr) public pure returns (uint256) {
            uint256 count = 0;
            for (uint256 i = 0; i < arr.length; i++) {
                if (arr[i] % 2 != 0) {
                    continue;
                }
                count++;
            }
            return count;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}
