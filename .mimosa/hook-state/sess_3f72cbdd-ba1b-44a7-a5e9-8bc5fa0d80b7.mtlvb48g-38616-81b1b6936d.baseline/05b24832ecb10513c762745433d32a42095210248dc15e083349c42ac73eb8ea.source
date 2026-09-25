//! ## StdLib ABI Encoding Helpers (Runtime)
//!
//! ABI encoding helpers used by the runtime StdLib handler. Extracted from
//! stdlib.rs to keep the dispatch module under the 800-line limit.
//!
//! Functions live in an `impl ExecutionContext` block but are `&self`-free;
//! they are called as associated functions from the main handler.

use super::*;

impl ExecutionContext {
    /// Convert a single ABI element into a 32-byte big-endian slot, matching
    /// the EVM ABI layout for `abi.encode` (non-packed).
    ///
    /// NeoVM stores integer-ish stack items (addresses, uint/int literals,
    /// bools, enums, small `bytesN`) as LITTLE-ENDIAN byte arrays — so to
    /// produce the EVM-canonical big-endian 32-byte slot we need to reverse
    /// the byte order. Cryptographic hashes (keccak256, sha256) are the
    /// exception: CryptoLib returns them as 32-byte BEs in natural order.
    ///
    /// Heuristic (content-based — the Solidity type is lost by the time the
    /// runtime handler runs):
    /// * `Integer` / `UnsignedInteger`: write via `to_be_bytes`, right-aligned.
    /// * `Boolean`: single byte 0/1 at `slot[31]`.
    /// * `ByteArray` of length >= 32 with trailing zeros (bytes 20..32 all
    ///   zero) — this is an `address` / small-int that was NewBuffer'd to
    ///   32 bytes. Reverse the leading 20 bytes (LE→BE) and right-align.
    /// * `ByteArray` of length >= 32 otherwise (keccak256 output, bytes32
    ///   literal): take the first 32 bytes verbatim (already BE).
    /// * `ByteArray` of length 20 (raw address from `LiteralValue::Address`):
    ///   reverse (LE→BE) and right-align into slot[12..32].
    /// * `ByteArray` of other lengths (< 32): treat as a LE-stored integer
    ///   (PUSHINT128, PUSHINT64-encoded-as-BA, etc.) — reverse and
    ///   right-align so the MSB lands at `slot[31]`.
    /// * Nested `Array` / `Map` / `Null`: zero slot (callers flatten).
    pub(crate) fn abi_pad32_be(item: &StackItem) -> [u8; 32] {
        let mut slot = [0u8; 32];
        match item {
            StackItem::Integer(v) => {
                if *v >= 0 {
                    slot[24..].copy_from_slice(&(*v as u64).to_be_bytes());
                } else {
                    // Two's-complement fill for negatives: sign-extend 0xff.
                    slot.fill(0xff);
                    slot[24..].copy_from_slice(&(*v as u64).to_be_bytes());
                }
            }
            StackItem::UnsignedInteger(v) => {
                slot[24..].copy_from_slice(&v.to_be_bytes());
            }
            StackItem::Boolean(b) => {
                slot[31] = if *b { 1 } else { 0 };
            }
            StackItem::ByteArray { data: bytes, .. } => {
                let b = bytes.borrow();
                if b.len() >= 32 {
                    // Address-slot normalisation: detect the 20-byte LE address
                    // zero-extended to 32 bytes (bytes 20..32 == 0) and emit it
                    // right-aligned big-endian at slot[12..32]. This is exactly
                    // the layout EVM expects for `address` in abi.encode. The
                    // same heuristic catches small uint256 literals that landed
                    // in a 32-byte NewBuffer — reversing the leading non-zero
                    // prefix recovers the BE integer value.
                    //
                    // Task #112 refinement — a left-aligned `bytesN` (N < 20)
                    // pre-padded to 32 bytes by the compiler's tuple-return
                    // path ALSO has `b[20..32] == 0`. It is distinguishable
                    // from an LE-address payload by a zero run *within*
                    // `b[12..20]`: LE-encoded addresses fill all 20 low bytes
                    // with "random" address bits, whereas `bytesN` with
                    // N ≤ 12 has zeros throughout `b[N..20]` (i.e. every byte
                    // of `b[12..20]` is zero). When that zero-mid pattern is
                    // present the buffer is copied verbatim — matching
                    // `abi.encode(bytesN)`'s left-aligned layout and
                    // unblocking `batch47_w4_msg_data_length_and_selector_via_call_method`.
                    let low_tail_zero = b[20..32].iter().all(|byte| *byte == 0);
                    let mid_zero = b[12..20].iter().all(|byte| *byte == 0);
                    if low_tail_zero && !mid_zero {
                        // LE-stored; reverse the first 20 bytes into slot[12..32].
                        let mut prefix: [u8; 20] = [0; 20];
                        prefix.copy_from_slice(&b[..20]);
                        prefix.reverse();
                        slot[12..32].copy_from_slice(&prefix);
                    } else {
                        // Natural BE payload (keccak256, bytes32 literal, or
                        // a compiler pre-padded `bytesN` with N ≤ 12).
                        slot.copy_from_slice(&b[..32]);
                    }
                } else if b.len() == 20 {
                    // Raw 20-byte address from `LiteralValue::Address` —
                    // stored LE, flip to BE and right-align into slot[12..32].
                    // This matches EVM `abi.encode(address)` which left-pads
                    // the 20-byte BE address to a 32-byte slot.
                    let mut rev = b.clone();
                    rev.reverse();
                    slot[12..32].copy_from_slice(&rev);
                } else if b.len() == 16 {
                    // 16 bytes — likely a LE-stored PUSHINT128 payload that
                    // NeoVM pushed as a ByteArray rather than an Integer. To
                    // recover the BE integer value, reverse and right-align
                    // so the MSB lands at slot[31]. String/bytes literals of
                    // exactly 16 bytes will render in reversed order under
                    // this heuristic — Solidity doesn't mint literals of
                    // this width by coincidence often, and the alternative
                    // (no-reverse) would break `abi.encode(uint128 x)` for
                    // values that don't fit in i64.
                    let mut rev = b.clone();
                    rev.reverse();
                    let start = 32 - rev.len();
                    slot[start..].copy_from_slice(&rev);
                } else {
                    // Short ByteArrays (< 16 bytes, and != 0 / != 20) are
                    // almost always string/bytes payloads from PUSHDATA1
                    // (e.g. `"fuzz"`, `hex"deadbeef"`). Left-pad to 32 bytes
                    // in NATURAL order so `abi.encode(string/bytes)` renders
                    // the content verbatim in the low bytes of the slot —
                    // matching both the Solidity spec intent and the fuzz
                    // harness expectations (`runtime_notify_emits_log_*`,
                    // `event_with_indexed_and_dynamic_args_lowers`).
                    //
                    // Integer literals that would naturally land here (e.g.
                    // raw `BigInt` fallback bytes) are uncommon enough that
                    // the simpler rule wins; the Integer stack-item path
                    // handles all PUSHINT8/16/32/64 values correctly above.
                    let start = 32 - b.len();
                    slot[start..].copy_from_slice(&b);
                }
            }
            StackItem::Null => {}
            StackItem::Array(_) | StackItem::Map(_) => {}
        }
        slot
    }

