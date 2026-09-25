//! Regression guards for the v0.31 audit/optimize/refactor pass. Runtime-side
//! tests drive the in-tree NeoVM simulator with hand-assembled bytecode; each
//! test exercises a defect found by the fresh audit (or the v0.30.3 report's
//! deferred follow-ups) and would fail on the pre-fix code.
//!
//! Compiler-side (IR/CLI/DevPack) guards for the same pass live further down
//! this file; C# runtime fixes are covered by `Neo.Sol.Runtime.Tests`.

#![allow(clippy::uninlined_format_args, non_snake_case)]

use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

// ---- opcode bytes (src/opcode/mod.rs) ----
const PUSHINT64: u8 = 0x03;
const PUSHDATA1: u8 = 0x0C;
const PUSHM1: u8 = 0x0F;
const PUSH0: u8 = 0x10;
const PUSH1: u8 = 0x11;
const PUSH2: u8 = 0x12;
const PUSH5: u8 = 0x15;
const PUSH7: u8 = 0x17;
const PUSH12: u8 = 0x1C;
const RET: u8 = 0x40;
const DUP: u8 = 0x4A;
const OVER: u8 = 0x4B;
const ADD: u8 = 0x9E;
const POW: u8 = 0xA3;
const MODMUL: u8 = 0xA5;
const MODPOW: u8 = 0xA6;
const NEWBUFFER: u8 = 0x88;
const NUMEQUAL: u8 = 0xB3;
const NUMNOTEQUAL: u8 = 0xB4;
const LE: u8 = 0xB6;
const NEWARRAY0: u8 = 0xC2;
const NEWARRAY: u8 = 0xC3;
const NEWARRAY_T: u8 = 0xC4;
const PICKITEM: u8 = 0xCE;
const APPEND: u8 = 0xCF;
const SETITEM: u8 = 0xD0;

