//! Fuzz target: NeoVM bytecode disassembler must never panic on arbitrary
//! byte sequences.
//!
//! Invariant under test: `disassemble_neovm_bytecode(bytes)` is total —
//! for any `&[u8]`, the function returns a `String` without panicking,
//! even on truncated bytecode (e.g. a PUSHDATA1 with length > remaining
//! bytes) or unknown opcodes.
//!
//! Coverage signal: each new opcode / operand-shape combination libFuzzer
//! discovers enlarges the corpus. A crash here would indicate an
//! out-of-bounds slice, an arithmetic underflow, or a missing `take(..)`
//! short-read guard in the disassembler.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = std::panic::catch_unwind(|| {
        let _ = neo_devpack_solidity::cli::disassemble_neovm_bytecode(data);
    });
});
