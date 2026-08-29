//! Shared helpers for the fuzz_tests integration-test submodules.
//!
//! Each sibling submodule imports from here via `use super::common::*;`.
//! This file holds only infrastructure (strategies, observers, decoders);
//! test functions live in the per-category submodules.

#![allow(dead_code)]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

pub fn is_solidity_reserved(s: &str) -> bool {
    matches!(
        s,
        "abstract"
            | "after"
            | "alias"
            | "anonymous"
            | "apply"
            | "as"
            | "assembly"
            | "async"
            | "auto"
            | "bool"
            | "break"
            | "byte"
            | "bytes"
            | "case"
            | "catch"
            | "constant"
            | "constructor"
            | "continue"
            | "contract"
            | "copyof"
            | "days"
            | "default"
            | "define"
            | "delete"
            | "do"
            | "else"
            | "emit"
            | "enum"
            | "error"
            | "event"
            | "external"
            | "fallback"
            | "false"
            | "final"
            | "for"
            | "from"
            | "function"
            | "hex"
            | "if"
            | "immutable"
            | "implements"
            | "import"
            | "in"
            | "indexed"
            | "inline"
            | "instance"
            | "interface"
            | "internal"
            | "is"
            | "let"
            | "library"
            | "macro"
            | "mapping"
            | "match"
            | "memory"
            | "modifier"
            | "mutable"
            | "new"
            | "null"
            | "of"
            | "override"
            | "partial"
            | "payable"
            | "persistent"
            | "pragma"
            | "private"
            | "promise"
            | "public"
            | "pure"
            | "receive"
            | "record"
            | "reference"
            | "relocatable"
            | "return"
            | "returns"
            | "revert"
            | "sealed"
            | "seconds"
            | "sizeof"
            | "static"
            | "storage"
            | "string"
            | "struct"
            | "super"
            | "supports"
            | "switch"
            | "temporary"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "type"
            | "typedef"
            | "typeof"
            | "unchecked"
            | "unicode"
            | "using"
            | "var"
            | "view"
            | "virtual"
            | "weeks"
            | "while"
            | "wei"
            | "years"
            | "address"
            | "fixed"
            | "int"
            | "int8"
            | "int16"
            | "int32"
            | "int64"
            | "int128"
            | "int256"
            | "uint"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "uint128"
            | "uint256"
            | "bytes1"
            | "bytes32"
    )
}

pub fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_filter("not a Solidity reserved keyword", |s| {
        !is_solidity_reserved(s)
    })
}

pub fn uint_value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("0".to_string()),
        "[1-9][0-9]{0,20}".prop_map(String::from),
    ]
}

/// Decode minimum-width little-endian `return_data` into a BigUint.
///
/// Scalar uint256 return values are emitted as variable-width LE bytes: the
/// probe set showed sizes of 8, 16, and 32 bytes depending on the value's
/// magnitude (e.g. 300 → 8 bytes, 2^64 → 16 bytes, ~2^128 → 32 bytes). An
/// empty slice decodes to 0 (treated as an absent scalar). `BigUint`
/// handles all three widths uniformly via `from_bytes_le`.
pub fn decode_uint_le(bytes: &[u8]) -> num_bigint::BigUint {
    use num_bigint::BigUint;
    if bytes.is_empty() {
        BigUint::from(0u8)
    } else {
        BigUint::from_bytes_le(bytes)
    }
}

/// Decode the state array of a NATIVE NEP notification from the bundled
/// emulator's log record.
///
/// NEP-17 / NEP-11 standard `Transfer` events are emitted in NATIVE Neo
/// shape (`Notify("Transfer", [from, to, amount(, tokenId)])` — no EVM
/// topic0). The emulator's `System.Runtime.Notify` handler records such
/// notifications on its legacy path: `topics = [eventName]` and `data` =
/// the serde-JSON encoding of the state array (tagged `StackItem` form,
/// e.g. `{"type":"Array","value":[{"type":"ByteArray","value":[...]},
/// {"type":"Null"},{"type":"Integer","value":100}]}`).
pub fn decode_native_notification_state(data: &[u8]) -> Vec<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_slice(data).unwrap_or_else(|e| {
        panic!(
            "native notification data must be the emulator's JSON state encoding: {e}; raw=0x{}",
            hex::encode(data)
        )
    });
    assert_eq!(
        value.get("type").and_then(|t| t.as_str()),
        Some("Array"),
        "native notification state must be an Array, got {value}"
    );
    value["value"]
        .as_array()
        .expect("native notification state array")
        .clone()
}

/// Extract a ByteArray state item's bytes from a decoded native
/// notification (see `decode_native_notification_state`).
pub fn native_state_bytes(item: &serde_json::Value) -> Vec<u8> {
    assert_eq!(
        item.get("type").and_then(|t| t.as_str()),
        Some("ByteArray"),
        "expected ByteArray state item, got {item}"
    );
    item["value"]
        .as_array()
        .expect("ByteArray value")
        .iter()
        .map(|b| b.as_u64().expect("byte") as u8)
        .collect()
}

/// Extract an Integer state item's value from a decoded native notification.
pub fn native_state_int(item: &serde_json::Value) -> i64 {
    assert_eq!(
        item.get("type").and_then(|t| t.as_str()),
        Some("Integer"),
        "expected Integer state item, got {item}"
    );
    item["value"].as_i64().expect("integer value")
}

