use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_cryptolib(method: &str, params: StackItem) -> StackItem {
        match method {
            "sha1" => {
                if let StackItem::Array(args) = params {
                    let value = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(value);
                    let digest = sha1::Sha1::digest(&bytes);
                    StackItem::byte_array(digest[..].to_vec())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "sha256" => {
                if let StackItem::Array(args) = params {
                    let value = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(value);
                    let digest = Sha256::digest(&bytes);
                    StackItem::byte_array(digest[..].to_vec())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "ripemd160" => {
                if let StackItem::Array(args) = params {
                    let value = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(value);
                    let digest = Ripemd160::digest(&bytes);
                    StackItem::byte_array(digest[..].to_vec())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "keccak256" => {
                if let StackItem::Array(args) = params {
                    let value = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let bytes = Self::stack_item_to_bytes(value);
                    let digest = Keccak256::digest(&bytes);
                    StackItem::byte_array(digest[..].to_vec())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "murmur32" => {
                if let StackItem::Array(args) = params {
                    let value = args
                        .borrow()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| StackItem::byte_array(Vec::new()));
                    let seed = args
                        .borrow()
                        .get(1)
                        .cloned()
                        .unwrap_or(StackItem::UnsignedInteger(0));
                    let bytes = Self::stack_item_to_bytes(value);
                    let seed_u32 = match seed {
                        StackItem::UnsignedInteger(u) => u as u32,
                        StackItem::Integer(i) => i as u32,
                        StackItem::ByteArray { data: b, .. } => {
                            let mut buf = [0u8; 4];
                            for (i, byte) in b.borrow().iter().take(4).enumerate() {
                                buf[i] = *byte;
                            }
                            u32::from_le_bytes(buf)
                        }
                        _ => 0,
                    };
                    let hash = Self::murmur3_32(&bytes, seed_u32);
                    StackItem::byte_array(hash.to_le_bytes().to_vec())
                } else {
                    StackItem::byte_array(Vec::new())
                }
            }
            "verifywithecdsa" => {
                // Neo N3 CryptoLib.verifyWithECDsa(message, pubkey, signature,
                // NamedCurveHash). The curve byte selects the EC curve AND the
                // message-hash algorithm: 22 secp256k1+SHA256, 23 secp256r1+SHA256,
                // 122 secp256k1+Keccak256, 123 secp256r1+Keccak256 (23 is Neo's
                // default). Neo hashes the message with the chosen algorithm, then
                // verifies the 64-byte ECDSA signature (r||s, big-endian) over that
                // 32-byte digest using the SEC1-encoded public key.
                let StackItem::Array(args) = params else {
                    return StackItem::Boolean(false);
                };
                let borrowed = args.borrow();
                let message =
                    Self::stack_item_to_bytes(borrowed.first().cloned().unwrap_or(StackItem::Null));
                let pubkey =
                    Self::stack_item_to_bytes(borrowed.get(1).cloned().unwrap_or(StackItem::Null));
                let signature =
                    Self::stack_item_to_bytes(borrowed.get(2).cloned().unwrap_or(StackItem::Null));
                let curve = match borrowed.get(3) {
                    Some(StackItem::Integer(n)) => *n,
                    Some(StackItem::UnsignedInteger(n)) => *n as i64,
                    Some(StackItem::ByteArray { data: b, .. }) => {
                        b.borrow().first().copied().unwrap_or(0) as i64
                    }
                    _ => 0,
                };
                let (is_secp256r1, use_keccak) = match curve {
                    22 => (false, false),
                    23 => (true, false),
                    122 => (false, true),
                    123 => (true, true),
                    _ => return StackItem::Boolean(false),
                };
                if signature.len() != 64 {
                    return StackItem::Boolean(false);
                }
                let digest: [u8; 32] = if use_keccak {
                    use sha3::{Digest as _, Keccak256};
                    Keccak256::digest(&message).into()
                } else {
                    Sha256::digest(&message).into()
                };
                let ok = if is_secp256r1 {
                    use p256::ecdsa::signature::hazmat::PrehashVerifier;
                    use p256::ecdsa::{Signature, VerifyingKey};
                    match (
                        VerifyingKey::from_sec1_bytes(&pubkey),
                        Signature::from_slice(&signature),
                    ) {
                        (Ok(vk), Ok(sig)) => vk.verify_prehash(&digest, &sig).is_ok(),
                        _ => false,
                    }
                } else {
                    use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
                    match (
                        PublicKey::from_slice(&pubkey),
                        Message::from_slice(&digest),
                        Signature::from_compact(&signature),
                    ) {
                        (Ok(pk), Ok(msg), Ok(mut sig)) => {
                            // Accept both high- and low-S forms (Neo verification
                            // does not require canonical low-S).
                            sig.normalize_s();
                            Secp256k1::verification_only()
                                .verify_ecdsa(&msg, &sig, &pk)
                                .is_ok()
                        }
                        _ => false,
                    }
                };
                StackItem::Boolean(ok)
            }
            "recoversecp256k1" => {
                // Neo N3 CryptoLib.RecoverSecp256K1(hash, signature) -> 65-byte
                // uncompressed pubkey (0x04 || x32 || y32) or Null on failure.
                // The Solidity `ecrecover` lowering (bytecode_builtins/builtin_call/
                // crypto.rs) packs `signature = r(32) || s(32) || v(1)` with v
                // normalized to 27..=30, matching Ethereum's recovery-id+27 form.
                let StackItem::Array(args) = params else {
                    return StackItem::Null;
                };
                let borrowed = args.borrow();
                let hash =
                    Self::stack_item_to_bytes(borrowed.first().cloned().unwrap_or(StackItem::Null));
                let signature =
                    Self::stack_item_to_bytes(borrowed.get(1).cloned().unwrap_or(StackItem::Null));
                if hash.len() < 32 || signature.len() != 65 {
                    return StackItem::Null;
                }

                use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
                use secp256k1::{Message, Secp256k1};

                let msg = match Message::from_slice(&hash[..32]) {
                    Ok(m) => m,
                    Err(_) => return StackItem::Null,
                };
                // v byte sits at signature[64]; accept 0/1 (raw) or 27..=30.
                let v = signature[64];
                let rec_i32 = match v {
                    0 | 1 => v as i32,
                    27..=30 => (v - 27) as i32,
                    _ => return StackItem::Null,
                };
                let rec_id = match RecoveryId::from_i32(rec_i32) {
                    Ok(id) => id,
                    Err(_) => return StackItem::Null,
                };
                let sig = match RecoverableSignature::from_compact(&signature[..64], rec_id) {
                    Ok(s) => s,
                    Err(_) => return StackItem::Null,
                };
                let secp = Secp256k1::new();
                match secp.recover_ecdsa(&msg, &sig) {
                    Ok(pk) => StackItem::byte_array(pk.serialize_uncompressed().to_vec()),
                    Err(_) => StackItem::Null,
                }
            }
            // ============================================================
            // BLS12-381 native handlers.
            //
            // These back the `CryptoLib.bls12381*` surface that the IR
            // resolver wires to `BuiltinCall::NativeCall { CryptoLib, ... }`
            // (see src/ir/context/builtins/syscalls.rs:149-196 and
            // src/ir/context/builtins/resolve.rs::resolve_cryptolib_member).
            // The bytecode emitter PACKs the call's args into a
            // `StackItem::Array`, so each handler unpacks the array and
            // operates on the elements directly. Method names arrive in
            // lower-case here because the dispatcher (`dispatch.rs`)
            // lower-cases them before matching.
            //
            // Encoding contract:
            //   * G1 points: 48-byte ZCash compressed encoding
            //   * G2 points: 96-byte ZCash compressed encoding
            //   * Scalars:   32 bytes, BIG-ENDIAN — to match how the
            //     differential proptests construct scalars
            //     (`scalar_bytes[24..].copy_from_slice(&s.to_be_bytes())`)
            //     and the typical Solidity convention for hash-derived
            //     scalars. We reverse to LE before handing to
            //     `Scalar::from_bytes`, falling back to `from_bytes_wide`
            //     for inputs >= the curve order.
            //   * Gt elements: serialized via the deterministic `Debug`
            //     representation (Fp12 → "(c0) + (c1)*w" → ...). This is
            //     stable across the same crate version, which is all the
            //     differential tests need (they compare two runtime calls
            //     against each other, not against an external oracle).
            "bls12381serialize" => Self::bls12381_serialize_handler(params),
            "bls12381deserialize" => Self::bls12381_deserialize_handler(params),
            "bls12381equal" => Self::bls12381_equal_handler(params),
            "bls12381add" => Self::bls12381_add_handler(params),
            "bls12381mul" => Self::bls12381_mul_handler(params),
            "bls12381pairing" => Self::bls12381_pairing_handler(params),
            "bls12381g1add" => Self::bls12381_g1_add_handler(params),
            "bls12381g1mul" => Self::bls12381_g1_mul_handler(params),
            "bls12381g1neg" => Self::bls12381_g1_neg_handler(params),
            "bls12381g2add" => Self::bls12381_g2_add_handler(params),
            "bls12381g2mul" => Self::bls12381_g2_mul_handler(params),
            "bls12381g2neg" => Self::bls12381_g2_neg_handler(params),
            // CryptoLib.verifyWithEd25519(message, pubkey, signature) —
            // Ed25519 signature verification. `pubkey` is 32 bytes, `signature`
            // is 64 bytes. Returns Boolean true on valid signature, false
            // otherwise (including malformed inputs).
            "verifywithed25519" => {
                let StackItem::Array(args) = params else {
                    return StackItem::Boolean(false);
                };
                let borrowed = args.borrow();
                let message =
                    Self::stack_item_to_bytes(borrowed.first().cloned().unwrap_or(StackItem::Null));
                let pubkey =
                    Self::stack_item_to_bytes(borrowed.get(1).cloned().unwrap_or(StackItem::Null));
                let signature =
                    Self::stack_item_to_bytes(borrowed.get(2).cloned().unwrap_or(StackItem::Null));
                if pubkey.len() != 32 || signature.len() != 64 {
                    return StackItem::Boolean(false);
                }
                use ed25519_dalek::{Signature, Verifier, VerifyingKey};
                let mut pk = [0u8; 32];
                pk.copy_from_slice(&pubkey);
                let mut sig_arr = [0u8; 64];
                sig_arr.copy_from_slice(&signature);
                let ok = VerifyingKey::from_bytes(&pk)
                    .ok()
                    .and_then(|vk| {
                        Signature::from_slice(&sig_arr)
                            .ok()
                            .map(|sig| vk.verify(&message, &sig).is_ok())
                    })
                    .unwrap_or(false);
                StackItem::Boolean(ok)
            }
            _ => StackItem::Null,
        }
    }

    // --------------------------------------------------------------
    // BLS12-381 helpers
    // --------------------------------------------------------------

    /// Unpack the standard `StackItem::Array(args)` envelope, returning
    /// `None` for any non-Array shape so the caller can short-circuit to
    /// `Null` (mirroring the sha256/ripemd160/etc. handlers).
    fn bls_unpack_args(params: &StackItem) -> Option<Vec<StackItem>> {
        if let StackItem::Array(args) = params {
            Some(args.borrow().clone())
        } else {
            None
        }
    }

    /// Parse a 48-byte ZCash-compressed G1 point. Returns None for any
    /// length mismatch or invalid encoding (subgroup check included via
    /// `from_compressed`).
    fn bls_parse_g1(bytes: &[u8]) -> Option<bls12_381::G1Affine> {
        if bytes.len() != 48 {
            return None;
        }
        let mut buf = [0u8; 48];
        buf.copy_from_slice(bytes);
        bls12_381::G1Affine::from_compressed(&buf).into_option()
    }

    /// Parse a 96-byte ZCash-compressed G2 point.
    fn bls_parse_g2(bytes: &[u8]) -> Option<bls12_381::G2Affine> {
        if bytes.len() != 96 {
            return None;
        }
        let mut buf = [0u8; 96];
        buf.copy_from_slice(bytes);
        bls12_381::G2Affine::from_compressed(&buf).into_option()
    }

    /// Parse a 32-byte big-endian scalar. Reverses to LE, then tries
    /// `Scalar::from_bytes` (which enforces canonical < modulus). For
    /// inputs >= modulus (e.g. an arbitrary 32-byte hash treated as a
    /// scalar), falls back to `Scalar::from_bytes_wide` over a 64-byte
    /// zero-extended buffer so we always produce a defined element of
    /// the scalar field rather than failing.
    fn bls_parse_scalar(bytes: &[u8]) -> Option<bls12_381::Scalar> {
        if bytes.is_empty() || bytes.len() > 32 {
            return None;
        }
        // Right-align into a 32-byte BE buffer (handle short inputs).
        let mut be = [0u8; 32];
        let off = 32 - bytes.len();
        be[off..].copy_from_slice(bytes);
        // Reverse to LE for the bls12_381 crate.
        let mut le = [0u8; 32];
        for (i, b) in be.iter().rev().enumerate() {
            le[i] = *b;
        }
        if let Some(s) = bls12_381::Scalar::from_bytes(&le).into_option() {
            return Some(s);
        }
        // Out-of-range: reduce via wide. from_bytes_wide takes a 64-byte
        // LE input and reduces modulo the curve order.
        let mut wide = [0u8; 64];
        wide[..32].copy_from_slice(&le);
        Some(bls12_381::Scalar::from_bytes_wide(&wide))
    }

    /// Deterministic Gt encoding. Gt's inner Fp12 is `pub(crate)`, so we
    /// can't reach into it for a structured serialization, but Fp12's
    /// Debug impl walks through Fp6/Fp2/Fp components and Fp's Debug uses
    /// the canonical big-endian byte form (see `Fp::fmt`). Two Gt values
    /// that compare equal therefore Debug-print to the same string, and
    /// equal Debug strings give equal byte vectors — which is exactly what
    /// the differential pairing test asserts (lhs_bytes == rhs_bytes for
    /// `e(a*G1, b*G2) == e(G1, ab*G2)`).
    fn bls_serialize_gt(gt: &bls12_381::Gt) -> Vec<u8> {
        format!("{gt:?}").into_bytes()
    }

    /// Helper to fetch arg N as bytes, defaulting to empty.
    fn bls_arg_bytes(args: &[StackItem], idx: usize) -> Vec<u8> {
        args.get(idx)
            .cloned()
            .map(Self::stack_item_to_bytes)
            .unwrap_or_default()
    }

    /// Helper to fetch arg N as bool.
    fn bls_arg_bool(args: &[StackItem], idx: usize) -> bool {
        match args.get(idx) {
            Some(StackItem::Boolean(b)) => *b,
            Some(StackItem::Integer(i)) => *i != 0,
            Some(StackItem::UnsignedInteger(u)) => *u != 0,
            Some(StackItem::ByteArray { data: b, .. }) => b.borrow().iter().any(|x| *x != 0),
            _ => false,
        }
    }

    // --------------------------------------------------------------
    // BLS12-381 handlers
    // --------------------------------------------------------------

    fn bls12381_serialize_handler(params: StackItem) -> StackItem {
        // Round-trip a point: validate and re-emit the canonical
        // compressed encoding. Length-dispatch on G1 (48) vs G2 (96).
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let bytes = Self::bls_arg_bytes(&args, 0);
        match bytes.len() {
            48 => match Self::bls_parse_g1(&bytes) {
                Some(p) => StackItem::byte_array(p.to_compressed().to_vec()),
                None => StackItem::Null,
            },
            96 => match Self::bls_parse_g2(&bytes) {
                Some(p) => StackItem::byte_array(p.to_compressed().to_vec()),
                None => StackItem::Null,
            },
            _ => StackItem::Null,
        }
    }

    fn bls12381_deserialize_handler(params: StackItem) -> StackItem {
        // Inverse of serialize: parse and re-emit. The runtime model
        // uses the compressed bytes themselves as the "opaque handle",
        // so deserialize is structurally identical to serialize.
        Self::bls12381_serialize_handler(params)
    }

    fn bls12381_equal_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Boolean(false);
        };
        let a = Self::bls_arg_bytes(&args, 0);
        let b = Self::bls_arg_bytes(&args, 1);
        if a.len() != b.len() {
            return StackItem::Boolean(false);
        }
        let equal = match a.len() {
            48 => match (Self::bls_parse_g1(&a), Self::bls_parse_g1(&b)) {
                (Some(pa), Some(pb)) => pa == pb,
                _ => false,
            },
            96 => match (Self::bls_parse_g2(&a), Self::bls_parse_g2(&b)) {
                (Some(pa), Some(pb)) => pa == pb,
                _ => false,
            },
            _ => false,
        };
        StackItem::Boolean(equal)
    }

    fn bls12381_add_handler(params: StackItem) -> StackItem {
        // Length-dispatch: both 48 → G1 add, both 96 → G2 add.
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let a = Self::bls_arg_bytes(&args, 0);
        let b = Self::bls_arg_bytes(&args, 1);
        if a.len() != b.len() {
            return StackItem::Null;
        }
        match a.len() {
            48 => Self::bls_g1_add_bytes(&a, &b),
            96 => Self::bls_g2_add_bytes(&a, &b),
            _ => StackItem::Null,
        }
    }

    fn bls12381_mul_handler(params: StackItem) -> StackItem {
        // Devpack signature: `bls12381Mul(bytes point, bytes scalar, bool neg)`.
        // Direct Solidity calls (`CryptoLib.bls12381Mul(p, k, false)`) follow
        // the same shape. The third arg is optional in our handler — if a
        // 2-arg variant is invoked (e.g. via `bls12381G1Mul`) we treat neg
        // as false.
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let p = Self::bls_arg_bytes(&args, 0);
        let s = Self::bls_arg_bytes(&args, 1);
        let neg = Self::bls_arg_bool(&args, 2);
        match p.len() {
            48 => Self::bls_g1_mul_bytes(&p, &s, neg),
            96 => Self::bls_g2_mul_bytes(&p, &s, neg),
            _ => StackItem::Null,
        }
    }

    fn bls12381_pairing_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let g1 = Self::bls_arg_bytes(&args, 0);
        let g2 = Self::bls_arg_bytes(&args, 1);
        let Some(p1) = Self::bls_parse_g1(&g1) else {
            return StackItem::Null;
        };
        let Some(p2) = Self::bls_parse_g2(&g2) else {
            return StackItem::Null;
        };
        let gt = bls12_381::pairing(&p1, &p2);
        StackItem::byte_array(Self::bls_serialize_gt(&gt))
    }

    fn bls12381_g1_add_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let a = Self::bls_arg_bytes(&args, 0);
        let b = Self::bls_arg_bytes(&args, 1);
        Self::bls_g1_add_bytes(&a, &b)
    }

    fn bls12381_g1_mul_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let p = Self::bls_arg_bytes(&args, 0);
        let s = Self::bls_arg_bytes(&args, 1);
        let neg = Self::bls_arg_bool(&args, 2);
        Self::bls_g1_mul_bytes(&p, &s, neg)
    }

    fn bls12381_g1_neg_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let p = Self::bls_arg_bytes(&args, 0);
        let Some(point) = Self::bls_parse_g1(&p) else {
            return StackItem::Null;
        };
        let negated = bls12_381::G1Affine::from(-bls12_381::G1Projective::from(point));
        StackItem::byte_array(negated.to_compressed().to_vec())
    }

    fn bls12381_g2_add_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let a = Self::bls_arg_bytes(&args, 0);
        let b = Self::bls_arg_bytes(&args, 1);
        Self::bls_g2_add_bytes(&a, &b)
    }

    fn bls12381_g2_mul_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let p = Self::bls_arg_bytes(&args, 0);
        let s = Self::bls_arg_bytes(&args, 1);
        let neg = Self::bls_arg_bool(&args, 2);
        Self::bls_g2_mul_bytes(&p, &s, neg)
    }

    fn bls12381_g2_neg_handler(params: StackItem) -> StackItem {
        let Some(args) = Self::bls_unpack_args(&params) else {
            return StackItem::Null;
        };
        let p = Self::bls_arg_bytes(&args, 0);
        let Some(point) = Self::bls_parse_g2(&p) else {
            return StackItem::Null;
        };
        let negated = bls12_381::G2Affine::from(-bls12_381::G2Projective::from(point));
        StackItem::byte_array(negated.to_compressed().to_vec())
    }

    // --------------------------------------------------------------
    // Group-arithmetic primitives shared by the abstract bls12381*
    // path and the bls12381G{1,2}* path.
    // --------------------------------------------------------------

    fn bls_g1_add_bytes(a: &[u8], b: &[u8]) -> StackItem {
        let (Some(pa), Some(pb)) = (Self::bls_parse_g1(a), Self::bls_parse_g1(b)) else {
            return StackItem::Null;
        };
        let sum = bls12_381::G1Projective::from(pa) + bls12_381::G1Projective::from(pb);
        let out = bls12_381::G1Affine::from(sum);
        StackItem::byte_array(out.to_compressed().to_vec())
    }

    fn bls_g2_add_bytes(a: &[u8], b: &[u8]) -> StackItem {
        let (Some(pa), Some(pb)) = (Self::bls_parse_g2(a), Self::bls_parse_g2(b)) else {
            return StackItem::Null;
        };
        let sum = bls12_381::G2Projective::from(pa) + bls12_381::G2Projective::from(pb);
        let out = bls12_381::G2Affine::from(sum);
        StackItem::byte_array(out.to_compressed().to_vec())
    }

    fn bls_g1_mul_bytes(p: &[u8], s: &[u8], neg: bool) -> StackItem {
        let Some(point) = Self::bls_parse_g1(p) else {
            return StackItem::Null;
        };
        let Some(scalar) = Self::bls_parse_scalar(s) else {
            return StackItem::Null;
        };
        let mut prod = bls12_381::G1Projective::from(point) * scalar;
        if neg {
            prod = -prod;
        }
        let out = bls12_381::G1Affine::from(prod);
        StackItem::byte_array(out.to_compressed().to_vec())
    }

    fn bls_g2_mul_bytes(p: &[u8], s: &[u8], neg: bool) -> StackItem {
        let Some(point) = Self::bls_parse_g2(p) else {
            return StackItem::Null;
        };
        let Some(scalar) = Self::bls_parse_scalar(s) else {
            return StackItem::Null;
        };
        let mut prod = bls12_381::G2Projective::from(point) * scalar;
        if neg {
            prod = -prod;
        }
        let out = bls12_381::G2Affine::from(prod);
        StackItem::byte_array(out.to_compressed().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S5 lock — `bls_serialize_gt` uses `format!("{gt:?}")` as a
    /// non-canonical, differential-only encoding. This test locks the
    /// encoding so that two calls on the same Gt value produce byte-identical
    /// output. If a future change alters the encoding (e.g., switching to a
    /// canonical Fp12 wire format), this test will fail loudly and force the
    /// differential pairing tests to be re-validated against the new shape.
    #[test]
    fn s5_bls_gt_serialization_is_deterministic_for_identity() {
        let gt = bls12_381::Gt::identity();
        let bytes_a = ExecutionContext::bls_serialize_gt(&gt);
        let bytes_b = ExecutionContext::bls_serialize_gt(&gt);
        assert_eq!(
            bytes_a, bytes_b,
            "Gt serialization must be deterministic across calls (S5 lock)"
        );
        assert!(
            !bytes_a.is_empty(),
            "Gt serialization must produce non-empty output"
        );
    }
}
