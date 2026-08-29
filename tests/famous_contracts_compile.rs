// clippy's `doc_lazy_continuation` and `doc_overindented_list_items` lints
// disagree on the correct indentation for list-item continuations in module
// docs (the former wants more, the latter wants less). Suppress both for
// this file — the docs are readable as-is and the lint signal is noise.
#![allow(clippy::doc_lazy_continuation, clippy::doc_overindented_list_items)]

//! Famous-contracts compilation suite.
//!
//! Verifies that the curated subset of upstream Solidity sources in
//! `third_party/famous-contracts/` continues to compile end-to-end through
//! `neo-solc`. The harness has two responsibilities:
//!
//! 1. Catch regressions in the import resolver / pragma combinator / library
//!    + sibling merge that previously turned 7 of 92 OZ-ecosystem contracts
//!    into "passing". As fixes land, the floor moves up and any regression
//!    is loud.
//! 2. Make it cheap to bisect compatibility losses across compiler versions
//!    by running the same sweep with the binary built for this revision.
//!
//! The Solidity sources in the vendored tree don't always come with their
//! transitive dependencies (OpenZeppelin pulls in `IERC20.sol`, Uniswap V4
//! pulls in `@uniswap/v4-core/...`, etc.). To keep the test hermetic without
//! requiring npm installs in CI, every assertion is "this many or more
//! contracts must compile" — we record a floor, not a tight equality. The
//! floor is raised whenever a real fix lands.
//!
//! Run with: `cargo test --release --test famous_contracts_compile`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn get_compiler_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_neo-solc"))
}

fn vendor_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("third_party/famous-contracts/sources")
}

fn unique_output_dir(scope: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = OUTPUT_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("neo-solc-{scope}-{nanos}-{seq}"))
}

/// Try to compile a single Solidity source through `neo-solc`. Returns
/// `Ok(true)` on success, `Ok(false)` if compilation failed for any reason,
/// or `Err` if the compiler binary couldn't be located. Errors during
/// compilation are intentionally swallowed (returned as `Ok(false)`) so the
/// caller can compute aggregate stats.
fn try_compile(source: &Path, include_paths: &[PathBuf]) -> Result<bool, String> {
    let compiler = get_compiler_path();
    if !compiler.exists() {
        return Err(format!("compiler not found at {}", compiler.display()));
    }
    let outdir = unique_output_dir("famous");
    std::fs::create_dir_all(&outdir)
        .map_err(|e| format!("failed to create {}: {e}", outdir.display()))?;
    let mut cmd = Command::new(&compiler);
    cmd.arg(source).arg("-O").arg("0").arg("-o").arg(&outdir);
    for inc in include_paths {
        cmd.arg("-I").arg(inc);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("failed to run compiler: {e}"))?;
    // Tidy up regardless of outcome; failing to clean is non-fatal because the
    // tempdir is in /tmp and the OS will eventually evict it.
    let _ = std::fs::remove_dir_all(&outdir);
    Ok(output.status.success())
}

fn iter_sol_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "sol") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

