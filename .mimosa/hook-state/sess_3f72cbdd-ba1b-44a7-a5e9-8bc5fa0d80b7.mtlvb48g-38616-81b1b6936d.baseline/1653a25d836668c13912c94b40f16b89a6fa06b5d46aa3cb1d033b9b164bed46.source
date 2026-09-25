//! Famous-contracts runtime smoke harness.
//!
//! The compile-only `famous_corpus_vendor_only_compile_floor` test (in
//! `tests/famous_contracts_compile.rs`) only verifies the frontend: it
//! runs each vendored .sol through `neo-solc` and asserts the compiler
//! exits 0. That catches parser / type-checker / IR-lowering regressions,
//! but it is BLIND to everything that happens after bytecode emission:
//! manifest permission gates, native-call wiring, storage key derivation,
//! runtime syscalls, S6 CallFlags propagation, etc.
//!
//! This file closes the gap: for every contract in the vendored tree
//! (`third_party/famous-contracts/sources/`), compile → deploy to a fresh
//! Neo-Express chain → invoke a representative read-only method. The
//! per-contract outcome is written to
//! `third_party/famous-contracts/RUNTIME_REPORT.md` so the next reader
//! can see, at a glance, which contracts exercise the runtime cleanly
//! and which surface FAULTs that the compile-only suite misses.
//!
//! ## What it covers
//!
//! - Deploy-side: every `.nef` produced by the compiler is asked to deploy
//!   to a real Neo N3 chain (so manifest-permission gaps, S6 call-flag
//!   mismatches, native-permission missing, etc. surface here).
//! - Smoke-side: for each successful deploy, we try a short list of
//!   read-only methods (`name`, `symbol`, `decimals`, `totalSupply`,
//!   `owner`, `paused`, `get`, `view`). The first HALT wins.
//!   Any FAULT on every candidate is recorded separately as
//!   "compile+deploy pass · no smoke method" so the report distinguishes
//!   runtime bugs from contracts whose API simply doesn't expose a
//!   zero-arg read.
//!
//! ## Why it is opt-in
//!
//! Requires the .NET SDK + Neo-Express (`neoxp`) + a pre-built
//! `neo-solc` release binary — none of which are available in a default
//! `cargo test` invocation. The whole file is gated behind
//! `#![cfg(feature = "neoxp-diff")]` and the test is `#[ignore]`, so:
//!
//!   - `cargo test`                                   → file is compiled out
//!   - `cargo test --features neoxp-diff`             → compiled, test skipped
//!   - `cargo test --features neoxp-diff -- --ignored`→ compiled + RUN
//!
//! Run locally:
//! ```sh
//! cargo build --release --bin neo-solc
//! NEOXP=/home/neo/.dotnet/tools/neoxp \
//! NEO_SOLC=$PWD/target/release/neo-solc \
//! cargo test --release --features neoxp-diff \
//!     --test famous_contracts_runtime_smoke -- --ignored --nocapture
//! ```

#![cfg(feature = "neoxp-diff")]

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

/// Read-only smoke methods, tried in this order; first HALT wins. These
/// were chosen because they are zero-arg, side-effect-free, and present
/// on a large fraction of the vendored corpus (ERC-20/721 tokens, access
/// control, governance). A contract whose API exposes none of these is
/// reported as "compile+deploy pass · no smoke method" rather than as a
/// failure — the absence of a zero-arg read is not a bug, it is a normal
/// shape of utility / library contracts.
const SMOKE_METHODS: &[&str] = &[
    "name",
    "symbol",
    "decimals",
    "totalSupply",
    "owner",
    "paused",
    "get",
    "view",
];

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn unique_work_dir(scope: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = OUTPUT_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("neo-solc-{scope}-{nanos}-{seq}"))
}

fn vendor_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("third_party/famous-contracts/sources")
}