fn run(code: &[u8]) -> Result<Vec<u8>, String> {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(code, &[]).expect("init");
    loop {
        match ctx.step() {
            Ok(state) if state.halted => return Ok(ctx.return_data().to_vec()),
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
}

fn pushdata(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![PUSHDATA1, bytes.len() as u8];
    out.extend_from_slice(bytes);
    out
}

fn int_le(v: i64) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

// ============================================================================
// NEWARRAY_T (0xC4) — must consume its 1-byte type operand and create typed
// defaults. Pre-fix the operand was left in the stream and executed as the
// NEXT opcode (the spec-typical 0x40 operand silently halted the script).
// ============================================================================

#[test]
fn newarray_t_consumes_type_operand_and_creates_integer_defaults() {
    // PUSH2; NEWARRAY_T Integer(0x21); PUSH1; PICKITEM; RET
    let code = [PUSH2, NEWARRAY_T, 0x21, PUSH1, PICKITEM, RET];
    assert_eq!(
        run(&code),
        Ok(int_le(0)),
        "NEWARRAY_T must consume its type operand and default elements to Integer(0)"
    );
}

#[test]
fn newarray_t_boolean_defaults_to_false() {
    // PUSH2; NEWARRAY_T Boolean(0x20); PUSH1; PICKITEM; RET
    let code = [PUSH2, NEWARRAY_T, 0x20, PUSH1, PICKITEM, RET];
    assert_eq!(
        run(&code),
        Ok(vec![0]),
        "NEWARRAY_T Boolean defaults must be false"
    );
}

#[test]
fn newarray_t_execution_continues_past_operand() {
    // Pre-fix, an operand of 0x40 (RET) executed immediately and the script
    // halted with EMPTY return data before the array was ever used.
    // PUSH1; NEWARRAY_T Integer(0x21); RET — array of 1 default is the result.
    let code = [PUSH1, NEWARRAY_T, 0x21, RET];
    let out = run(&code).expect("script must halt normally, not on the operand");
    // The single-element array serializes via JSON; just assert non-empty.
    assert!(
        !out.is_empty(),
        "RET must see the created array, not an operand-halved script"
    );
}

// ============================================================================
// NUMEQUAL / NUMNOTEQUAL (0xB3/0xB4) — numeric comparison per NeoVM: coerce
// both operands to BigInteger. Pre-fix this was structural equality, so
// Integer(5) != ByteString[0x05] and [0x05] != [0x05, 0x00].
// ============================================================================

#[test]
fn numequal_integer_vs_bytestring_is_numeric() {
    // PUSH5; PUSHDATA1 [0x05]; NUMEQUAL; RET
    let mut code = vec![PUSH5];
    code.extend(pushdata(&[0x05]));
    code.extend_from_slice(&[NUMEQUAL, RET]);
    assert_eq!(
        run(&code),
        Ok(vec![1]),
        "NUMEQUAL(Integer(5), ByteString[0x05]) must be true (numeric coercion)"
    );
}

#[test]
fn numequal_short_bytestrings_compare_numerically() {
    // [0x05] vs [0x05, 0x00] — numerically both are 5.
    let mut code = pushdata(&[0x05]);
    code.extend(pushdata(&[0x05, 0x00]));
    code.extend_from_slice(&[NUMEQUAL, RET]);
    assert_eq!(
        run(&code),
        Ok(vec![1]),
        "NUMEQUAL([0x05], [0x05,0x00]) must be true (BigInt numeric equality)"
    );
}

#[test]
fn numnotequal_negative_bytestring_matches_integer() {
    // PUSHM1 (-1); PUSHDATA1 [0xFF] (little-endian two's complement -1);
    // NUMNOTEQUAL; RET → false.
    let mut code = vec![PUSHM1];
    code.extend(pushdata(&[0xFF]));
    code.extend_from_slice(&[NUMNOTEQUAL, RET]);
    assert_eq!(
        run(&code),
        Ok(vec![0]),
        "NUMNOTEQUAL(-1, [0xFF]) must be false: [0xFF] decodes to -1"
    );
}

#[test]
fn numequal_on_arrays_faults_like_real_neovm() {
    // PUSH2; NEWARRAY; DUP; NUMEQUAL — NeoVM faults on non-numeric operands.
    let code = [PUSH2, NEWARRAY, DUP, NUMEQUAL];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    let mut faulted = false;
    for _ in 0..16 {
        match ctx.step() {
            Ok(state) if state.halted => break,
            Ok(_) => {}
            Err(_) => {
                faulted = true;
                break;
            }
        }
    }
    assert!(faulted, "NUMEQUAL on Array operands must FAULT");
}

#[test]
fn le_integer_bytestring_uses_numeric_equality() {
    // PUSH5; PUSHDATA1 [0x05]; LE; RET → 5 <= 5 is true. Pre-fix the equality
    // half was structural so LE returned false for equal values of mixed
    // numeric types.
    let mut code = vec![PUSH5];
    code.extend(pushdata(&[0x05]));
    code.extend_from_slice(&[LE, RET]);
    assert_eq!(run(&code), Ok(vec![1]), "LE(5, [0x05]) must be true");
}

// ============================================================================
// SETITEM (0xD0) — ByteStrings are immutable on a real node; only Buffer
// (0x30) accepts in-place byte writes.
// ============================================================================

#[test]
fn setitem_on_bytestring_faults() {
    // PUSHDATA1 [AA BB]; DUP; PUSH0 (index); PUSH12 (value); SETITEM
    let mut code = pushdata(&[0xAA, 0xBB]);
    code.extend_from_slice(&[DUP, PUSH0, PUSH12, SETITEM]);
    let err = run(&code).expect_err("SETITEM on immutable ByteString must FAULT");
    assert!(
        err.contains("immutable"),
        "fault should name the immutable ByteString target, got: {err}"
    );
}

#[test]
fn setitem_on_buffer_still_roundtrips() {
    // PUSH2; NEWBUFFER; DUP; PUSH0; PUSH7; SETITEM; PUSH0; PICKITEM; RET
    let code = [
        PUSH2, NEWBUFFER, DUP, PUSH0, PUSH7, SETITEM, PUSH0, PICKITEM, RET,
    ];
    assert_eq!(
        run(&code),
        Ok(vec![7]),
        "compiler-emitted NEWBUFFER + SETITEM writes must keep working (PICKITEM yields a 1-byte ByteString)"
    );
}

// ============================================================================
// Circular-reference rejection — `NEWARRAY0; DUP; APPEND` and friends must
// FAULT like NeoVM's reference counter instead of building a self-referential
// collection that hangs every recursive walker (serialize/EQUAL).
// ============================================================================

#[test]
fn append_self_referential_array_faults() {
    let code = [NEWARRAY0, DUP, APPEND];
    let err = run(&code).expect_err("APPEND of an array into itself must FAULT");
    assert!(
        err.contains("circular reference"),
        "fault should name the circular reference, got: {err}"
    );
}

#[test]
fn setitem_self_referential_array_faults() {
    // NEWARRAY0; DUP; PUSH0 (index); OVER (value = array); SETITEM
    let code = [NEWARRAY0, DUP, PUSH0, OVER, SETITEM];
    let err = run(&code).expect_err("SETITEM of an array into itself must FAULT");
    assert!(err.contains("circular reference"), "got: {err}");
}

#[test]
fn setitem_self_referential_map_faults() {
    // NEWMAP; DUP; PUSHDATA1 [0x01] (key); OVER (value = map); SETITEM
    let mut code = vec![0xC8, DUP];
    code.extend(pushdata(&[0x01]));
    code.extend_from_slice(&[OVER, SETITEM]);
    let err = run(&code).expect_err("SETITEM of a map into itself must FAULT");
    assert!(err.contains("circular reference"), "got: {err}");
}

// ============================================================================
// Arithmetic overflow/edge fidelity.
// ============================================================================

#[test]
fn modmul_modulus_i64_min_does_not_panic() {
    // a = i64::MIN, b = 1, modulus = i64::MIN → product % |m| = 0.
    // Pre-fix `m.abs()` on i64::MIN panicked (debug) / produced a negative
    // modulus (release).
    let mut code = vec![PUSHINT64];
    code.extend_from_slice(&int_le(i64::MIN));
    code.push(PUSH1);
    code.push(PUSHINT64);
    code.extend_from_slice(&int_le(i64::MIN));
    code.extend_from_slice(&[MODMUL, RET]);
    assert_eq!(run(&code), Ok(int_le(0)));
}

#[test]
fn modpow_modulus_i64_min_does_not_panic() {
    // base = 2, exponent = 1, modulus = i64::MIN → 2 % |m| = 2.
    let mut code = vec![PUSH2, PUSH1, PUSHINT64];
    code.extend_from_slice(&int_le(i64::MIN));
    code.extend_from_slice(&[MODPOW, RET]);
    assert_eq!(run(&code), Ok(int_le(2)));
}

#[test]
fn pow_exponent_above_u32_faults_instead_of_wrapping() {
    // base = 2, exponent = 2^32 + 1. Pre-fix `e as u32` wrapped to 1 and the
    // script "succeeded" with 2 instead of faulting.
    let mut code = vec![PUSH2, PUSHINT64];
    code.extend_from_slice(&int_le(4_294_967_297));
    code.extend_from_slice(&[POW]);
    let err = run(&code).expect_err("POW exponent >= 2^32 must FAULT");
    assert!(err.contains("exponent too large"), "got: {err}");
}

#[test]
fn add_sign_extends_short_negative_bytestring() {
    // [0xFF] (little-endian two's complement -1) + 1 = 0. Pre-fix the narrow
    // coercion zero-padded, yielding 255 + 1 = 256.
    let mut code = pushdata(&[0xFF]);
    code.extend_from_slice(&[PUSH1, ADD, RET]);
    assert_eq!(
        run(&code),
        Ok(int_le(0)),
        "short negative ByteStrings must sign-extend in arithmetic"
    );
}

// ============================================================================
// Memory model — reads entirely past the allocation high-water mark must
// zero-fill (EVM untouched-memory semantics), not fault.
// ============================================================================

#[test]
fn read_memory_past_high_water_zero_fills() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    let word = ctx
        .read_memory(1024, 32)
        .expect("read past high-water mark");
    assert!(
        word.iter().all(|&b| b == 0),
        "untouched memory must read as zeros"
    );
}

// ============================================================================
// Compiler-side guards (IR / CLI / DevPack) for the same audit pass.
// ============================================================================

use neo_devpack_solidity::cli::compile_contracts;

fn wrap(body: &str) -> String {
    format!(
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\ncontract C {{ function run() public pure returns (uint256) {{ {body} }} }}"
    )
}

/// Negative `**` exponents used to compile to a square-and-multiply loop whose
/// arithmetic SHR never brings a negative exponent to zero — a non-terminating
/// on-chain loop. They must now be rejected at compile time (solc rejects them
/// too).
#[test]
fn pow_signed_exponent_is_a_compile_error() {
    let src = wrap("uint256 b = 2; int256 e = -1; return b ** e;");
    let result = compile_contracts(&src, false, 2);
    assert!(
        result.is_err(),
        "signed ** exponent must fail compilation (pre-fix it emitted an infinite runtime loop)"
    );
    let errs = format!("{:?}", result.unwrap_err());
    assert!(
        errs.contains("exponent"),
        "error should mention the signed exponent, got: {errs}"
    );
}

/// Cyclic constant initializers referenced in selector position used to
/// recurse forever in `resolve_selector_method_name` and abort the compiler.
#[test]
fn cyclic_constant_initializers_do_not_abort_the_compiler() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    bytes4 constant A = B;
    bytes4 constant B = A;
    function pick() public pure returns (bytes4) {
        return bytes4(uint32(A));
    }
}
"#;
    // Must terminate with a diagnostic (or success), never a stack overflow.
    let _ = compile_contracts(src, false, 2);
}

