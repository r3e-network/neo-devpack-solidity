//! Regression tests for function-selector ABI canonicalization of struct, enum
//! and integer-shorthand parameters.
//!
//! The EVM function selector is `keccak256(name(t1,t2,...))[..4]` where each
//! parameter type uses its ABI-canonical spelling: a struct expands to a
//! `(field0,field1,...)` tuple, an enum is `uint8`, and integer widths are
//! explicit (`uint256`, not `uint`). Previously the manifest selector used the
//! bare struct/enum NAME (e.g. `f(S)`), producing selectors that disagree with
//! Ethereum tooling and with the contract's own `this.f.selector` literal.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use sha3::{Digest, Keccak256};

fn selector_of(src: &str, contract: &str, method: &str) -> [u8; 4] {
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == contract)
        .expect("contract artifact");
    let m = art
        .metadata
        .methods
        .iter()
        .find(|m| m.name == method)
        .unwrap_or_else(|| panic!("method {method} not found"));
    m.selector
}

fn keccak4(sig: &str) -> [u8; 4] {
    let mut h = Keccak256::new();
    h.update(sig.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

#[test]
fn struct_param_selector_expands_to_tuple() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    struct S { uint256 id; address owner; }
    function f(S calldata s) external pure returns (uint256) { return s.id; }
}"#;
    assert_eq!(
        selector_of(src, "C", "f"),
        keccak4("f((uint256,address))"),
        "struct param must expand to (uint256,address) in the selector signature"
    );
}

#[test]
fn nested_struct_param_selector_expands_recursively() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    struct Inner { uint128 a; bool b; }
    struct Outer { Inner inner; address who; }
    function g(Outer calldata o) external pure returns (bool) { return o.inner.b; }
}"#;
    assert_eq!(
        selector_of(src, "C", "g"),
        keccak4("g(((uint128,bool),address))"),
        "nested struct must expand recursively"
    );
}

#[test]
fn enum_param_selector_is_uint8() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    enum Color { Red, Green, Blue }
    function h(Color c) external pure returns (uint256) { return uint256(uint8(c)); }
}"#;
    assert_eq!(
        selector_of(src, "C", "h"),
        keccak4("h(uint8)"),
        "enum param must canonicalize to uint8 in the selector signature"
    );
}

#[test]
fn uint_shorthand_selector_is_uint256() {
    // `uint`/`int` shorthand must canonicalize to the explicit width.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function k(uint a, int b) external pure returns (uint) { return a; }
}"#;
    assert_eq!(
        selector_of(src, "C", "k"),
        keccak4("k(uint256,int256)"),
        "uint/int shorthand must canonicalize to uint256/int256"
    );
}

#[test]
fn plain_types_selector_unchanged() {
    // Regression guard: ordinary types keep their canonical EVM selector.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function transfer(address to, uint256 amount) external returns (bool) { return true; }
}"#;
    // keccak256("transfer(address,uint256)")[..4] == 0xa9059cbb
    assert_eq!(selector_of(src, "C", "transfer"), [0xa9, 0x05, 0x9c, 0xbb]);
}

// Event `topic0 = keccak256(name(t1,...))` must expand a struct event parameter
// to its `(field0,field1,...)` tuple, matching Ethereum log filters. The emit
// lowering pushes the 32-byte topic0 as a literal into the bytecode, so the
// canonical hash must appear there.
#[test]
fn event_topic0_expands_struct_param_to_tuple() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    struct Pt { uint256 x; address who; }
    event Moved(Pt p);
    function go() external { emit Moved(Pt(7, address(0))); }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let mut h = Keccak256::new();
    h.update(b"Moved((uint256,address))");
    let topic0 = h.finalize();
    let needle: &[u8] = &topic0[..];
    assert!(
        art.bytecode.windows(32).any(|w| w == needle),
        "event topic0 keccak(\"Moved((uint256,address))\") must be embedded as a 32-byte literal; \
         the struct param was not expanded to a tuple"
    );
}

// The `this.f.selector` LITERAL (resolved from the selector registry in
// solidity_analyse.rs) must equal the MANIFEST selector and the EVM-canonical
// keccak. This guards against the manifest and the in-contract `.selector`
// disagreeing for struct/enum/shorthand params (they are computed by separate
// code paths and must stay in lockstep, since both drive on-chain dispatch).
#[test]
fn dot_selector_literal_matches_canonical_for_struct_and_shorthand() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    struct S { uint256 id; address owner; }
    enum Color { Red, Green, Blue }
    function f(S calldata s) external pure returns (uint256) { return s.id; }
    function k(uint a, int b) external pure returns (uint) { return a; }
    function c(Color x) external pure returns (uint256) { return uint256(uint8(x)); }
    function selF() external pure returns (bytes4) { return this.f.selector; }
    function selK() external pure returns (bytes4) { return this.k.selector; }
    function selC() external pure returns (bytes4) { return this.c.selector; }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");

    for (probe, sig) in [
        ("selF", "f((uint256,address))"),
        ("selK", "k(uint256,int256)"),
        ("selC", "c(uint8)"),
    ] {
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, probe, &[])
            .unwrap_or_else(|e| panic!("{probe} call: {e:?}"));
        assert!(r.success, "{probe} must succeed; exc={:?}", r.exception);
        // bytes4 return is the 4 selector bytes (big-endian on the wire).
        let got = &r.return_data;
        assert_eq!(
            &got[..4.min(got.len())],
            &keccak4(sig)[..],
            "{probe}: this.<m>.selector literal must equal keccak(\"{sig}\")[..4]; \
             got {got:02x?}"
        );
    }
}
