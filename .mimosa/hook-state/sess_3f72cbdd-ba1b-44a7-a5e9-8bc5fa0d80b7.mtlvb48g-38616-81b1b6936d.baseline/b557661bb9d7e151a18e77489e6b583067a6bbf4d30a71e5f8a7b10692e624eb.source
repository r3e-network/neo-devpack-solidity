//! IR (Intermediate Representation) unit tests.
//!
//! Tests IR generation through the public compilation API.

use neo_devpack_solidity::cli::compile_contracts;

#[test]
fn ir_compiles_simple_contract() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Simple {
        uint256 public value;
        function setValue(uint256 v) public {
            value = v;
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    assert!(result.is_ok(), "Compilation should succeed");
}

#[test]
fn ir_compiles_contract_with_mapping() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Mapping {
        mapping(address => uint256) public balances;
        function set(address a, uint256 v) public {
            balances[a] = v;
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    assert!(result.is_ok(), "Mapping contract should compile");
}

#[test]
fn ir_compiles_contract_with_events() {
    let source = r#"
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;
    contract Events {
        event Transfer(address indexed from, address indexed to, uint256 value);
        function emit_event() public {
            emit Transfer(msg.sender, address(0), 100);
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    assert!(result.is_ok(), "Event contract should compile");
}
