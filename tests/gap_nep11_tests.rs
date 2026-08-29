//! Regression tests for the NEP-11 manifest-conformance gap (`gap key: nep11`).
//!
//! Layers covered:
//! 1. `tokenId` retyped from `bytes32` (manifest `Hash256`) to dynamic `bytes`
//!    (manifest `ByteArray`, NEP-11 spec: ByteString up to 64 bytes).
//! 2. `tokensOf` / `tokens` returning a NeoVM storage iterator declared as
//!    manifest returntype `InteropInterface` (NEP-11 spec: iterator), backed
//!    by a raw-storage token index scanned with
//!    `FindOptions.KeysOnly | FindOptions.RemovePrefix`.
//! 3. `properties` returning Map is intentionally NOT implemented (no sane
//!    Solidity-side construct produces a NeoVM Map stack item return);
//!    the devpack keeps `bytes` / manifest `ByteArray` and documents it.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::cli::CompilationArtifacts;
use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::storage::StorageManager;
use neo_devpack_solidity::runtime::{types::StackItem, NeoRuntime, RuntimeConfig};
use serde_json::Value;

const CALLER: &str = "0x1122334455667788990011223344556677889900";

fn devpack_nep11_source() -> String {
    [
        include_str!("../devpack/contracts/Syscalls.sol"),
        include_str!("../devpack/contracts/NativeContracts.sol"),
        include_str!("../devpack/contracts/NativeCalls.sol"),
        include_str!("../devpack/libraries/Storage.sol"),
        include_str!("../devpack/libraries/Runtime.sol"),
        include_str!("../devpack/libraries/Neo.sol"),
        include_str!("../devpack/contracts/FrameworkBase.sol"),
        include_str!("../devpack/standards/NEP11.sol"),
    ]
    .join("\n")
}

/// NeoVM integers come back as little-endian bytes from the bundled runtime.
fn uint_le(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    for (i, b) in bytes.iter().take(8).enumerate() {
        buf[i] = *b;
    }
    u64::from_le_bytes(buf)
}

