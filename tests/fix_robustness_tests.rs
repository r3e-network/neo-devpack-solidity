//! Regression tests for compiler robustness fixes (agent key: robustness).
//!
//! 1. Unbounded constant folding: `1 << 18000000000000000000` (and friends)
//!    used to make the `-O2` Shl fold allocate an exabyte-scale BigInt and
//!    abort the whole compiler with SIGABRT; sub-abort shifts like
//!    `1 << 200000000` ballooned the .nef to ~25 MB. The fold now declines
//!    once the result would exceed `MAX_FOLDED_LITERAL_BITS` and leaves the
//!    op to runtime (the same code `-O0`/`-O1` emit).
//!
//! 2. Unguarded parse recursion: a ~1500-term nested expression used to
//!    overflow the main-thread stack inside `solang_parser::parse` (and,
//!    once parsing was guarded, inside the AST clone in
//!    `analyse_all_sources`), aborting the process instead of producing a
//!    diagnostic. Parsing now runs through
//!    `frontend::parse_solidity_guarded` on a large-stack worker thread,
//!    and the CLI driver itself runs on a large-stack thread.

use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

fn compiler_path() -> &'static str {
    env!("CARGO_BIN_EXE_neo-solc")
}

/// Compile `source` at the given `-O` level. The returned `TempDir` keeps
/// the output files alive for inspection until the end of the test.
fn compile_source(source: &str, opt: &str) -> (std::process::Output, tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let input = dir.path().join("input.sol");
    std::fs::write(&input, source).expect("write source");
    let prefix = dir.path().join("out");

    let output = Command::new(compiler_path())
        .arg(&input)
        .arg(opt)
        .arg("-o")
        .arg(&prefix)
        .output()
        .expect("run compiler");

    (output, dir, prefix)
}

fn assert_not_aborted(output: &std::process::Output, what: &str) {
    assert!(
        output.status.code().is_some(),
        "{what}: compiler was killed by a signal (stack overflow / OOM abort): {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

// --- Finding 1: unbounded constant folding -------------------------------

#[test]
fn huge_shift_literal_compiles_cleanly_at_o2() {
    // Used to abort with `memory allocation of 2250000000000000016 bytes
    // failed` (SIGABRT, exit 134) at the default -O2.
    let sources = [
        "contract C { function f() public pure returns (uint) { return 1 << 18000000000000000000; } }",
        "contract C { function f() public pure returns (uint) { return 1 << 0xffffffffffffffff; } }",
        "contract C { function f() public pure returns (uint) { return (2**1024) << 5000000000; } }",
    ];
    for source in sources {
        let (output, _dir, _) = compile_source(source, "-O2");
        assert_not_aborted(&output, "huge shift literal");
        assert!(
            output.status.success(),
            "huge shift literal must compile (runtime op), got: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn sub_abort_shift_no_longer_balloons_the_nef() {
    // `1 << 200000000` used to fold into a 25 MB push literal (.nef of
    // ~25,000,116 bytes at ~100 MB peak RSS). With the fold declined, the
    // emitted contract stays tiny.
    let source =
        "contract C { function f() public pure returns (uint) { return 1 << 200000000; } }";
    let (output, _dir, prefix) = compile_source(source, "-O2");
    assert!(
        output.status.success(),
        "sub-abort shift must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let nef = prefix.with_extension("nef");
    let size = std::fs::metadata(&nef)
        .unwrap_or_else(|e| panic!("missing {}: {e}", nef.display()))
        .len();
    assert!(
        size < 1024 * 1024,
        "oversized shift must not be folded into the bytecode (nef is {size} bytes)"
    );
}

#[test]
fn chained_huge_literal_multiplies_compile_cleanly_at_o2() {
    // Each `1 << 4000` folds (just under the ceiling); the multiplies of the
    // resulting >4000-bit literals must decline to fold instead of
    // ballooning compile-time memory / the emitted bytecode.
    let source = "contract C { function f() public pure returns (uint) { \
                  return (1 << 4000) * (1 << 4000) * (1 << 4000) * (1 << 4000); } }";
    let (output, _dir, prefix) = compile_source(source, "-O2");
    assert_not_aborted(&output, "chained huge multiplies");
    assert!(
        output.status.success(),
        "chained huge multiplies must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let nef = prefix.with_extension("nef");
    let size = std::fs::metadata(&nef).expect("nef metadata").len();
    assert!(
        size < 1024 * 1024,
        "huge-literal product must not be folded into the bytecode (nef is {size} bytes)"
    );
}

#[test]
fn legal_uint256_constants_still_compile_at_o2() {
    // The fold guard must not reject legal uint256 constant arithmetic.
    // (Direct fold-still-happens assertions live in the unit tests in
    // src/cli/ir_optimize/constant_folding.rs.)
    let source = "contract C { function f() public pure returns (uint) { \
                  return (1 << 255) + (3 * 5) + (115792089237316195423570985008687907853269984665640564039457584007913129639935 & 0xff); } }";
    let (output, _dir, _) = compile_source(source, "-O2");
    assert!(
        output.status.success(),
        "legal uint256 constants must keep compiling at -O2: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// --- Finding 2: unguarded parse recursion ---------------------------------

fn deeply_nested_source(terms: usize) -> String {
    let mut source =
        String::from("contract C { function f() public pure returns (uint) { return 1 ");
    for _ in 0..terms {
        source.push_str("+1");
    }
    source.push_str("; } }");
    source
}

#[test]
fn deeply_nested_expression_compiles_without_stack_overflow() {
    // ~1500 terms used to abort the whole process with
    // `thread 'main' has overflowed its stack` (exit 134) inside
    // `solang_parser::parse`. 3000 terms must now produce a clean result.
    let (output, _dir, _) = compile_source(&deeply_nested_source(3000), "-O2");
    assert_not_aborted(&output, "3000-term nested expression");
    assert!(
        output.status.success(),
        "3000-term nested expression must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn deeply_nested_expression_standard_json_no_abort() {
    // Same shape through the standard-json entry point
    // (src/cli/standard_json/standard_json_process/imports.rs).
    let dir = tempdir().expect("tempdir");
    let input_path = dir.path().join("input.json");
    let output_path = dir.path().join("out.json");

    let input = serde_json::json!({
        "language": "Solidity",
        "sources": {
            "C.sol": { "content": deeply_nested_source(3000) }
        },
        "settings": {}
    });
    std::fs::write(
        &input_path,
        serde_json::to_string(&input).expect("serialize"),
    )
    .expect("write input");

    let output = Command::new(compiler_path())
        .arg("--standard-json")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("run compiler");

    assert_not_aborted(&output, "standard-json 3000-term nested expression");
    assert!(
        output.status.success(),
        "standard-json deep nesting must produce a clean result: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&output_path).expect("read standard-json output"),
    )
    .expect("parse standard-json output");
    assert!(
        json["contracts"].is_object(),
        "expected compiled contracts in standard-json output: {json}"
    );
}

#[test]
fn parse_source_library_api_survives_deep_nesting() {
    // The guarded parse must protect library consumers too: this test runs
    // on a default-sized libtest thread, well below the depth the parser
    // needs for 3000 nesting levels.
    let source = deeply_nested_source(3000);
    let contracts = neo_devpack_solidity::frontend::parse_source(&source)
        .expect("deeply nested source must parse via the guarded helper");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].name, "C");
}