/// An indexed param of CONTRACT type must lower to a static left-padded
/// `address` topic slot (matching topic0's `E(address)`), not
/// `keccak256(abi.encode(...))`. Pre-fix the per-param canonical type was
/// derived from the declared string ("MyToken" → user-defined → dynamic).
#[test]
fn indexed_contract_typed_event_param_uses_static_address_topic() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract Token {}
contract C {
    event Made(Token indexed t, uint256 x);
    function emit_it() public {
        emit Made(Token(address(0x1000)), 1);
    }
}
"#;
    let arts = compile_contracts(src, false, 2).expect("compile must succeed");
    // The source declares two contracts (Token + C); pick C's artifact.
    let arts: Vec<_> = arts
        .into_iter()
        .filter(|a| a.metadata.name == "C")
        .collect();
    let code = &arts[0].bytecode;
    // The indexed contract-type arg must take the SAME static path as a plain
    // indexed `address`: a 20-byte left-padded address push (0x1000 as BE).
    let addr20: [u8; 20] = [
        0x00, 0x10, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00,
    ];
    assert!(
        code.windows(20).any(|w| w == addr20),
        "indexed contract-type topic must push the raw address (static slot)"
    );
    // topic0 must hash the RESOLVED signature `Made(address,uint256)`.
    use sha3::{Digest, Keccak256};
    let mut h = Keccak256::new();
    h.update(b"Made(address,uint256)");
    let topic0: Vec<u8> = h.finalize().to_vec();
    assert!(
        code.windows(32).any(|w| w == topic0.as_slice()),
        "topic0 must be keccak256(Made(address,uint256))"
    );
    // And the topic must NOT be hashed at runtime: pre-fix the declared-string
    // canonicalizer classified the param as user-defined/dynamic and emitted a
    // CryptoLib.keccak256 native call for the topic value.
    assert!(
        !code.windows(9).any(|w| w == b"keccak256"),
        "indexed contract-type topic must not go through a runtime keccak256 call"
    );
}

