//! Regression tests for manifest-correctness fixes (agent: manifest).
//!
//! Covers:
//! 1. Permission inference visits catch handlers (try/catch contract calls
//!    get manifest permissions instead of silently under-permissioning).
//! 2. Manifest event parameters describe the actual System.Runtime.Notify
//!    payload (EVM shape: topic0 + indexed + data), matching post-Basilisk
//!    notification validation.
//! 3. Safe-flag soundness: a `view` method that reaches a storage write
//!    through a function pointer is rejected instead of shipping
//!    `"safe": true`.
//! 4. Standards detection no longer claims NEP-11/NEP-24/NEP-17 for
//!    contracts with incomplete or non-conformant method sets.
//! 5. The devpack `Any` UDVT (`type Any is bytes;`) reaches the manifest as
//!    `Any`, per the NEP-17/NEP-11 specs for `data` parameters.
//! 6. Distinct-arity overloads keep their original Solidity name in the
//!    manifest (Neo dispatches on name + parameter count); only same-arity
//!    collisions are mangled.

use neo_devpack_solidity::cli::compile_contracts;
use serde_json::Value;

// ── 1. Catch-handler permission inference ───────────────────────────────

#[test]
fn catch_handler_contract_call_gets_manifest_permission() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IPrimary {
        function fetch(uint256 x) external returns (uint256);
    }

    interface IFallback {
        function report(uint256 x) external returns (uint256);
    }

    contract CatchPerm {
        bytes20 constant PRIMARY_LE = hex"14131211100f0e0d0c0b0a090807060504030201";
        bytes20 constant BACKUP_LE = hex"2423222120191817161514131211100f0e0d0c0b";

        function run(uint256 x) public returns (uint256) {
            try IPrimary(Syscalls.scriptHashToAddress(PRIMARY_LE)).fetch(x) returns (uint256 v) {
                return v;
            } catch {
                return IFallback(Syscalls.scriptHashToAddress(BACKUP_LE)).report(x);
            }
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let primary = "0x0102030405060708090a0b0c0d0e0f1011121314";
    let backup = "0x0b0c0d0e0f101112131415161718192021222324";

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String(primary.into())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "fetch"))
        }),
        "expected permission for the try-path call, got {permissions:?}"
    );

    // The catch-path call must be covered too: either by the precise entry
    // (constant hash + method literal inside the catch body) or — at worst —
    // by a wildcard. Silently missing means the error-recovery path FAULTs
    // on-chain with a disallowed-method exception.
    assert!(
        permissions.iter().any(|entry| {
            let contract_ok = entry["contract"] == Value::String(backup.into())
                || entry["contract"] == Value::String("*".into());
            let methods_ok = entry["methods"] == Value::String("*".into())
                || entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "report"));
            contract_ok && methods_ok
        }),
        "expected the catch-path call (BACKUP.report) to be covered by manifest permissions, \
         got {permissions:?}"
    );
}

// ── 2. Manifest event parameters describe the Notify payload ────────────