/// Look for an `@openzeppelin/contracts` install reachable from the project
/// root, climbing `node_modules` directories. Returns `None` when no install
/// is available — the test then runs in vendor-only mode.
fn find_oz_install() -> Option<PathBuf> {
    let env = std::env::var("NEO_SOL_OZ_INSTALL_DIR")
        .ok()
        .map(PathBuf::from);
    if let Some(p) = env {
        if p.join("@openzeppelin/contracts").is_dir() {
            return Some(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cursor: Option<&Path> = Some(manifest.as_path());
    while let Some(dir) = cursor {
        let candidate = dir.join("node_modules");
        if candidate.join("@openzeppelin/contracts").is_dir() {
            return Some(candidate);
        }
        cursor = dir.parent();
    }
    None
}

#[test]
fn famous_corpus_vendor_only_compile_floor() {
    // Hermetic floor: with only the vendored sources (no npm install), at
    // least this many contracts must continue to compile end-to-end. The
    // vendored tree is intentionally minimal (only the "leaf" sources, not
    // their transitive deps), so the realistic floor here is small — but any
    // drop from it signals a *real* regression in the compiler's ability to
    // process self-contained contracts.
    let vendor = vendor_root();
    if !vendor.exists() {
        eprintln!("skip: vendor tree missing at {}", vendor.display());
        return;
    }
    let include_paths = vec![
        vendor.clone(),
        vendor.join("@openzeppelin/contracts"),
        vendor.join("@openzeppelin/contracts-upgradeable"),
        vendor.join("@uniswap/v2-core/contracts"),
        vendor.join("@uniswap/v4-core/src"),
        vendor.join("@uniswap/v4-periphery/src"),
        vendor.join("@safe-global/safe-contracts/contracts"),
    ];
    let mut total = 0usize;
    let mut passed = 0usize;
    for sol in iter_sol_files(&vendor) {
        total += 1;
        if try_compile(&sol, &include_paths).unwrap_or(false) {
            passed += 1;
        }
    }
    const FLOOR: usize = 5;
    assert!(
        passed >= FLOOR,
        "famous-corpus vendor-only compile-pass count regressed: got {passed}/{total}, floor is {FLOOR}."
    );
    eprintln!("famous corpus (vendor-only): {passed}/{total} compiled");
}

#[test]
fn oz_corpus_with_install_compile_floor() {
    // Higher-fidelity floor: when an `@openzeppelin/contracts` install is
    // reachable (either from a `node_modules` ancestor of the project root
    // or via the `NEO_SOL_OZ_INSTALL_DIR` env var), the OZ subset of the
    // famous corpus must compile far more contracts. This is the test that
    // would have caught the pragma/using-directive/sibling-modifier bugs the
    // refactor fixed. Without an install the test no-ops; CI is expected to
    // run `npm install` in a helper step to exercise this floor.
    let Some(oz_install) = find_oz_install() else {
        eprintln!(
            "skip: no @openzeppelin/contracts install found; set NEO_SOL_OZ_INSTALL_DIR or \
             run `npm install @openzeppelin/contracts@5.4.0 @openzeppelin/contracts-upgradeable@5.4.0` in the project root"
        );
        return;
    };
    let vendor = vendor_root();
    if !vendor.exists() {
        eprintln!("skip: vendor tree missing at {}", vendor.display());
        return;
    }
    let oz_dir = vendor.join("@openzeppelin");
    if !oz_dir.exists() {
        eprintln!("skip: @openzeppelin vendor tree missing");
        return;
    }
    let include_paths = vec![
        oz_install.clone(),
        vendor.clone(),
        vendor.join("@openzeppelin/contracts"),
        vendor.join("@openzeppelin/contracts-upgradeable"),
    ];
    let mut total = 0usize;
    let mut passed = 0usize;
    for sol in iter_sol_files(&oz_dir) {
        total += 1;
        if try_compile(&sol, &include_paths).unwrap_or(false) {
            passed += 1;
        }
    }
    // Before the refactor, only 7 of the 40 OZ contracts compiled even with
    // a full install (most blocked by the pragma min-combinator bug and the
    // library-`using` / sibling-modifier merge bugs). After the two-round
    // refactor the entire OZ corpus compiles (40/40) — delegatecall and
    // opaque address.call paths are now downgraded from compile errors to
    // runtime traps (ABORTMSG), so dead-code paths that include them no
    // longer block deployment of the surrounding contract. The floor is
    // conservative to leave headroom for one or two intentional rejections
    // as the refactor settles.
    const FLOOR: usize = 35;
    assert!(
        passed >= FLOOR,
        "OZ corpus (with install) compile-pass count regressed: got {passed}/{total}, floor is {FLOOR}."
    );
    eprintln!(
        "OZ corpus (install at {}): {passed}/{total} compiled",
        oz_install.display()
    );
}
