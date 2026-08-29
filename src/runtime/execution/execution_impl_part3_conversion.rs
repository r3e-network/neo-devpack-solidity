use super::*;

impl ExecutionContext {
    pub(crate) fn convert_item(
        &self,
        item: StackItem,
        target_code: u8,
    ) -> Result<StackItem, RuntimeError> {
        match target_code {
            0x00 => Ok(item), // Any/no-op
            0x20 => Ok(StackItem::Boolean(item.is_truthy())),
            0x21 | 0x22 => {
                // NeoVM CONVERT→Integer is only valid from Boolean / Integer /
                // ByteString / Buffer; a compound (Array/Map) operand throws
                // ("not supported"). Faulting here matches real NeoVM instead of
                // silently producing a degenerate integer.
                if matches!(item, StackItem::Array(_) | StackItem::Map(_)) {
                    return Err(RuntimeError::ExecutionError {
                        message: "CONVERT: cannot convert a compound type (Array/Map) to Integer"
                            .to_string(),
                    });
                }
                // NeoVM CONVERT→Integer: interpret the byte buffer as a signed
                // little-endian arbitrary-precision integer. Narrow results
                // (≤ 8 bytes) fit in `StackItem::Integer(i64)`; wider results
                // are preserved as signed-LE `StackItem::ByteArray` so the
                // downstream BigInt arithmetic path (`coerce_item_to_bigint`,
                // `bigint_to_stack_item`) keeps full precision for values that
                // exceed 64 bits (e.g. `uint256(bytes32(...))` with high bytes
                // populated). Truncating to the first 8 bytes here was Task
                // #111's root cause — for a 32-byte LE buffer whose magnitude
                // lives above byte 7, the naive `from_le_bytes([..8])` path
                // returned 0.
                let bytes = Self::stack_item_to_bytes(item);
                if bytes.is_empty() {
                    return Ok(StackItem::Integer(0));
                }
                // NeoVM CONVERT→Integer rejects ByteStrings wider than 32 bytes
                // (the signed-256-bit maximum). Real nodes fault here; matching
                // that behaviour surfaces `[2^255, 2^256-1]`-class lowering bugs
                // in the simulator instead of letting them slip to on-chain
                // faults (the POW `**` overflow guard's 33-byte `2^256` literal
                // was the canonical example — caught by the neoxp differential
                // harness). Values that legitimately fit (≤ 32 bytes) are
                // preserved as a signed-LE ByteArray below so the wide BigInt
                // arithmetic path keeps full precision.
                if bytes.len() > 32 {
                    return Err(RuntimeError::ExecutionError {
                        message: format!(
                            "CONVERT: ByteString length {} exceeds the 32-byte Integer maximum",
                            bytes.len()
                        ),
                    });
                }
                if bytes.is_empty() {
                    return Ok(StackItem::Integer(0.into()));
                }
                if bytes.len() <= 8 {
                    // Sign-extend from the high bit of the last byte so signed
                    // LE encodings shorter than 8 bytes round-trip correctly.
                    let mut buf = [0u8; 8];
                    buf[..bytes.len()].copy_from_slice(&bytes);
                    let sign = *bytes.last().expect("guarded by non-empty check") & 0x80;
                    if sign != 0 {
                        for byte in buf.iter_mut().skip(bytes.len()) {
                            *byte = 0xFF;
                        }
                    }
                    Ok(StackItem::Integer(i64::from_le_bytes(buf)))
                } else {
                    // Preserve the signed-LE encoding verbatim — the wide
                    // arithmetic path (Task #30) already decodes ByteArray via
                    // `BigInt::from_signed_bytes_le`.
                    Ok(StackItem::byte_array(bytes))
                }
            }
            0x28 | 0x30 => {
                // NeoVM CONVERT→ByteString/Buffer of an Integer yields the
                // MINIMAL two's-complement little-endian encoding (zero ⇒ empty
                // span), NOT a fixed 8-byte word. Match a real node here so a
                // contract that converts an integer to bytes (and inspects its
                // length, hashes it, or concatenates it) sees on-chain widths.
                // The output type_tag mirrors the target: 0x28→ByteString, 0x30→Buffer.
                let bytes = match item {
                    StackItem::Integer(_) | StackItem::UnsignedInteger(_) => {
                        let n = self.coerce_item_to_bigint(&item).unwrap_or_default();
                        if n.sign() == num_bigint::Sign::NoSign {
                            Vec::new()
                        } else {
                            n.to_signed_bytes_le()
                        }
                    }
                    _ => Self::stack_item_to_bytes(item),
                };
                if target_code == 0x30 {
                    Ok(StackItem::buffer(bytes))
                } else {
                    Ok(StackItem::byte_array(bytes))
                }
            }
            0x40 | 0x41 => match item {
                StackItem::Array(items) => Ok(StackItem::Array(items)),
                StackItem::Map(map) => Ok(StackItem::array(
                    map.borrow()
                        .iter()
                        .map(|(k, v)| {
                            StackItem::array(vec![StackItem::byte_array(k.clone()), v.clone()])
                        })
                        .collect(),
                )),
                other => Ok(StackItem::array(vec![other])),
            },
            0x48 => match item {
                StackItem::Map(map) => Ok(StackItem::Map(map)),
                StackItem::Array(items) => {
                    let mut map = std::collections::HashMap::new();
                    for pair in items.borrow().iter() {
                        let StackItem::Array(kv) = pair else {
                            continue;
                        };
                        let kv = kv.borrow();
                        if kv.len() < 2 {
                            continue;
                        }
                        let key = kv.first().cloned().unwrap_or(StackItem::Null);
                        let value = kv.last().cloned().unwrap_or(StackItem::Null);
                        map.insert(Self::stack_item_to_bytes(key), value);
                    }
                    Ok(StackItem::map(map))
                }
                other => {
                    let mut map = std::collections::HashMap::new();
                    map.insert(Vec::new(), other);
                    Ok(StackItem::map(map))
                }
            },
            0x80 => Ok(item), // iterator tokens already byte arrays; leave untouched
            _ => Ok(item),
        }
    }

    pub(crate) fn is_iterator_token(&self, item: &StackItem) -> bool {
        if let Some(id) = Self::iterator_id_from_item(item) {
            return self.iterators.contains_key(&id);
        }
        false
    }
}
