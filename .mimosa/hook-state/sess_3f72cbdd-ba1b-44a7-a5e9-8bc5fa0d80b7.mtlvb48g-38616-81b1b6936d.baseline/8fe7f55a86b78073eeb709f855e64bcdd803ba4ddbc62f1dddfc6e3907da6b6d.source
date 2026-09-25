//! ## Native StdLib & ABI Runtime
//!
//! This module implements the Neo N3 `StdLib` native contract interface
//! dispatcher and the base64 helper methods used by its dispatch arms.
//!
//! The large ABI-encoding and BinarySerializer helper blocks that previously
//! lived here have been split out to keep each file under the 800-line limit:
//!
//! * `stdlib_abi.rs` — EVM-canonical ABI encoding helpers used by the
//!   `abiencode`, `abiencodepacked`, `abidecode`, and `itoa` / `atoi` arms.
//! * `stdlib_binary.rs` — Neo N3 `BinarySerializer` wire-format helpers used
//!   by the `serialize` and `deserialize` arms, plus their unit tests.
//! * `stdlib.rs` — this file, containing only the dispatcher and the base64
//!   encode/decode helpers.
//!
//! All three modules contribute `impl ExecutionContext` blocks; the associated
//! functions are called as `Self::abi_*` and `Self::neo_binary_*` from the
//! dispatcher below.

