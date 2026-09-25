//! Boundary condition tests for compiler robustness.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_empty_contract() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Empty {}
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_large_integer_literal() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract BigInt {
        uint256 constant MAX = 115792089237316195423570985008687907853269984665640564039457584007913129639935;
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_deeply_nested_expressions() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Nested {
        function deep() public pure returns (uint256) {
            return ((((1 + 2) * 3) - 4) / 5) % 6;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_many_parameters() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract ManyParams {
        function f(uint a, uint b, uint c, uint d, uint e, uint f, uint g, uint h) 
            public pure returns (uint) {
            return a + b + c + d + e + f + g + h;
        }
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}

#[test]
fn test_unicode_in_strings() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Unicode {
        string public greeting = unicode"你好世界 🌍";
    }
    "#;
    assert!(compile_contracts(source, false, 2).is_ok());
}