    /// Convert a single ABI element into its `encodePacked` byte-width.
    ///
    /// For `uint256`/`int256`/`address` (the widths we can't recover from the
    /// runtime type) we emit the full 32-byte BE slot so the fixed-width
    /// signature-based consumers (e.g. EIP-191 / `keccak256(...encodePacked)`)
    /// agree with the EVM reference. Dynamic bytes/strings pass through as-is
    /// without any length prefix / padding (Ethereum spec: `encodePacked` on
    /// dynamic operands is a raw concatenation of their contents).
    ///
    /// This mirrors `abi_pad32_be`'s LE→BE normalisation for integer-shaped
    /// ByteArrays so `keccak256(abi.encode(x, y)) == keccak256(abi.encodePacked(x, y))`
    /// holds for uint256 args (Solidity invariant — see fuzz harness
    /// `hash_consistency_keccak_of_encoded_matches_packed`).
    pub(crate) fn abi_packed_bytes(item: &StackItem) -> Vec<u8> {
        match item {
            StackItem::Integer(_) | StackItem::UnsignedInteger(_) | StackItem::Null => {
                Self::abi_pad32_be(item).to_vec()
            }
            StackItem::ByteArray { data: bytes, .. } => {
                // Dynamic bytes/strings (`bytes`, `string`): raw concat per the
                // EVM `abi.encodePacked` spec — no length prefix, no padding.
                // `abi_is_dynamic` classifies any ByteArray whose length is
                // NOT in `{16, 20, 32}` as dynamic; those three static widths
                // correspond to PUSHINT128 / raw `address` / bytes32-sized
                // integer slots which DO need LE→BE normalisation into a
                // 32-byte packed slot (batch-#30 H2; see
                // `batch30_h2_keccak_abi_encode_packed_dynamic`).
                if Self::abi_is_dynamic(item) {
                    bytes.borrow().clone()
                } else {
                    Self::abi_pad32_be(item).to_vec()
                }
            }
            StackItem::Boolean(b) => vec![if *b { 1 } else { 0 }],
            // Task #193 — `abi.encodePacked(T[])` on dynamic arrays: per the
            // Solidity spec, array elements are padded to 32 bytes each
            // (distinct from the direct-scalar packed widths), concatenated
            // with NO length prefix and NO offset. For a `uint256[] a = [1, 2,
            // 3]` input that yields 96 bytes = BE(1, 32) || BE(2, 32) ||
            // BE(3, 32). Iterate the inner StackItem list and emit each
            // element's 32-byte BE slot via `abi_pad32_be`; nested Array /
            // Map elements fall back to zero-slot (same convention as the
            // encode path — nested dynamic shapes are out of scope until a
            // harness exercises them).
            //
            // Distinct from the `abiencode` dispatch which wraps `T[]` in
            // offset+length+elements (via `abi_dynamic_tail_bytes`); the
            // packed variant suppresses both the offset and the length.
            StackItem::Array(arr) => {
                let elements = arr.borrow().clone();
                let mut out = Vec::with_capacity(elements.len() * 32);
                for el in elements.iter() {
                    out.extend_from_slice(&Self::abi_pad32_be(el));
                }
                out
            }
            StackItem::Map(_) => Vec::new(),
        }
    }

