pub(crate) fn take<'a>(bytecode: &'a [u8], pc: &mut usize, len: usize) -> Option<&'a [u8]> {
    if pc.saturating_add(len) > bytecode.len() {
        return None;
    }
    let slice = &bytecode[*pc..*pc + len];
    *pc += len;
    Some(slice)
}

pub(crate) fn take_u8(bytecode: &[u8], pc: &mut usize) -> Option<u8> {
    take(bytecode, pc, 1).map(|b| b[0])
}

pub(crate) fn take_i8(bytecode: &[u8], pc: &mut usize) -> Option<i8> {
    take_u8(bytecode, pc).map(|b| b as i8)
}

pub(crate) fn take_u16(bytecode: &[u8], pc: &mut usize) -> Option<u16> {
    let bytes = take(bytecode, pc, 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub(crate) fn take_i16(bytecode: &[u8], pc: &mut usize) -> Option<i16> {
    take_u16(bytecode, pc).map(|v| v as i16)
}

pub(crate) fn take_u32(bytecode: &[u8], pc: &mut usize) -> Option<u32> {
    let bytes = take(bytecode, pc, 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub(crate) fn take_i32(bytecode: &[u8], pc: &mut usize) -> Option<i32> {
    take_u32(bytecode, pc).map(|v| v as i32)
}

pub(crate) fn take_i64(bytecode: &[u8], pc: &mut usize) -> Option<i64> {
    let bytes = take(bytecode, pc, 8)?;
    Some(i64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

pub(crate) fn fmt_target(bytecode_len: usize, base: usize, rel: i32) -> String {
    let base_i64 = base as i64;
    let rel_i64 = rel as i64;
    let target = base_i64.checked_add(rel_i64);
    match target {
        Some(t) if (0..bytecode_len as i64).contains(&t) => format!("0x{t:06X}"),
        _ => format!("(out of bounds: base=0x{base:06X} rel={rel})"),
    }
}