// ============================================================================
// CLI behavior guards — exit codes, batch tolerance, strict -O validation.
// ============================================================================

use std::process::Command;

fn compiler_path() -> &'static str {
    env!("CARGO_BIN_EXE_neo-solc")
}

fn write_temp(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).expect("write temp file");
    path
}

/// `--standard-json` must exit NON-ZERO when the emitted JSON carries
/// error-severity entries (solc parity). Pre-fix it exited 0, so CI builds
/// passed with zero artifacts.
#[test]
fn standard_json_exits_nonzero_on_compile_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_path = write_temp(
        dir.path(),
        "input.json",
        &serde_json::json!({
            "language": "Solidity",
            "sources": {
                "Bad.sol": { "content": "contract Bad { function f( unknown syntax" }
            },
            "settings": {}
        })
        .to_string(),
    );
    let output_path = dir.path().join("out.json");

    let out = Command::new(compiler_path())
        .arg("--standard-json")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("run compiler");

    assert!(
        !out.status.success(),
        "standard-json must exit non-zero when errors are reported (got exit 0)"
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).expect("read output json"))
            .expect("parse output json");
    assert!(
        json["errors"]
            .as_array()
            .expect("errors")
            .iter()
            .any(|e| e["severity"] == "error"),
        "output json must still carry the error entries"
    );
}

