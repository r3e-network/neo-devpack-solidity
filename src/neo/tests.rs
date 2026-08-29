use super::constants::MAX_SOURCE_LENGTH;
use super::encoding::calculate_checksum;
use super::*;
use crate::opcode::OpCode;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

fn read_varint(data: &[u8], offset: &mut usize) -> u64 {
    match data[*offset] {
        0xFD => {
            let start = *offset + 1;
            *offset += 3;
            u16::from_le_bytes(data[start..start + 2].try_into().unwrap()) as u64
        }
        0xFE => {
            let start = *offset + 1;
            *offset += 5;
            u32::from_le_bytes(data[start..start + 4].try_into().unwrap()) as u64
        }
        0xFF => {
            let start = *offset + 1;
            *offset += 9;
            u64::from_le_bytes(data[start..start + 8].try_into().unwrap())
        }
        value => {
            *offset += 1;
            value as u64
        }
    }
}

#[test]
fn builds_nef_payload_matching_spec() {
    let script = vec![OpCode::PUSHDATA1.byte(), 0x01, 0x2A, OpCode::RET.byte()];
    let token = MethodToken::new([0x11; 20], "transfer", 2, true, 0x07);
    let source = "https://example.com/src.sol";
    let compiler = "neo-devpack-solidity-test";

    let nef = build_nef_with_tokens(&script, compiler, source, &[token]).unwrap();
    let mut offset = 0usize;

    // Magic
    assert_eq!(&nef[offset..offset + 4], b"NEF3");
    offset += 4;

    // Compiler (padded to 64 bytes)
    let compiler_bytes = &nef[offset..offset + 64];
    assert_eq!(
        &compiler_bytes[..compiler.len()],
        compiler.as_bytes(),
        "compiler prefix should match"
    );
    offset += 64;

    // Source
    let source_len = read_varint(&nef, &mut offset) as usize;
    let source_bytes = &nef[offset..offset + source_len];
    assert_eq!(std::str::from_utf8(source_bytes).unwrap(), source);
    offset += source_len;

    // Reserved byte
    assert_eq!(nef[offset], 0);
    offset += 1;

    // Tokens
    let token_count = read_varint(&nef, &mut offset);
    assert_eq!(token_count, 1);
    assert_eq!(&nef[offset..offset + 20], &[0x11u8; 20]);
    offset += 20;

    let method_len = read_varint(&nef, &mut offset) as usize;
    let method = &nef[offset..offset + method_len];
    assert_eq!(std::str::from_utf8(method).unwrap(), "transfer");
    offset += method_len;

    let params = u16::from_le_bytes(nef[offset..offset + 2].try_into().expect("param bytes"));
    offset += 2;
    let has_return = nef[offset] != 0;
    offset += 1;
    let call_flags = nef[offset];
    offset += 1;

    assert_eq!(params, 2);
    assert!(has_return);
    assert_eq!(call_flags, 0x07);

    // Reserved 2 bytes
    assert_eq!(&nef[offset..offset + 2], &[0u8; 2]);
    offset += 2;

    // Script payload
    let script_len = read_varint(&nef, &mut offset) as usize;
    assert_eq!(script_len, script.len());
    assert_eq!(&nef[offset..offset + script_len], script.as_slice());
    offset += script_len;

    // Checksum
    let checksum = u32::from_le_bytes(nef[offset..offset + 4].try_into().unwrap());
    let expected_checksum = calculate_checksum(&nef[..offset]);
    assert_eq!(checksum, expected_checksum);
    offset += 4;

    assert_eq!(offset, nef.len(), "parsed all bytes");
}

#[test]
fn rejects_invalid_call_flags() {
    let script = vec![OpCode::RET.byte()];
    let bad_token = MethodToken::new([0x22; 20], "x", 0, false, 0x80);
    let result = build_nef_with_tokens(&script, "compiler", "", &[bad_token]);
    assert!(result.is_err(), "expected Err for invalid call flags");
    assert!(
        result.unwrap_err().contains("call flags"),
        "error should mention call flags"
    );
}

#[test]
fn clamps_long_source_field() {
    let script = vec![OpCode::RET.byte()];
    let long_source = "a".repeat(MAX_SOURCE_LENGTH + 20);
    let nef = build_nef_with_tokens(&script, "compiler", &long_source, &[]).unwrap();
    let mut offset = 4 + 64; // magic + compiler
    let len = read_varint(&nef, &mut offset) as usize;
    assert_eq!(len, MAX_SOURCE_LENGTH);
    let source = std::str::from_utf8(&nef[offset..offset + len]).unwrap();
    assert!(source.len() <= MAX_SOURCE_LENGTH);
}

#[test]
fn computes_contract_hash_for_small_checksum_matches_hash160_of_scriptbuilder_payload() {
    let sender = [0x11u8; 20];
    let checksum = 1u32;
    let name = "TestContract";

    let computed = compute_contract_hash(sender, checksum, name);

    // ScriptBuilder emits:
    //   ABORT
    //   PUSH sender (ByteString)
    //   PUSH checksum (Integer)
    //   PUSH name (String)
    let mut script = Vec::new();
    script.push(OpCode::ABORT.byte());
    script.push(OpCode::PUSHDATA1.byte());
    script.push(20);
    script.extend_from_slice(&sender);
    script.push(OpCode::PUSH1.byte());
    script.push(OpCode::PUSHDATA1.byte());
    script.push(name.len() as u8);
    script.extend_from_slice(name.as_bytes());

    let sha = Sha256::digest(&script);
    let expected = Ripemd160::digest(sha);
    assert_eq!(computed.as_slice(), expected.as_slice());
}

#[test]
fn computes_contract_hash_for_large_checksum_matches_hash160_of_scriptbuilder_payload() {
    let sender = [0xABu8; 20];
    let checksum = u32::MAX;
    let name = "C";

    let computed = compute_contract_hash(sender, checksum, name);

    let mut script = Vec::new();
    script.push(OpCode::ABORT.byte());
    script.push(OpCode::PUSHDATA1.byte());
    script.push(20);
    script.extend_from_slice(&sender);

    // u32::MAX does not fit in signed i32, so ScriptBuilder uses a wider PUSHINT.
    script.push(OpCode::PUSHINT64.byte());
    script.extend_from_slice(&(checksum as i64).to_le_bytes());

    script.push(OpCode::PUSHDATA1.byte());
    script.push(1);
    script.extend_from_slice(b"C");

    let sha = Sha256::digest(&script);
    let expected = Ripemd160::digest(sha);
    assert_eq!(computed.as_slice(), expected.as_slice());
}

#[test]
fn uint160_hex_parsing_roundtrips_big_endian_string() {
    let input = "0x0102030405060708090a0b0c0d0e0f1011121314";
    let parsed = parse_uint160_hex_be(input).expect("parse uint160");
    assert_eq!(format_uint160_hex_be(&parsed), input);
}

#[test]
fn uint160_hex_parsing_accepts_uppercase_prefix() {
    let input = "0X0102030405060708090a0b0c0d0e0f1011121314";
    let parsed = parse_uint160_hex_be(input).expect("parse uint160");
    assert_eq!(
        format_uint160_hex_be(&parsed),
        "0x0102030405060708090a0b0c0d0e0f1011121314"
    );
}
