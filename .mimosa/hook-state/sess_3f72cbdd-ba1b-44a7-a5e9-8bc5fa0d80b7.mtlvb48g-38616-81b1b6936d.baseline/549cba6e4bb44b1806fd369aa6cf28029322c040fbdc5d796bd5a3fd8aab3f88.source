use sha2::{Digest, Sha256};

pub(super) fn write_fixed_string(buffer: &mut Vec<u8>, value: &str, width: usize) {
    let mut padded = vec![0u8; width];
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(width);
    padded[..copy_len].copy_from_slice(&bytes[..copy_len]);
    buffer.extend_from_slice(&padded);
}

pub(super) fn write_varstring(buffer: &mut Vec<u8>, value: &str) {
    write_varbytes(buffer, value.as_bytes());
}

pub(super) fn write_varbytes(buffer: &mut Vec<u8>, data: &[u8]) {
    write_varint(buffer, data.len() as u64);
    buffer.extend_from_slice(data);
}

pub(super) fn write_varint(buffer: &mut Vec<u8>, value: u64) {
    match value {
        0x0000_0000..=0x0000_00FC => buffer.push(value as u8),
        0x0000_00FD..=0x0000_FFFF => {
            buffer.push(0xFD);
            buffer.extend_from_slice(&(value as u16).to_le_bytes());
        }
        0x0001_0000..=0xFFFF_FFFF => {
            buffer.push(0xFE);
            buffer.extend_from_slice(&(value as u32).to_le_bytes());
        }
        _ => {
            buffer.push(0xFF);
            buffer.extend_from_slice(&value.to_le_bytes());
        }
    }
}

pub(super) fn calculate_checksum(payload: &[u8]) -> u32 {
    let first = Sha256::digest(payload);
    let second = Sha256::digest(first);
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&second[..4]);
    u32::from_le_bytes(checksum)
}
