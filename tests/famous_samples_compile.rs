//! Hermetic compile test for the curated famous-contract sample corpus under
//! `third_party/famous-contracts/samples/`.
//!
//! Unlike `famous_contracts_compile::famous_corpus_vendor_only_compile_floor`
//! (which tolerates missing transitive dependencies and only asserts a low
//! floor), every file in `samples/` is deliberately SELF-CONTAINED — any small
//! imports were inlined — so each supported file must compile with NO include
//! path. This test therefore asserts **100%** of the *version-supported* subset:
//! a single failure on a supported version is a real compiler regression
//! against a real-world contract.
//!
//! The corpus spans DeFi / NFT / GameFi / zero-knowledge / infrastructure-DAO
//! plus `patterns/` (minimal shapes that pinned specific fixes), and Solidity
//! 0.5.x–0.8.x. Samples whose pragma is outside the compiler's supported range
//! (`>=0.8.19 <0.8.28`) are recorded as skipped rather than failures.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

fn compiler() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_neo-solc"))
}

fn samples_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("third_party/famous-contracts/samples")
}

fn iter_sol(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            iter_sol(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sol") {
            out.push(path);
        }
    }
}

/// Compile `source` hermetically (no include paths). Returns `Ok(())` on
/// success, or `Err(first-error-line)` on failure.
fn compile_hermetic(source: &Path) -> Result<(), String> {
    let outdir = std::env::temp_dir().join(format!(
        "neo-sample-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&outdir).map_err(|e| e.to_string())?;
    let output = Command::new(compiler())
        .arg(source)
        .arg("-O")
        .arg("0")
        .arg("-o")
        .arg(&outdir)
        .output()
        .map_err(|e| format!("failed to run compiler: {e}"));
    let _ = std::fs::remove_dir_all(&outdir);
    let output = output?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_error = stderr
        .lines()
        .find(|l| l.to_ascii_lowercase().contains("error"))
        .unwrap_or("<no error line>")
        .trim()
        .to_string();
    Err(first_error)
}

/// Returns `true` if the compiler error indicates the source's Solidity
/// version is outside the compiler's supported range. Such samples are
/// recorded as skipped, not failures, because the compiler explicitly does
/// not support them.
fn is_unsupported_solidity_version(err: &str) -> bool {
    err.to_ascii_lowercase()
        .contains("unsupported solidity version")
}

#[test]
fn famous_samples_compile_hermetically() {
    let root = samples_root();
    assert!(root.exists(), "sample corpus missing at {}", root.display());
    let compiler = compiler();
    assert!(
        compiler.exists(),
        "neo-solc binary not found at {}",
        compiler.display()
    );

    let mut files = Vec::new();
    iter_sol(&root, &mut files);
    files.sort();

    // Guard against the corpus silently emptying (e.g. a bad rebase).
    assert!(
        files.len() >= 25,
        "expected a substantial self-contained sample corpus, found only {}",
        files.len()
    );

    let mut failures = Vec::new();
    let mut skipped = Vec::new();
    for f in &files {
        if let Err(err) = compile_hermetic(f) {
            let rel = f.strip_prefix(&root).unwrap_or(f);
            if is_unsupported_solidity_version(&err) {
                skipped.push(format!("  {} :: {}", rel.display(), err));
            } else {
                failures.push(format!("  {} :: {}", rel.display(), err));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} supported famous samples failed hermetic compilation:\n{}\n\n{} samples skipped due to unsupported Solidity version",
        failures.len(),
        files.len() - skipped.len(),
        failures.join("\n"),
        skipped.len(),
    );

    eprintln!(
        "famous samples: {} compiled, {} skipped (unsupported version), {} total",
        files.len() - skipped.len() - failures.len(),
        skipped.len(),
        files.len()
    );
}
