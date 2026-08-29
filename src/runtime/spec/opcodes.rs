use super::*;
use crate::opcode::OpCode;

/// Canonical Neo N3 opcode table, derived from the [`OpCode`] enum.
///
/// The enum in [`crate::opcode`] is the single source of truth: the
/// `HashMap` below is a thin lookup view keyed by byte, kept so
/// existing call sites that take a raw `u8` (the runtime's
/// per-byte interpreter, the disassembler, …) can keep their
/// `HashMap<u8, _>` shape without re-listing every opcode.
pub static OPCODES: Lazy<HashMap<u8, OpcodeSpec>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Walk every byte 0x00..=0xFF. OpCode::try_from maps each assigned
    // byte to its variant; unassigned bytes are skipped (the spec
    // leaves holes — e.g. 0x06, 0x07, 0x42, 0x44, 0x47, 0x4C, 0x4F,
    // 0x8A, 0xA7, 0xAD, 0xB2, 0xBC, 0xBD, 0xD5..=0xD7, 0xDA, 0xDC,
    // 0xDD, 0xDE, 0xDF, 0xE2..=0xFF). Iterating is the cheapest way
    // to derive the table without a 200-line `match`.
    for byte in 0u8..=u8::MAX {
        if let Ok(op) = OpCode::try_from(byte) {
            m.insert(
                byte,
                OpcodeSpec {
                    code: byte,
                    name: op.name(),
                    gas: op.gas(),
                },
            );
        }
    }
    m
});

// PERF: `opcode_gas` is consulted once per executed instruction (the very top
// of `execute_instruction`) and `opcode_name` on every debug/disassembly step.
// Back them with flat 256-entry arrays — a bounds-free `[u8 as usize]` index —
// instead of hashing into the `HashMap` above, which costs a hash + probe per
// instruction. The arrays are derived from the same `OpCode` enum once.
static OPCODE_NAME: Lazy<[Option<&'static str>; 256]> = Lazy::new(|| {
    let mut t = [None; 256];
    for byte in 0u8..=u8::MAX {
        if let Ok(op) = OpCode::try_from(byte) {
            t[byte as usize] = Some(op.name());
        }
    }
    t
});

static OPCODE_GAS: Lazy<[Option<u64>; 256]> = Lazy::new(|| {
    let mut t = [None; 256];
    for byte in 0u8..=u8::MAX {
        if let Ok(op) = OpCode::try_from(byte) {
            t[byte as usize] = Some(op.gas());
        }
    }
    t
});

pub fn opcode_name(code: u8) -> Option<&'static str> {
    OPCODE_NAME[code as usize]
}

pub fn opcode_gas(code: u8) -> Option<u64> {
    OPCODE_GAS[code as usize]
}
