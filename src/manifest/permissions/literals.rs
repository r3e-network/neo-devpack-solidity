use super::*;

pub(crate) fn descriptor_from_literal(lit: &ir::LiteralValue) -> Option<String> {
    let bytes_le: &[u8] = match lit {
        ir::LiteralValue::Address(bytes) => bytes.as_slice(),
        ir::LiteralValue::ByteArray(bytes) if bytes.len() == 20 => bytes.as_slice(),
        _ => return None,
    };

    if bytes_le.len() != 20 {
        return None;
    }

    let bytes_be: Vec<u8> = bytes_le.iter().rev().copied().collect();
    Some(format!("0x{}", hex::encode(bytes_be)))
}

pub(crate) fn method_name_from_literal(lit: &ir::LiteralValue) -> Option<String> {
    match lit {
        ir::LiteralValue::String(bytes) => {
            let name = String::from_utf8_lossy(bytes).to_string();
            let name = name.trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }
        _ => None,
    }
}
