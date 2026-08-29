//! Regression tests for the `events-native` gap:
//!
//! 1. NEP-17 / NEP-11 standard `Transfer` events are emitted in NATIVE Neo
//!    notification shape (`Notify("Transfer", [from, to, amount(, tokenId)])`,
//!    no EVM topic0, zero address mapped to Null) and declared natively in
//!    the manifest (`from: Hash160, to: Hash160, amount: Integer(, tokenId)`).
//! 2. Anonymous events Notify under their DECLARED name (the previous empty
//!    `""` name faults on Neo nodes >= 3.6) and the manifest declares their
//!    wire shape `[indexed..., data]` (no topic0 slot).
//! 3. Non-standard events keep the EVM log shape and the manifest declares
//!    that wire shape truthfully (`[topic0, indexed..., data]`, ByteArray).
//! 4. Emits that cannot match a declared manifest event (undeclared event,
//!    arg-count mismatch) are compile errors instead of faulting on-chain.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use serde_json::Value;

/// Decode the emulator's record of a native notification: the legacy Notify
/// path stores `topics = [eventName]` and `data` = serde-JSON of the state
/// array in the tagged `StackItem` encoding.
fn decode_state(data: &[u8]) -> Vec<Value> {
    let value: Value = serde_json::from_slice(data).unwrap_or_else(|e| {
        panic!(
            "native notification data must be JSON: {e}; raw=0x{}",
            hex::encode(data)
        )
    });
    assert_eq!(value["type"], "Array", "state must be an Array: {value}");
    value["value"].as_array().expect("state array").clone()
}

fn state_bytes(item: &Value) -> Vec<u8> {
    assert_eq!(item["type"], "ByteArray", "expected ByteArray, got {item}");
    item["value"]
        .as_array()
        .expect("bytes")
        .iter()
        .map(|b| b.as_u64().expect("byte") as u8)
        .collect()
}

fn state_int(item: &Value) -> i64 {
    assert_eq!(item["type"], "Integer", "expected Integer, got {item}");
    item["value"].as_i64().expect("int")
}

fn is_null(item: &Value) -> bool {
    item["type"] == "Null"
}

const NEP17_LIKE: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Token {
    event Transfer(address indexed from, address indexed to, uint256 amount);

    function mint(address to, uint256 amount) external {
        emit Transfer(address(0), to, amount);
    }

    function burn(address from, uint256 amount) external {
        emit Transfer(from, address(0), amount);
    }

    function move(address from, address to, uint256 amount) external {
        emit Transfer(from, to, amount);
    }
}"#;

fn invoke(method: &str, args: &[StackItem]) -> neo_devpack_solidity::runtime::ExecutionResult {
    let arts = compile_contracts(NEP17_LIKE, false, 2).expect("compile");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    rt.call_method(
        &arts[0].bytecode,
        &arts[0].tokens,
        &arts[0].manifest,
        method,
        args,
    )
    .expect("invoke")
}

#[test]
fn nep17_transfer_mint_maps_zero_from_to_null() {
    let to = [0x22u8; 20];
    let r = invoke(
        "mint",
        &[StackItem::byte_array(to.to_vec()), StackItem::Integer(1000)],
    );
    assert!(r.success, "mint: {:?}", r.exception.map(|e| e.message));
    assert_eq!(r.logs.len(), 1, "exactly one Transfer log");
    let log = &r.logs[0];
    assert_eq!(&log.topics[0][..], b"Transfer" as &[u8]);
    let state = decode_state(&log.data);
    assert_eq!(
        state.len(),
        3,
        "native state must be [from, to, amount]: {state:?}"
    );
    assert!(
        is_null(&state[0]),
        "mint: `from` = address(0) must map to Null, got {:?}",
        state[0]
    );
    assert_eq!(
        state_bytes(&state[1]),
        to.to_vec(),
        "mint: `to` 20-byte UInt160"
    );
    assert_eq!(state_int(&state[2]), 1000, "mint: amount Integer");
}

#[test]
fn nep17_transfer_burn_maps_zero_to_to_null() {
    let from = [0x33u8; 20];
    let r = invoke(
        "burn",
        &[StackItem::byte_array(from.to_vec()), StackItem::Integer(7)],
    );
    assert!(r.success, "burn: {:?}", r.exception.map(|e| e.message));
    let state = decode_state(&r.logs[0].data);
    assert_eq!(
        state_bytes(&state[0]),
        from.to_vec(),
        "burn: `from` verbatim"
    );
    assert!(
        is_null(&state[1]),
        "burn: `to` = address(0) must map to Null, got {:?}",
        state[1]
    );
    assert_eq!(state_int(&state[2]), 7);
}

