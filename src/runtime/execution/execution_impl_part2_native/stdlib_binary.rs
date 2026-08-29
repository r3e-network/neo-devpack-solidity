//! ## Neo N3 BinarySerializer (Runtime)
//!
//! Implementation of `StdLib.serialize` and `StdLib.deserialize` for the Neo N3
//! wire format. Extracted from stdlib.rs to keep the dispatch module under the
//! 800-line limit.
//!
//! Functions live in an `impl ExecutionContext` block but are `&self`-free.

use super::*;
use crate::runtime::execution::types::stack::ByteArrayType;

// Section 2 — Neo N3 BinarySerializer (StdLib.serialize / deserialize)
// ============================================================================
//
// Byte format per neo-vm `StackItem.Serialize` / `BinarySerializer`:
//   0x00 Null               : (no payload)
//   0x20 Boolean            : 1 byte (0x00 / 0x01)
//   0x21 Integer            : varbytes(minimal signed two's-complement LE)
//   0x28 ByteString         : varbytes(bytes)
//   0x30 Buffer             : varbytes(bytes)
//   0x40 Array              : varint(count) || item…             (recursive)
//   0x41 Struct             : varint(count) || item…             (recursive)
//   0x48 Map                : varint(count) || (key value)…      (recursive)
//
// `varint` is the canonical Bitcoin/Neo little-endian length prefix used by
// `BinaryWriter.WriteVarInt`: values < 0xFD are one byte; 0xFD/0xFE/0xFF
// prefixes introduce a 2/4/8-byte little-endian payload. `varbytes` is a
// `varint` length followed by that many raw bytes.
//
// The pre-fix implementation used invented tags (`ByteArray` 0x00, `Boolean`
// 0x01, `Integer` 0x02 as a FIXED 8-byte LE, `Null` 0x03, `Map` 0x80) and a
// non-Neo MSB-first 7-bit continuation varint. Round-trips succeeded only
// inside the simulator; every `serialize`-derived storage key or interop
// payload was byte-incompatible with a real node. This is now aligned to the
// real format, and the deserializer enforces the memory-safety bounds the real
// `BinarySerializer` relies on (lengths clamped to the remaining buffer, a
// nesting-depth cap, no preallocation from an unvalidated count, trailing
// bytes and unknown tags rejected).

/// `BinarySerializer.MaxNestedDepth` (neo-vm).
const MAX_NESTED_DEPTH: usize = 10;
/// The simulator models `Integer`/`UnsignedInteger` as 64-bit items; a decoded
/// integer longer than 9 bytes cannot be represented (a u64 above `i64::MAX`
/// serializes to 9 bytes once the non-negative sign byte is added). Real Neo
/// allows up to 32-byte BigIntegers — a separate simulator-model limitation.
const MAX_INTEGER_BYTES: usize = 9;

/// StackItemType byte for the type-tagged `ByteArray` variant.
fn byte_array_type_tag(tag: ByteArrayType) -> u8 {
    match tag {
        ByteArrayType::ByteString => 0x28,
        ByteArrayType::Buffer => 0x30,
    }
}

/// Minimal-length signed two's-complement little-endian encoding of an `i64`,
/// matching .NET `BigInteger.ToByteArray()` (and thus Neo's `Integer` payload).
/// Zero is encoded as the single byte `0x00`.
fn signed_le_bytes(value: i64) -> Vec<u8> {
    if value == 0 {
        return vec![0u8];
    }
    num_bigint::BigInt::from(value).to_signed_bytes_le()
}

impl ExecutionContext {
    /// Encode `value` in the Neo N3 BinarySerializer wire format.
    pub(crate) fn neo_binary_serialize(value: &StackItem) -> Vec<u8> {
        let mut out = Vec::new();
        Self::neo_binary_serialize_into(value, &mut out);
        out
    }

