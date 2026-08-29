//! Pathological corpus compile smoke test (wave-#34 follow-up).
//!
//! Wave #34 added 15 pathological / stress-test seed contracts under
//! `fuzz/corpus/fuzz_target_1/seed_pathological_*.sol` to drive deeper
//! cargo-fuzz coverage (stack-overflow guards, big-integer caps, recursive
//! type definitions, oversized literals, etc.). cargo-fuzz only exercises
//! these inside its libFuzzer rounds — a regression that re-introduces a
//! panic on one of these inputs might not be caught within a single
//! 2-minute fuzz run.
//!
//! This test compiles every `seed_pathological_*.sol` file in-process via
//! `compile_contracts` and asserts the compiler never PANICS. Both a
//! successful compile AND a graceful `Err` return are acceptable — what
//! we are guarding against is host-level faults (stack overflow, integer
//! underflow in helpers, unbounded recursion, etc.).
//!
//! Reference shape: `tests/fuzz_tests/examples_smoke_props.rs` (wave-#30
//! examples gate).

#![allow(unused_imports)]

use neo_devpack_solidity::cli::compile_contracts;
use std::fs::read_to_string;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

/// Directory holding the pathological seed corpus, relative to the
/// workspace root (cargo sets `CARGO_MANIFEST_DIR` to the workspace root
/// when running tests).
const CORPUS_DIR: &str = "fuzz/corpus/fuzz_target_1";

/// Filename prefix marking a wave-#34 pathological seed.
const SEED_PREFIX: &str = "seed_pathological_";

/// Collect every `seed_pathological_*.sol` file in `dir`, sorted by path
/// (deterministic test output across runs).
fn collect_pathological_seeds(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with(SEED_PREFIX) && name.ends_with(".sol") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// Outcome bucket for a single seed file.
enum SeedOutcome {
    /// Compiler returned `Ok(...)` — the contract compiled cleanly.
    Compiled,
    /// Compiler returned `Err(...)` — graceful, well-formed rejection.
    /// Pathological inputs are EXPECTED to land here in many cases.
    GracefulError,
    /// Compiler panicked (host-level fault, stack overflow, unwrap on
    /// `None`, etc.). The panic message is captured for reporting.
    /// THIS IS A REAL BUG.
    Panic(String),
}

/// Run `compile_contracts` on `src` inside `catch_unwind`. Returns the
/// observed outcome bucket. The unwind catches Rust panics; arithmetic
/// overflows in debug builds become panics, so this gate catches them too.
fn compile_with_panic_guard(src: &str) -> SeedOutcome {
    let result = catch_unwind(AssertUnwindSafe(|| compile_contracts(src, false, 2)));
    match result {
        Ok(Ok(_)) => SeedOutcome::Compiled,
        Ok(Err(_)) => SeedOutcome::GracefulError,
        Err(payload) => {
            // Best-effort extract of the panic message — try `&str` and
            // `String` payloads (the standard panic types).
            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "<non-string panic payload>".to_string()
            };
            SeedOutcome::Panic(msg)
        }
    }
}

/// Iterate every `seed_pathological_*.sol` under
/// `fuzz/corpus/fuzz_target_1/`, run the compiler on each inside a
/// panic-catching scope, and assert ZERO panics. Successful compile and
/// graceful `Err` returns are both acceptable — pathological seeds are
/// designed to stress-test the front-end's rejection paths, not to
/// produce shipped artifacts.
///
/// On any panic, fail the test with the file path + truncated panic
/// message — that is a real regression and should be filed immediately.
#[test]
fn pathological_corpus_compile_smoke() {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let corpus = workspace_root.join(CORPUS_DIR);

    let seeds = collect_pathological_seeds(&corpus);
    assert!(
        !seeds.is_empty(),
        "pathological_corpus_compile_smoke surveyed 0 seed files at {} — \
         the wave-#34 corpus is missing or the path moved.",
        corpus.display(),
    );

    let mut total = 0usize;
    let mut compiled = 0usize;
    let mut graceful = 0usize;
    let mut panics: Vec<(PathBuf, String)> = Vec::new();

    for path in seeds {
        total += 1;
        let src = match read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                // I/O error reading a checked-in seed file is itself a
                // regression — surface it as a panic-class failure.
                panics.push((path.clone(), format!("read_to_string: {}", e)));
                continue;
            }
        };
        match compile_with_panic_guard(&src) {
            SeedOutcome::Compiled => compiled += 1,
            SeedOutcome::GracefulError => graceful += 1,
            SeedOutcome::Panic(msg) => {
                let truncated: String = msg.chars().take(400).collect();
                panics.push((path.clone(), truncated));
            }
        }
    }

    eprintln!(
        "pathological_corpus_compile_smoke: surveyed {} seed(s), compiled {}, \
         graceful-error {}, panics {}",
        total,
        compiled,
        graceful,
        panics.len(),
    );

    if !panics.is_empty() {
        let summary = panics
            .iter()
            .map(|(p, e)| format!("  - {}: {}", p.display(), e))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "pathological_corpus_compile_smoke regression: {} pathological seed(s) \
             PANICKED the compiler. These seeds are designed to be REJECTED \
             gracefully — a panic means the front-end has an unguarded \
             stack-overflow / unwrap / arithmetic-underflow path. File as a \
             regression.\n{}",
            panics.len(),
            summary,
        );
    }
}
