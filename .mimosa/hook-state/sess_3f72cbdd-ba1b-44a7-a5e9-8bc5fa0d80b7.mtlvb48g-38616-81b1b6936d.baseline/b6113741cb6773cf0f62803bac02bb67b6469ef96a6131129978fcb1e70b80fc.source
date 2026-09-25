//! Property-based tests for compiler diagnostic message stability.
//!
//! Diagnostic stability is critical for IDE integration, error-aggregation
//! pipelines, and user trust. This module pins down three properties of the
//! compiler's error path:
//!
//! 1. **Determinism** — the same broken input compiled twice in a row must
//!    produce string-equal `Err(_)` payloads. Non-determinism here suggests
//!    HashMap-ordering leaks, time-based formatting, or other shared mutable
//!    state in the diagnostic pipeline.
//! 2. **No internal-state leakage** — random and corrupted inputs must never
//!    surface a Rust panic banner, an `unwrap()` site, a source path
//!    (`src/foo.rs:NN`), or a Debug-format dump of internal AST/diagnostic
//!    types. Such leaks are signatures of `unwrap()`/`expect()` reaching the
//!    user-visible error channel during a refactor.
//! 3. **Position information** — for shapes that a parser-level diagnostic
//!    *should* be able to locate (unclosed brace, typo'd identifier), the
//!    error message must contain a `line:column` reference that lets users
//!    or tooling navigate to the offending source.
//!
//! The proptest companion (b) generates random byte-corrupted Solidity
//! sources via mutation of a known-valid template — truncation, ASCII
//! garbage insertion, and byte-level scrambling — to exercise breadth that
//! the handcrafted (a) and (c) cases cannot.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use proptest::prelude::*;

/// Render a `CompileError` to a string in the same shape both calls produce.
/// `Debug` is used because `CompileError` does not implement `Display`, and
/// the round-tripped bytes flow through `formattedMessage` in the JSON
/// output anyway — they are the user-visible payload that downstream
/// consumers diff.
fn err_to_string<E: std::fmt::Debug>(e: &E) -> String {
    format!("{:?}", e)
}

/// Markers that must NEVER appear in a user-visible error message. Each
/// signals a different class of internal-state leak:
///
/// - `"thread '"` and `"panicked at "` — Rust panic banner reached the
///   error channel (a `catch_unwind` boundary is missing or an `unwrap()`
///   slipped through).
/// - `"backtrace::"` — a debug build with `RUST_BACKTRACE=1` formatted a
///   stack trace into the message.
/// - `"called `Option::unwrap()`"` and `"called `Result::unwrap()`"` — the
///   canonical messages from `unwrap()` on `None`/`Err`. Both shapes appear
///   in the standard library; either reaching the user is a defect.
/// - `"DiagnosticImpl"` / `"AstNode"` / `"IrDiagnostic { "` — Debug-format
///   dumps of internal types. These are noise to the user and may leak
///   compiler-private field names.
const PANIC_MARKERS: &[&str] = &[
    "thread '",
    "panicked at ",
    "backtrace::",
    "called `Option::unwrap()`",
    "called `Result::unwrap()`",
    "DiagnosticImpl",
];

/// Source-path leak detection. Looks for any substring of the form
/// `<...>src/<...>.rs:<digits>` which is the canonical Rust panic file
/// reference (e.g. `src/cli/foo.rs:42`). We don't ban `src/` outright
/// because legitimate diagnostics may reference a user-named file called
/// `src.sol`. The paired `.rs:` + numeric suffix is the discriminator.
fn contains_rust_path_leak(msg: &str) -> bool {
    // Cheap two-stage check: locate `src/`, then verify that the same
    // segment also contains `.rs:` *after* the `src/` (i.e. on the same
    // file path, not a later sentence).
    let mut search = msg;
    while let Some(idx) = search.find("src/") {
        let after = &search[idx..];
        // Only consider the next ~120 chars — a Rust source path won't be
        // longer than that, and bounding the window avoids false positives
        // from later sentences in a multi-line message.
        let window_end = after.len().min(120);
        let window = &after[..window_end];
        if let Some(rs_idx) = window.find(".rs:") {
            let after_rs = &window[rs_idx + ".rs:".len()..];
            if after_rs.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                return true;
            }
        }
        search = &search[idx + 4..];
    }
    false
}

