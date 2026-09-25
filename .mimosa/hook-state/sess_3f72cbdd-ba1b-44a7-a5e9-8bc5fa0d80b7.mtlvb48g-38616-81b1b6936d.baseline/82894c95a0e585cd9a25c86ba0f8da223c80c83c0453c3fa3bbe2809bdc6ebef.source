//! OpCode method implementations.
//!
//! This file contains the `impl OpCode` block with all
//! constructors, helpers, and utility methods.

use super::OpCode;

impl OpCode {
    /// The raw byte value of this opcode, as it appears in a script.
    ///
    /// `#[repr(u8)]` makes the layout trivially `as u8`-castable, but
    /// we expose this explicit `const fn` so call sites do not need to
    /// write `as u8` everywhere.
    #[inline]
    pub const fn byte(self) -> u8 {
        self as u8
    }

    /// Static, canonical name (e.g. `"PUSH0"`, `"JMP_L"`).
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            OpCode::PUSHINT8 => "PUSHINT8",
            OpCode::PUSHINT16 => "PUSHINT16",
            OpCode::PUSHINT32 => "PUSHINT32",
            OpCode::PUSHINT64 => "PUSHINT64",
            OpCode::PUSHINT128 => "PUSHINT128",
            OpCode::PUSHINT256 => "PUSHINT256",
            OpCode::PUSHT => "PUSHT",
            OpCode::PUSHF => "PUSHF",
            OpCode::PUSHA => "PUSHA",
            OpCode::PUSHNULL => "PUSHNULL",
            OpCode::PUSHDATA1 => "PUSHDATA1",
            OpCode::PUSHDATA2 => "PUSHDATA2",
            OpCode::PUSHDATA4 => "PUSHDATA4",
            OpCode::PUSHM1 => "PUSHM1",
            OpCode::PUSH0 => "PUSH0",
            OpCode::PUSH1 => "PUSH1",
            OpCode::PUSH2 => "PUSH2",
            OpCode::PUSH3 => "PUSH3",
            OpCode::PUSH4 => "PUSH4",
            OpCode::PUSH5 => "PUSH5",
            OpCode::PUSH6 => "PUSH6",
            OpCode::PUSH7 => "PUSH7",
            OpCode::PUSH8 => "PUSH8",
            OpCode::PUSH9 => "PUSH9",
            OpCode::PUSH10 => "PUSH10",
            OpCode::PUSH11 => "PUSH11",
            OpCode::PUSH12 => "PUSH12",
            OpCode::PUSH13 => "PUSH13",
            OpCode::PUSH14 => "PUSH14",
            OpCode::PUSH15 => "PUSH15",
            OpCode::PUSH16 => "PUSH16",
            OpCode::NOP => "NOP",
            OpCode::JMP => "JMP",
            OpCode::JMP_L => "JMP_L",
            OpCode::JMPIF => "JMPIF",
            OpCode::JMPIF_L => "JMPIF_L",
            OpCode::JMPIFNOT => "JMPIFNOT",
            OpCode::JMPIFNOT_L => "JMPIFNOT_L",
            OpCode::JMPEQ => "JMPEQ",
            OpCode::JMPEQ_L => "JMPEQ_L",
            OpCode::JMPNE => "JMPNE",
            OpCode::JMPNE_L => "JMPNE_L",
            OpCode::JMPGT => "JMPGT",
            OpCode::JMPGT_L => "JMPGT_L",
            OpCode::JMPGE => "JMPGE",
            OpCode::JMPGE_L => "JMPGE_L",
            OpCode::JMPLT => "JMPLT",
            OpCode::JMPLT_L => "JMPLT_L",
            OpCode::JMPLE => "JMPLE",
            OpCode::JMPLE_L => "JMPLE_L",
            OpCode::CALL => "CALL",
            OpCode::CALL_L => "CALL_L",
            OpCode::CALLA => "CALLA",
            OpCode::CALLT => "CALLT",
            OpCode::ABORT => "ABORT",
            OpCode::ASSERT => "ASSERT",
            OpCode::THROW => "THROW",
            OpCode::TRY => "TRY",
            OpCode::TRY_L => "TRY_L",
            OpCode::ENDTRY => "ENDTRY",
            OpCode::ENDTRY_L => "ENDTRY_L",
            OpCode::ENDFINALLY => "ENDFINALLY",
            OpCode::RET => "RET",
            OpCode::SYSCALL => "SYSCALL",
            OpCode::DEPTH => "DEPTH",
            OpCode::DROP => "DROP",
            OpCode::NIP => "NIP",
            OpCode::XDROP => "XDROP",
            OpCode::CLEAR => "CLEAR",
            OpCode::DUP => "DUP",
            OpCode::OVER => "OVER",
            OpCode::PICK => "PICK",
            OpCode::TUCK => "TUCK",
            OpCode::SWAP => "SWAP",
            OpCode::ROT => "ROT",
            OpCode::ROLL => "ROLL",
            OpCode::REVERSE3 => "REVERSE3",
            OpCode::REVERSE4 => "REVERSE4",
            OpCode::REVERSEN => "REVERSEN",
            OpCode::INITSSLOT => "INITSSLOT",
            OpCode::INITSLOT => "INITSLOT",
            OpCode::LDSFLD0 => "LDSFLD0",
            OpCode::LDSFLD1 => "LDSFLD1",
            OpCode::LDSFLD2 => "LDSFLD2",
            OpCode::LDSFLD3 => "LDSFLD3",
            OpCode::LDSFLD4 => "LDSFLD4",
            OpCode::LDSFLD5 => "LDSFLD5",
            OpCode::LDSFLD6 => "LDSFLD6",
            OpCode::LDSFLD => "LDSFLD",
            OpCode::STSFLD0 => "STSFLD0",
            OpCode::STSFLD1 => "STSFLD1",
            OpCode::STSFLD2 => "STSFLD2",
            OpCode::STSFLD3 => "STSFLD3",
            OpCode::STSFLD4 => "STSFLD4",
            OpCode::STSFLD5 => "STSFLD5",
            OpCode::STSFLD6 => "STSFLD6",
            OpCode::STSFLD => "STSFLD",
            OpCode::LDLOC0 => "LDLOC0",
            OpCode::LDLOC1 => "LDLOC1",
            OpCode::LDLOC2 => "LDLOC2",
            OpCode::LDLOC3 => "LDLOC3",
            OpCode::LDLOC4 => "LDLOC4",
            OpCode::LDLOC5 => "LDLOC5",
            OpCode::LDLOC6 => "LDLOC6",
            OpCode::LDLOC => "LDLOC",
            OpCode::STLOC0 => "STLOC0",
            OpCode::STLOC1 => "STLOC1",
            OpCode::STLOC2 => "STLOC2",
            OpCode::STLOC3 => "STLOC3",
            OpCode::STLOC4 => "STLOC4",
            OpCode::STLOC5 => "STLOC5",
            OpCode::STLOC6 => "STLOC6",
            OpCode::STLOC => "STLOC",
            OpCode::LDARG0 => "LDARG0",
            OpCode::LDARG1 => "LDARG1",
            OpCode::LDARG2 => "LDARG2",
            OpCode::LDARG3 => "LDARG3",
            OpCode::LDARG4 => "LDARG4",
            OpCode::LDARG5 => "LDARG5",
            OpCode::LDARG6 => "LDARG6",
            OpCode::LDARG => "LDARG",
            OpCode::STARG0 => "STARG0",
            OpCode::STARG1 => "STARG1",
            OpCode::STARG2 => "STARG2",
            OpCode::STARG3 => "STARG3",
            OpCode::STARG4 => "STARG4",
            OpCode::STARG5 => "STARG5",
            OpCode::STARG6 => "STARG6",
            OpCode::STARG => "STARG",
            OpCode::NEWBUFFER => "NEWBUFFER",
            OpCode::MEMCPY => "MEMCPY",
            OpCode::CAT => "CAT",
            OpCode::SUBSTR => "SUBSTR",
            OpCode::LEFT => "LEFT",
            OpCode::RIGHT => "RIGHT",
            OpCode::INVERT => "INVERT",
            OpCode::AND => "AND",
            OpCode::OR => "OR",
            OpCode::XOR => "XOR",
            OpCode::EQUAL => "EQUAL",
            OpCode::NOTEQUAL => "NOTEQUAL",
            OpCode::SIGN => "SIGN",
            OpCode::ABS => "ABS",
            OpCode::NEGATE => "NEGATE",
            OpCode::INC => "INC",
            OpCode::DEC => "DEC",
            OpCode::ADD => "ADD",
            OpCode::SUB => "SUB",
            OpCode::MUL => "MUL",
            OpCode::DIV => "DIV",
            OpCode::MOD => "MOD",
            OpCode::POW => "POW",
            OpCode::SQRT => "SQRT",
            OpCode::MODMUL => "MODMUL",
            OpCode::MODPOW => "MODPOW",
            OpCode::SHL => "SHL",
            OpCode::SHR => "SHR",
            OpCode::NOT => "NOT",
            OpCode::BOOLAND => "BOOLAND",
            OpCode::BOOLOR => "BOOLOR",
            OpCode::NZ => "NZ",
            OpCode::NUMEQUAL => "NUMEQUAL",
            OpCode::NUMNOTEQUAL => "NUMNOTEQUAL",
            OpCode::LT => "LT",
            OpCode::LE => "LE",
            OpCode::GT => "GT",
            OpCode::GE => "GE",
            OpCode::MIN => "MIN",
            OpCode::MAX => "MAX",
            OpCode::WITHIN => "WITHIN",
            OpCode::PACKMAP => "PACKMAP",
            OpCode::PACKSTRUCT => "PACKSTRUCT",
            OpCode::PACK => "PACK",
            OpCode::UNPACK => "UNPACK",
            OpCode::NEWARRAY0 => "NEWARRAY0",
            OpCode::NEWARRAY => "NEWARRAY",
            OpCode::NEWARRAY_T => "NEWARRAY_T",
            OpCode::NEWSTRUCT0 => "NEWSTRUCT0",
            OpCode::NEWSTRUCT => "NEWSTRUCT",
            OpCode::NEWMAP => "NEWMAP",
            OpCode::SIZE => "SIZE",
            OpCode::HASKEY => "HASKEY",
            OpCode::KEYS => "KEYS",
            OpCode::VALUES => "VALUES",
            OpCode::PICKITEM => "PICKITEM",
            OpCode::APPEND => "APPEND",
            OpCode::SETITEM => "SETITEM",
            OpCode::REVERSEITEMS => "REVERSEITEMS",
            OpCode::REMOVE => "REMOVE",
            OpCode::CLEARITEMS => "CLEARITEMS",
            OpCode::POPITEM => "POPITEM",
            OpCode::ISNULL => "ISNULL",
            OpCode::ISTYPE => "ISTYPE",
            OpCode::CONVERT => "CONVERT",
            OpCode::ABORTMSG => "ABORTMSG",
            OpCode::ASSERTMSG => "ASSERTMSG",
        }
    }

    /// Default gas cost (matches the canonical Neo N3 spec).
    ///
    /// `0` is returned for opcodes that do not consume gas on their
    /// own (e.g. `RET`); the actual cost depends on context for
    /// opcodes like `CALL` and `SYSCALL` whose pricing is dynamic
    /// — callers that care about exact cost should special-case.
    #[inline]
    pub const fn gas(self) -> u64 {
        match self {
            OpCode::PUSHINT8
            | OpCode::PUSHINT16
            | OpCode::PUSHINT32
            | OpCode::PUSHINT64
            | OpCode::PUSHINT128
            | OpCode::PUSHINT256
            | OpCode::PUSHT
            | OpCode::PUSHF
            | OpCode::PUSHA
            | OpCode::PUSHNULL
            | OpCode::PUSHM1
            | OpCode::PUSH0
            | OpCode::PUSH1
            | OpCode::PUSH2
            | OpCode::PUSH3
            | OpCode::PUSH4
            | OpCode::PUSH5
            | OpCode::PUSH6
            | OpCode::PUSH7
            | OpCode::PUSH8
            | OpCode::PUSH9
            | OpCode::PUSH10
            | OpCode::PUSH11
            | OpCode::PUSH12
            | OpCode::PUSH13
            | OpCode::PUSH14
            | OpCode::PUSH15
            | OpCode::PUSH16 => 1,
            OpCode::PUSHDATA1 | OpCode::PUSHDATA2 | OpCode::PUSHDATA4 => 2,
            OpCode::NOP
            | OpCode::JMP
            | OpCode::JMP_L
            | OpCode::JMPIF
            | OpCode::JMPIF_L
            | OpCode::JMPIFNOT
            | OpCode::JMPIFNOT_L
            | OpCode::JMPEQ
            | OpCode::JMPEQ_L
            | OpCode::JMPNE
            | OpCode::JMPNE_L
            | OpCode::JMPGT
            | OpCode::JMPGT_L
            | OpCode::JMPGE
            | OpCode::JMPGE_L
            | OpCode::JMPLT
            | OpCode::JMPLT_L
            | OpCode::JMPLE
            | OpCode::JMPLE_L => 2,
            OpCode::CALL | OpCode::CALL_L | OpCode::CALLA | OpCode::CALLT => 512,
            OpCode::ABORT | OpCode::ASSERT | OpCode::THROW => 1,
            OpCode::TRY
            | OpCode::TRY_L
            | OpCode::ENDTRY
            | OpCode::ENDTRY_L
            | OpCode::ENDFINALLY => 1,
            OpCode::RET => 0,
            OpCode::SYSCALL => 10,
            OpCode::DEPTH
            | OpCode::DROP
            | OpCode::NIP
            | OpCode::XDROP
            | OpCode::CLEAR
            | OpCode::DUP
            | OpCode::OVER
            | OpCode::PICK
            | OpCode::TUCK
            | OpCode::SWAP
            | OpCode::ROT
            | OpCode::ROLL
            | OpCode::REVERSE3
            | OpCode::REVERSE4
            | OpCode::REVERSEN
            | OpCode::INITSSLOT
            | OpCode::INITSLOT
            | OpCode::LDSFLD0
            | OpCode::LDSFLD1
            | OpCode::LDSFLD2
            | OpCode::LDSFLD3
            | OpCode::LDSFLD4
            | OpCode::LDSFLD5
            | OpCode::LDSFLD6
            | OpCode::LDSFLD
            | OpCode::STSFLD0
            | OpCode::STSFLD1
            | OpCode::STSFLD2
            | OpCode::STSFLD3
            | OpCode::STSFLD4
            | OpCode::STSFLD5
            | OpCode::STSFLD6
            | OpCode::STSFLD
            | OpCode::LDLOC0
            | OpCode::LDLOC1
            | OpCode::LDLOC2
            | OpCode::LDLOC3
            | OpCode::LDLOC4
            | OpCode::LDLOC5
            | OpCode::LDLOC6
            | OpCode::LDLOC
            | OpCode::STLOC0
            | OpCode::STLOC1
            | OpCode::STLOC2
            | OpCode::STLOC3
            | OpCode::STLOC4
            | OpCode::STLOC5
            | OpCode::STLOC6
            | OpCode::STLOC
            | OpCode::LDARG0
            | OpCode::LDARG1
            | OpCode::LDARG2
            | OpCode::LDARG3
            | OpCode::LDARG4
            | OpCode::LDARG5
            | OpCode::LDARG6
            | OpCode::LDARG
            | OpCode::STARG0
            | OpCode::STARG1
            | OpCode::STARG2
            | OpCode::STARG3
            | OpCode::STARG4
            | OpCode::STARG5
            | OpCode::STARG6
            | OpCode::STARG
            | OpCode::ISNULL
            | OpCode::ISTYPE
            | OpCode::CONVERT
            | OpCode::NOT
            | OpCode::BOOLAND
            | OpCode::BOOLOR
            | OpCode::NZ => 2,
            OpCode::NEWBUFFER
            | OpCode::MEMCPY
            | OpCode::CAT
            | OpCode::SUBSTR
            | OpCode::LEFT
            | OpCode::RIGHT
            | OpCode::PACKMAP
            | OpCode::PACKSTRUCT
            | OpCode::PACK
            | OpCode::UNPACK
            | OpCode::NEWARRAY0
            | OpCode::NEWARRAY
            | OpCode::NEWARRAY_T
            | OpCode::NEWSTRUCT0
            | OpCode::NEWSTRUCT
            | OpCode::NEWMAP
            | OpCode::SIZE
            | OpCode::HASKEY
            | OpCode::KEYS
            | OpCode::VALUES
            | OpCode::PICKITEM
            | OpCode::APPEND
            | OpCode::SETITEM
            | OpCode::REVERSEITEMS
            | OpCode::REMOVE
            | OpCode::CLEARITEMS
            | OpCode::POPITEM => 4,
            OpCode::INVERT
            | OpCode::AND
            | OpCode::OR
            | OpCode::XOR
            | OpCode::EQUAL
            | OpCode::NOTEQUAL => 3,
            OpCode::SIGN
            | OpCode::ABS
            | OpCode::NEGATE
            | OpCode::INC
            | OpCode::DEC
            | OpCode::ADD
            | OpCode::SUB
            | OpCode::SHL
            | OpCode::SHR
            | OpCode::NUMEQUAL
            | OpCode::NUMNOTEQUAL
            | OpCode::LT
            | OpCode::LE
            | OpCode::GT
            | OpCode::GE
            | OpCode::MIN
            | OpCode::MAX
            | OpCode::WITHIN => 3,
            OpCode::MUL | OpCode::DIV | OpCode::MOD => 5,
            OpCode::POW | OpCode::SQRT | OpCode::MODMUL | OpCode::MODPOW => match self {
                OpCode::POW => 8,
                OpCode::SQRT => 6,
                _ => 8,
            },
            OpCode::ABORTMSG | OpCode::ASSERTMSG => 1,
        }
    }

    /// `PUSH0`..`PUSH16` for `n` in `0..=16`, or `None` for any other
    /// value. Use this anywhere the codegen computes a small integer
    /// push instead of writing `0x10 + n` by hand.
    #[inline]
    pub const fn push_small(n: u8) -> Option<Self> {
        match n {
            0 => Some(OpCode::PUSH0),
            1 => Some(OpCode::PUSH1),
            2 => Some(OpCode::PUSH2),
            3 => Some(OpCode::PUSH3),
            4 => Some(OpCode::PUSH4),
            5 => Some(OpCode::PUSH5),
            6 => Some(OpCode::PUSH6),
            7 => Some(OpCode::PUSH7),
            8 => Some(OpCode::PUSH8),
            9 => Some(OpCode::PUSH9),
            10 => Some(OpCode::PUSH10),
            11 => Some(OpCode::PUSH11),
            12 => Some(OpCode::PUSH12),
            13 => Some(OpCode::PUSH13),
            14 => Some(OpCode::PUSH14),
            15 => Some(OpCode::PUSH15),
            16 => Some(OpCode::PUSH16),
            _ => None,
        }
    }

    /// `LDLOC0`..`LDLOC6` for `n` in `0..=6`, otherwise the generic
    /// `LDLOC` opcode (which is followed by a 1-byte index operand).
    #[inline]
    pub const fn ldloc(n: u8) -> Self {
        match n {
            0 => OpCode::LDLOC0,
            1 => OpCode::LDLOC1,
            2 => OpCode::LDLOC2,
            3 => OpCode::LDLOC3,
            4 => OpCode::LDLOC4,
            5 => OpCode::LDLOC5,
            6 => OpCode::LDLOC6,
            _ => OpCode::LDLOC,
        }
    }

    /// `STLOC0`..`STLOC6` for `n` in `0..=6`, otherwise the generic
    /// `STLOC` opcode (which is followed by a 1-byte index operand).
    #[inline]
    pub const fn stloc(n: u8) -> Self {
        match n {
            0 => OpCode::STLOC0,
            1 => OpCode::STLOC1,
            2 => OpCode::STLOC2,
            3 => OpCode::STLOC3,
            4 => OpCode::STLOC4,
            5 => OpCode::STLOC5,
            6 => OpCode::STLOC6,
            _ => OpCode::STLOC,
        }
    }

    /// `LDARG0`..`LDARG6` for `n` in `0..=6`, otherwise the generic
    /// `LDARG` opcode (which is followed by a 1-byte index operand).
    #[inline]
    pub const fn ldarg(n: u8) -> Self {
        match n {
            0 => OpCode::LDARG0,
            1 => OpCode::LDARG1,
            2 => OpCode::LDARG2,
            3 => OpCode::LDARG3,
            4 => OpCode::LDARG4,
            5 => OpCode::LDARG5,
            6 => OpCode::LDARG6,
            _ => OpCode::LDARG,
        }
    }

    /// `STARG0`..`STARG6` for `n` in `0..=6`, otherwise the generic
    /// `STARG` opcode (which is followed by a 1-byte index operand).
    #[inline]
    pub const fn starg(n: u8) -> Self {
        match n {
            0 => OpCode::STARG0,
            1 => OpCode::STARG1,
            2 => OpCode::STARG2,
            3 => OpCode::STARG3,
            4 => OpCode::STARG4,
            5 => OpCode::STARG5,
            6 => OpCode::STARG6,
            _ => OpCode::STARG,
        }
    }

    /// `LDSFLD0`..`LDSFLD6` for `n` in `0..=6`, otherwise the generic
    /// `LDSFLD` opcode (which is followed by a 1-byte index operand).
    #[inline]
    pub const fn ldsfld(n: u8) -> Self {
        match n {
            0 => OpCode::LDSFLD0,
            1 => OpCode::LDSFLD1,
            2 => OpCode::LDSFLD2,
            3 => OpCode::LDSFLD3,
            4 => OpCode::LDSFLD4,
            5 => OpCode::LDSFLD5,
            6 => OpCode::LDSFLD6,
            _ => OpCode::LDSFLD,
        }
    }

    /// `STSFLD0`..`STSFLD6` for `n` in `0..=6`, otherwise the generic
    /// `STSFLD` opcode (which is followed by a 1-byte index operand).
    #[inline]
    pub const fn stsfld(n: u8) -> Self {
        match n {
            0 => OpCode::STSFLD0,
            1 => OpCode::STSFLD1,
            2 => OpCode::STSFLD2,
            3 => OpCode::STSFLD3,
            4 => OpCode::STSFLD4,
            5 => OpCode::STSFLD5,
            6 => OpCode::STSFLD6,
            _ => OpCode::STSFLD,
        }
    }

    /// Pick the right `PUSHDATA*` opcode for a payload of `len` bytes.
    ///
    /// - `len ≤ 0xFF`  → [`OpCode::PUSHDATA1`] (1-byte length prefix)
    /// - `len ≤ 0xFFFF` → [`OpCode::PUSHDATA2`] (2-byte length prefix)
    /// - else           → [`OpCode::PUSHDATA4`] (4-byte length prefix)
    #[inline]
    pub const fn push_data(len: usize) -> Self {
        if len <= u8::MAX as usize {
            OpCode::PUSHDATA1
        } else if len <= u16::MAX as usize {
            OpCode::PUSHDATA2
        } else {
            OpCode::PUSHDATA4
        }
    }

    /// Pick the smallest `PUSHINT*` opcode that fits `value` (as
    /// little-endian two's-complement, signed).
    ///
    /// - `0..=0xFF`        → [`OpCode::PUSHINT8`]
    /// - `0..=0xFFFF`      → [`OpCode::PUSHINT16`]
    /// - `0..=0xFFFF_FFFF` → [`OpCode::PUSHINT32`]
    /// - else              → [`OpCode::PUSHINT64`]
    ///
    /// Values that do not fit in `i64` (or that are negative) should
    /// be handled by the caller via [`OpCode::PUSHINT128`] or
    /// [`OpCode::PUSHINT256`].
    #[inline]
    pub const fn push_int(value: u64) -> Self {
        if value <= u8::MAX as u64 {
            OpCode::PUSHINT8
        } else if value <= u16::MAX as u64 {
            OpCode::PUSHINT16
        } else if value <= u32::MAX as u64 {
            OpCode::PUSHINT32
        } else {
            OpCode::PUSHINT64
        }
    }
}
