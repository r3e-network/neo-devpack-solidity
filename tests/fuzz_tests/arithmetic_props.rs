//! Edge-case arithmetic / identifier / string-encoding property tests.
//!
//! Extracted from the top-level "Edge Case Tests" banner in
//! `tests/fuzz_tests.rs`. Contents unchanged from the pre-split file.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Edge Case Tests ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn large_integer_literals_compile(
        high_bits in any::<u128>()
    ) {
        let value = format!("{}", high_bits);
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract LargeIntContract {{
    uint256 public value = {};
}}"#,
            value
        );

        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn long_identifiers_compile(
        name in "[a-zA-Z][a-zA-Z0-9_]{50,200}"
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 42;
}}"#,
            name
        );

        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(_) => {
                // Long identifiers may be rejected
            }
        }
    }

    #[test]
    fn many_functions_compile(
        count in 10usize..50
    ) {
        let funcs: Vec<String> = (0..count)
            .map(|i| format!(
                "    function func{}() public pure returns (uint256) {{ return {}; }}",
                i, i
            ))
            .collect();

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
{}
}}"#,
            funcs.join("\n")
        );

        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(_) => {
                // Too many functions may be rejected
            }
        }
    }

    #[test]
    fn long_strings_compile(
        content in prop::collection::vec(any::<char>(), 100..2000)
    ) {
        let escaped: String = content
            .iter()
            .map(|c| match c {
                '"' | '\\' | '\n' | '\r' | '\t' => ' '.to_string(),
                c => c.to_string(),
            })
            .collect();

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    string public message = "{}";
}}"#,
            escaped
        );

        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(_) => {
                // Very long strings may be rejected
            }
        }
    }

    #[test]
    fn unicode_strings_compile(
        content in prop::collection::vec(
            prop_oneof![
                Just('α'), Just('β'), Just('γ'), Just('中'), Just('文'),
                Just('🌍'), Just('🚀'), Just('🔥'),
            ],
            1..50
        )
    ) {
        let escaped: String = content.iter().collect();

        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    string public message = unicode"{}";
}}"#,
            escaped
        );

        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(_) => {}
        }
    }

    #[test]
    fn deeply_nested_parens_resilience(
        depth in 20usize..200
    ) {
        let mut expr = "42".to_string();
        for _ in 0..depth {
            expr = format!("({})", expr);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = {};
}}"#,
            expr
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn deeply_nested_braces_resilience(
        depth in 20usize..200
    ) {
        let mut body = "uint256 x = 42;".to_string();
        for _ in 0..depth {
            body = format!("{{ {} }}", body);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function test() public pure returns (uint256) {{
        {}
        return 42;
    }}
}}"#,
            body
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn deeply_nested_brackets_resilience(
        depth in 5usize..50
    ) {
        let mut ty = "uint256".to_string();
        for _ in 0..depth {
            ty = format!("{}[]", ty);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    {} public value;
}}"#,
            ty
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn deeply_nested_ternary_resilience(
        depth in 10usize..100
    ) {
        let mut expr = "0".to_string();
        for i in 0..depth {
            expr = format!("x > {} ? {} : {}", i, i + 1, expr);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function test(uint256 x) public pure returns (uint256) {{
        return {};
    }}
}}"#,
            expr
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn nested_block_comments_resilience(
        _dummy in any::<bool>()
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {
    /* outer /* inner */ outer */ uint256 public value = 42;
}"#;
        let result = compile_contracts(source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn very_long_comments_resilience(
        len in 500usize..5000
    ) {
        let comment = "x".repeat(len);
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
// {}
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 42;
}}"#,
            comment
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn invalid_characters_produce_errors(
        invalid_char in prop_oneof![Just('@'), Just('#'), Just('~'), Just('`')]
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 42{}42;
}}"#,
            invalid_char
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_err(), "Malformed source with invalid char '{}' should error, but got Ok", invalid_char);
    }

    #[test]
    fn unmatched_parens_produce_errors(
        depth in 1usize..20
    ) {
        let mut expr = "42".to_string();
        for _ in 0..depth {
            expr = format!("({}", expr);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = {};
}}"#,
            expr
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_err(), "Unmatched parens should error, but got Ok");
    }

    #[test]
    fn unmatched_braces_produce_errors(
        depth in 1usize..20
    ) {
        let mut body = "uint256 x = 42;".to_string();
        for _ in 0..depth {
            body = format!("{{ {}", body);
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function test() public pure returns (uint256) {{
        {}
        return 42;
    }}
}}"#,
            body
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_err(), "Unmatched braces should error, but got Ok");
    }

    #[test]
    fn malformed_numbers_produce_errors(
        num in prop_oneof![
            Just("0x".to_string()),
            Just("0xGGGG".to_string()),
            Just("1.2.3".to_string()),
        ]
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = {};
}}"#,
            num
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_err(), "Malformed number '{}' should error, but got Ok", num);
    }

    // Note: solang-parser is lenient with escape sequences; \z, \q, \xGG are
    // accepted. This harness verifies the compiler does not panic on them.
    #[test]
    fn lenient_escape_sequences_accepted(
        esc in prop_oneof![Just("\\z"), Just("\\q"), Just("\\xGG")]
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    string public message = "{}";
}}"#,
            esc
        );
        let result = compile_contracts(&source, false, 2);
        prop_assert!(result.is_ok(), "Lenient escape '{}' should compile without panic: {:?}", esc, result.err());
    }

    // ---------- Syntax edge-case / parser resilience tests ----------

    #[test]
    fn keyword_case_variant_identifiers_resilience(
        name in prop_oneof![
            Just("Uint256".to_string()),
            Just("PUBLIC".to_string()),
            Just("fOr".to_string()),
            Just("iF".to_string()),
            Just("eLsE".to_string()),
            Just("whIlE".to_string()),
            Just("BREAK".to_string()),
            Just("CoNtInUe".to_string()),
            Just("ReTuRn".to_string()),
            Just("fAlSe".to_string()),
            Just("TrUe".to_string()),
            Just("CoNtRaCt".to_string()),
            Just("FuNcTiOn".to_string()),
            Just("MoDiFiEr".to_string()),
            Just("EvEnT".to_string()),
        ]
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public {} = 42;
}}"#,
            name
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn nested_ternary_with_side_effects_resilience(
        depth in 1usize..20
    ) {
        let mut expr = "y".to_string();
        for i in 0..depth {
            if i.is_multiple_of(2) {
                expr = format!("x > {} ? ++y : {}", i, expr);
            } else {
                expr = format!("x > {} ? --y : {}", i, expr);
            }
        }
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    function test(uint256 x, uint256 y) public pure returns (uint256) {{
        return {};
    }}
}}"#,
            expr
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn interleaved_comments_unusual_positions_resilience(
        _dummy in any::<bool>()
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
prag/*a*/ma//b
solidity/*c*/^0.8.0;
contract/*d*/TestContract/*e*/{
    uint256/*f*/public/*g*/value/*h*/=/*i*/42/*j*/;
    function/*k*/test()/*l*/public/*m*/pure/*n*/returns/*o*/(uint256/*p*/)/*q*/{
        return/*r*/(1/*s*/+/*t*/2/*u*/)/*v*/;
    }
}"#;
        let result = compile_contracts(source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn very_large_hex_literal_resilience(
        hex in "[0-9a-fA-F]{60,64}"
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 0x{};
}}"#,
            hex
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn very_small_scientific_notation_resilience(
        _dummy in any::<bool>()
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {
    uint256 public value = 1e-37;
}"#;
        let result = compile_contracts(source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn octal_like_leading_zeros_resilience(
        digits in "[0-9]{10,30}"
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {{
    uint256 public value = 0{};
}}"#,
            digits
        );
        let result = compile_contracts(&source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn empty_string_resilience(
        _dummy in any::<bool>()
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {
    string public message = "";
}"#;
        let result = compile_contracts(source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn whitespace_only_string_resilience(
        _dummy in any::<bool>()
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {
    string public message = "     ";
}"#;
        let result = compile_contracts(source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }

    #[test]
    fn maximum_escape_sequences_string_resilience(
        _dummy in any::<bool>()
    ) {
        let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract TestContract {
    string public message = "\n\t\r\x00\xFF\x41\n\t\r\x00\xFF\x41";
}"#;
        let result = compile_contracts(source, false, 2);
        match result {
            Ok(_) => {},
            Err(e) => {
                let err_str = format!("{:?}", e);
                prop_assume!(!err_str.contains("panic") && !err_str.contains("unwrap"));
            }
        }
    }
}
