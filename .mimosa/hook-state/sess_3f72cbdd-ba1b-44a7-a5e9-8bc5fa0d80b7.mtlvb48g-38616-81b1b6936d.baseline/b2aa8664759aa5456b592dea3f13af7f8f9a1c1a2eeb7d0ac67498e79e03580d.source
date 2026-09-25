//! OpCode conversion implementations.
//!
//! `TryFrom<u8>` conversion for parsing raw opcode bytes.

use super::OpCode;

impl TryFrom<u8> for OpCode {
    type Error = ();

    /// Convert a raw byte into the canonical [`OpCode`].
    ///
    /// Returns `Err(())` for bytes that are not assigned a variant
    /// (e.g. `0x06`, `0x07`, `0x42`, `0x44`, … — the unused slots in
    /// the NeoVM spec). The runtime simulator and disassembler use
    /// this to detect unknown instructions and report them honestly
    /// rather than silently mapping to a placeholder.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => OpCode::PUSHINT8,
            0x01 => OpCode::PUSHINT16,
            0x02 => OpCode::PUSHINT32,
            0x03 => OpCode::PUSHINT64,
            0x04 => OpCode::PUSHINT128,
            0x05 => OpCode::PUSHINT256,
            0x08 => OpCode::PUSHT,
            0x09 => OpCode::PUSHF,
            0x0A => OpCode::PUSHA,
            0x0B => OpCode::PUSHNULL,
            0x0C => OpCode::PUSHDATA1,
            0x0D => OpCode::PUSHDATA2,
            0x0E => OpCode::PUSHDATA4,
            0x0F => OpCode::PUSHM1,
            0x10 => OpCode::PUSH0,
            0x11 => OpCode::PUSH1,
            0x12 => OpCode::PUSH2,
            0x13 => OpCode::PUSH3,
            0x14 => OpCode::PUSH4,
            0x15 => OpCode::PUSH5,
            0x16 => OpCode::PUSH6,
            0x17 => OpCode::PUSH7,
            0x18 => OpCode::PUSH8,
            0x19 => OpCode::PUSH9,
            0x1A => OpCode::PUSH10,
            0x1B => OpCode::PUSH11,
            0x1C => OpCode::PUSH12,
            0x1D => OpCode::PUSH13,
            0x1E => OpCode::PUSH14,
            0x1F => OpCode::PUSH15,
            0x20 => OpCode::PUSH16,
            0x21 => OpCode::NOP,
            0x22 => OpCode::JMP,
            0x23 => OpCode::JMP_L,
            0x24 => OpCode::JMPIF,
            0x25 => OpCode::JMPIF_L,
            0x26 => OpCode::JMPIFNOT,
            0x27 => OpCode::JMPIFNOT_L,
            0x28 => OpCode::JMPEQ,
            0x29 => OpCode::JMPEQ_L,
            0x2A => OpCode::JMPNE,
            0x2B => OpCode::JMPNE_L,
            0x2C => OpCode::JMPGT,
            0x2D => OpCode::JMPGT_L,
            0x2E => OpCode::JMPGE,
            0x2F => OpCode::JMPGE_L,
            0x30 => OpCode::JMPLT,
            0x31 => OpCode::JMPLT_L,
            0x32 => OpCode::JMPLE,
            0x33 => OpCode::JMPLE_L,
            0x34 => OpCode::CALL,
            0x35 => OpCode::CALL_L,
            0x36 => OpCode::CALLA,
            0x37 => OpCode::CALLT,
            0x38 => OpCode::ABORT,
            0x39 => OpCode::ASSERT,
            0x3A => OpCode::THROW,
            0x3B => OpCode::TRY,
            0x3C => OpCode::TRY_L,
            0x3D => OpCode::ENDTRY,
            0x3E => OpCode::ENDTRY_L,
            0x3F => OpCode::ENDFINALLY,
            0x40 => OpCode::RET,
            0x41 => OpCode::SYSCALL,
            0x43 => OpCode::DEPTH,
            0x45 => OpCode::DROP,
            0x46 => OpCode::NIP,
            0x48 => OpCode::XDROP,
            0x49 => OpCode::CLEAR,
            0x4A => OpCode::DUP,
            0x4B => OpCode::OVER,
            0x4D => OpCode::PICK,
            0x4E => OpCode::TUCK,
            0x50 => OpCode::SWAP,
            0x51 => OpCode::ROT,
            0x52 => OpCode::ROLL,
            0x53 => OpCode::REVERSE3,
            0x54 => OpCode::REVERSE4,
            0x55 => OpCode::REVERSEN,
            0x56 => OpCode::INITSSLOT,
            0x57 => OpCode::INITSLOT,
            0x58 => OpCode::LDSFLD0,
            0x59 => OpCode::LDSFLD1,
            0x5A => OpCode::LDSFLD2,
            0x5B => OpCode::LDSFLD3,
            0x5C => OpCode::LDSFLD4,
            0x5D => OpCode::LDSFLD5,
            0x5E => OpCode::LDSFLD6,
            0x5F => OpCode::LDSFLD,
            0x60 => OpCode::STSFLD0,
            0x61 => OpCode::STSFLD1,
            0x62 => OpCode::STSFLD2,
            0x63 => OpCode::STSFLD3,
            0x64 => OpCode::STSFLD4,
            0x65 => OpCode::STSFLD5,
            0x66 => OpCode::STSFLD6,
            0x67 => OpCode::STSFLD,
            0x68 => OpCode::LDLOC0,
            0x69 => OpCode::LDLOC1,
            0x6A => OpCode::LDLOC2,
            0x6B => OpCode::LDLOC3,
            0x6C => OpCode::LDLOC4,
            0x6D => OpCode::LDLOC5,
            0x6E => OpCode::LDLOC6,
            0x6F => OpCode::LDLOC,
            0x70 => OpCode::STLOC0,
            0x71 => OpCode::STLOC1,
            0x72 => OpCode::STLOC2,
            0x73 => OpCode::STLOC3,
            0x74 => OpCode::STLOC4,
            0x75 => OpCode::STLOC5,
            0x76 => OpCode::STLOC6,
            0x77 => OpCode::STLOC,
            0x78 => OpCode::LDARG0,
            0x79 => OpCode::LDARG1,
            0x7A => OpCode::LDARG2,
            0x7B => OpCode::LDARG3,
            0x7C => OpCode::LDARG4,
            0x7D => OpCode::LDARG5,
            0x7E => OpCode::LDARG6,
            0x7F => OpCode::LDARG,
            0x80 => OpCode::STARG0,
            0x81 => OpCode::STARG1,
            0x82 => OpCode::STARG2,
            0x83 => OpCode::STARG3,
            0x84 => OpCode::STARG4,
            0x85 => OpCode::STARG5,
            0x86 => OpCode::STARG6,
            0x87 => OpCode::STARG,
            0x88 => OpCode::NEWBUFFER,
            0x89 => OpCode::MEMCPY,
            0x8B => OpCode::CAT,
            0x8C => OpCode::SUBSTR,
            0x8D => OpCode::LEFT,
            0x8E => OpCode::RIGHT,
            0x90 => OpCode::INVERT,
            0x91 => OpCode::AND,
            0x92 => OpCode::OR,
            0x93 => OpCode::XOR,
            0x97 => OpCode::EQUAL,
            0x98 => OpCode::NOTEQUAL,
            0x99 => OpCode::SIGN,
            0x9A => OpCode::ABS,
            0x9B => OpCode::NEGATE,
            0x9C => OpCode::INC,
            0x9D => OpCode::DEC,
            0x9E => OpCode::ADD,
            0x9F => OpCode::SUB,
            0xA0 => OpCode::MUL,
            0xA1 => OpCode::DIV,
            0xA2 => OpCode::MOD,
            0xA3 => OpCode::POW,
            0xA4 => OpCode::SQRT,
            0xA5 => OpCode::MODMUL,
            0xA6 => OpCode::MODPOW,
            0xA8 => OpCode::SHL,
            0xA9 => OpCode::SHR,
            0xAA => OpCode::NOT,
            0xAB => OpCode::BOOLAND,
            0xAC => OpCode::BOOLOR,
            0xB1 => OpCode::NZ,
            0xB3 => OpCode::NUMEQUAL,
            0xB4 => OpCode::NUMNOTEQUAL,
            0xB5 => OpCode::LT,
            0xB6 => OpCode::LE,
            0xB7 => OpCode::GT,
            0xB8 => OpCode::GE,
            0xB9 => OpCode::MIN,
            0xBA => OpCode::MAX,
            0xBB => OpCode::WITHIN,
            0xBE => OpCode::PACKMAP,
            0xBF => OpCode::PACKSTRUCT,
            0xC0 => OpCode::PACK,
            0xC1 => OpCode::UNPACK,
            0xC2 => OpCode::NEWARRAY0,
            0xC3 => OpCode::NEWARRAY,
            0xC4 => OpCode::NEWARRAY_T,
            0xC5 => OpCode::NEWSTRUCT0,
            0xC6 => OpCode::NEWSTRUCT,
            0xC8 => OpCode::NEWMAP,
            0xCA => OpCode::SIZE,
            0xCB => OpCode::HASKEY,
            0xCC => OpCode::KEYS,
            0xCD => OpCode::VALUES,
            0xCE => OpCode::PICKITEM,
            0xCF => OpCode::APPEND,
            0xD0 => OpCode::SETITEM,
            0xD1 => OpCode::REVERSEITEMS,
            0xD2 => OpCode::REMOVE,
            0xD3 => OpCode::CLEARITEMS,
            0xD4 => OpCode::POPITEM,
            0xD8 => OpCode::ISNULL,
            0xD9 => OpCode::ISTYPE,
            0xDB => OpCode::CONVERT,
            0xE0 => OpCode::ABORTMSG,
            0xE1 => OpCode::ASSERTMSG,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_roundtrips_through_try_from() {
        // Every variant must round-trip byte -> variant -> byte.
        let cases = [
            OpCode::PUSHINT8,
            OpCode::PUSHINT256,
            OpCode::PUSHDATA1,
            OpCode::PUSHDATA4,
            OpCode::PUSH0,
            OpCode::PUSH16,
            OpCode::NOP,
            OpCode::JMP_L,
            OpCode::CALL,
            OpCode::SYSCALL,
            OpCode::RET,
            OpCode::DUP,
            OpCode::LDLOC0,
            OpCode::LDLOC6,
            OpCode::LDLOC,
            OpCode::STARG,
            OpCode::ADD,
            OpCode::MODPOW,
            OpCode::ISNULL,
            OpCode::CONVERT,
            OpCode::ABORTMSG,
            OpCode::ASSERTMSG,
        ];
        for op in cases {
            let byte = op.byte();
            let parsed = OpCode::try_from(byte).expect("known opcode");
            assert_eq!(parsed, op, "byte 0x{byte:02X} round-trip mismatch");
        }
    }

