//! End-to-end regression guards for the v0.30.3 independent-audit fixes
//! A1 / A2 / A7 (see the audit report). These are behavioral: they compile a
//! probe with the *real* compiler at a chosen optimizer level and (where the
//! path is executable) run it on the in-tree NeoVM simulator, asserting the
//! Solidity-mandated `Panic(0x11)` / correct value. Each test would fail on the
//! pre-fix code, so they are genuine guards rather than tautologies.
//!
//! - **A1** — `type(int256).max + 1` must revert with `Panic(0x11)` at the
//!   *default* `-O2` (previously the constant folder collapsed the checked add
//!   into the literal `2^255`, which re-emitted as `int256::MIN` and defeated the
//!   runtime overflow guard, silently returning the wrapped value).
//! - **A2** — narrow integer widths outside {8,16,32,64,128} (uint96 / uint112 /
//!   uint160 / uint224 …, common in Uniswap/Maker) must get checked-overflow
//!   guards and `<<` width truncation, which the hardcoded set silently omitted.
//! - **A7** — a qualified reference to a *nonexistent* library/contract member
//!   (e.g. `Lib.TYPO`) must fail compilation, not silently fold to `0` /
//!   `address(0)`.

#![allow(clippy::uninlined_format_args, non_snake_case)]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::codegen::disassemble_neovm_bytecode;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

fn wrap(body: &str, ret: &str) -> String {
    format!(
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\ncontract C {{ function run() public pure returns ({ret}) {{ {body} }} }}"
    )
}

/// Compile `src` at `level` and execute its no-arg `run()`, returning
/// `Ok(value)` (little-endian unsigned, up to 16 bytes) on success or
/// `Err(panic_code)` when the contract faults with a `Panic(uint256)` envelope.
fn run_at(src: &str, level: u8) -> Result<u128, u8> {
    let arts = compile_contracts(src, false, level).expect("compile must succeed");
    assert!(!arts.is_empty(), "no artifacts");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let res = rt
        .execute(&arts[0].bytecode, &[])
        .expect("host-level execute must not error");
    if res.success {
        let mut v: u128 = 0;
        for (i, b) in res.return_data.iter().enumerate().take(16) {
            v |= (*b as u128) << (8 * i);
        }
        Ok(v)
    } else if res.return_data.len() >= 36 && res.return_data[..4] == [0x4e, 0x48, 0x7b, 0x71] {
        Err(res.return_data[35])
    } else {
        Err(0xff)
    }
}

fn assemble(src: &str, level: u8) -> String {
    let arts = compile_contracts(src, false, level).expect("compile must succeed");
    disassemble_neovm_bytecode(&arts[0].bytecode)
}

// ============================================================================
// A1 — checked int256 overflow must revert at every optimizer level
// ============================================================================

#[test]
fn a1_int256_max_plus_one_panics_at_default_O2() {
    let src = wrap("return type(int256).max + 1;", "int256");
    // Level 2 is the CLI / standard-JSON default.
    assert_eq!(
        run_at(&src, 2),
        Err(0x11),
        "type(int256).max + 1 must Panic(0x11) at -O2 (was silently folded to int256::MIN)"
    );
}

#[test]
fn a1_int256_overflow_parity_across_levels() {
    let src = wrap("int256 r = type(int256).max + 1; return r;", "int256");
    let o0 = run_at(&src, 0);
    let o2 = run_at(&src, 2);
    assert_eq!(o0, Err(0x11), "-O0 must revert");
    assert_eq!(
        o0, o2,
        "-O0 and -O2 must agree (the fold may not change semantics)"
    );
}

#[test]
fn a1_int256_underflow_panics_across_levels() {
    let src = wrap("return type(int256).min - 1;", "int256");
    assert_eq!(run_at(&src, 2), Err(0x11));
    assert_eq!(run_at(&src, 0), Err(0x11));
}

#[test]
fn a1_O2_still_emits_add_and_guard_for_int_overflow() {
    let src = wrap("return type(int256).max + 1;", "int256");
    let asm = assemble(&src, 2);
    // The checked add + its Panic(0x11) envelope must survive the optimizer:
    // an ADD and a THROW both present (previously folded to a lone PUSHINT).
    assert!(asm.contains(" ADD"), "expected a runtime ADD: {asm}");
    assert!(
        asm.contains("THROW"),
        "expected the overflow THROW guard: {asm}"
    );
}