    fn neo_binary_serialize_into(value: &StackItem, out: &mut Vec<u8>) {
        match value {
            StackItem::ByteArray { data: rc, type_tag } => {
                let bytes = rc.borrow();
                out.push(byte_array_type_tag(*type_tag));
                Self::neo_write_varint(out, bytes.len() as u64);
                out.extend_from_slice(&bytes);
            }
            StackItem::Boolean(b) => {
                out.push(0x20); // Boolean
                out.push(if *b { 1 } else { 0 });
            }
            StackItem::Integer(i) => {
                out.push(0x21); // Integer
                let bytes = signed_le_bytes(*i);
                Self::neo_write_varint(out, bytes.len() as u64);
                out.extend_from_slice(&bytes);
            }
            StackItem::UnsignedInteger(u) => {
                // Neo has no unsigned stack type; encode the value as a
                // non-negative BigInteger (minimal signed LE keeps the sign
                // byte, so a u64 in (i64::MAX, u64::MAX] round-trips).
                out.push(0x21); // Integer
                let bytes = num_bigint::BigInt::from(*u).to_signed_bytes_le();
                Self::neo_write_varint(out, bytes.len() as u64);
                out.extend_from_slice(&bytes);
            }
            StackItem::Null => {
                out.push(0x00); // Any / Null
            }
            StackItem::Array(rc) => {
                out.push(0x40); // Array
                let items = rc.borrow();
                Self::neo_write_varint(out, items.len() as u64);
                for item in items.iter() {
                    Self::neo_binary_serialize_into(item, out);
                }
            }
            StackItem::Map(rc) => {
                out.push(0x48); // Map
                let map = rc.borrow();
                Self::neo_write_varint(out, map.len() as u64);
                for (k, v) in map.iter() {
                    Self::neo_binary_serialize_into(&StackItem::byte_array(k.clone()), out);
                    Self::neo_binary_serialize_into(v, out);
                }
            }
        }
    }

    /// Decode a Neo N3 BinarySerializer byte stream back into a StackItem.
    /// Returns `None` on truncated / malformed / over-nested / trailing-byte
    /// input (mirrors a real node faulting). Never preallocates from an
    /// unvalidated count.
    pub(crate) fn neo_binary_deserialize(bytes: &[u8]) -> Option<StackItem> {
        let mut cursor = 0usize;
        let item = Self::neo_binary_deserialize_from(bytes, &mut cursor, 0)?;
        // Neo's BinarySerializer requires the whole buffer to be consumed; a
        // well-formed stream has exactly one top-level item with no trailer.
        if cursor != bytes.len() {
            return None;
        }
        Some(item)
    }

