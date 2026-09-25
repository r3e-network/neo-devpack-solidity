//! Property-based tests for the NeoVM disassembler's *inverse-stability*.
//!
//! Motivation: a disassembler that produces inconsistent output is a debugging
//! hazard — the same bytecode rendered differently across calls suggests
//! reliance on shared mutable state (a real correctness bug). These tests pin
//! down three properties:
//!
//! 1. `disasm_idempotent_on_compiler_output` — for any contract source the
//!    compiler accepts, disassembling the emitted bytecode twice yields the
//!    exact same string. This is the closest analogue to a true
//!    `disasm -> asm -> disasm` round-trip given that no NeoVM text-assembler
//!    is exposed from this crate (only `disassemble_neovm_bytecode`).
//! 2. `disasm_handles_arbitrary_bytes_without_panic` — proptest companion to
//!    the libFuzzer harness `fuzz/fuzz_targets/fuzz_target_disasm.rs`. Random
//!    byte slices of length 0..=512 must not panic and must produce a
//!    non-empty string for any non-empty input.
//! 3. `disasm_output_includes_known_opcodes` — sanity check that mnemonics
//!    `PUSH0` (`0x10`) and `RET` (`0x40`) appear in the output for the
//!    minimal bytecode that contains them, guarding against accidental
//!    mnemonic-table regressions.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]

use super::common::*;
use neo_devpack_solidity::cli::{compile_contracts, disassemble_neovm_bytecode};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Disassembling real compiler-produced bytecode twice must yield byte-
    /// identical strings. If this ever fails, the disassembler depends on
    /// shared mutable state and rendering is non-deterministic.
    #[test]
    fn disasm_idempotent_on_compiler_output(
        var_name in identifier_strategy()
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract D {{
    uint256 public {} = 7;
}}"#,
            var_name
        );

        let artifacts = match compile_contracts(&source, false, 2) {
            Ok(a) => a,
            Err(_) => return Ok(()), // skip rejected sources
        };
        prop_assert!(!artifacts.is_empty(), "compiler returned no artifacts");

        for a in &artifacts {
            let s1 = disassemble_neovm_bytecode(&a.bytecode);
            let s2 = disassemble_neovm_bytecode(&a.bytecode);
            let s3 = disassemble_neovm_bytecode(&a.bytecode);
            prop_assert_eq!(&s1, &s2,
                "disassembler output differed between back-to-back calls (1 vs 2)");
            prop_assert_eq!(&s2, &s3,
                "disassembler output differed between back-to-back calls (2 vs 3)");
            // For any non-trivial contract bytecode, output is non-empty.
            if !a.bytecode.is_empty() {
                prop_assert!(!s1.is_empty(),
                    "non-empty bytecode produced empty disassembly");
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Random byte sequences (any length 0..=512) must not panic and must
    /// produce a non-empty rendered string when the input is non-empty.
    /// Also asserts idempotency on the same arbitrary input — re-runs of the
    /// disassembler on the same byte slice must agree.
    #[test]
    fn disasm_handles_arbitrary_bytes_without_panic(
        bytes in prop::collection::vec(any::<u8>(), 0..=512)
    ) {
        let s1 = disassemble_neovm_bytecode(&bytes);
        let s2 = disassemble_neovm_bytecode(&bytes);

        prop_assert_eq!(&s1, &s2,
            "disassembler diverged on identical arbitrary input ({} bytes)",
            bytes.len());

        if !bytes.is_empty() {
            prop_assert!(!s1.is_empty(),
                "non-empty input ({} bytes) produced empty disassembly",
                bytes.len());
        } else {
            prop_assert!(s1.is_empty(),
                "empty input produced non-empty disassembly: {:?}", s1);
        }
    }
}

/// Spot-check: minimal bytecodes containing well-known opcodes must surface
/// the canonical mnemonic substring. Catches regressions in the opcode-name
/// table (`runtime::spec::opcode_name`).
#[test]
fn disasm_output_includes_known_opcodes() {
    // 0x10 = PUSH0
    let s_push0 = disassemble_neovm_bytecode(&[0x10]);
    assert!(
        s_push0.contains("PUSH0"),
        "expected PUSH0 mnemonic in disasm of [0x10], got: {:?}",
        s_push0
    );

    // 0x40 = RET
    let s_ret = disassemble_neovm_bytecode(&[0x40]);
    assert!(
        s_ret.contains("RET"),
        "expected RET mnemonic in disasm of [0x40], got: {:?}",
        s_ret
    );

    // Combined: PUSH0 ; RET — both mnemonics present, in order.
    let s_combo = disassemble_neovm_bytecode(&[0x10, 0x40]);
    let push_idx = s_combo
        .find("PUSH0")
        .expect("PUSH0 missing from combined disasm");
    let ret_idx = s_combo
        .find("RET")
        .expect("RET missing from combined disasm");
    assert!(
        push_idx < ret_idx,
        "PUSH0 should precede RET in combined disasm; got: {:?}",
        s_combo
    );

    // Unknown opcode (0xFF is not in the opcode table) must fall back to the
    // documented `OP_FF` placeholder rather than producing empty output.
    let s_unknown = disassemble_neovm_bytecode(&[0xFF]);
    assert!(
        s_unknown.contains("OP_FF"),
        "expected OP_FF placeholder for unknown opcode 0xFF, got: {:?}",
        s_unknown
    );
}

/// Determinism on a hand-built bytecode that exercises a variable-length
/// opcode (PUSHDATA1) — important because length-prefixed forms are where
/// state-dependent rendering bugs would most plausibly hide.
#[test]
fn disasm_idempotent_on_pushdata1() {
    // PUSHDATA1 len=4 [0xDE 0xAD 0xBE 0xEF] ; RET
    let bc = [0x0C, 0x04, 0xDE, 0xAD, 0xBE, 0xEF, 0x40];
    let a = disassemble_neovm_bytecode(&bc);
    let b = disassemble_neovm_bytecode(&bc);
    assert_eq!(a, b, "disasm of PUSHDATA1 sequence not idempotent");
    assert!(
        a.contains("PUSHDATA1"),
        "PUSHDATA1 mnemonic missing: {:?}",
        a
    );
    assert!(a.contains("deadbeef"), "PUSHDATA1 payload missing: {:?}", a);
    assert!(a.contains("RET"), "trailing RET missing: {:?}", a);
}