#[test]
fn manifest_event_params_match_emitted_notification_payload() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract Token {
        // NOTE: deliberately NOT named `Transfer` with the NEP-17 signature —
        // that exact shape is emitted as a NATIVE NEP transfer notification
        // (covered by tests/gap_events_native_tests.rs). This test pins the
        // truthful EVM wire-shape declaration for all OTHER events.
        event Moved(address indexed from, address indexed to, uint256 amount);
        event Burned(uint256 amount);

        function emitIt(address to, uint256 amount) public {
            emit Moved(msg.sender, to, amount);
            emit Burned(amount);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let events = artifacts[0].manifest["abi"]["events"]
        .as_array()
        .expect("events array");

    // Moved is notified as [topic0, from-slot, to-slot, data] — 4 items,
    // all ByteStrings — so the manifest must declare exactly that, or every
    // `emit` FAULTs on Neo nodes >= 3.6 (HF_Basilisk notification checks).
    let moved = events
        .iter()
        .find(|e| e["name"] == "Moved")
        .expect("Moved event");
    let params = moved["parameters"].as_array().expect("parameters");
    let shapes: Vec<(&str, &str)> = params
        .iter()
        .map(|p| {
            (
                p["name"].as_str().expect("param name"),
                p["type"].as_str().expect("param type"),
            )
        })
        .collect();
    assert_eq!(
        shapes,
        vec![
            ("topic0", "ByteArray"),
            ("from", "ByteArray"),
            ("to", "ByteArray"),
            ("data", "ByteArray"),
        ],
        "Moved manifest params must mirror the notified EVM-shape state array"
    );

    // No indexed parameters → [topic0, data].
    let burned = events
        .iter()
        .find(|e| e["name"] == "Burned")
        .expect("Burned event");
    let burned_params = burned["parameters"].as_array().expect("parameters");
    assert_eq!(
        burned_params.len(),
        2,
        "non-indexed event payload is [topic0, data], got {burned_params:?}"
    );
    assert_eq!(burned_params[0]["name"], "topic0");
    assert_eq!(burned_params[1]["name"], "data");
}

// ── 3. Safe-flag: function-pointer calls taint hazard analysis ──────────

#[test]
fn view_method_writing_storage_through_function_pointer_is_rejected() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract FpSafeBypass {
        uint256 private counter;

        function bump() internal returns (uint256) {
            counter += 1;
            return counter;
        }

        function apply_(function() internal returns (uint256) fp) internal returns (uint256) {
            return fp();
        }

        function peek() public view returns (uint256) {
            return apply_(bump);
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    let err = match result {
        Ok(_) => panic!(
            "compilation must fail: `peek` is declared view but reaches a storage \
             write through a function pointer (would ship \"safe\": true)"
        ),
        Err(err) => format!("{err:?}"),
    };
    assert!(
        err.contains("writes contract storage"),
        "expected a view-purity violation mentioning storage writes, got: {err}"
    );
}

// ── 4. Standards not claimed when method sets incomplete ────────────────

#[test]
fn erc721_shape_claims_neither_nep11_nor_nep24() {
    // balanceOf + ownerOf + transferFrom + tokenURI is an ERC-721 surface:
    // NEP-11 mandates symbol/decimals/totalSupply/tokensOf/transfer (and
    // transferFrom is not a NEP-11 method); NEP-24's only mandatory method
    // is royaltyInfo, which tokenURI does not imply.
    let source = r#"
    pragma solidity ^0.8.19;

    contract Erc721ish {
        mapping(uint256 => address) private owners;
        mapping(address => uint256) private balances;

        function balanceOf(address owner) public view returns (uint256) { return balances[owner]; }
        function ownerOf(uint256 tokenId) public view returns (address) { return owners[tokenId]; }
        function tokenURI(uint256 tokenId) public pure returns (string memory) { tokenId; return ""; }

        function transferFrom(address from, address to, uint256 tokenId) public {
            owners[tokenId] = to;
            balances[from] -= 1;
            balances[to] += 1;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let standards = artifacts[0].manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");

    for bogus in ["NEP-11", "NEP-24", "NEP-17"] {
        assert!(
            !standards.iter().any(|s| s.as_str() == Some(bogus)),
            "ERC-721-shaped contract must not advertise {bogus}; got {standards:?}"
        );
    }
}

#[test]
fn vanilla_erc20_signature_does_not_claim_nep17() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract VanillaERC20 {
        mapping(address => uint256) private balances;

        event Transfer(address indexed from, address indexed to, uint256 amount);

        function symbol() public pure returns (string memory) { return "VAN"; }
        function decimals() public pure returns (uint8) { return 18; }
        function totalSupply() public pure returns (uint256) { return 0; }
        function balanceOf(address account) public view returns (uint256) { return balances[account]; }

        function transfer(address to, uint256 amount) public returns (bool) {
            balances[msg.sender] -= amount;
            balances[to] += amount;
            emit Transfer(msg.sender, to, amount);
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let standards = artifacts[0].manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");

    assert!(
        !standards.iter().any(|s| s.as_str() == Some("NEP-17")),
        "a 2-parameter ERC-20 transfer must not be advertised as NEP-17 \
         (wallets invoke transfer(from, to, amount, data)); got {standards:?}"
    );
}

// ── 5. Devpack `Any` UDVT reaches the manifest as Any ───────────────────

#[test]
fn devpack_any_type_emits_manifest_any_for_transfer_data() {
    let source = r#"
    pragma solidity ^0.8.19;

    type Any is bytes;

    contract AnyToken {
        mapping(address => uint256) private balances;

        event Transfer(address indexed from, address indexed to, uint256 amount);

        function symbol() public pure returns (string memory) { return "ANY"; }
        function decimals() public pure returns (uint8) { return 8; }
        function totalSupply() public pure returns (uint256) { return 0; }
        function balanceOf(address account) public view returns (uint256) { return balances[account]; }

        function transfer(address from, address to, uint256 amount, Any data) public returns (bool) {
            data;
            require(from == msg.sender, "bad from");
            balances[from] -= amount;
            balances[to] += amount;
            emit Transfer(from, to, amount);
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let manifest = &artifacts[0].manifest;

    let methods = manifest["abi"]["methods"].as_array().expect("methods");
    let transfer = methods
        .iter()
        .find(|m| m["name"] == "transfer")
        .expect("transfer method");
    let params = transfer["parameters"].as_array().expect("parameters");
    assert_eq!(params.len(), 4, "NEP-17 transfer takes 4 parameters");
    assert_eq!(
        params[3]["type"],
        Value::String("Any".to_string()),
        "the `data` parameter declared with the devpack `Any` UDVT must reach \
         the manifest as `Any` (NEP-17 spec), not ByteArray; got {params:?}"
    );

    // With the conformant signature + event, NEP-17 should also be detected.
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert!(
        standards.iter().any(|s| s.as_str() == Some("NEP-17")),
        "conformant NEP-17 surface should be advertised; got {standards:?}"
    );
}

#[test]
fn user_defined_any_alias_with_non_bytes_underlying_keeps_alias_semantics() {
    // The `Any` special case is scoped to an underlying type of `bytes`
    // (the devpack declaration). An unrelated `type Any is bytes32;` must
    // keep resolving through the alias (bytes32 → Hash256).
    let source = r#"
    pragma solidity ^0.8.19;

    type Any is bytes32;

    contract Custom {
        function poke(Any value) public pure returns (bool) {
            value;
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods");
    let poke = methods
        .iter()
        .find(|m| m["name"] == "poke")
        .expect("poke method");
    assert_eq!(
        poke["parameters"][0]["type"],
        Value::String("Hash256".to_string()),
        "`type Any is bytes32;` must keep its alias semantics (Hash256)"
    );
}

// ── 6. Distinct-arity overloads keep their original names ───────────────

#[test]
fn distinct_arity_overloads_keep_original_manifest_names() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract OverloadedApi {
        function foo(uint256 value) public pure returns (uint256) {
            return value;
        }

        function foo(uint256 value, uint256 extra) public pure returns (uint256) {
            return value + extra;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods");

    let foo_arities: Vec<usize> = methods
        .iter()
        .filter(|m| m["name"] == "foo")
        .map(|m| m["parameters"].as_array().map(|p| p.len()).unwrap_or(0))
        .collect();
    assert_eq!(
        foo_arities.len(),
        2,
        "both distinct-arity overloads must keep the original name `foo`; methods={:?}",
        methods
            .iter()
            .map(|m| m["name"].clone())
            .collect::<Vec<_>>()
    );
    assert!(
        foo_arities.contains(&1) && foo_arities.contains(&2),
        "expected arities 1 and 2 under the shared name, got {foo_arities:?}"
    );

    assert!(
        !methods.iter().any(|m| m["name"] == "foo(uint256)"),
        "distinct-arity overloads must not be mangled in the manifest"
    );
}

#[test]
fn same_arity_overloads_still_get_unique_manifest_names() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract OverloadedApi {
        function foo(uint256 value) public pure returns (uint256) {
            return value;
        }

        function foo(address account) public pure returns (address) {
            return account;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods");

    // Same (name, arity): Neo cannot disambiguate, so exactly one keeps the
    // clean name and the other carries the mangled signature name.
    let clean = methods.iter().filter(|m| m["name"] == "foo").count();
    let mangled = methods
        .iter()
        .filter(|m| {
            m["name"]
                .as_str()
                .is_some_and(|n| n == "foo(uint256)" || n == "foo(address)")
        })
        .count();
    assert_eq!(clean, 1, "exactly one primary keeps the clean name");
    assert_eq!(mangled, 1, "the colliding overload keeps its mangled name");
}