#[test]
fn nep17_transfer_keeps_nonzero_addresses_verbatim() {
    let from = [0x44u8; 20];
    let to = [0x55u8; 20];
    let r = invoke(
        "move",
        &[
            StackItem::byte_array(from.to_vec()),
            StackItem::byte_array(to.to_vec()),
            StackItem::Integer(123),
        ],
    );
    assert!(r.success, "move: {:?}", r.exception.map(|e| e.message));
    let state = decode_state(&r.logs[0].data);
    assert_eq!(state_bytes(&state[0]), from.to_vec());
    assert_eq!(state_bytes(&state[1]), to.to_vec());
    assert_eq!(state_int(&state[2]), 123);
}

#[test]
fn nep17_transfer_manifest_declares_native_shape() {
    let arts = compile_contracts(NEP17_LIKE, false, 2).expect("compile");
    let events = arts[0].manifest["abi"]["events"]
        .as_array()
        .expect("events");
    let transfer = events
        .iter()
        .find(|e| e["name"] == "Transfer")
        .expect("Transfer declared");
    let params = transfer["parameters"].as_array().expect("params");
    assert_eq!(params.len(), 3);
    assert_eq!(params[0]["name"], "from");
    assert_eq!(params[0]["type"], "Hash160");
    assert_eq!(params[1]["name"], "to");
    assert_eq!(params[1]["type"], "Hash160");
    assert_eq!(params[2]["name"], "amount");
    assert_eq!(params[2]["type"], "Integer");
}