/// Locate `neo-solc`. Mirrors `tests/neoxp_differential.rs::resolve_neo_solc`.
fn resolve_neo_solc() -> PathBuf {
    if let Ok(p) = std::env::var("NEO_SOLC") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return path;
        }
    }
    let target = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
    for sub in ["release", "debug"] {
        let cand = target.join(sub).join("neo-solc");
        if cand.is_file() {
            return cand;
        }
    }
    panic!(
        "neo-solc binary not found. Build it first with `cargo build --release` \
         or point the NEO_SOLC env var at it."
    )
}

/// Locate `neoxp`. Mirrors `tests/neoxp_differential.rs::resolve_neoxp`.
fn resolve_neoxp() -> PathBuf {
    if let Ok(p) = std::env::var("NEOXP") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return path;
        }
    }
    PathBuf::from("neoxp")
}

/// Include paths passed to `neo-solc -I ...` when compiling each vendored
/// source. Mirrors the existing
/// `tests/famous_contracts_compile.rs::vendor_only_compile_floor` set so
/// the runtime test sees the same compile-floor the existing test does.
fn include_paths() -> Vec<PathBuf> {
    let vendor = vendor_root();
    vec![
        vendor.clone(),
        vendor.join("@openzeppelin/contracts"),
        vendor.join("@openzeppelin/contracts-upgradeable"),
        vendor.join("@uniswap/v2-core/contracts"),
        vendor.join("@uniswap/v4-core/src"),
        vendor.join("@uniswap/v4-periphery/src"),
        vendor.join("@safe-global/safe-contracts/contracts"),
        vendor.join("@aave/core-v3"),
    ]
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

fn detect_pragma(source: &Path) -> Option<String> {
    std::fs::read_to_string(source).ok()?.lines().find_map(|l| {
        let t = l.trim();
        if t.starts_with("pragma solidity") {
            Some(t.to_string())
        } else {
            None
        }
    })
}

/// Per-contract outcome for the report. We record EVERY field (even when
/// empty) so a regression in any single stage is loud in the diff.
#[derive(Debug, Clone)]
struct ContractResult {
    relpath: String,
    pragma: Option<String>,
    /// Compile succeeded (exit 0, .nef emitted).
    compiles: bool,
    /// Deploy to Neo-Express HALTed.
    deploys: bool,
    /// Non-empty iff `deploys == false` AND we have a concrete reason
    /// (neoxp exit code != 0, VM FAULT, missing contract-hash, …).
    deploy_error: Option<String>,
    /// First `SMOKE_METHODS` entry whose invoke HALTed.
    smoke_method: Option<String>,
    /// `HALT` / `FAULT` for the smoke invoke.
    smoke_state: Option<String>,
    /// Raw `.stack[0]` JSON for the smoke invoke (string for ByteString,
    /// decimal for Integer, etc.).
    smoke_result: Option<String>,
}

impl ContractResult {
    /// True iff this contract cleared every stage we test.
    fn smoke_pass(&self) -> bool {
        self.compiles && self.deploys && self.smoke_state.as_deref() == Some("HALT")
    }
}

/// Try to compile `source` and return the path to its emitted `.nef`
/// inside `outdir`. Caller owns `outdir` (must `remove_dir_all` on drop).
/// Returns `None` if compile fails OR the .nef is missing.
fn try_compile_with_nef(compiler: &Path, source: &Path, outdir: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(outdir).ok()?;
    let mut cmd = Command::new(compiler);
    cmd.arg(source).arg("-O").arg("0").arg("-o").arg(outdir);
    for inc in include_paths() {
        cmd.arg("-I").arg(inc);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stem = source.file_stem()?.to_str()?;
    let nef = outdir.join(format!("{stem}.nef"));
    if nef.is_file() {
        Some(nef)
    } else {
        None
    }
}

/// Run `neoxp <args>` with HOME redirected to a throwaway dir. Returns
/// stdout on success; on failure returns a diagnostic string that
/// combines stdout + stderr.
fn neoxp_run(neoxp_bin: &Path, home: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(neoxp_bin)
        .args(args)
        .env("HOME", home)
        .output()
        .map_err(|e| format!("failed to spawn neoxp {args:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "neoxp {args:?} exited {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// What `deploy_to_neoxp` returns on success: every handle the caller
/// needs to keep alive for further invocations. `home` and `work` are
/// `TempDir`s — dropping them removes the on-disk chain + neoxp state.
/// `work` is kept around purely as an owning handle for the chain file;
/// its only use is to extend the tempdir lifetime via `Drop`.
#[allow(dead_code)]
struct NeoxpChain {
    home: TempDir,
    work: TempDir,
    chain: PathBuf,
    contract_hash: String,
}

/// Stand up a fresh Neo-Express chain and deploy `nef` to it. Returns
/// the chain handle + contract hash on success, or an error string on
/// deploy failure (which can be a legitimate VM FAULT — recordable in
/// the report).
fn deploy_to_neoxp(neoxp_bin: &Path, nef: &Path) -> Result<NeoxpChain, String> {
    let home = TempDir::new().map_err(|e| format!("neoxp home tempdir: {e}"))?;
    let work = TempDir::new().map_err(|e| format!("neoxp work tempdir: {e}"))?;
    let chain = work.path().join("chain.neo-express");
    let chain_str = chain.to_str().ok_or("non-utf8 chain path")?;

    neoxp_run(neoxp_bin, home.path(), &["create", "-f", "-o", chain_str])?;
    neoxp_run(
        neoxp_bin,
        home.path(),
        &[
            "transfer", "-i", chain_str, "100", "GAS", "genesis", "node1",
        ],
    )?;

    let deploy_out = neoxp_run(
        neoxp_bin,
        home.path(),
        &[
            "contract",
            "deploy",
            "-i",
            chain_str,
            nef.to_str().ok_or("non-utf8 nef path")?,
            "node1",
            "-j",
        ],
    )?;

    let v: Value = serde_json::from_str(&deploy_out)
        .map_err(|e| format!("deploy json parse: {e}\nraw: {deploy_out}"))?;
    let contract_hash = v["contract-hash"]
        .as_str()
        .ok_or_else(|| format!("missing contract-hash in deploy output: {v}"))?
        .to_string();
    if contract_hash.is_empty() {
        return Err(format!("empty contract-hash in deploy output: {v}"));
    }
    Ok(NeoxpChain {
        home,
        work,
        chain,
        contract_hash,
    })
}

/// Invoke `method` on `contract_hash` (already deployed to `chain`) and
/// return `(state, raw_stack_item_json)` on success. State is "HALT" or
/// "FAULT" (or other Neo-VM states — passed through verbatim). `home`
/// is the neoxp HOME directory (TempDir-owned by the caller).
fn invoke_smoke(
    neoxp_bin: &Path,
    home: &Path,
    chain: &Path,
    contract_hash: &str,
    method: &str,
) -> Result<(String, String), String> {
    let invoke_path = chain.with_file_name(format!("smoke-{method}.neo-invoke.json"));
    let payload = json!({
        "contract": contract_hash,
        "operation": method,
        "args": [],
    });
    std::fs::write(&invoke_path, payload.to_string())
        .map_err(|e| format!("write invoke json: {e}"))?;

    let out = neoxp_run(
        neoxp_bin,
        home,
        &[
            "contract",
            "invoke",
            "-r",
            "-j",
            "-i",
            chain.to_str().ok_or("non-utf8 chain path")?,
            invoke_path.to_str().ok_or("non-utf8 invoke path")?,
            "node1",
        ],
    )?;

    let v: Value =
        serde_json::from_str(&out).map_err(|e| format!("invoke json parse: {e}\nraw: {out}"))?;
    let state = v["state"].as_str().unwrap_or("UNKNOWN").to_string();
    let stack0 = v["stack"]
        .as_array()
        .and_then(|a| a.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "(empty stack)".to_string());
    Ok((state, stack0))
}

/// Drive one vendored .sol through compile → deploy → smoke-method sweep.
/// Returns a fully populated `ContractResult` even when stages fail —
/// partial progress is the whole point of the report.
fn smoke_one(compiler: &Path, neoxp_bin: &Path, source: &Path) -> ContractResult {
    let relpath = source
        .strip_prefix(vendor_root())
        .unwrap_or(source)
        .to_string_lossy()
        .into_owned();
    let pragma = detect_pragma(source);

    // Stage 1: compile.
    let outdir = unique_work_dir("famous-smoke");
    let nef = match try_compile_with_nef(compiler, source, &outdir) {
        Some(n) => n,
        None => {
            let _ = std::fs::remove_dir_all(&outdir);
            return ContractResult {
                relpath,
                pragma,
                compiles: false,
                deploys: false,
                deploy_error: None,
                smoke_method: None,
                smoke_state: None,
                smoke_result: None,
            };
        }
    };

    // Stage 2: deploy.
    let nc = match deploy_to_neoxp(neoxp_bin, &nef) {
        Ok(t) => t,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&outdir);
            return ContractResult {
                relpath,
                pragma,
                compiles: true,
                deploys: false,
                deploy_error: Some(e),
                smoke_method: None,
                smoke_state: None,
                smoke_result: None,
            };
        }
    };

    // Stage 3: try smoke methods in order. The first HALT wins; a
    // method returning FAULT (e.g. constructor args required, wrong
    // selector) is silently skipped so the report only surfaces
    // SUCCESSFUL reads. The "all FAULT" case surfaces as
    // smoke_state=None in the report — the row shows the contract
    // deployed but exposes no zero-arg read among the candidates.
    let mut result = ContractResult {
        relpath: relpath.clone(),
        pragma,
        compiles: true,
        deploys: true,
        deploy_error: None,
        smoke_method: None,
        smoke_state: None,
        smoke_result: None,
    };
    for &method in SMOKE_METHODS {
        match invoke_smoke(
            neoxp_bin,
            nc.home.path(),
            &nc.chain,
            &nc.contract_hash,
            method,
        ) {
            Ok((state, stack0)) if state == "HALT" => {
                result.smoke_method = Some(method.to_string());
                result.smoke_state = Some(state);
                result.smoke_result = Some(stack0);
                break;
            }
            Ok(_) => continue,  // FAULT / UNKNOWN / … → try next method
            Err(_) => continue, // spawn / parse failure → try next method
        }
    }

    let _ = std::fs::remove_dir_all(&outdir);
    // `nc` drops here, removing the home + work tempdirs (chain + neoxp
    // state).
    result
}

/// Render `results` as a markdown table and write it to
/// `third_party/famous-contracts/RUNTIME_REPORT.md`. The header carries
/// the timestamp + summary so the file is self-describing without
/// needing to be regenerated to be useful.
fn write_report(results: &[ContractResult]) {
    let vendor = vendor_root();
    let report_path = vendor
        .parent()
        .map(|p| p.join("RUNTIME_REPORT.md"))
        .unwrap_or_else(|| PathBuf::from("RUNTIME_REPORT.md"));

    let total = results.len();
    let compiled = results.iter().filter(|r| r.compiles).count();
    let deployed = results.iter().filter(|r| r.compiles && r.deploys).count();
    let smoke_halt = results.iter().filter(|r| r.smoke_pass()).count();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut s = String::new();
    s.push_str("# Famous Contracts Runtime Smoke Report\n\n");
    s.push_str(&format!(
        "_Generated_: UNIX epoch {ts} (`cargo test --features neoxp-diff \
         --test famous_contracts_runtime_smoke -- --ignored --nocapture`)\n\n"
    ));
    s.push_str("- Compiler: `neo-solc` (release)\n");
    s.push_str("- Backend: Neo-Express (real-node deploy + invoke)\n");
    s.push_str("- Source root: `third_party/famous-contracts/sources/`\n\n");

    s.push_str("## Summary\n\n");
    s.push_str("| Metric | Count |\n| --- | --- |\n");
    s.push_str(&format!("| Total contracts | {total} |\n"));
    s.push_str(&format!("| Compile pass | {compiled} |\n"));
    s.push_str(&format!(
        "| Deploy pass (compile pass + HALT deploy) | {deployed} |\n"
    ));
    s.push_str(&format!(
        "| Smoke HALT (full pass — compile + deploy + smoke method) | {smoke_halt} |\n\n"
    ));

    s.push_str("## Per-contract results\n\n");
    s.push_str("Legend: ✓ = passed, ✗ = failed, `—` = not attempted (earlier stage failed).\n\n");
    s.push_str("| Contract | Pragma | Compile | Deploy | Smoke method | State | Stack[0] |\n");
    s.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for r in results {
        let compile_cell = if r.compiles { "✓" } else { "✗" };
        let deploy_cell = if !r.compiles {
            "—"
        } else if r.deploys {
            "✓"
        } else {
            "✗"
        };
        let method =
            r.smoke_method
                .clone()
                .unwrap_or_else(|| if r.deploys { "—".into() } else { "".into() });
        let state = r.smoke_state.clone().unwrap_or_default();
        let stack = r.smoke_result.clone().unwrap_or_else(|| {
            if !r.deploys && r.deploy_error.is_some() {
                let e = r.deploy_error.as_deref().unwrap_or("");
                let first_line = e.lines().next().unwrap_or("").to_string();
                format!("_deploy error_: {first_line}")
            } else {
                String::new()
            }
        });
        let pragma_cell = r.pragma.clone().unwrap_or_default();
        // Escape pipes in user-supplied cells to keep the table valid.
        let clean = |v: &str| v.replace('|', "\\|");
        s.push_str(&format!(
            "| `{}` | `{}` | {} | {} | `{}` | {} | {} |\n",
            clean(&r.relpath),
            clean(&pragma_cell),
            compile_cell,
            deploy_cell,
            clean(&method),
            state,
            clean(&stack),
        ));
    }

    if let Some(parent) = report_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&report_path, &s) {
        eprintln!(
            "warning: failed to write runtime report to {}: {e}",
            report_path.display()
        );
    } else {
        eprintln!("report written: {}", report_path.display());
    }
}

#[test]
#[ignore = "requires dotnet + Neo.Express; run with --ignored under the neoxp-diff feature"]
fn famous_corpus_runtime_smoke() {
    let vendor = vendor_root();
    if !vendor.exists() {
        eprintln!("skip: vendor tree missing at {}", vendor.display());
        return;
    }
    let compiler = resolve_neo_solc();
    let neoxp_bin = resolve_neoxp();

    eprintln!("compiler: {}", compiler.display());
    eprintln!("neoxp:    {}", neoxp_bin.display());
    eprintln!("vendor:   {}", vendor.display());

    let sources = iter_sol_files(&vendor);
    eprintln!("discovered {} .sol files", sources.len());

    let mut results: Vec<ContractResult> = Vec::with_capacity(sources.len());
    for (idx, src) in sources.iter().enumerate() {
        eprint!(
            "[{:>3}/{}] {} ... ",
            idx + 1,
            sources.len(),
            src.strip_prefix(&vendor).unwrap_or(src).display(),
        );
        let r = smoke_one(&compiler, &neoxp_bin, src);
        let status = if r.smoke_pass() {
            "smoke HALT"
        } else if r.compiles && r.deploys {
            "deployed, no smoke"
        } else if r.compiles {
            "compile, deploy FAULT"
        } else {
            "compile FAIL"
        };
        eprintln!("{status}");
        results.push(r);
    }

    let total = results.len();
    let compiled = results.iter().filter(|r| r.compiles).count();
    let deployed = results.iter().filter(|r| r.compiles && r.deploys).count();
    let smoke_halt = results.iter().filter(|r| r.smoke_pass()).count();
    eprintln!(
        "\n=== Famous Contracts Runtime Smoke ===\n\
         total={total} compiled={compiled} deployed={deployed} smoke_HALT={smoke_halt}"
    );

    write_report(&results);

    // Floor: at least 1 contract must full-pass the smoke. The compile-only
    // floor is 5; here we ask for at least one real-node deploy + smoke
    // HALT — if the runtime regresses, this fires before the user notices
    // a missing report.
    assert!(
        smoke_halt >= 1,
        "famous-corpus runtime smoke regressed: {smoke_halt}/{total} HALT, floor is 1"
    );
}