use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_stdlib(method: &str, params: StackItem) -> StackItem {
        match method {
            "serialize" => {
                // S1 fix: emit the Neo N3 BinarySerializer wire format, NOT
                // JSON. Real Neo N3 nodes serialize StackItems as type-tagged
                // little-endian; the previous serde_json path produced
                // `{"type":"ByteArray","value":[...]}` which round-trips inside
                // the simulator but is byte-incompatible on-chain (storage
                // keys, length checks, and inter-contract interop all
                // diverged). `jsonSerialize` below still produces JSON for
                // callers that explicitly want the JSON form.
                if let StackItem::Array(args) = params {
                    let value = args.borrow().first().cloned().unwrap_or(StackItem::Null);
                    let bytes = Self::neo_binary_serialize(&value);
                    StackItem::byte_array(bytes)
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "deserialize" => {
                // S1 fix: decode the Neo N3 BinarySerializer wire format (the
                // inverse of `serialize` above).
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    Self::neo_binary_deserialize(&bytes).unwrap_or(StackItem::Null)
                } else {
                    StackItem::Null
                }
            }
            "jsonserialize" => {
                if let StackItem::Array(args) = params {
                    let value = args.borrow().first().cloned().unwrap_or(StackItem::Null);
                    let json = serde_json::to_string(&value).unwrap_or_default();
                    StackItem::byte_array(json.into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "jsondeserialize" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let json_str = String::from_utf8(bytes).unwrap_or_default();
                    serde_json::from_str::<StackItem>(&json_str).unwrap_or(StackItem::Null)
                } else {
                    StackItem::Null
                }
            }
            // Tasks #44/#72/#73 — EVM-canonical ABI encoder. Given an outer
            // `StackItem::Array` of arguments, emit the spec-compliant
            // head+tail layout:
            //
            //   * Static args (integer-shaped, address, bytesN, bool): one
            //     32-byte BE-padded slot in the head section. Tail: none.
            //   * Dynamic args (string, bytes, dynamic arrays): the head
            //     slot carries a 32-byte BE offset pointing at the tail
            //     section (measured from the start of the encoded data);
            //     the tail section itself is `len(32) || data(padded to a
            //     32-byte multiple)`. The first dynamic arg's tail starts
            //     immediately after the head (offset = 32 * arg_count).
            //
            // When EVERY arg is static (e.g. `abi.encode(uint256, uint256)`
            // or `abi.encode(address)`) the output collapses to the classic
            // `concat(pad32_be(arg_i))` Task #44 shape — preserves existing
            // static-path tests.
            //
            // Heuristic (the Solidity type is lost by runtime dispatch time):
            // a `StackItem::ByteArray` is classified dynamic ONLY for the
            // short-literal / empty-buffer / long-buffer widths that fall
            // outside the integer/address/bytes32 static heuristics baked
            // into `abi_pad32_be` (see `abi_is_dynamic` in stdlib_abi.rs).
            "abiencode" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    let inner = if borrowed.len() == 1 {
                        // The runtime wraps the positional args in an outer
                        // Array; when the caller passed a single Array to
                        // `abiEncode` (how the bytecode emitter packs things),
                        // unwrap one level.
                        match &borrowed[0] {
                            StackItem::Array(nested) => nested.borrow().clone(),
                            other => vec![other.clone()],
                        }
                    } else {
                        borrowed.clone()
                    };
                    let is_dynamic: Vec<bool> = inner.iter().map(Self::abi_is_dynamic).collect();
                    if !is_dynamic.iter().any(|b| *b) {
                        // Fast path: static-only encoding (Task #44 shape).
                        let mut out = Vec::with_capacity(inner.len() * 32);
                        for item in inner.iter() {
                            out.extend_from_slice(&Self::abi_pad32_be(item));
                        }
                        return StackItem::byte_array(out);
                    }
                    // Head+tail encoding (EVM spec, Tasks #72/#73).
                    let head_len = inner.len() * 32;
                    let mut tails: Vec<Vec<u8>> = Vec::with_capacity(inner.len());
                    let mut running_offset: u64 = head_len as u64;
                    let mut head = Vec::with_capacity(head_len);
                    for (i, item) in inner.iter().enumerate() {
                        if is_dynamic[i] {
                            let tail = Self::abi_dynamic_tail_bytes(item);
                            let mut offset_slot = [0u8; 32];
                            offset_slot[24..].copy_from_slice(&running_offset.to_be_bytes());
                            head.extend_from_slice(&offset_slot);
                            running_offset += tail.len() as u64;
                            tails.push(tail);
                        } else {
                            head.extend_from_slice(&Self::abi_pad32_be(item));
                        }
                    }
                    let total = head.len() + tails.iter().map(|t| t.len()).sum::<usize>();
                    let mut out = Vec::with_capacity(total);
                    out.extend_from_slice(&head);
                    for tail in &tails {
                        out.extend_from_slice(tail);
                    }
                    StackItem::byte_array(out)
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // Task #44 — packed variant. Same input convention; each element
            // is serialised to its packed width and concatenated.
            "abiencodepacked" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    let inner = if borrowed.len() == 1 {
                        match &borrowed[0] {
                            StackItem::Array(nested) => nested.borrow().clone(),
                            other => vec![other.clone()],
                        }
                    } else {
                        borrowed.clone()
                    };
                    let mut out = Vec::new();
                    for item in inner.iter() {
                        out.extend_from_slice(&Self::abi_packed_bytes(item));
                    }
                    StackItem::byte_array(out)
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // Task #44 — round-trip decoder. Accepts a ByteArray produced by
            // `abiencode` (32*N BE-packed) and returns an `Array` of
            // `UnsignedInteger` slots (one per 32-byte chunk). This preserves
            // `abi.decode(abi.encode(x))` equivalence when the argument is
            // integer-typed; bytes/address decoding falls back to the raw
            // ByteArray slot for the caller to re-interpret.
            "abidecode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    if bytes.is_empty() || !bytes.len().is_multiple_of(32) {
                        // Not a canonical payload — return as-is so callers
                        // that passed through a non-32-aligned buffer still
                        // get something sensible.
                        return StackItem::byte_array(bytes);
                    }
                    // Task #121 — dynamic-array detection: an EVM-canonical
                    // `abi.encode(T[])` payload is `[offset=32][length=N]
                    // [element_0]..[element_{N-1}]` — total = 64 + N*32 bytes,
                    // with the first 32-byte slot equal to 0x20 (offset to
                    // the length field) and the second 32-byte slot equal to
                    // N (which exactly accounts for the remaining data).
                    //
                    // When that signature matches, return the bytes verbatim
                    // as a `ByteArray` so the outer RET path emits the
                    // EVM-canonical encoding of the `T[]` unchanged. The
                    // calling site (e.g. `return abi.decode(encoded, (uint[]))`
                    // from batch53_cc2) would otherwise round-trip through
                    // `Array → serde_json` and leak JSON as return data.
                    //
                    // Does NOT recurse into dynamic-array-of-dynamic (`T[][]`
                    // / `string[]`) or struct tuples — those patterns don't
                    // match the simple offset=32 + single-length signature
                    // and stay on the scalar-slot path.
                    if bytes.len() >= 64 {
                        let first_is_offset_20 = bytes[..24].iter().all(|b| *b == 0) && {
                            let mut buf = [0u8; 8];
                            buf.copy_from_slice(&bytes[24..32]);
                            u64::from_be_bytes(buf) == 32
                        };
                        let declared_len = {
                            let mut buf = [0u8; 8];
                            buf.copy_from_slice(&bytes[56..64]);
                            u64::from_be_bytes(buf)
                        };
                        let len_high_zero = bytes[32..56].iter().all(|b| *b == 0);
                        let payload_matches =
                            len_high_zero && (64 + declared_len as usize * 32) == bytes.len();
                        if first_is_offset_20 && payload_matches {
                            return StackItem::byte_array(bytes);
                        }
                    }
                    let slots: Vec<StackItem> = bytes
                        .chunks_exact(32)
                        .map(|chunk| {
                            // Decode as u64 when high bits are zero — lossless
                            // for the common uint64/uint256<=u64::MAX path.
                            let high_zero = chunk[..24].iter().all(|b| *b == 0);
                            if high_zero {
                                let mut buf = [0u8; 8];
                                buf.copy_from_slice(&chunk[24..]);
                                StackItem::UnsignedInteger(u64::from_be_bytes(buf))
                            } else {
                                // A 32-byte big-endian integer slot whose high
                                // bit is set is a uint256 >= 2^255: store it as
                                // the canonical 32-byte TWO'S-COMPLEMENT (as the
                                // rest of the runtime does) so it compares equal
                                // to the same value produced by literals/arith,
                                // instead of a positive-magnitude 33-byte form.
                                let v = num_bigint::BigInt::from_bytes_be(
                                    num_bigint::Sign::Plus,
                                    chunk,
                                );
                                Self::u256_twos_complement_item(v)
                            }
                        })
                        .collect();
                    if slots.len() == 1 {
                        slots.into_iter().next().expect("guarded by len() == 1")
                    } else {
                        StackItem::array(slots)
                    }
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // Task #51 — StdLib.itoa(value[, base]): format integer as
            // decimal (or hex when base=16) ASCII string, returned as a
            // `ByteArray`. Matches `callt_stdlib_itoa_roundtrip_via_token`
            // which compares against `N.to_string().as_bytes()`. Bases
            // other than {10,16} collapse to decimal (no exception channel).
            "itoa" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    // Decode the value as a full arbitrary-precision BigInteger,
                    // NOT via `stack_item_to_int` (which reads only the low 8
                    // bytes of a ByteArray-backed wide integer). int256/uint256
                    // values live on the stack as ByteArrays, so the i64 path
                    // silently formatted the wrong number for magnitudes above
                    // bit 63 — real Neo's StdLib.itoa formats the whole value.
                    let value = borrowed
                        .first()
                        .map(Self::stack_item_to_bigint)
                        .unwrap_or_else(|| num_bigint::BigInt::from(0));
                    let base = borrowed
                        .get(1)
                        .map(|it| Self::stack_item_to_int(it.clone()))
                        .unwrap_or(10);
                    // Hex: uppercase, no `0x`; negatives as `-ABS` (StdLib).
                    // num-bigint's `UpperHex` already prefixes a `-` and prints
                    // the magnitude, matching that convention.
                    let text = if base == 16 {
                        format!("{value:X}")
                    } else {
                        value.to_string()
                    };
                    StackItem::byte_array(text.into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // Task #51 — StdLib.atoi(string[, base]): parse decimal (or
            // hex) ASCII back into an integer. Returns `StackItem::Integer`
            // so `stack_item_to_bytes` LE-encodes it; `atoi(itoa(N)) == N`
            // round-trips via `decode_uint_le`. Malformed input yields 0.
            "atoi" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    let input_bytes = borrowed
                        .first()
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    let base = borrowed
                        .get(1)
                        .map(|it| Self::stack_item_to_int(it.clone()))
                        .unwrap_or(10);
                    let s = std::str::from_utf8(&input_bytes).unwrap_or("");
                    let trimmed = s.trim();
                    let radix: u32 = if base == 16 { 16 } else { 10 };
                    // Optional leading `-`; hex may also carry `0x`/`0X`.
                    let (neg, body) = match trimmed.strip_prefix('-') {
                        Some(rest) => (true, rest),
                        None => (false, trimmed),
                    };
                    let body = if radix == 16 {
                        body.strip_prefix("0x")
                            .or_else(|| body.strip_prefix("0X"))
                            .unwrap_or(body)
                    } else {
                        body
                    };
                    // Bug #21: `atoi("--42", 10)` previously returned 42 because
                    // `i128::from_str_radix` accepts a leading `-` in `body`,
                    // making the outer `neg` flag a second negation. Parse the
                    // magnitude as **unsigned** so any sign character in `body`
                    // (including a second `-`, a `+`, or anything else
                    // non-digit) yields 0.
                    let magnitude = u128::from_str_radix(body, radix).unwrap_or(0) as i128;
                    let signed = if neg { -magnitude } else { magnitude };
                    // Saturate into i64 (Integer slot width).
                    let clamped = signed.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
                    StackItem::Integer(clamped)
                } else {
                    StackItem::Integer(0)
                }
            }
            // Task #51 — StdLib.base64Encode(bytes): RFC 4648 base64 with
            // `=` padding. Hand-rolled (no new crate). UTF-8 `ByteArray`.
            "base64encode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    StackItem::byte_array(Self::base64_encode(&bytes).into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // Task #51 — StdLib.base64Decode(string): inverse encoder.
            // Invalid input yields an empty buffer (no exception channel).
            "base64decode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let s = std::str::from_utf8(&bytes).unwrap_or("");
                    StackItem::byte_array(Self::base64_decode(s).unwrap_or_default())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.hexEncode(bytes): UPPERCASE hex string (matches C#
            // `ToHexString`). Exposed on the real StdLib native contract.
            "hexencode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
                    StackItem::byte_array(hex.into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.hexDecode(string): inverse of hexEncode (accepts either
            // case, like C# `HexToBytes`). Invalid input (odd length or
            // non-hex characters) yields an empty buffer, mirroring the other
            // decoder arms' no-exception convention.
            "hexdecode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let s = std::str::from_utf8(&bytes).unwrap_or("");
                    let decoded = if s.len().is_multiple_of(2) {
                        (0..s.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
                            .collect::<Result<Vec<u8>, _>>()
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    StackItem::byte_array(decoded)
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.base64UrlEncode(bytes): RFC 4648 base64url variant
            // (URL-safe alphabet: `-` and `_` instead of `+` and `/`, no padding).
            "base64urlencode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let encoded = Self::base64_encode(&bytes);
                    let url_safe = encoded
                        .replace('+', "-")
                        .replace('/', "_")
                        .trim_end_matches('=')
                        .to_string();
                    StackItem::byte_array(url_safe.into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.base64UrlDecode(string): inverse of base64UrlEncode.
            "base64urldecode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let s = std::str::from_utf8(&bytes).unwrap_or("");
                    // Restore standard alphabet and padding for the decoder.
                    let standard = s.replace('-', "+").replace('_', "/");
                    let padded = match standard.len() % 4 {
                        2 => format!("{standard}=="),
                        3 => format!("{standard}="),
                        _ => standard,
                    };
                    StackItem::byte_array(Self::base64_decode(&padded).unwrap_or_default())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.base58Encode(bytes): Bitcoin base58 encoding.
            "base58encode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let encoded = bs58::encode(&bytes).into_string();
                    StackItem::byte_array(encoded.into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.base58Decode(string): Bitcoin base58 decoding.
            "base58decode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let s = std::str::from_utf8(&bytes).unwrap_or("");
                    match bs58::decode(s).into_vec() {
                        Ok(decoded) => StackItem::byte_array(decoded),
                        Err(_) => StackItem::byte_array(Vec::new()),
                    }
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.base58CheckEncode(bytes): base58 with 4-byte SHA256
            // double-hash checksum appended.
            "base58checkencode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let hash1 = Sha256::digest(&bytes);
                    let hash2 = Sha256::digest(hash1);
                    let mut payload = bytes;
                    payload.extend_from_slice(&hash2[..4]);
                    let encoded = bs58::encode(&payload).into_string();
                    StackItem::byte_array(encoded.into_bytes())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.base58CheckDecode(string): inverse of base58CheckEncode.
            // Verifies the 4-byte checksum; returns empty on mismatch.
            "base58checkdecode" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    let s = std::str::from_utf8(&bytes).unwrap_or("");
                    match bs58::decode(s).into_vec() {
                        Ok(decoded) if decoded.len() >= 4 => {
                            let data_len = decoded.len() - 4;
                            let hash1 = Sha256::digest(&decoded[..data_len]);
                            let hash2 = Sha256::digest(hash1);
                            if decoded[data_len..] == hash2[..4] {
                                StackItem::byte_array(decoded[..data_len].to_vec())
                            } else {
                                StackItem::byte_array(Vec::new())
                            }
                        }
                        _ => StackItem::byte_array(Vec::new()),
                    }
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            // StdLib.memoryCompare(str1, str2): lexicographic byte comparison.
            // Returns -1 if str1 < str2, 0 if equal, 1 if str1 > str2.
            "memorycompare" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    let a = borrowed
                        .first()
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    let b = borrowed
                        .get(1)
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    let result = match a.cmp(&b) {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    };
                    StackItem::Integer(result)
                } else {
                    StackItem::Integer(0)
                }
            }
            // StdLib.memorySearch(haystack, needle[, start]): finds the first
            // occurrence of needle in haystack starting from `start`.
            // Returns the index or -1 if not found.
            "memorysearch" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    let haystack = borrowed
                        .first()
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    let needle = borrowed
                        .get(1)
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    let start = borrowed.get(2).map(Self::extract_first_int).unwrap_or(0) as usize;
                    if needle.is_empty() {
                        return StackItem::Integer(start as i64);
                    }
                    if start >= haystack.len() {
                        return StackItem::Integer(-1);
                    }
                    match haystack[start..]
                        .windows(needle.len())
                        .position(|w| w == needle.as_slice())
                    {
                        Some(idx) => StackItem::Integer((start + idx) as i64),
                        None => StackItem::Integer(-1),
                    }
                } else {
                    StackItem::Integer(-1)
                }
            }
            // StdLib.stringSplit(str, separator): splits the string by the
            // separator and returns an Array of parts. An empty separator
            // returns the original string as a single-element Array.
            "stringsplit" => {
                if let StackItem::Array(args) = params {
                    let borrowed = args.borrow();
                    let input = borrowed
                        .first()
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    let sep = borrowed
                        .get(1)
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .unwrap_or_default();
                    if sep.is_empty() {
                        return StackItem::array(vec![StackItem::byte_array(input)]);
                    }
                    let s = String::from_utf8_lossy(&input);
                    let sep_str = String::from_utf8_lossy(&sep);
                    let parts: Vec<StackItem> = s
                        .split(sep_str.as_ref())
                        .map(|part| StackItem::byte_array(part.as_bytes().to_vec()))
                        .collect();
                    StackItem::array(parts)
                } else {
                    StackItem::array(Vec::new())
                }
            }
            // StdLib.strLen(str): returns the byte length of the input string.
            "strlen" => {
                if let StackItem::Array(args) = params {
                    let input = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(input);
                    StackItem::Integer(bytes.len() as i64)
                } else {
                    StackItem::Integer(0)
                }
            }
            _ => StackItem::Null,
        }
    }

    /// RFC 4648 base64 encode — standard alphabet with `=` padding.
    fn base64_encode(input: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
        let mut chunks = input.chunks_exact(3);
        for chunk in &mut chunks {
            let b = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
            out.push(ALPHABET[((b >> 18) & 0x3f) as usize] as char);
            out.push(ALPHABET[((b >> 12) & 0x3f) as usize] as char);
            out.push(ALPHABET[((b >> 6) & 0x3f) as usize] as char);
            out.push(ALPHABET[(b & 0x3f) as usize] as char);
        }
        let rem = chunks.remainder();
        match rem.len() {
            1 => {
                let b = (rem[0] as u32) << 16;
                out.push(ALPHABET[((b >> 18) & 0x3f) as usize] as char);
                out.push(ALPHABET[((b >> 12) & 0x3f) as usize] as char);
                out.push('=');
                out.push('=');
            }
            2 => {
                let b = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
                out.push(ALPHABET[((b >> 18) & 0x3f) as usize] as char);
                out.push(ALPHABET[((b >> 12) & 0x3f) as usize] as char);
                out.push(ALPHABET[((b >> 6) & 0x3f) as usize] as char);
                out.push('=');
            }
            _ => {}
        }
        out
    }

    /// RFC 4648 base64 decode — standard alphabet only. Returns `None` on
    /// malformed input (bad character or bad padding) rather than partial
    /// output; callers in the native-dispatch path substitute an empty
    /// buffer in that case.
    fn base64_decode(input: &str) -> Option<Vec<u8>> {
        fn decode_char(c: u8) -> Option<u8> {
            match c {
                b'A'..=b'Z' => Some(c - b'A'),
                b'a'..=b'z' => Some(c - b'a' + 26),
                b'0'..=b'9' => Some(c - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                _ => None,
            }
        }
        // Strip whitespace (spaces, tabs, newlines) — PEM/MIME interop.
        let cleaned: Vec<u8> = input
            .bytes()
            .filter(|b| !matches!(*b, b' ' | b'\t' | b'\r' | b'\n'))
            .collect();
        if !cleaned.len().is_multiple_of(4) {
            return None;
        }
        let mut out = Vec::with_capacity(cleaned.len() / 4 * 3);
        for (chunk_idx, chunk) in cleaned.chunks_exact(4).enumerate() {
            let pad = chunk.iter().rev().take_while(|b| **b == b'=').count();
            if pad > 2 {
                return None;
            }
            // Bug #22: previously this loop accepted `=` at any position in
            // the chunk and treated it as a literal zero sextet, so inputs
            // like "AB=C" decoded to non-empty bytes instead of erroring.
            // Reject `=` anywhere except the trailing-padding region of the
            // FINAL chunk (mid-chunk and non-final-chunk pad bytes are
            // illegal per RFC 4648).
            let total_eq = chunk.iter().filter(|b| **b == b'=').count();
            if total_eq != pad {
                return None; // `=` outside trailing-pad position
            }
            let is_final_chunk = chunk_idx + 1 == cleaned.len() / 4;
            if pad > 0 && !is_final_chunk {
                return None; // pad only allowed in final chunk
            }
            let mut vals = [0u8; 4];
            for (i, &c) in chunk.iter().enumerate() {
                vals[i] = if c == b'=' { 0 } else { decode_char(c)? };
            }
            let b = ((vals[0] as u32) << 18)
                | ((vals[1] as u32) << 12)
                | ((vals[2] as u32) << 6)
                | (vals[3] as u32);
            out.push(((b >> 16) & 0xff) as u8);
            if pad < 2 {
                out.push(((b >> 8) & 0xff) as u8);
            }
            if pad < 1 {
                out.push((b & 0xff) as u8);
            }
        }
        Some(out)
    }
}