#[test]
fn nep11_transfer_with_bytes32_token_id_is_native() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract NFT {
    event Transfer(address indexed from, address indexed to, uint256 amount, bytes32 tokenId);
    function mint(address to, bytes32 tokenId) external {
        emit Transfer(address(0), to, 1, tokenId);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");

    // Manifest declares the native 4-param shape.
    let events = arts[0].manifest["abi"]["events"]
        .as_array()
        .expect("events");
    let transfer = events
        .iter()
        .find(|e| e["name"] == "Transfer")
        .expect("Transfer declared");
    let params = transfer["parameters"].as_array().expect("params");
    assert_eq!(
        params.len(),
        4,
        "native NEP-11 shape: from, to, amount, tokenId"
    );
    assert_eq!(params[0]["type"], "Hash160");
    assert_eq!(params[1]["type"], "Hash160");
    assert_eq!(params[2]["type"], "Integer");
    assert_eq!(params[3]["type"], "Hash256"); // bytes32 tokenId

    // Wire shape: [Null(from=0), to, 1, tokenId].
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let to = [0x66u8; 20];
    let token = [0xABu8; 32];
    let r = rt
        .call_method(
            &arts[0].bytecode,
            &arts[0].tokens,
            &arts[0].manifest,
            "mint",
            &[
                StackItem::byte_array(to.to_vec()),
                StackItem::byte_array(token.to_vec()),
            ],
        )
        .expect("mint");
    assert!(r.success, "mint: {:?}", r.exception.map(|e| e.message));
    assert_eq!(r.logs.len(), 1);
    assert_eq!(&r.logs[0].topics[0][..], b"Transfer" as &[u8]);
    let state = decode_state(&r.logs[0].data);
    assert_eq!(state.len(), 4, "native NEP-11 state: {state:?}");
    assert!(
        is_null(&state[0]),
        "mint from must be Null, got {:?}",
        state[0]
    );
    assert_eq!(state_bytes(&state[1]), to.to_vec());
    assert_eq!(state_int(&state[2]), 1);
    assert_eq!(state_bytes(&state[3]), token.to_vec(), "tokenId verbatim");
}

#[test]
fn erc721_style_transfer_with_uint256_token_id_is_native_nep17_shape() {
    // ERC-721's `Transfer(address,address,uint256 tokenId)` is signature-
    // identical to NEP-17's Transfer (indexed-ness is ignored), so it is
    // emitted natively too — pinned here so the equivalence is deliberate.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    function go(address to) external {
        emit Transfer(address(0), to, 5);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let events = arts[0].manifest["abi"]["events"]
        .as_array()
        .expect("events");
    let transfer = events
        .iter()
        .find(|e| e["name"] == "Transfer")
        .expect("Transfer declared");
    let params = transfer["parameters"].as_array().expect("params");
    assert_eq!(params.len(), 3, "native shape [from, to, tokenId]");
    assert_eq!(params[0]["type"], "Hash160");
    assert_eq!(params[1]["type"], "Hash160");
    assert_eq!(params[2]["type"], "Integer");
}

#[test]
fn non_standard_transfer_keeps_evm_shape() {
    // amount is uint128 — NOT the NEP-17 `uint256` signature, so the EVM
    // log shape is preserved and the manifest declares the wire shape.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Transfer(address indexed from, address indexed to, uint128 amount);
    function go(address to) external {
        emit Transfer(address(0), to, 5);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let events = arts[0].manifest["abi"]["events"]
        .as_array()
        .expect("events");
    let transfer = events
        .iter()
        .find(|e| e["name"] == "Transfer")
        .expect("Transfer declared");
    let params = transfer["parameters"].as_array().expect("params");
    // EVM wire shape: topic0 + 2 indexed + data.
    assert_eq!(
        params.len(),
        4,
        "EVM wire shape [topic0, from, to, data]: {params:?}"
    );
    assert_eq!(params[0]["name"], "topic0");
    assert_eq!(params[3]["name"], "data");
    assert!(params.iter().all(|p| p["type"] == "ByteArray"));

    // Runtime: EVM LogEntry reconstruction (3 topics: sig + 2 indexed).
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &arts[0].bytecode,
            &arts[0].tokens,
            &arts[0].manifest,
            "go",
            &[StackItem::byte_array(vec![0x77u8; 20])],
        )
        .expect("go");
    assert!(r.success, "go: {:?}", r.exception.map(|e| e.message));
    assert_eq!(r.logs.len(), 1);
    let log = &r.logs[0];
    assert_eq!(log.topics.len(), 3, "EVM shape: sig + 2 indexed topics");
    use sha3::Digest;
    let sig = sha3::Keccak256::digest(b"Transfer(address,address,uint128)");
    assert_eq!(&log.topics[0][..], &sig[..], "topic0 = keccak of signature");
}

#[test]
fn anonymous_event_notifies_with_declared_name() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event Ping(address indexed who, uint256 value) anonymous;
    function go() external {
        emit Ping(address(0x01), 9);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");

    // Manifest declares the anonymous wire shape: [who, data] (no topic0).
    let events = arts[0].manifest["abi"]["events"]
        .as_array()
        .expect("events");
    let ping = events
        .iter()
        .find(|e| e["name"] == "Ping")
        .expect("Ping declared");
    let params = ping["parameters"].as_array().expect("params");
    assert_eq!(
        params.len(),
        2,
        "anonymous wire shape [who, data]: {params:?}"
    );
    assert_eq!(params[0]["name"], "who");
    assert_eq!(params[0]["type"], "ByteArray");
    assert_eq!(params[1]["name"], "data");

    // The Notify event name must be the declared name (an empty name faults
    // on Neo >= 3.6). The bytecode must therefore contain the "Ping" string
    // and the emulator must record one log whose EVM topics are just the
    // indexed slot (no signature-hash topic0).
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
    let r = rt
        .call_method(
            &arts[0].bytecode,
            &arts[0].tokens,
            &arts[0].manifest,
            "go",
            &[] as &[StackItem],
        )
        .expect("go");
    assert!(r.success, "go: {:?}", r.exception.map(|e| e.message));
    assert_eq!(r.logs.len(), 1);
    let log = &r.logs[0];
    use sha3::Digest;
    let sig = sha3::Keccak256::digest(b"Ping(address,uint256)");
    assert!(
        log.topics.iter().all(|t| t[..] != sig[..]),
        "anonymous event must not carry the signature-hash topic0"
    );
}

#[test]
fn emit_of_undeclared_event_is_compile_error() {
    // `emit` with a wrong arg count cannot match the manifest declaration
    // and would fault on post-Basilisk nodes — must be a compile error.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event E(uint256 a, uint256 b);
    function go() external {
        emit E(1);
    }
}"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_err(),
        "emit with mismatched arg count must fail compilation, got {result:?}"
    );
}