    #[test]
    fn try_from_rejects_unassigned_bytes() {
        // Holes in the opcode space: 0x06, 0x07, 0x42, 0x44, 0x47, 0x4C, 0x4F, …
        for byte in [
            0x06u8, 0x07, 0x42, 0x44, 0x47, 0x4C, 0x4F, 0x8A, 0xA7, 0xAD, 0xD5,
        ] {
            assert!(
                OpCode::try_from(byte).is_err(),
                "byte 0x{byte:02X} should be unassigned"
            );
        }
    }

    #[test]
    fn indexed_constructors_match_byte_layout() {
        // PUSHn
        for n in 0u8..=16 {
            assert_eq!(OpCode::push_small(n).unwrap().byte(), 0x10 + n);
        }
        assert!(OpCode::push_small(17).is_none());

        // LDLOCn / STLOCn
        for n in 0u8..=6 {
            assert_eq!(OpCode::ldloc(n).byte(), 0x68 + n);
            assert_eq!(OpCode::stloc(n).byte(), 0x70 + n);
        }
        assert_eq!(OpCode::ldloc(7).byte(), OpCode::LDLOC.byte());
        assert_eq!(OpCode::stloc(255).byte(), OpCode::STLOC.byte());

        // LDARGn / STARGn
        for n in 0u8..=6 {
            assert_eq!(OpCode::ldarg(n).byte(), 0x78 + n);
            assert_eq!(OpCode::starg(n).byte(), 0x80 + n);
        }
        assert_eq!(OpCode::ldarg(7).byte(), OpCode::LDARG.byte());
        assert_eq!(OpCode::starg(7).byte(), OpCode::STARG.byte());

        // LDSFLDn / STSFLDn
        for n in 0u8..=6 {
            assert_eq!(OpCode::ldsfld(n).byte(), 0x58 + n);
            assert_eq!(OpCode::stsfld(n).byte(), 0x60 + n);
        }
        assert_eq!(OpCode::ldsfld(7).byte(), OpCode::LDSFLD.byte());
        assert_eq!(OpCode::stsfld(7).byte(), OpCode::STSFLD.byte());
    }