    /// Decode a stack item to a full `BigInt` (the value's true magnitude,
    /// signed-little-endian for ByteArray-backed wide integers). Static mirror
    /// of `coerce_item_to_bigint`'s arms for the `&self`-free native handlers.
    pub(crate) fn stack_item_to_bigint(item: &StackItem) -> num_bigint::BigInt {
        use num_bigint::BigInt;
        match item {
            StackItem::Integer(v) => BigInt::from(*v),
            StackItem::UnsignedInteger(v) => BigInt::from(*v),
            StackItem::Boolean(b) => BigInt::from(if *b { 1 } else { 0 }),
            StackItem::ByteArray { data: bytes, .. } => {
                let bytes = bytes.borrow();
                if bytes.is_empty() {
                    BigInt::from(0)
                } else {
                    BigInt::from_signed_bytes_le(&bytes)
                }
            }
            _ => BigInt::from(0),
        }
    }

    /// Classify whether a `StackItem` should encode as a DYNAMIC type under
    /// EVM ABI rules. Used by `abiencode` (Tasks #72/#73) to decide whether
    /// the head slot carries the value directly or an offset to a tail.
    ///
    /// Since the Solidity source type is lost by dispatch time, we rely on
    /// the same content-based heuristics `abi_pad32_be` already uses for
    /// byte-shape disambiguation, inverted to identify strings/bytes:
    ///
    /// * `Integer`, `UnsignedInteger`, `Boolean`, `Null` → STATIC always.
    /// * `ByteArray` of length 16 → STATIC (PUSHINT128 payload).
    /// * `ByteArray` of length 20 → STATIC (raw `address`).
    /// * `ByteArray` of length 32 → STATIC (bytes32 literal, keccak output,
    ///   or an address/small-int NewBuffer-padded to 32 bytes).
    /// * `ByteArray` of any OTHER length (0, 1..=15, 17..=19, 21..=31, >32)
    ///   → DYNAMIC string/bytes payload. Empty buffers encode as a
    ///   zero-length dynamic tail (spec: `len=0` + no data).
    /// * `Array` / `Map` → DYNAMIC (dynamic arrays/maps; no ABI slot shape).
    pub(crate) fn abi_is_dynamic(item: &StackItem) -> bool {
        match item {
            StackItem::Integer(_)
            | StackItem::UnsignedInteger(_)
            | StackItem::Boolean(_)
            | StackItem::Null => false,
            StackItem::ByteArray { data: bytes, .. } => {
                let len = bytes.borrow().len();
                !matches!(len, 16 | 20 | 32)
            }
            StackItem::Array(_) | StackItem::Map(_) => true,
        }
    }

