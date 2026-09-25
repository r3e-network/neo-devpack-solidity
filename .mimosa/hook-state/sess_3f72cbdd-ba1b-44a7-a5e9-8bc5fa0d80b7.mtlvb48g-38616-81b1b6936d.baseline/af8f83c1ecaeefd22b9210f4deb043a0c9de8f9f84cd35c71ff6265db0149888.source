//! Lexer unit tests.
//!
//! Tests tokenization through compilation.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn test_basic_syntax_parsing() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Basic {
        uint256 public x;
    }
    "#;

    let result = compile_contracts(source, false, 0);
    assert!(result.is_ok());
}

#[test]
fn test_number_literals() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Numbers {
        uint256 constant HEX = 0x42;
        uint256 constant DEC = 123;
    }
    "#;

    let result = compile_contracts(source, false, 0);
    assert!(result.is_ok());
}

#[test]
fn test_string_literals() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Strings {
        string public hello = "hello world";
    }
    "#;

    let result = compile_contracts(source, false, 0);
    assert!(result.is_ok());
}