    fn neo_binary_deserialize_from(
        bytes: &[u8],
        cursor: &mut usize,
        depth: usize,
    ) -> Option<StackItem> {
        if depth > MAX_NESTED_DEPTH {
            return None;
        }
        if *cursor >= bytes.len() {
            return None;
        }
        let tag = bytes[*cursor];
        *cursor += 1;
        match tag {
            0x00 => Some(StackItem::Null),
            0x20 => {
                // Boolean
                if *cursor >= bytes.len() {
                    return None;
                }
                let raw = bytes[*cursor];
                *cursor += 1;
                match raw {
                    0x00 => Some(StackItem::Boolean(false)),
                    0x01 => Some(StackItem::Boolean(true)),
                    // Neo only ever writes 0/1; anything else is malformed.
                    _ => None,
                }
            }
            0x21 => {
                // Integer: varbytes(minimal signed LE), bounded by the 64-bit
                // simulator item model.
                let data = Self::neo_read_varbytes(bytes, cursor)?;
                if data.is_empty() || data.len() > MAX_INTEGER_BYTES {
                    return None;
                }
                // Reject non-minimal (over-long) encodings to match the reader.
                let value = num_bigint::BigInt::from_signed_bytes_le(data);
                if let Ok(i) = i64::try_from(value.clone()) {
                    Some(StackItem::Integer(i))
                } else if let Ok(u) = u64::try_from(value) {
                    Some(StackItem::UnsignedInteger(u))
                } else {
                    None
                }
            }
            0x28 | 0x30 => {
                // ByteString (0x28) / Buffer (0x30): varbytes.
                let data = Self::neo_read_varbytes(bytes, cursor)?;
                let owned = data.to_vec();
                if tag == 0x30 {
                    Some(StackItem::buffer(owned))
                } else {
                    Some(StackItem::byte_array(owned))
                }
            }
            0x40 | 0x41 => {
                // Array (0x40) / Struct (0x41): varint(count) || item…
                // The simulator has no distinct Struct item, so both decode to
                // an Array (matching the compiler's Array↔Struct coercion).
                let count = Self::neo_read_varint(bytes, cursor)?;
                // A valid count can never exceed the remaining bytes (every
                // element is ≥ 1 byte). Clamp BEFORE any allocation so a
                // crafted ~10-byte stream cannot force a multi-gigabyte reserve.
                let remaining = bytes.len().saturating_sub(*cursor);
                if count > remaining as u64 {
                    return None;
                }
                let count = count as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(Self::neo_binary_deserialize_from(bytes, cursor, depth + 1)?);
                }
                Some(StackItem::array(items))
            }
            0x48 => {
                // Map (0x48): varint(count) || (key value)…
                // Each entry needs ≥ 2 item headers, so `count > remaining / 2`
                // is impossible for well-formed input; clamp before allocating.
                let count = Self::neo_read_varint(bytes, cursor)?;
                let remaining = bytes.len().saturating_sub(*cursor);
                if count > (remaining as u64) / 2 {
                    return None;
                }
                let count = count as usize;
                let mut map = std::collections::HashMap::with_capacity(count);
                for _ in 0..count {
                    let key_item = Self::neo_binary_deserialize_from(bytes, cursor, depth + 1)?;
                    let key = Self::stack_item_to_bytes(key_item);
                    let value = Self::neo_binary_deserialize_from(bytes, cursor, depth + 1)?;
                    map.insert(key, value);
                }
                Some(StackItem::map(map))
            }
            // Unknown / unrepresentable type tag (Pointer 0x10, InteropInterface
            // 0x60, etc.) — Neo faults on these.
            _ => None,
        }
    }

    /// Canonical Bitcoin/Neo little-endian length prefix (`WriteVarInt`).
    fn neo_write_varint(out: &mut Vec<u8>, value: u64) {
        match value {
            0x0000_0000_0000_0000..=0x0000_0000_0000_00FC => out.push(value as u8),
            0x0000_0000_0000_00FD..=0x0000_0000_0000_FFFF => {
                out.push(0xFD);
                out.extend_from_slice(&(value as u16).to_le_bytes());
            }
            0x0000_0000_0001_0000..=0x0000_0000_FFFF_FFFF => {
                out.push(0xFE);
                out.extend_from_slice(&(value as u32).to_le_bytes());
            }
            _ => {
                out.push(0xFF);
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    /// Inverse of [`Self::neo_write_varint`] (`ReadVarInt`), rejecting a
    /// truncated prefix and rejecting oversized 64-bit lengths that cannot fit
    /// in the remaining buffer (callers additionally clamp against `remaining`).
    fn neo_read_varint(bytes: &[u8], cursor: &mut usize) -> Option<u64> {
        let b = *bytes.get(*cursor)?;
        *cursor += 1;
        match b {
            0x00..=0xFC => Some(b as u64),
            0xFD => {
                let end = cursor.checked_add(2)?;
                let raw = bytes.get(*cursor..end)?;
                let v = u16::from_le_bytes([raw[0], raw[1]]) as u64;
                // Canonical: 0xFD must not encode a value that fits one byte.
                if v < 0xFD {
                    return None;
                }
                *cursor = end;
                Some(v)
            }
            0xFE => {
                let end = cursor.checked_add(4)?;
                let raw = bytes.get(*cursor..end)?;
                let v = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as u64;
                if v < 0x0001_0000 {
                    return None;
                }
                *cursor = end;
                Some(v)
            }
            0xFF => {
                let end = cursor.checked_add(8)?;
                let raw = bytes.get(*cursor..end)?;
                let v = u64::from_le_bytes([
                    raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
                ]);
                if v < 0x0000_0001_0000_0000 {
                    return None;
                }
                *cursor = end;
                Some(v)
            }
        }
    }

    /// `ReadVarBytes`: a varint length clamped against the remaining buffer
    /// (overflow-free), then that many raw bytes. Never trusts the length to
    /// index past the end.
    fn neo_read_varbytes<'a>(bytes: &'a [u8], cursor: &mut usize) -> Option<&'a [u8]> {
        let len = Self::neo_read_varint(bytes, cursor)? as usize;
        let remaining = bytes.len().checked_sub(*cursor)?;
        if len > remaining {
            return None;
        }
        let end = cursor.checked_add(len)?;
        let slice = &bytes[*cursor..end];
        *cursor = end;
        Some(slice)
    }
}