fn method<'a>(manifest: &'a Value, name: &str) -> &'a Value {
    manifest["abi"]["methods"]
        .as_array()
        .expect("methods array")
        .iter()
        .find(|m| m["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("method '{name}' not found in manifest"))
}

fn param_types(method: &Value) -> Vec<&str> {
    method["parameters"]
        .as_array()
        .expect("parameters array")
        .iter()
        .map(|p| p["type"].as_str().expect("param type"))
        .collect()
}

fn new_runtime() -> NeoRuntime {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    // Pin msg.sender: without an explicit caller override the bundled
    // runtime's GetCallingScriptHash handler re-points storage_account at
    // default_account mid-execution (pre-existing harness behaviour).
    runtime
        .override_caller_account(CALLER)
        .expect("caller override");
    runtime
}

// ============================================================================
// Devpack NEP11.sol — manifest conformance
// ============================================================================

#[test]
fn devpack_nep11_manifest_is_spec_conformant() {
    let artifacts =
        compile_contracts(&devpack_nep11_source(), false, 2).expect("devpack NEP11 compile");
    let nep11 = artifacts
        .iter()
        .find(|a| a.metadata.name == "NEP11")
        .expect("NEP11 artifact");
    let manifest = &nep11.manifest;

    // tokenId is a NEP-11 ByteString => manifest ByteArray (was Hash256).
    assert_eq!(
        param_types(method(manifest, "ownerOf")),
        vec!["ByteArray"],
        "ownerOf(tokenId) parameter must be ByteArray"
    );
    assert_eq!(
        param_types(method(manifest, "transfer")),
        vec!["Hash160", "ByteArray", "Any"],
        "transfer(to, tokenId, data) parameter types (data is spec `Any`)"
    );
    assert_eq!(
        method(manifest, "ownerOf")["returntype"].as_str(),
        Some("Hash160")
    );

    // tokensOf / tokens return NeoVM iterators => InteropInterface.
    for name in ["tokensOf", "tokens"] {
        let m = method(manifest, name);
        assert_eq!(
            m["returntype"].as_str(),
            Some("InteropInterface"),
            "{name} must declare manifest returntype InteropInterface"
        );
        assert_eq!(m["safe"].as_bool(), Some(true), "{name} must be safe");
    }
    assert_eq!(param_types(method(manifest, "tokensOf")), vec!["Hash160"]);

    // properties: documented deviation — ByteArray (spec says Map; Solidity
    // cannot build a NeoVM Map return value).
    let props = method(manifest, "properties");
    assert_eq!(param_types(props), vec!["ByteArray"]);
    assert_eq!(props["returntype"].as_str(), Some("ByteArray"));

    // No method may surface a Hash256-typed tokenId anymore.
    for m in manifest["abi"]["methods"].as_array().unwrap() {
        for p in m["parameters"].as_array().unwrap() {
            if p["name"].as_str() == Some("tokenId") {
                assert_eq!(
                    p["type"].as_str(),
                    Some("ByteArray"),
                    "method {} still types tokenId as {:?}",
                    m["name"],
                    p["type"]
                );
            }
        }
    }

    // The NEP-11 standard must still be auto-detected.
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("standards");
    assert!(
        standards.iter().any(|s| s.as_str() == Some("NEP-11")),
        "NEP11 must advertise NEP-11, got {standards:?}"
    );

    // The 4-parameter Transfer event keeps tokenId as ByteArray.
    let events = manifest["abi"]["events"].as_array().expect("events");
    let transfer = events
        .iter()
        .find(|e| e["name"].as_str() == Some("Transfer"))
        .expect("Transfer event");
    let transfer_params = transfer["parameters"].as_array().expect("event params");
    assert_eq!(transfer_params.len(), 4, "NEP-11 Transfer event arity");
    assert_eq!(
        transfer_params[3]["type"].as_str(),
        Some("ByteArray"),
        "Transfer.tokenId event parameter must be ByteArray"
    );
}

// ============================================================================
// Devpack NEP11.sol — bundled-runtime execution smoke
// ============================================================================

fn nep11_deploy_args() -> Vec<StackItem> {
    vec![
        StackItem::byte_array(b"Test NFT".to_vec()),
        StackItem::byte_array(b"TNFT".to_vec()),
        StackItem::Integer(0),
        StackItem::byte_array(b"https://example.com/".to_vec()),
        StackItem::Integer(0),
        StackItem::Boolean(false),
    ]
}

fn call(
    runtime: &mut NeoRuntime,
    artifact: &CompilationArtifacts,
    name: &str,
    args: &[StackItem],
) -> neo_devpack_solidity::runtime::ExecutionResult {
    let result = runtime
        .call_method_with_deploy_args(
            &artifact.bytecode,
            &artifact.tokens,
            &artifact.manifest,
            name,
            args,
            Some(&nep11_deploy_args()),
        )
        .unwrap_or_else(|e| panic!("{name} invocation failed: {e:?}"));
    assert!(result.success, "{name} reverted: {:?}", result.exception);
    result
}

#[test]
fn devpack_nep11_mint_and_iterate_in_bundled_runtime() {
    let artifacts =
        compile_contracts(&devpack_nep11_source(), false, 2).expect("devpack NEP11 compile");
    let nep11 = artifacts
        .iter()
        .find(|a| a.metadata.name == "NEP11")
        .expect("NEP11 artifact");

    let mut runtime = new_runtime();
    let holder = StackItem::byte_array(vec![7u8; 20]);
    let empty_props = StackItem::byte_array(Vec::new());

    // Mint two tokens with explicit ByteString ids (msg.sender == _minter
    // because the deploy and the mint share the overridden caller).
    for id in [b"id-1".to_vec(), b"id-2".to_vec()] {
        call(
            &mut runtime,
            nep11,
            "mint",
            &[
                holder.clone(),
                StackItem::byte_array(id),
                empty_props.clone(),
            ],
        );
    }

    let r = call(&mut runtime, nep11, "totalSupply", &[]);
    assert_eq!(uint_le(&r.return_data), 2, "totalSupply after two mints");

    let r = call(
        &mut runtime,
        nep11,
        "balanceOf",
        std::slice::from_ref(&holder),
    );
    assert_eq!(uint_le(&r.return_data), 2, "balanceOf(holder)");

    let r = call(
        &mut runtime,
        nep11,
        "ownerOf",
        &[StackItem::byte_array(b"id-1".to_vec())],
    );
    assert_eq!(r.return_data, vec![7u8; 20], "ownerOf(id-1)");

    // tokensOf / tokens return the raw iterator handle stack item.
    let r = call(&mut runtime, nep11, "tokens", &[]);
    assert!(
        !r.return_data.is_empty(),
        "tokens() must return the iterator handle"
    );
    let r = call(
        &mut runtime,
        nep11,
        "tokensOf",
        std::slice::from_ref(&holder),
    );
    assert!(
        !r.return_data.is_empty(),
        "tokensOf(owner) must return the iterator handle"
    );

    // The enumeration helpers consume those iterators in-contract; the
    // KeysOnly|RemovePrefix index scan must yield the bare token ids.
    let r = call(
        &mut runtime,
        nep11,
        "tokenOfOwnerByIndex",
        &[holder.clone(), StackItem::Integer(0)],
    );
    assert_eq!(r.return_data, b"id-1", "tokenOfOwnerByIndex(holder, 0)");

    let r = call(
        &mut runtime,
        nep11,
        "tokenByIndex",
        &[StackItem::Integer(1)],
    );
    assert_eq!(r.return_data, b"id-2", "tokenByIndex(1)");

    // properties round-trips the stored blob (documented ByteArray shape).
    call(
        &mut runtime,
        nep11,
        "setProperties",
        &[
            StackItem::byte_array(b"id-1".to_vec()),
            StackItem::byte_array(b"{\"name\":\"one\"}".to_vec()),
        ],
    );
    let r = call(
        &mut runtime,
        nep11,
        "properties",
        &[StackItem::byte_array(b"id-1".to_vec())],
    );
    assert_eq!(r.return_data, b"{\"name\":\"one\"}");
}

// ============================================================================
// Compiler/runtime mechanics pinned by a self-contained probe contract
// ============================================================================

/// A self-contained contract exercising the exact patterns the reworked
/// devpack NEP11.sol relies on — dynamic-bytes mapping keys, `bytes.concat`
/// over (constant prefix, address, dynamic id), a raw-storage token index,
/// `Storage.find(prefix, options)` returned from a public method, and
/// `Syscalls.iteratorNext` / `iteratorValue` consumption.
const ITERATOR_PROBE: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract IterProbe {
    bytes constant TOKEN_PREFIX = "probe.t.";
    bytes constant ACCOUNT_PREFIX = "probe.a.";
    // FindOptions.KeysOnly | FindOptions.RemovePrefix
    uint8 constant FIND_KEYS_REMOVE_PREFIX = 0x03;

    mapping(bytes => address) private owners;

    function add(bytes memory id) public returns (bool) {
        owners[id] = msg.sender;
        Storage.put(bytes.concat(TOKEN_PREFIX, id), hex"01");
        Storage.put(bytes.concat(ACCOUNT_PREFIX, msg.sender, id), hex"01");
        return true;
    }

    function tokens() public view returns (Syscalls.Iterator memory) {
        return Storage.find(TOKEN_PREFIX, FIND_KEYS_REMOVE_PREFIX);
    }

    function tokensOf(address owner) public view returns (Syscalls.Iterator memory) {
        return Storage.find(bytes.concat(ACCOUNT_PREFIX, owner), FIND_KEYS_REMOVE_PREFIX);
    }

    function countTokens() public returns (uint256 n) {
        Syscalls.Iterator memory it = Storage.find(TOKEN_PREFIX, FIND_KEYS_REMOVE_PREFIX);
        while (Syscalls.iteratorNext(it)) {
            n++;
        }
        return n;
    }

    function firstTokenId() public returns (bytes memory) {
        Syscalls.Iterator memory it = Storage.find(TOKEN_PREFIX, FIND_KEYS_REMOVE_PREFIX);
        require(Syscalls.iteratorNext(it), "empty");
        return Syscalls.iteratorValue(it);
    }

    function ownerOf(bytes memory id) public view returns (address) {
        return owners[id];
    }
}
"#;

fn probe_source() -> String {
    [
        include_str!("../devpack/contracts/Syscalls.sol"),
        include_str!("../devpack/contracts/NativeContracts.sol"),
        include_str!("../devpack/contracts/NativeCalls.sol"),
        include_str!("../devpack/libraries/Storage.sol"),
        include_str!("../devpack/libraries/Runtime.sol"),
        ITERATOR_PROBE,
    ]
    .join("\n")
}

#[test]
fn iterator_probe_manifest_declares_interop_interface_returns() {
    let artifacts = compile_contracts(&probe_source(), false, 2).expect("probe compile failed");
    let probe = artifacts
        .iter()
        .find(|a| a.metadata.name == "IterProbe")
        .expect("IterProbe artifact");

    for name in ["tokens", "tokensOf"] {
        assert_eq!(
            method(&probe.manifest, name)["returntype"].as_str(),
            Some("InteropInterface"),
            "{name} must declare manifest returntype InteropInterface"
        );
    }

    // tokenId-style dynamic bytes params/returns stay ByteArray.
    assert_eq!(
        method(&probe.manifest, "ownerOf")["parameters"][0]["type"].as_str(),
        Some("ByteArray")
    );
    assert_eq!(
        method(&probe.manifest, "firstTokenId")["returntype"].as_str(),
        Some("ByteArray")
    );
}

#[test]
fn iterator_probe_executes_in_bundled_runtime() {
    let artifacts = compile_contracts(&probe_source(), false, 2).expect("probe compile failed");
    let probe = artifacts
        .iter()
        .find(|a| a.metadata.name == "IterProbe")
        .expect("IterProbe artifact");

    let mut runtime = new_runtime();

    let id1 = StackItem::byte_array(b"token-1".to_vec());
    let id2 = StackItem::byte_array(b"token-2".to_vec());

    for id in [&id1, &id2] {
        let result = runtime
            .call_method(
                &probe.bytecode,
                &probe.tokens,
                &probe.manifest,
                "add",
                std::slice::from_ref(id),
            )
            .expect("add invocation");
        assert!(result.success, "add failed: {:?}", result.exception);
    }

    // Returning the iterator from a public method must succeed and leave a
    // stack item (the iterator handle) as the NeoVM return value.
    let result = runtime
        .call_method(
            &probe.bytecode,
            &probe.tokens,
            &probe.manifest,
            "tokens",
            &[],
        )
        .expect("tokens invocation");
    assert!(result.success, "tokens failed: {:?}", result.exception);
    assert!(
        !result.return_data.is_empty(),
        "tokens() must return the iterator handle stack item"
    );

    // Consuming the iterator in-contract observes both index entries.
    let result = runtime
        .call_method(
            &probe.bytecode,
            &probe.tokens,
            &probe.manifest,
            "countTokens",
            &[],
        )
        .expect("countTokens invocation");
    assert!(result.success, "countTokens failed: {:?}", result.exception);
    let count = uint_le(&result.return_data);
    assert_eq!(count, 2, "countTokens saw {count} entries");

    // FindOptions.KeysOnly | RemovePrefix must yield the bare token id
    // (prefix stripped, no [key, value] struct) — matches real Neo N3.
    let result = runtime
        .call_method(
            &probe.bytecode,
            &probe.tokens,
            &probe.manifest,
            "firstTokenId",
            &[],
        )
        .expect("firstTokenId invocation");
    assert!(
        result.success,
        "firstTokenId failed: {:?}",
        result.exception
    );
    assert_eq!(
        result.return_data, b"token-1",
        "iterator value must be the prefix-stripped token id"
    );
}

// ============================================================================
// Runtime FindOptions semantics (System.Storage.Find)
// ============================================================================

/// Raw-bytecode driver: Put "aa"=>"1" in-VM, then
/// Find(prefix="a", options) ; Next ; Value ; RET.
fn run_find_with_options(options: u8) -> Result<Vec<u8>, String> {
    let mut storage = StorageManager::new(&RuntimeConfig::default()).map_err(|e| e.to_string())?;
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).map_err(|e| e.to_string())?;
    let account = "0x0000000000000000000000000000000000000000";
    ctx.bind_storage(account, &mut storage)
        .map_err(|e| e.to_string())?;

    // Neo N3 syscall convention: first argument at the top of the stack —
    // Put(context, key, value) is pushed as [value, key, context] and
    // Find(context, prefix, options) as [options, prefix, context].
    let bytecode = vec![
        // Put key "aa" = "1"
        0x0C, 0x01, b'1', // value "1"
        0x0C, 0x02, b'a', b'a', // key "aa"
        0x41, 155, 246, 103, 206, // Storage.GetContext
        0x41, 230, 63, 24, 132, // Storage.Put
        // Find with prefix "a"
        0x0C, 0x01, options, // options (1-byte integer-coercible string)
        0x0C, 0x01, b'a', // prefix "a"
        0x41, 155, 246, 103, 206, // Storage.GetContext
        0x41, 223, 48, 184, 154,  // Storage.Find
        0x4A, // DUP iterator token
        0x41, 156, 8, 237, 156,  // Iterator.Next
        0x45, // DROP bool
        0x41, 243, 84, 191, 29,   // Iterator.Value
        0x40, // RET
    ];
    ctx.initialize(&bytecode, &[]).map_err(|e| e.to_string())?;
    loop {
        let state = ctx.step().map_err(|e| e.to_string())?;
        if state.halted {
            break;
        }
    }
    // ByteArray returns surface raw; Array/Map returns surface as JSON.
    Ok(ctx.return_data().to_vec())
}

