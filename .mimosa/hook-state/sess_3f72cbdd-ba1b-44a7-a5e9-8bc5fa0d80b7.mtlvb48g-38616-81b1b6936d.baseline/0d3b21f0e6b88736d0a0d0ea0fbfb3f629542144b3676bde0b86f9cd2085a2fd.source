use crate::opcode::OpCode;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

/// Compute the Neo N3 contract hash for a deployment transaction.
///
/// Neo derives the deployed contract hash from:
/// - the deploy transaction sender (UInt160)
/// - the NEF checksum (u32)
/// - the manifest name (string)
///
/// This matches `Neo.SmartContract.Helper.GetContractHash(sender, nefChecksum, name)` in the
/// Neo node implementation:
/// - ScriptBuilder emits: `ABORT`, `PUSH sender`, `PUSH nefChecksum`, `PUSH name`
/// - Contract hash is `Hash160(script)` (RIPEMD160(SHA256(script))) interpreted as UInt160.
///
/// Inputs and outputs use the VM byte order for UInt160 (little-endian 20 bytes).
pub fn compute_contract_hash(sender_le: [u8; 20], nef_checksum: u32, name: &str) -> [u8; 20] {
    let mut script = Vec::new();
    script.push(OpCode::ABORT.byte());
    emit_push_bytes(&mut script, &sender_le);
    emit_push_u32(&mut script, nef_checksum);
    emit_push_bytes(&mut script, name.as_bytes());
    hash160(&script)
}

/// Parse a Neo UInt160 from a 0x-prefixed (or raw) big-endian hex string.
///
/// Returns the UInt160 in NeoVM byte order (little-endian 20 bytes).
pub fn parse_uint160_hex_be(value: &str) -> Result<[u8; 20], String> {
    let trimmed = value.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if without_prefix.len() != 40 {
        return Err(format!(
            "expected 40 hex characters for UInt160 (got {})",
            without_prefix.len()
        ));
    }
    let bytes_be = hex::decode(without_prefix)
        .map_err(|err| format!("invalid hex UInt160 '{value}': {err}"))?;
    if bytes_be.len() != 20 {
        return Err(format!(
            "expected 20 bytes for UInt160 (got {})",
            bytes_be.len()
        ));
    }
    let mut le = [0u8; 20];
    for (dst, src) in le.iter_mut().zip(bytes_be.into_iter().rev()) {
        *dst = src;
    }
    Ok(le)
}

/// Format a Neo UInt160 (little-endian bytes) as a 0x-prefixed big-endian hex string.
pub fn format_uint160_hex_be(value_le: &[u8; 20]) -> String {
    let be: Vec<u8> = value_le.iter().rev().copied().collect();
    format!("0x{}", hex::encode(be))
}

fn hash160(data: &[u8]) -> [u8; 20] {
    let sha = Sha256::digest(data);
    let digest = Ripemd160::digest(sha);
    let mut out = [0u8; 20];
    out.copy_from_slice(&digest[..20]);
    out
}

/// Emit a NeoVM PUSHDATA1 / PUSHDATA2 / PUSHDATA4 + payload. Bytecode-
/// identical to `cli::bytecode::bytecode_builtins::data::push_data` but
/// kept local: exposing it from the `cli` module would widen its
/// visibility (currently private) just to share 13 lines of stable,
/// well-tested opcode emission. The duplication risk is small because
/// the compiler pipeline drives both call sites through the same
/// deployment path; any divergence would surface immediately in the
/// `compute_contract_hash` unit tests.
fn emit_push_bytes(script: &mut Vec<u8>, data: &[u8]) {
    script.push(OpCode::push_data(data.len()).byte());
    match data.len() {
        len if len <= u8::MAX as usize => {
            script.push(data.len() as u8);
        }
        len if len <= u16::MAX as usize => {
            script.extend_from_slice(&(data.len() as u16).to_le_bytes());
        }
        _ => {
            script.extend_from_slice(&(data.len() as u32).to_le_bytes());
        }
    }
    script.extend_from_slice(data);
}

fn emit_push_u32(script: &mut Vec<u8>, value: u32) {
    if value == 0 {
        script.push(OpCode::PUSH0.byte());
        return;
    }
    if value <= 16 {
        // # Invariant: `value` is 1..=16 (0 is handled above), which is
        // exactly the range `push_small` accepts. This cannot be None.
        script.push(
            OpCode::push_small(value as u8)
                .expect("value in 1..=16")
                .byte(),
        );
        return;
    }
    if value <= i8::MAX as u32 {
        script.push(OpCode::PUSHINT8.byte());
        script.push(value as u8);
        return;
    }
    if value <= i16::MAX as u32 {
        script.push(OpCode::PUSHINT16.byte());
        script.extend_from_slice(&(value as i16).to_le_bytes());
        return;
    }
    if value <= i32::MAX as u32 {
        script.push(OpCode::PUSHINT32.byte());
        script.extend_from_slice(&(value as i32).to_le_bytes());
        return;
    }
    script.push(OpCode::PUSHINT64.byte());
    script.extend_from_slice(&(value as i64).to_le_bytes());
}