    /// Produce the EVM-canonical tail-section bytes for a DYNAMIC ABI arg:
    /// a 32-byte BE length prefix followed by the raw content padded with
    /// trailing zeros to the next 32-byte boundary. Used by `abiencode`.
    ///
    /// * `StackItem::ByteArray`: length + raw content (string / bytes).
    /// * `StackItem::Array`: length + each element as a 32-byte BE slot
    ///   (Task #121 — dynamic-array variant, e.g. `uint[]`). Nested
    ///   dynamic arrays are NOT handled — each element is flattened via
    ///   `abi_pad32_be`, which zeros nested Array/Map elements rather
    ///   than recursing. That covers the common `uintN[]` / `bytesN[]` /
    ///   `address[]` / `bool[]` shapes; deeper nesting (`uint[][]`) is
    ///   out of scope until a harness exists for it.
    /// * Anything else: empty tail (keeps head + tail consistent with
    ///   the existing fallback semantics for Map / Null).
    pub(crate) fn abi_dynamic_tail_bytes(item: &StackItem) -> Vec<u8> {
        match item {
            StackItem::ByteArray { data: bytes, .. } => {
                let content = bytes.borrow().clone();
                let len = content.len();
                let padded_len = len.div_ceil(32) * 32;
                let mut out = Vec::with_capacity(32 + padded_len);
                let mut len_slot = [0u8; 32];
                len_slot[24..].copy_from_slice(&(len as u64).to_be_bytes());
                out.extend_from_slice(&len_slot);
                out.extend_from_slice(&content);
                out.resize(32 + padded_len, 0);
                out
            }
            StackItem::Array(arr) => {
                // Task #121 — EVM-canonical `T[]` encoding: 32-byte BE
                // length prefix followed by N × 32-byte BE-padded element
                // slots (no padding between slots; each element already
                // occupies a full 32-byte word).
                //
                // Task #192 — nested struct-array shape. When the element is
                // itself a `StackItem::Array` of SCALARS (i.e. a struct value
                // whose fields were flattened onto a NeoVM Array on the stack,
                // per Task #181's boundary convention), the EVM canonical
                // encoding inlines each struct's fields as consecutive 32-byte
                // slots: `length || (field0 || field1 || ... || fieldK-1)*N`.
                // This matches `abi.encode((uint256,bool)[])` for all-static
                // struct fields, where the struct contributes K head slots
                // and the OUTER array is just length + N*K flat slots (no
                // per-element offset indirection because each struct is
                // static — it occupies exactly K*32 bytes inline).
                //
                // Heuristic: an element is treated as a struct value when it
                // is `StackItem::Array` AND every inner field is itself a
                // scalar (Integer / UnsignedInteger / Boolean / Null, or a
                // ByteArray of address/bytesN/integer width). Nested dynamic
                // elements (strings, bytes, deeper arrays) fall outside this
                // static-struct fast path and keep the existing zero-slot
                // fallback (same as pre-#192 behaviour).
                let elements = arr.borrow().clone();
                let n = elements.len();
                let mut out = Vec::new();
                let mut len_slot = [0u8; 32];
                len_slot[24..].copy_from_slice(&(n as u64).to_be_bytes());
                out.extend_from_slice(&len_slot);
                for el in elements.iter() {
                    if let StackItem::Array(fields) = el {
                        let field_items = fields.borrow().clone();
                        if !field_items.is_empty()
                            && field_items.iter().all(|item| !Self::abi_is_dynamic(item))
                        {
                            for field in field_items.iter() {
                                out.extend_from_slice(&Self::abi_pad32_be(field));
                            }
                            continue;
                        }
                    }
                    out.extend_from_slice(&Self::abi_pad32_be(el));
                }
                out
            }
            _ => {
                // Map / Null: degrade to an empty zero-length tail to
                // match the existing head-side fallback (abi_pad32_be
                // returns zeros for these, and abi_is_dynamic classifies
                // Map as dynamic — so we emit a 32-byte length=0 slot).
                vec![0u8; 32]
            }
        }
    }
}