#[cfg(test)]
mod neo_binary_tests {
    use super::*;

    /// ByteString serializes as `[0x28, varint(len), bytes...]`.
    #[test]
    fn serialize_bytearray_format() {
        let item = StackItem::byte_array(vec![0xAA, 0xBB]);
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&item),
            vec![0x28, 0x02, 0xAA, 0xBB]
        );
        let empty = StackItem::byte_array(vec![]);
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&empty),
            vec![0x28, 0x00]
        );
    }

    /// Buffer uses the 0x30 tag, distinct from ByteString 0x28 (type-strict on
    /// the real node).
    #[test]
    fn serialize_buffer_tag() {
        let buf = StackItem::ByteArray {
            data: std::rc::Rc::new(std::cell::RefCell::new(vec![0x01])),
            type_tag: ByteArrayType::Buffer,
        };
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&buf),
            vec![0x30, 0x01, 0x01]
        );
    }

    /// Integer uses tag 0x21 and a MINIMAL signed two's-complement LE payload
    /// (NOT a fixed 8-byte word).
    #[test]
    fn serialize_integer_format() {
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Integer(2)),
            vec![0x21, 0x01, 0x02]
        );
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Integer(-1)),
            vec![0x21, 0x01, 0xFF]
        );
        // 255 needs a sign byte to stay positive → 0x02 payload bytes.
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Integer(255)),
            vec![0x21, 0x02, 0xFF, 0x00]
        );
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Integer(0)),
            vec![0x21, 0x01, 0x00]
        );
    }

    #[test]
    fn serialize_boolean_and_null_format() {
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Boolean(true)),
            vec![0x20, 0x01]
        );
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Boolean(false)),
            vec![0x20, 0x00]
        );
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&StackItem::Null),
            vec![0x00]
        );
    }

    #[test]
    fn serialize_array_format() {
        // Array [Integer 1, Integer 2] =>
        //   0x40, varint(2)=0x02, 0x21 0x01 0x01, 0x21 0x01 0x02
        let item = StackItem::array(vec![StackItem::Integer(1), StackItem::Integer(2)]);
        assert_eq!(
            ExecutionContext::neo_binary_serialize(&item),
            vec![0x40, 0x02, 0x21, 0x01, 0x01, 0x21, 0x01, 0x02]
        );
    }

    /// Canonical Bitcoin/Neo little-endian varint (was previously a non-Neo
    /// MSB-first 7-bit scheme).
    #[test]
    fn varint_canonical_format() {
        let enc = |v: u64| {
            let mut out = Vec::new();
            ExecutionContext::neo_write_varint(&mut out, v);
            out
        };
        assert_eq!(enc(0), vec![0x00]);
        assert_eq!(enc(0xFC), vec![0xFC]);
        assert_eq!(enc(0xFD), vec![0xFD, 0xFD, 0x00]);
        assert_eq!(enc(200), vec![0xC8]);
        assert_eq!(enc(0xFFFF), vec![0xFD, 0xFF, 0xFF]);
        assert_eq!(enc(0x0001_0000), vec![0xFE, 0x00, 0x00, 0x01, 0x00]);

        for v in [
            0u64,
            1,
            0xFC,
            0xFD,
            200,
            0xFFFF,
            0x0001_0000,
            0xFFFF_FFFF,
            0x0000_0001_0000_0000,
            u64::MAX,
        ] {
            let bytes = enc(v);
            let mut cur = 0;
            assert_eq!(
                ExecutionContext::neo_read_varint(&bytes, &mut cur),
                Some(v),
                "varint rt {v}"
            );
            assert_eq!(cur, bytes.len());
        }
    }

    #[test]
    fn roundtrip_all_scalar_types() {
        let cases = vec![
            StackItem::byte_array(vec![0xDE, 0xAD, 0xBE, 0xEF]),
            StackItem::byte_array(vec![]),
            StackItem::Integer(0),
            StackItem::Integer(123456789),
            StackItem::Integer(-42),
            StackItem::UnsignedInteger(9_000_000_000_000_000_000), // > i64::MAX
            StackItem::Boolean(true),
            StackItem::Boolean(false),
            StackItem::Null,
            StackItem::array(vec![
                StackItem::Integer(1),
                StackItem::byte_array(vec![0x01, 0x02]),
            ]),
        ];
        for item in cases {
            let ser = ExecutionContext::neo_binary_serialize(&item);
            let de = ExecutionContext::neo_binary_deserialize(&ser)
                .unwrap_or_else(|| panic!("deserialize failed for {item:?} (bytes {ser:?})"));
            let re_ser = ExecutionContext::neo_binary_serialize(&de);
            assert_eq!(re_ser, ser, "round-trip not canonical for {item:?}");
        }
    }

    #[test]
    fn deserialize_truncated_input_returns_none() {
        // Integer tag with no payload.
        assert!(ExecutionContext::neo_binary_deserialize(&[0x21]).is_none());
        // Integer claiming 8 payload bytes but only 1 present.
        assert!(ExecutionContext::neo_binary_deserialize(&[0x21, 0x08, 0xAA]).is_none());
        // ByteArray claiming 5 bytes with only 1 present.
        assert!(ExecutionContext::neo_binary_deserialize(&[0x28, 0x05, 0xAA]).is_none());
        // Empty input.
        assert!(ExecutionContext::neo_binary_deserialize(&[]).is_none());
    }

    /// A crafted Array count must not force a huge `with_capacity` reserve.
    #[test]
    fn deserialize_oversized_array_count_faults_not_oom() {
        // 0x40 (Array) + 0xFF + 8-byte LE = 2^40 element count, ~9 bytes input.
        let mut payload = vec![0x40, 0xFF];
        payload.extend_from_slice(&0x0000_0100_0000_0000u64.to_le_bytes());
        assert!(ExecutionContext::neo_binary_deserialize(&payload).is_none());
    }

    /// The `*cursor + len` overflow that previously slipped the bounds check.
    #[test]
    fn deserialize_length_overflow_no_panic() {
        // ByteArray tag, varint length near u64::MAX, no real bytes.
        let mut payload = vec![0x28, 0xFF];
        payload.extend_from_slice(&u64::MAX.to_le_bytes());
        assert!(ExecutionContext::neo_binary_deserialize(&payload).is_none());
    }

    /// Nested arrays beyond `MaxNestedDepth` must fault, not stack-overflow.
    #[test]
    fn deserialize_exceeding_depth_faults() {
        // Build depth+2 nested single-element arrays: [0x40,0x01] repeated,
        // terminated by a Null.
        let mut payload = Vec::new();
        for _ in 0..(MAX_NESTED_DEPTH + 5) {
            payload.push(0x40);
            payload.push(0x01);
        }
        payload.push(0x00);
        assert!(ExecutionContext::neo_binary_deserialize(&payload).is_none());
    }

    #[test]
    fn deserialize_rejects_trailing_bytes_and_unknown_tag() {
        // Valid Null (0x00) followed by a stray byte → reject.
        assert!(ExecutionContext::neo_binary_deserialize(&[0x00, 0x00]).is_none());
        // Unknown type tag 0x10 (Pointer) → fault (None), not silent Null.
        assert!(ExecutionContext::neo_binary_deserialize(&[0x10]).is_none());
    }
}