/// `--standard-json` success case must still exit 0 (guard against overreach).
#[test]
fn standard_json_exits_zero_on_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_path = write_temp(
        dir.path(),
        "input.json",
        &serde_json::json!({
            "language": "Solidity",
            "sources": {
                "Ok.sol": { "content": "contract Ok { function f() public pure returns (uint256) { return 1; } }" }
            },
            "settings": {}
        })
        .to_string(),
    );
    let out = Command::new(compiler_path())
        .arg("--standard-json")
        .arg("--input")
        .arg(&input_path)
        .output()
        .expect("run compiler");
    assert!(out.status.success(), "clean compile must exit 0");
}

/// A batch (`neo-solc a.sol b.sol`) must still compile the files AFTER a
/// failing one and exit non-zero at the end. Pre-fix the first failure
/// aborted the run.
#[test]
fn batch_mode_compiles_remaining_files_after_a_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let good = write_temp(
        dir.path(),
        "Good.sol",
        "contract Good { function f() public pure returns (uint256) { return 1; } }",
    );
    let bad = write_temp(dir.path(), "Bad.sol", "contract Bad { this is not solidity");
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).expect("mkdir");

    let out = Command::new(compiler_path())
        .arg(good)
        .arg(&bad)
        .arg("-o")
        .arg(&out_dir)
        .output()
        .expect("run compiler");

    assert!(
        !out.status.success(),
        "batch with one failing file must exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Bad.sol"),
        "failure summary must name the failing file, got: {stderr}"
    );
    let good_nef = std::fs::read_dir(&out_dir)
        .expect("list out dir")
        .filter_map(Result::ok)
        .any(|e| e.file_name().to_string_lossy().contains("Good"));
    assert!(
        good_nef,
        "the file after the failing one must still have been compiled+emitted"
    );
}

/// Invalid `-O` levels must be rejected by the CLI (clap value parser), not
/// silently coerced to the default.
#[test]
fn invalid_optimizer_level_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_temp(
        dir.path(),
        "C.sol",
        "contract C { function f() public pure returns (uint256) { return 1; } }",
    );
    let out = Command::new(compiler_path())
        .arg(&src)
        .arg("-O")
        .arg("9")
        .output()
        .expect("run compiler");
    assert!(
        !out.status.success(),
        "-O 9 must be rejected (pre-fix it silently fell back to -O2)"
    );
    let out4 = Command::new(compiler_path())
        .arg(&src)
        .arg("-O")
        .arg("4")
        .output()
        .expect("run compiler");
    assert!(!out4.status.success(), "-O 4 must also be rejected");
}

/// Human-readable error output must carry the stable NSH code (pre-fix only
/// the message was printed, so users couldn't feed codes to --Wno/--Werror).
#[test]
fn human_error_output_includes_nsh_code() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Parseable but semantically invalid, so the failure surfaces through the
    // compile stage (`CompileError::Semantic` → emit_error), not import
    // resolution.
    let src = write_temp(
        dir.path(),
        "C.sol",
        "library L { function f() internal pure returns (uint256) { return 1; } }\ncontract C { function g() public pure returns (uint256) { return L.TYPO; } }",
    );
    let out = Command::new(compiler_path())
        .arg(&src)
        .output()
        .expect("run compiler");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("error[NSH-"),
        "human error output must embed the NSH code, got: {stderr}"
    );
}

// ============================================================================
// DevPack guards — the Solidity library surface shipped with the compiler.
// ============================================================================

