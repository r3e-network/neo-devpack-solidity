use super::*;

#[test]
fn parse_natspec_extracts_title() {
    let text = "@title My Token\n@author Alice";
    let doc = parse_natspec(text);
    assert_eq!(doc.title, Some("My Token".to_string()));
    assert_eq!(doc.author, Some("Alice".to_string()));
}

#[test]
fn parse_natspec_extracts_notice_and_dev() {
    let text = "@notice This is a token\n@dev Internal implementation";
    let doc = parse_natspec(text);
    assert_eq!(doc.notice, Some("This is a token".to_string()));
    assert_eq!(doc.dev, Some("Internal implementation".to_string()));
}

#[test]
fn parse_natspec_extracts_params() {
    let text = "@param to The recipient address\n@param amount The amount to send";
    let doc = parse_natspec(text);
    assert_eq!(doc.params.len(), 2);
    assert_eq!(
        doc.params[0],
        ("to".to_string(), "The recipient address".to_string())
    );
    assert_eq!(
        doc.params[1],
        ("amount".to_string(), "The amount to send".to_string())
    );
}

#[test]
fn parse_natspec_extracts_return() {
    let text = "@return The balance amount\n@return success Whether it succeeded";
    let doc = parse_natspec(text);
    assert_eq!(doc.returns.len(), 2);
    assert_eq!(doc.returns[0], "The balance amount");
    assert_eq!(doc.returns[1], "success Whether it succeeded");
}

#[test]
fn parse_natspec_extracts_custom_tags() {
    let text = "@custom:security-contact security@example.com\n@custom:version 1.0.0";
    let doc = parse_natspec(text);
    assert_eq!(doc.custom.len(), 2);
    assert_eq!(
        doc.custom[0],
        (
            "security-contact".to_string(),
            "security@example.com".to_string()
        )
    );
    assert_eq!(doc.custom[1], ("version".to_string(), "1.0.0".to_string()));
}

#[test]
fn clean_doc_comment_removes_prefixes() {
    assert_eq!(clean_doc_comment("/// Hello"), "Hello");
    assert_eq!(clean_doc_comment("/** Hello */"), "Hello");
    assert_eq!(clean_doc_comment("   * Hello"), "Hello");
    assert_eq!(clean_doc_comment("Hello"), "Hello");
}

#[test]
fn parse_source_captures_contract_doc() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Test Contract
/// @author Test Author
contract Test {
    function foo() public pure returns (uint256) {
        return 42;
    }
}
"#;
    let contracts = parse_source(source).expect("should parse");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].doc.title, Some("Test Contract".to_string()));
    assert_eq!(contracts[0].doc.author, Some("Test Author".to_string()));
}

#[test]
fn parse_source_captures_function_doc() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
    /// @notice Get the value
    /// @param x The input value
    /// @return The output value
    function getValue(uint256 x) public pure returns (uint256) {
        return x * 2;
    }
}
"#;

    let contracts = parse_source(source).expect("should parse");
    assert_eq!(contracts.len(), 1);
    let func = &contracts[0].functions[0];
    assert_eq!(func.doc.notice, Some("Get the value".to_string()));
    assert_eq!(func.doc.params.len(), 1);
    assert_eq!(func.doc.params[0].0, "x");
    assert_eq!(func.doc.returns.len(), 1);
}

#[test]
fn parse_source_rejects_unsupported_solidity_pragma() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.4.0;

contract Test {
    function value() public pure returns (uint256) {
        return 1;
    }
}
"#;

    let err = parse_source(source).expect_err("unsupported pragma should fail");
    assert!(matches!(err, FrontendError::UnsupportedVersion(_)));
}

#[test]
fn parse_source_accepts_supported_solidity_pragma() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Test {
    function value() public pure returns (uint256) {
        return 1;
    }
}
"#;

    let contracts = parse_source(source).expect("supported pragma should parse");
    assert_eq!(contracts.len(), 1);
}

#[test]
fn parse_source_accepts_mixed_range_including_0_8() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

contract Test {
    function value() public pure returns (uint256) {
        return 1;
    }
}
"#;

    let contracts = parse_source(source).expect("0.8 range should parse");
    assert_eq!(contracts.len(), 1);
}

#[test]
fn parse_source_rejects_range_excluding_0_8() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity >=0.9.0;

contract Test {
    function value() public pure returns (uint256) {
        return 1;
    }
}
"#;

    let err = parse_source(source).expect_err("0.9-only range should fail");
    assert!(matches!(err, FrontendError::UnsupportedVersion(_)));
}

#[test]
fn parse_source_rejects_disjoint_or_without_0_8() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.4.0 || ^0.9.0;

contract Test {
    function value() public pure returns (uint256) {
        return 1;
    }
}
"#;

    let err = parse_source(source).expect_err("OR branches without 0.8 should fail");
    assert!(matches!(err, FrontendError::UnsupportedVersion(_)));
}

#[test]
fn parse_source_tracks_using_directives_with_targets_and_function_lists() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

library MathLib {
    function add(uint256 self, uint256 rhs) internal pure returns (uint256) {
        return self + rhs;
    }
}

contract UsesUsingDirectives {
    using MathLib for uint256;
    using {MathLib.add} for *;
}
"#;

    let contracts = parse_source(source).expect("using directives should parse");
    let contract = contracts
        .iter()
        .find(|contract| contract.name == "UsesUsingDirectives")
        .expect("expected target contract");

    assert!(contract.has_using_for_star);
    assert!(contract.has_using_function_list);
    assert_eq!(contract.using_directives.len(), 2);

    assert!(contract.using_directives.iter().any(|directive| {
        directive.target_type.as_deref() == Some("uint256") && directive.function_names.is_none()
    }));

    assert!(contract.using_directives.iter().any(|directive| {
        directive.target_type.is_none()
            && directive
                .function_names
                .as_ref()
                .is_some_and(|names| names.iter().any(|name| name == "add"))
    }));
}

#[test]
fn parse_source_combines_multi_file_pragmas_via_intersection() {
    // Regression: when imports are concatenated, the cumulative pragma min
    // must be the MAX (intersection of constraints) — not the MIN. OpenZeppelin
    // ships many helper files with `>=0.4.16`, while feature-using files
    // declare `^0.8.20`. Taking the min would let the broader pragma
    // illegally widen the effective range and disable feature gates the entry
    // contract requires (e.g. `string.concat`, introduced in 0.8.12).
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity >=0.4.16;
pragma solidity ^0.8.20;

contract Test {
    function concat() public pure returns (string memory) {
        return string.concat("a", "b");
    }
}
"#;

    let contracts = parse_source(source).expect("string.concat should be allowed under 0.8.20");
    assert_eq!(contracts.len(), 1);
}

#[test]
fn unsupported_construct_variant_is_loud() {
    let err = FrontendError::UnsupportedConstruct("StrawmanVariant".into());
    let msg = err.to_string();
    assert!(
        msg.contains("StrawmanVariant"),
        "UnsupportedConstruct must surface the kind in its message; got: {msg}"
    );
    assert!(
        !err.is_recoverable(),
        "UnsupportedConstruct must be a hard error so silent drops are impossible"
    );
}