#[test]
fn storage_find_honours_find_options() {
    // None (0x00): legacy [key, value] struct entries (JSON-encoded Array).
    let raw = run_find_with_options(0x00).expect("None options");
    match serde_json::from_slice::<StackItem>(&raw).expect("Array JSON") {
        StackItem::Array(items) => assert_eq!(items.borrow().len(), 2),
        other => panic!("FindOptions.None must yield [key, value], got {other:?}"),
    }

    // KeysOnly (0x01): the full key.
    assert_eq!(
        run_find_with_options(0x01).expect("KeysOnly"),
        b"aa".to_vec()
    );

    // KeysOnly | RemovePrefix (0x03): the prefix-stripped key.
    assert_eq!(
        run_find_with_options(0x03).expect("KeysOnly|RemovePrefix"),
        b"a".to_vec()
    );

    // ValuesOnly (0x04): the bare value.
    assert_eq!(
        run_find_with_options(0x04).expect("ValuesOnly"),
        b"1".to_vec()
    );

    // Invalid combination faults, matching the C# node's validation.
    assert!(
        run_find_with_options(0x05).is_err(),
        "KeysOnly|ValuesOnly must be rejected"
    );
}

// ============================================================================
// Example contract keeps compiling with dynamic-bytes token ids
// ============================================================================