use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Compile `source` against the repo-root devpack (`import "devpack/..."`)
/// and succeed; panics with the compiler stderr otherwise.
fn compile_with_devpack(scope: &str, source: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let contract_path = write_temp(dir.path(), "Probe.sol", source);
    let out = Command::new(compiler_path())
        .arg(&contract_path)
        .arg("-I")
        .arg(manifest_dir())
        .output()
        .expect("run compiler");
    assert!(
        out.status.success(),
        "{scope}: devpack probe must compile; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `NativeRole.sol` declared `address constant ROLE_CONTRACT =
/// NativeContracts.ROLE_CONTRACT;` — a constant that never existed — so ANY
/// contract importing the library failed compilation. It must compile and
/// expose the role-management entrypoints.
#[test]
fn devpack_native_role_library_compiles() {
    compile_with_devpack(
        "native-role",
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
import "devpack/contracts/native/NativeRole.sol";
contract RoleProbe {
    function designate(bytes1 role, bytes[] memory pubkeys) public {
        NativeRole.designateAsRole(role, pubkeys);
    }
    function designated(bytes1 role) public view returns (bytes[] memory) {
        return NativeRole.getDesignatedByRole(role, 0);
    }
}
"#,
    );
}

/// The split syscall libraries must be surface-equivalent to the monolith:
/// the six helpers that existed only in monolith `Syscalls.sol` now also
/// compile from the split modules.
#[test]
fn devpack_split_syscalls_surface_matches_monolith() {
    compile_with_devpack(
        "split-syscalls",
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
import "devpack/contracts/syscalls/SyscallsRuntime.sol";
import "devpack/contracts/syscalls/SyscallsContract.sol";
contract SplitProbe {
    function random() public view returns (uint256) {
        return SyscallsRuntime.getCurrentRandom();
    }
    function toAddress(bytes20 h) public pure returns (address) {
        return SyscallsRuntime.scriptHashToAddress(h);
    }
    function toScriptHash(address a) public pure returns (bytes20) {
        return SyscallsRuntime.addressToScriptHash(a);
    }
    function validAddress(address a) public pure returns (bool) {
        return SyscallsRuntime.isValidAddress(a);
    }
    function script(address c) public view returns (bytes memory) {
        return SyscallsContract.getContractScript(c);
    }
    function exists(address c) public view returns (bool) {
        return SyscallsContract.contractExists(c);
    }
}
"#,
    );
}

/// NEP-22 declares `update(bytes nefFile, bytes manifest, bytes data)` — the
/// manifest used to be a `string`, advertising a nonconforming `String`
/// parameter in the manifest ABI.
#[test]
fn devpack_nep22_manifest_param_is_bytes() {
    compile_with_devpack(
        "nep22",
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
import "devpack/standards/NEP22.sol";
contract UpgradeProbe is NEP22Upgradeable {
    function _update(bytes calldata, bytes calldata, bytes calldata) internal override {}
}
"#,
    );
}

/// The example NFT marketplace must compile with the `_updateFloorPrice`
/// underflow clamp (a fresh long-duration listing used to make `buyToken`
/// revert with Panic(0x11)).
#[test]
fn devpack_complete_nep11_example_still_compiles() {
    let example = manifest_dir().join("devpack/examples/CompleteNEP11NFT.sol");
    let out = Command::new(compiler_path())
        .arg(&example)
        .arg("-I")
        .arg(manifest_dir())
        .output()
        .expect("run compiler");
    assert!(
        out.status.success(),
        "CompleteNEP11NFT example must compile; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// An indexed struct param whose declared (lowercase) struct name made the
/// string-derived canonicalizer classify it as STATIC used to emit a raw
/// un-hashed encoding blob as the topic; it must be keccak256(abi.encode(s)).
#[test]
fn indexed_struct_event_param_is_hashed() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    struct point { uint256 a; }
    event Made(point indexed p);
    function emit_it() public {
        emit Made(point(1));
    }
}
"#;
    let arts = compile_contracts(src, false, 2).expect("compile must succeed");
    // topic0 = keccak256("Made((uint256))") must appear in the bytecode.
    use sha3::{Digest, Keccak256};
    let mut h = Keccak256::new();
    h.update(b"Made((uint256))");
    let topic0: Vec<u8> = h.finalize().to_vec();
    let code = &arts[0].bytecode;
    assert!(
        code.windows(32).any(|w| w == topic0.as_slice()),
        "topic0 must hash the tuple-expanded signature"
    );
    // The indexed struct topic itself must be a Keccak256 application, not a
    // raw encoding: at least one keccak256 syscall must follow the slot build.
    // (Byte-level check: the value 1 padded to 32 bytes must NOT appear as a
    // pushed literal right before the notify, which is the pre-fix blob.)
    let mut raw_slot = vec![0u8; 31];
    raw_slot.push(1);
    assert!(
        !code.windows(32).any(|w| w == raw_slot.as_slice()),
        "indexed struct topic must be hashed, not the raw abi.encode blob"
    );
}