    #[test]
    fn push_data_picks_narrowest_prefix() {
        assert_eq!(OpCode::push_data(0).byte(), 0x0C);
        assert_eq!(OpCode::push_data(0xFF).byte(), 0x0C);
        assert_eq!(OpCode::push_data(0x100).byte(), 0x0D);
        assert_eq!(OpCode::push_data(0xFFFF).byte(), 0x0D);
        assert_eq!(OpCode::push_data(0x1_0000).byte(), 0x0E);
    }

    #[test]
    fn push_int_picks_smallest_signed_fit() {
        assert_eq!(OpCode::push_int(0).byte(), 0x00);
        assert_eq!(OpCode::push_int(0xFF).byte(), 0x00);
        assert_eq!(OpCode::push_int(0x100).byte(), 0x01);
        assert_eq!(OpCode::push_int(0xFFFF).byte(), 0x01);
        assert_eq!(OpCode::push_int(0x1_0000).byte(), 0x02);
        assert_eq!(OpCode::push_int(0xFFFF_FFFF).byte(), 0x02);
        assert_eq!(OpCode::push_int(0x1_0000_0000).byte(), 0x03);
        assert_eq!(OpCode::push_int(u64::MAX).byte(), 0x03);
    }

    #[test]
    fn name_is_stable_and_nonempty() {
        // Every defined variant must have a stable, non-empty name.
        for byte in 0x00u8..=0xE1 {
            if let Ok(op) = OpCode::try_from(byte) {
                let n = op.name();
                assert!(!n.is_empty(), "byte 0x{byte:02X} has empty name");
                assert!(n.is_ascii(), "byte 0x{byte:02X} name '{n}' must be ASCII");
            }
        }
    }
}