#[test]
fn complete_nep11_example_manifest_uses_bytearray_token_ids() {
    let source = [
        include_str!("../devpack/contracts/Syscalls.sol"),
        include_str!("../devpack/contracts/NativeContracts.sol"),
        include_str!("../devpack/contracts/NativeCalls.sol"),
        include_str!("../devpack/libraries/Storage.sol"),
        include_str!("../devpack/libraries/Runtime.sol"),
        include_str!("../devpack/libraries/Neo.sol"),
        include_str!("../devpack/contracts/FrameworkBase.sol"),
        include_str!("../devpack/standards/NEP11.sol"),
        include_str!("../devpack/examples/CompleteNEP11NFT.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("CompleteNEP11NFT compile");
    let nft = artifacts
        .iter()
        .find(|a| a.metadata.name == "CompleteNEP11NFT")
        .expect("CompleteNEP11NFT artifact");

    for m in nft.manifest["abi"]["methods"].as_array().unwrap() {
        for p in m["parameters"].as_array().unwrap() {
            if p["name"].as_str() == Some("tokenId") {
                assert_eq!(
                    p["type"].as_str(),
                    Some("ByteArray"),
                    "method {} still types tokenId as {:?}",
                    m["name"],
                    p["type"]
                );
            }
        }
    }
    for name in ["tokensOf", "tokens"] {
        assert_eq!(
            method(&nft.manifest, name)["returntype"].as_str(),
            Some("InteropInterface"),
            "inherited {name} must keep the iterator return"
        );
    }
}