#[test]
fn a1_in_range_int256_add_still_folds_ok() {
    // A non-overflowing checked add must still evaluate correctly at -O2.
    assert_eq!(
        run_at(&wrap("return 1000000 + 12345;", "int256"), 2),
        Ok(1_012_345)
    );
}

#[test]
fn a1_uint256_max_plus_one_panics() {
    // uint256 overflow (result 2^256) already declined to fold (union window) —
    // assert it still reverts at -O2 as a companion of the A1 band fix.
    assert_eq!(
        run_at(&wrap("return type(uint256).max + 1;", "uint256"), 2),
        Err(0x11)
    );
}

// ============================================================================
// A2 — odd narrow widths (∉ {8,16,32,64,128}) get checked guards + truncation
// ============================================================================

#[test]
fn a2_uint96_checked_add_overflow_panics() {
    assert_eq!(
        run_at(
            &wrap(
                "uint96 x = type(uint96).max; x = x + 1; return x;",
                "uint96"
            ),
            2
        ),
        Err(0x11)
    );
}

#[test]
fn a2_uint224_checked_add_overflow_panics() {
    assert_eq!(
        run_at(
            &wrap(
                "uint224 x = type(uint224).max; x = x + 1; return x;",
                "uint224"
            ),
            2
        ),
        Err(0x11)
    );
}

#[test]
fn a2_uint112_checked_mul_overflow_panics() {
    assert_eq!(
        run_at(
            &wrap(
                "uint112 a = type(uint112).max; uint112 b = 3; a = a * b; return a;",
                "uint112"
            ),
            2
        ),
        Err(0x11)
    );
}

#[test]
fn a2_int160_checked_sub_underflow_panics() {
    assert_eq!(
        run_at(
            &wrap(
                "int160 x = type(int160).min; x = x - 1; return x;",
                "int160"
            ),
            2
        ),
        Err(0x11)
    );
}

#[test]
fn a2_uint96_shl_truncates_to_width() {
    // uint96(1) << 96 → 0 (width truncation), never a spurious panic.
    assert_eq!(
        run_at(&wrap("uint96 x = 1; x = x << 96; return x;", "uint96"), 2),
        Ok(0)
    );
}

#[test]
fn a2_uint224_shl_truncates_to_width() {
    // uint224(1) << 224 → 0.
    assert_eq!(
        run_at(
            &wrap("uint224 x = 1; x = x << 224; return x;", "uint224"),
            2
        ),
        Ok(0)
    );
}

#[test]
fn a2_uint96_unchecked_wraps_mod_2_96() {
    // unchecked overflow truncates mod 2^96: max + 1 == 0. Fits u128 for decode.
    assert_eq!(
        run_at(
            &wrap(
                "uint96 x = type(uint96).max; unchecked { x = x + 1; } return x;",
                "uint96"
            ),
            2
        ),
        Ok(0)
    );
}

#[test]
fn a2_uint96_in_range_add_ok() {
    assert_eq!(
        run_at(
            &wrap("uint96 x = 1000000000000; x = x + 5; return x;", "uint96"),
            2
        ),
        Ok(1_000_000_000_005)
    );
}

// ============================================================================
// A7 — dangling qualified constant reference is a compile error
// ============================================================================

#[test]
fn a7_valid_library_constant_compiles() {
    let src = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\
        library Lib { address public constant GOOD = 0x1234567890123456789012345678901234567890; }\n\
        contract C { address public g; function f() public { g = Lib.GOOD; } }";
    assert!(
        compile_contracts(src, false, 2).is_ok(),
        "a real library constant must still compile"
    );
}

#[test]
fn a7_dangling_library_constant_is_hard_error() {
    let src = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\
        library Lib { address public constant GOOD = 0x1234567890123456789012345678901234567890; }\n\
        contract C { address public g; function f() public { g = Lib.TYPO; } }";
    let err = compile_contracts(src, false, 2)
        .expect_err("a reference to a nonexistent library constant must fail compilation");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("TYPO") || msg.contains("unknown member"),
        "error must name the bad member; got: {msg}"
    );
}

#[test]
fn a7_dangling_constant_is_not_silently_zero() {
    // Belt-and-suspenders: confirm the pre-fix symptom (silent address(0) fold)
    // cannot be produced — a bad constant reference never yields an artifact.
    let src = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\
        library Lib { address public constant GOOD = 0x0000000000000000000000000000000000000001; }\n\
        contract C { address public g; function f() public { g = Lib.MISSING; } }";
    assert!(compile_contracts(src, false, 2).is_err());
}