/// Whether a decoded native-notification state item is `Null` (the NEP
/// mint/burn convention: `from == null` for mint, `to == null` for burn).
pub fn native_state_is_null(item: &serde_json::Value) -> bool {
    item.get("type").and_then(|t| t.as_str()) == Some("Null")
}

/// Observed runtime behavior for a single-function arithmetic harness.
///
/// The fourth variant (`FaultOther`) exists because batch #10 harness #8
/// surfaced a non-panic fault shape (`"Shift amount exceeds maximum (255)"`)
/// that is neither `Panic(0x11)` nor `Panic(0x12)` nor a clean return — it
/// is a VM-level guard on the shift-amount argument. Future additions to
/// this enum should follow the same pattern: add a variant only when a
/// harness observes a genuinely new outcome shape.
#[derive(Debug, PartialEq, Eq)]
pub enum ObservedBehavior {
    /// Execution reverted via THROW with a Solidity panic selector.
    /// `u8` holds the selector byte (0x11, 0x12, ...).
    Panicked(u8),
    /// Execution succeeded and returned a scalar `uint256` value decoded
    /// from the minimum-width LE `return_data`.
    Returned(num_bigint::BigUint),
    /// Execution faulted but the exception message did NOT match the
    /// Panic(0x<sig>) shape. The string carries the raw exception message
    /// for downstream diffing.
    FaultOther(String),
}

/// Compile a tiny single-function contract and execute offset 0.
///
/// Any compile failure panics immediately (these harnesses bake known-good
/// source in as literals; a compile failure signals a regression in the
/// frontend, not in the arithmetic lowering under test). Runtime execution
/// errors (vs. execution-that-produced-a-fault) also panic, matching the
/// `execute(..).expect(...)` idiom used by batches #5/#8/#9.
pub fn compile_and_execute(source: &str) -> neo_devpack_solidity::runtime::ExecutionResult {
    let artifacts = compile_contracts(source, false, 2)
        .unwrap_or_else(|e| panic!("arith-scope compile failed: {:?}\nsource:\n{}", e, source));
    assert!(
        !artifacts.is_empty(),
        "arith-scope compile produced no artifacts"
    );
    let mut runtime = NeoRuntime::new(RuntimeConfig::default())
        .expect("arith-scope runtime construction must not fail");
    runtime
        .execute(&artifacts[0].bytecode, &[])
        .expect("arith-scope execute must not fail at host level (a fault != host error)")
}

/// Parse an `ExecutionResult` into an `ObservedBehavior` using the same
/// message shape batches #8/#9 established: revert panics surface as
/// `"Execution failed: THROW: Panic: 0x<hex>"` on the exception message.
///
/// The selector byte is parsed out of the first `"Panic: 0x"` occurrence
/// (case-sensitive — the runtime produces exactly that capitalization; see
/// src/runtime/execution/instruction/flow/exceptions.rs). Two-digit selectors
/// like `0x11` and `0x12` are supported directly; future single-digit
/// selectors (none currently defined by Solidity) would need padding.
pub fn observe(result: &neo_devpack_solidity::runtime::ExecutionResult) -> ObservedBehavior {
    if result.success {
        return ObservedBehavior::Returned(decode_uint_le(&result.return_data));
    }
    let exc = match result.exception.as_ref() {
        Some(e) => e,
        // Failed execution with no exception is unexpected; treat as a
        // degenerate FaultOther so the match in each harness can still fire.
        None => return ObservedBehavior::FaultOther("no exception populated".to_string()),
    };
    // Task #103 — prefer the EVM-canonical envelope on `return_data` when
    // present. The compiler emits `keccak256("Panic(uint256)")[..4] ||
    // abi.encode(code)` (selector = 0x4e487b71) for assert/div-by-zero/
    // mod-by-zero/empty-pop/enum-cast panics, so reading the selector +
    // last byte of the 32-byte BE uint is the shape-agnostic path.
    if result.return_data.len() >= 36 && &result.return_data[..4] == &[0x4eu8, 0x48, 0x7b, 0x71] {
        // High bits must be zero for the code to fit in a byte — which is
        // true for every panic code Solidity 0.8 defines (0x01..=0x51).
        if result.return_data[4..35].iter().all(|b| *b == 0) {
            return ObservedBehavior::Panicked(result.return_data[35]);
        }
    }
    // Legacy path — pre-Task-#103 Panic 0x11/0x41 emitters still push
    // `"Panic: 0x<hex>"` as a raw ByteString. Parse the hex out of the
    // exception message until those callers migrate to the canonical
    // envelope too.
    let msg = &exc.message;
    if let Some(idx) = msg.find("Panic: 0x") {
        let tail = &msg[idx + "Panic: 0x".len()..];
        // Selector is two hex digits followed by end-of-string or whitespace.
        let hex_part: String = tail
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .take(2)
            .collect();
        if !hex_part.is_empty() {
            if let Ok(sig) = u8::from_str_radix(&hex_part, 16) {
                return ObservedBehavior::Panicked(sig);
            }
        }
    }
    ObservedBehavior::FaultOther(msg.clone())
}