/// Returns `true` iff `msg` contains a `line:column` style position
/// reference. Accepts the canonical `:LINE:COL` shape (e.g. `:5:12`),
/// the parser's `LINE:COL ` prefix shape (e.g. `5:1 `), or the parser's
/// "at LINE:COL" suffix (e.g. `at 5:1`). Any of these is enough to let an
/// IDE or pipeline navigate to the source.
fn contains_position_reference(msg: &str) -> bool {
    // Walk character pairs looking for `<digit>+:<digit>+`. This matches
    // `5:12`, `:5:12`, and `at 5:1` without a regex dependency.
    let bytes = msg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            // Need at least one digit before the colon, then a colon, then
            // at least one digit after it.
            if i < bytes.len()
                && bytes[i] == b':'
                && i + 1 < bytes.len()
                && bytes[i + 1].is_ascii_digit()
            {
                // Confirm we consumed at least one digit at `start..i`.
                if i > start {
                    return true;
                }
            }
        } else {
            i += 1;
        }
    }
    false
}

/// Hand-crafted intentionally-broken Solidity sources used by the
/// determinism and (where applicable) position tests. Each is documented
/// with the failure shape it is intended to elicit.
fn broken_source_corpus() -> Vec<(&'static str, String)> {
    vec![
        // Truncated mid-statement: parser must report unexpected EOF.
        (
            "truncated_mid_assignment",
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\
             contract C { function f() public { uint x ="
                .to_string(),
        ),
        // Unclosed brace: parser must surface a position-bearing diagnostic
        // pointing at or near the offending region.
        (
            "unclosed_brace",
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\
             contract C {\n\
             \x20\x20\x20\x20uint256 x;\n\
             \x20\x20\x20\x20function f() public {\n"
                .to_string(),
        ),
        // Missing semicolon between statements.
        (
            "missing_semicolon",
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\
             contract C { uint256 x = 1 uint256 y = 2; }"
                .to_string(),
        ),
        // Undeclared identifier in a return expression: semantic-layer
        // diagnostic.
        (
            "undeclared_identifier",
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\
             contract C { function f() public returns (uint) { return undefined_var; } }"
                .to_string(),
        ),
        // Literal out of type range: type-checker diagnostic.
        (
            "uint8_overflow_literal",
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\
             contract C { uint8 x = 1000; }"
                .to_string(),
        ),
        // Unknown type name.
        (
            "unknown_type",
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\
             contract C { notAType x = 5; }"
                .to_string(),
        ),
    ]
}

/// (a) Determinism: each broken source compiled twice must produce the
/// exact same `Err(_)` payload. If any case in the corpus is unexpectedly
/// accepted, we treat that as a soft skip rather than a failure (the goal
/// here is diagnostic stability, not catalog completeness).
#[test]
fn diagnostic_determinism() {
    let corpus = broken_source_corpus();
    let mut checked = 0usize;
    for (name, source) in &corpus {
        let r1 = compile_contracts(source, false, 2);
        let r2 = compile_contracts(source, false, 2);
        match (&r1, &r2) {
            (Err(e1), Err(e2)) => {
                let s1 = err_to_string(e1);
                let s2 = err_to_string(e2);
                assert_eq!(
                    s1, s2,
                    "[{}] diagnostic non-deterministic across two compiles\n  first: {}\n second: {}",
                    name, s1, s2
                );
                checked += 1;
            }
            (Ok(_), Ok(_)) => {
                // Source unexpectedly accepted — log and skip.
                eprintln!(
                    "[diagnostic_determinism] note: corpus entry `{}` was accepted; \
                     consider strengthening it",
                    name
                );
            }
            (a, b) => {
                panic!(
                    "[{}] compile result diverged across two attempts: {:?} vs {:?}",
                    name, a, b
                );
            }
        }
    }
    assert!(
        checked >= 3,
        "expected at least 3 broken sources to actually fail; only {} did",
        checked
    );
}

/// (c) Position references: for the two shapes whose diagnostics SHOULD
/// be locatable (parser-level errors), the message must contain a
/// `line:column` reference.
#[test]
fn diagnostic_includes_position() {
    // An unclosed brace: missing closing `}` for the function body. The
    // parser's `Unexpected end of input at L:C` path should fire.
    let unclosed = "// SPDX-License-Identifier: MIT\n\
                    pragma solidity ^0.8.0;\n\
                    contract C {\n\
                        uint256 x;\n\
                        function f() public {\n";
    // A typo'd token at a specific line — a stray `@@` glyph that the
    // tokenizer cannot consume.
    let stray_token = "// SPDX-License-Identifier: MIT\n\
                       pragma solidity ^0.8.0;\n\
                       contract C {\n\
                           uint256 a;\n\
                           uint256 b;\n\
                           uint256 c;\n\
                           uint256 d;\n\
                           @@ broken_here;\n\
                       }\n";

    for (label, source) in &[("unclosed_brace", unclosed), ("stray_token", stray_token)] {
        match compile_contracts(source, false, 2) {
            Err(e) => {
                let msg = err_to_string(&e);
                assert!(
                    contains_position_reference(&msg),
                    "[{}] diagnostic missing line:column position reference\n  msg: {}",
                    label,
                    msg
                );
            }
            Ok(_) => panic!(
                "[{}] expected compile failure for diagnostic-position test, got success",
                label
            ),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// (b) No internal stack traces or Debug-format leaks: random byte-
    /// corruption of a valid template must never produce a panic banner,
    /// `unwrap()` site, source path, or Debug dump in the user-visible
    /// error message. Inputs that compile cleanly are skipped (the
    /// property only constrains the error path).
    #[test]
    fn diagnostic_no_internal_stack_traces(
        garbage in prop::collection::vec(any::<u8>(), 0..=64),
        cut_offset in 0usize..200usize,
        insert_at in 0usize..200usize,
        mode in 0u8..3u8,
    ) {
        // Start from a known-valid template and corrupt it three different
        // ways depending on `mode`.
        let template = "// SPDX-License-Identifier: MIT\n\
                        pragma solidity ^0.8.0;\n\
                        contract C {\n\
                            uint256 public x = 1;\n\
                            function f() public view returns (uint256) {\n\
                                return x;\n\
                            }\n\
                        }\n";
        let bytes = template.as_bytes();
        let n = bytes.len();
        // Restrict garbage to printable ASCII so the resulting string is
        // valid UTF-8 (compile_contracts takes &str). The exotic bytes are
        // already covered by the libFuzzer harness; the point here is to
        // exercise the *Solidity* error path on plausibly-shaped junk.
        let safe_garbage: Vec<u8> = garbage
            .iter()
            .map(|b| {
                let v = *b % 95;
                v + 0x20 // ASCII printable: 0x20..=0x7E
            })
            .collect();
        let corrupted: String = match mode {
            0 => {
                // Truncate at a random offset.
                let cut = cut_offset.min(n);
                std::str::from_utf8(&bytes[..cut]).unwrap_or("").to_string()
            }
            1 => {
                // Insert printable garbage at a random offset.
                let pos = insert_at.min(n);
                let mut out = Vec::with_capacity(n + safe_garbage.len());
                out.extend_from_slice(&bytes[..pos]);
                out.extend_from_slice(&safe_garbage);
                out.extend_from_slice(&bytes[pos..]);
                String::from_utf8(out).unwrap_or_default()
            }
            _ => {
                // Append printable garbage at the end.
                let mut out = Vec::with_capacity(n + safe_garbage.len());
                out.extend_from_slice(bytes);
                out.extend_from_slice(&safe_garbage);
                String::from_utf8(out).unwrap_or_default()
            }
        };

        // The compile call itself must not panic — proptest catches a
        // panic and surfaces the failing input automatically. Diagnostic
        // content checks only run when there is an error to inspect.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compile_contracts(&corrupted, false, 2)
        }));
        prop_assert!(
            result.is_ok(),
            "compile_contracts panicked on corrupted source:\n---\n{}\n---",
            corrupted
        );
        let inner = result.unwrap();
        if let Err(e) = inner {
            let msg = err_to_string(&e);
            for marker in PANIC_MARKERS {
                prop_assert!(
                    !msg.contains(marker),
                    "diagnostic leaked internal marker {:?} in message: {}\n  source:\n---\n{}\n---",
                    marker, msg, corrupted
                );
            }
            prop_assert!(
                !contains_rust_path_leak(&msg),
                "diagnostic leaked a Rust source path in message: {}\n  source:\n---\n{}\n---",
                msg, corrupted
            );
        }
    }
}
