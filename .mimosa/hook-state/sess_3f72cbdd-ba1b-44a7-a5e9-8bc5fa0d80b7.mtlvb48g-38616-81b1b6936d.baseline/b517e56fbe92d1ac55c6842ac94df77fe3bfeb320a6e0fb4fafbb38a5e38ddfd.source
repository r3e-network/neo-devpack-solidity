//! S1 fix tests — `StdLib.serialize` must emit the Neo N3 BinarySerializer
//! wire format (type-tagged little-endian), NOT JSON.
//!
//! Before this fix, `serialize` ran the StackItem through `serde_json`, which
//! produced `{"type":"ByteArray","value":[...]}` etc. That round-trips inside
//! the simulator but is byte-incompatible with real Neo N3 nodes: storage keys
//! derived from serialized values, length checks, and inter-contract interop
//! all silently diverged on-chain.
//!
//! The Neo N3 BinarySerializer format (neo-vm `StackItem.Serialize`, tags =
//! `StackItemType`) is:
//!   - 0x00 Null                : (nothing)
//!   - 0x20 Boolean             : 1 byte (0/1)
//!   - 0x21 Integer             : varbytes(minimal signed two's-complement LE)
//!   - 0x28 ByteString          : varint(len) || bytes
//!   - 0x30 Buffer              : varint(len) || bytes
//!   - 0x40 Array / 0x41 Struct : varint(count) || items...
//!   - 0x48 Map                 : varint(count) || (key value)... pairs
//!
//! with the canonical Bitcoin/Neo little-endian varint (0xFD/0xFE/0xFF prefixes).
//!
//! These tests reach `serialize`/`deserialize` via CALLT into the StdLib
//! native contract — the same path the compiled Solidity `StdLib.serialize`
//! lowering takes. CALLT args arrive as ByteArray StackItems (PUSHDATA1), so
//! the end-to-end assertions focus on ByteArray round-trips; the typed-scalar
//! (Integer/Boolean/Null/Array/Map) format is pinned by the unit tests inside
//! `stdlib.rs`.

use neo_devpack_solidity::neo::MethodToken;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

/// StdLib native-contract hash (UInt160 LE) — copied from
/// `src/runtime/spec/native_contracts.rs`. Same bytes used by every other
/// StdLib CALLT harness in this crate.
const STDLIB_HASH: [u8; 20] = [
    0xc0, 0xef, 0x39, 0xce, 0xe0, 0xe4, 0xe9, 0x25, 0xc6, 0xc2, 0xa0, 0x6a, 0x79, 0xe1, 0x44, 0x0d,
    0xd8, 0x6f, 0xce, 0xac,
];

fn build_callt_script(args: &[Vec<u8>]) -> Vec<u8> {
    let mut script = Vec::with_capacity(8 + args.iter().map(|a| 2 + a.len()).sum::<usize>());
    for arg in args {
        debug_assert!(arg.len() <= 255, "PUSHDATA1 max length is 255");
        script.push(0x0C); // PUSHDATA1
        script.push(arg.len() as u8);
        script.extend_from_slice(arg);
    }
    script.extend_from_slice(&[0x37, 0x00, 0x00, 0x40]); // CALLT token 0, RET
    script
}

fn call_stdlib(method: &str, args: &[Vec<u8>]) -> (bool, Vec<u8>, Option<String>) {
    let tokens = vec![MethodToken::new(
        STDLIB_HASH,
        method,
        args.len() as u16,
        true,
        0x0F,
    )];
    let script = build_callt_script(args);
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let result = rt
        .execute_with_tokens(&script, &[], &tokens)
        .expect("execute_with_tokens must not fail at host level");
    (
        result.success,
        result.return_data,
        result.exception.map(|e| e.message),
    )
}

#[test]
fn serialize_bytearray_emits_neo_binary_varint_prefixed() {
    // serialize(ByteString [0xAA, 0xBB]) => [0x28, 0x02, 0xAA, 0xBB]
    //   tag=0x28 (ByteString), varint len=2, raw bytes.
    let (ok, out, exc) = call_stdlib("serialize", &[vec![0xAA, 0xBB]]);
    assert!(ok, "serialize must succeed; exc={exc:?}");
    assert_eq!(
        out,
        vec![0x28, 0x02, 0xAA, 0xBB],
        "serialize(ByteArray [AA,BB]) must emit Neo binary [0x28, varint(2), AA, BB], not JSON"
    );
}

#[test]
fn serialize_empty_bytearray_emits_tag_and_zero_len() {
    // serialize(ByteString []) => [0x28, 0x00]
    let (ok, out, exc) = call_stdlib("serialize", &[vec![]]);
    assert!(ok, "serialize must succeed; exc={exc:?}");
    assert_eq!(out, vec![0x28, 0x00]);
}

#[test]
fn serialize_deserialize_bytearray_roundtrips() {
    let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let (ok_s, ser, exc_s) = call_stdlib("serialize", std::slice::from_ref(&bytes));
    assert!(ok_s, "serialize must succeed; exc={exc_s:?}");
    assert_eq!(ser, vec![0x28, 0x04, 0xDE, 0xAD, 0xBE, 0xEF]);

    let (ok_d, deser, exc_d) = call_stdlib("deserialize", &[ser]);
    assert!(ok_d, "deserialize must succeed; exc={exc_d:?}");
    assert_eq!(
        deser, bytes,
        "deserialize(serialize(ByteArray)) returns the same bytes"
    );
}

#[test]
fn serialize_output_is_not_json() {
    // Regression guard: any JSON-shaped output (starts with '{' or '[') means
    // the implementation reverted to serde_json.
    let (ok, out, _) = call_stdlib("serialize", &[vec![42]]);
    assert!(ok);
    assert!(
        !out.starts_with(b"{") && !out.starts_with(b"["),
        "serialize must NOT produce JSON; got: {:?}",
        String::from_utf8_lossy(&out)
    );
}

#[test]
fn jsonserialize_still_emits_json() {
    // S1 fix contract: jsonSerialize stays JSON (only `serialize` switches to
    // the Neo binary format). This guards against accidentally changing both.
    let (ok, out, _) = call_stdlib("jsonserialize", &[vec![0x42]]);
    assert!(ok);
    assert!(
        out.starts_with(b"{") || out.starts_with(b"["),
        "jsonSerialize must still emit JSON; got: {:?}",
        String::from_utf8_lossy(&out)
    );
}