#[test]
fn declared_nep17_with_wrong_transfer_types_is_manifest_error() {
    // Explicitly declared standards are hard-validated: a Transfer event
    // with non-NEP types will not be emitted natively, so the declared
    // standard would be a lie.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
/// @custom:neo.manifest.supportedstandards ["NEP-17"]
contract Bad {
    event Transfer(address indexed from, address indexed to, uint8 amount);
    function symbol() external pure returns (string memory) { return "B"; }
    function decimals() external pure returns (uint8) { return 0; }
    function totalSupply() external pure returns (uint256) { return 0; }
    function balanceOf(address) external pure returns (uint256) { return 0; }
    function transfer(address, address, uint256, bytes memory) external pure returns (bool) { return true; }
}"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_err(),
        "declared NEP-17 with non-native Transfer types must fail, got {result:?}"
    );
}

#[test]
fn unresolved_member_call_is_compile_error_not_silent_zero() {
    // A typo'd / missing member function must be a hard compile error, not a
    // silent fallback that drops the arguments and returns 0.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint256 x) public pure returns (uint256) {
        return x.frobnicate();
    }
}"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_err(),
        "an unresolved member call must fail compilation, got {result:?}"
    );
}

#[test]
fn fixed_size_array_event_param_is_not_misclassified_as_integer() {
    // `uint256[3]` ends with `]` but not `[]`; it must classify as Array, not
    // Integer, so emitting a fixed-array event param does not raise a spurious
    // "expected Integer, got Array" type error.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event E(uint256[3] vals);
    function go() external {
        uint256[3] memory a;
        a[0] = 1; a[1] = 2; a[2] = 3;
        emit E(a);
    }
}"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_ok(),
        "fixed-size-array event param must compile, got {result:?}"
    );
}

#[test]
fn indexed_dynamic_array_event_param_compiles() {
    // An indexed dynamic-array param must hash keccak256(abi.encode(value)),
    // not the raw Array stack item (which a real node's CryptoLib.keccak256
    // rejects). At minimum it must compile and lower.
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    event E(uint256[] indexed vals, uint256 x);
    function go() external {
        uint256[] memory a = new uint256[](2);
        a[0] = 1; a[1] = 2;
        emit E(a, 7);
    }
}"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_ok(),
        "indexed dynamic-array event param must compile, got {result:?}"
    );
}

#[test]
fn return_tuple_count_mismatch_is_compile_error() {
    // `return (1, 2)` from a single-return function is a definite arity
    // mismatch (solc rejects it) — must now be a hard error, not a warning.
    let two_for_one = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() public pure returns (uint256) { return (1, 2); }
}"#;
    assert!(
        compile_contracts(two_for_one, false, 2).is_err(),
        "returning a 2-tuple where 1 value is declared must fail compilation"
    );

    // But `return;` with a NAMED return is valid Solidity and must still
    // compile (the named return's current value is returned).
    let named_return = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f() public pure returns (uint256 x) { x = 5; return; }
}"#;
    assert!(
        compile_contracts(named_return, false, 2).is_ok(),
        "`return;` with a named return must compile"
    );
}

#[test]
fn same_shape_struct_overloads_flagged_as_selector_collision() {
    // struct A{uint256 x} and struct B{uint256 y} both canonicalize to the ABI
    // tuple (uint256), so f(A) and f(B) share a 4-byte selector and collide
    // on-chain — solc rejects this; a name-keyed check (f(A) != f(B)) missed it.
    let collide = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct A { uint256 x; }
    struct B { uint256 y; }
    function f(A memory a) public pure returns (uint256) { return a.x; }
    function f(B memory b) public pure returns (uint256) { return b.y; }
}"#;
    assert!(
        compile_contracts(collide, false, 2).is_err(),
        "same-shape struct overloads must be flagged as a selector collision"
    );

    // Distinct-shape struct overloads have distinct selectors -> still compile.
    let ok = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    struct A { uint256 x; }
    struct B { address y; }
    function f(A memory a) public pure returns (uint256) { return a.x; }
    function f(B memory b) public pure returns (uint256) { b; return 0; }
}"#;
    assert!(
        compile_contracts(ok, false, 2).is_ok(),
        "distinct-shape struct overloads must compile, got err"
    );
}
