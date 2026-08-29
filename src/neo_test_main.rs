//! `neo-test` — a Foundry-style Solidity test runner for Neo N3.
//!
//! A Solidity developer on Ethereum writes test contracts (`function testFoo()`
//! / `setUp()`) and runs `forge test`; each test is a transaction that must not
//! revert. `neo-test` brings that exact workflow to Neo: it compiles each test
//! source with the in-tree `neo-solc` compiler and executes every `test*`
//! function on the in-tree NeoVM (`NeoRuntime`) — the same VM the compiler's own
//! test-suite trusts — reporting PASS/FAIL, gas, decoded revert reasons, and
//! `console.log` / `Runtime.Log` output.
//!
//! Conventions (Foundry-compatible):
//!   * A *test contract* is any contract that declares a no-argument public
//!     function whose name starts with `test`.
//!   * `setUp()` (if present) runs before EACH test, against fresh state
//!     (per-test isolation), exactly like Foundry.
//!   * `test*`      — passes when the call does NOT revert.
//!   * `testFail*`  — passes when the call DOES revert.
//!   * Assertions are ordinary Solidity `require(cond, "msg")` / `assert(cond)`
//!     (and `revert`), which fault the VM on failure and surface as the FAIL
//!     reason. (A `Test`/assert helper library can layer on top later.)
//!
//! Usage:
//!   neo-test [PATH ...] [--match-test `<substr>`] [--match-contract `<substr>`]
//!            [-v|--verbose] [--gas] [--no-color]
//! PATH may be a `.sol` file or a directory (scanned recursively for `*.t.sol`,
//! falling back to `*.sol`). With no PATH, scans `./test` then `.`.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

struct Opts {
    paths: Vec<String>,
    match_test: Option<String>,
    match_contract: Option<String>,
    include: Vec<String>,
    verbose: bool,
    gas: bool,
    color: bool,
}

fn parse_args() -> Opts {
    let mut o = Opts {
        paths: Vec::new(),
        match_test: None,
        match_contract: None,
        include: Vec::new(),
        verbose: false,
        gas: false,
        color: std::env::var_os("NO_COLOR").is_none(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--match-test" | "-m" => o.match_test = it.next(),
            "--match-contract" | "-c" => o.match_contract = it.next(),
            "--include" | "-I" => {
                if let Some(p) = it.next() {
                    o.include.push(p);
                }
            }
            "-v" | "--verbose" => o.verbose = true,
            "--gas" | "--gas-report" => o.gas = true,
            "--no-color" => o.color = false,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => o.paths.push(other.to_string()),
        }
    }
    o
}

fn print_help() {
    println!(
        "neo-test — Foundry-style Solidity test runner for Neo N3\n\n\
         USAGE:\n  neo-test [PATH ...] [OPTIONS]\n\n\
         PATH        A .sol file or a directory (scanned for *.t.sol, else *.sol).\n\
         OPTIONS:\n\
         \x20 -m, --match-test <S>      Only run test fns whose name contains <S>\n\
         \x20 -c, --match-contract <S>  Only run contracts whose name contains <S>\n\
         \x20 -I, --include <DIR>       Extra import root (repeatable)\n\
         \x20 -v, --verbose             Show logs and revert reasons for every test\n\
         \x20 --gas                     Show per-test gas usage\n\
         \x20 --no-color                Disable ANSI color\n\n\
         CONVENTIONS:\n\
         \x20 test*      passes when the call does not revert\n\
         \x20 testFail*  passes when the call reverts\n\
         \x20 setUp()    runs before each test (fresh state per test)\n\n\
         STD LIBRARY (bundled, no setup):\n\
         \x20 import \"neo-std/Test.sol\";     // assertEq / assertTrue / assertGt / fail ... + `vm`\n\
         \x20 import \"neo-std/console.sol\";  // console.log(...) debug output\n\
         \x20 import \"neo-std/Vm.sol\";       // cheatcodes (auto via `is Test`)\n\n\
         CHEATCODES (vm.*, available when `is Test`):\n\
         \x20 vm.prank(addr) / startPrank / stopPrank   set msg.sender\n\
         \x20 vm.warp(secs) / vm.roll(n)                set block.timestamp / number\n\
         \x20 vm.deal(addr, amount)                     set GAS balance\n\
         \x20 vm.expectRevert()                         next call must revert\n\
         \x20 vm.label(addr, name) / vm.assume(cond)"
    );
}

/// Write the embedded `neo-std` test library to a temp dir and return the
/// include root, so `import \"neo-std/Test.sol\"` resolves to
/// `<root>/neo-std/Test.sol` without any project setup.
fn setup_std_lib() -> Option<PathBuf> {
    let root = std::env::temp_dir().join(format!("neo-test-std-{}", std::process::id()));
    let dir = root.join("neo-std");
    std::fs::create_dir_all(&dir).ok()?;
    std::fs::write(dir.join("Test.sol"), include_str!("neo_test_std/Test.sol")).ok()?;
    std::fs::write(
        dir.join("console.sol"),
        include_str!("neo_test_std/console.sol"),
    )
    .ok()?;
    std::fs::write(dir.join("Vm.sol"), include_str!("neo_test_std/Vm.sol")).ok()?;
    Some(root)
}

struct Paint {
    on: bool,
}
impl Paint {
    fn c(&self, code: &str, s: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
    fn green(&self, s: &str) -> String {
        self.c("32", s)
    }
    fn red(&self, s: &str) -> String {
        self.c("31", s)
    }
    fn dim(&self, s: &str) -> String {
        self.c("2", s)
    }
    fn bold(&self, s: &str) -> String {
        self.c("1", s)
    }
}

fn main() -> ExitCode {
    let opts = parse_args();
    let paint = Paint { on: opts.color };

    let files = collect_sol_files(&opts.paths);
    if files.is_empty() {
        eprintln!(
            "neo-test: no Solidity test files found. Pass a .sol file or a directory (looked for *.t.sol / *.sol)."
        );
        return ExitCode::from(2);
    }

    // Materialize the embedded `neo-std` test library to a temp dir and add it
    // to the include path so `import "neo-std/Test.sol"` / `"neo-std/console.sol"`
    // resolve with zero project setup. User `-I` paths are appended.
    let mut include_paths: Vec<PathBuf> = Vec::new();
    if let Some(std_root) = setup_std_lib() {
        include_paths.push(std_root);
    }
    include_paths.extend(opts.include.iter().map(PathBuf::from));

    let started = Instant::now();
    let mut total_pass = 0usize;
    let mut total_fail = 0usize;
    let mut total_skipped_contracts = 0usize;
    let mut any_compile_error = false;

    for file in &files {
        // Resolve `import` directives from disk so multi-file projects work
        // (the contract under test, a shared test base, OpenZeppelin, etc.).
        let src = match neo_devpack_solidity::cli::resolve_source_with_imports(file, &include_paths)
        {
            Ok(s) => s,
            Err(e) => {
                println!(
                    "{} {}",
                    paint.red("Import resolution failed:"),
                    file.display()
                );
                println!("  {}", e.replace('\n', "\n  "));
                any_compile_error = true;
                continue;
            }
        };

        let artifacts = match compile_contracts(&src, false, 2) {
            Ok(a) => a,
            Err(e) => {
                println!("{} {}", paint.red("Compilation failed:"), file.display());
                println!("  {}", format!("{e:?}").replace('\n', "\n  "));
                any_compile_error = true;
                continue;
            }
        };

        for art in &artifacts {
            let cname = contract_name(&art.manifest);
            if let Some(mc) = &opts.match_contract {
                if !cname.contains(mc) {
                    continue;
                }
            }
            let methods = method_names(&art.manifest);
            let has_setup = methods.iter().any(|m| m == "setUp");
            let mut tests: Vec<&String> = methods
                .iter()
                .filter(|m| is_test_fn(m))
                .filter(|m| {
                    opts.match_test
                        .as_ref()
                        .map(|s| m.contains(s))
                        .unwrap_or(true)
                })
                .collect();
            tests.sort();
            if tests.is_empty() {
                total_skipped_contracts += 1;
                continue;
            }

            println!(
                "\n{} {}",
                paint.dim(&format!("Running {} test(s) for", tests.len())),
                paint.bold(&format!("{}:{}", file.display(), cname))
            );

            let mut contract_pass = 0usize;
            let mut contract_fail = 0usize;
            for tname in tests {
                let r = run_one_test(art, tname, has_setup);
                let expect_revert = tname.starts_with("testFail");
                let passed = if expect_revert { !r.success } else { r.success };

                if passed {
                    contract_pass += 1;
                    let gas = if opts.gas {
                        paint.dim(&format!(" (gas: {})", r.gas_used))
                    } else {
                        String::new()
                    };
                    println!("  {} {}{}", paint.green("[PASS]"), tname, gas);
                } else {
                    contract_fail += 1;
                    let why = r.fail_reason(expect_revert);
                    println!(
                        "  {} {} {}",
                        paint.red("[FAIL]"),
                        tname,
                        paint.dim(&format!("({why})"))
                    );
                }

                if opts.verbose || !passed {
                    for line in &r.logs {
                        println!("    {} {}", paint.dim("log:"), line);
                    }
                }
            }
            total_pass += contract_pass;
            total_fail += contract_fail;
        }
    }

    let elapsed = started.elapsed();
    println!();
    if total_skipped_contracts > 0 {
        println!(
            "{}",
            paint.dim(&format!(
                "({total_skipped_contracts} contract(s) had no test* functions — skipped)"
            ))
        );
    }
    let verdict = if total_fail == 0 && !any_compile_error {
        paint.green("ok")
    } else {
        paint.red("FAILED")
    };
    println!(
        "Test result: {verdict}. {} passed; {} failed; finished in {:.2?}",
        paint.bold(&total_pass.to_string()),
        paint.bold(&total_fail.to_string()),
        elapsed
    );

    if total_fail == 0 && !any_compile_error {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Result of one test invocation, distilled for reporting.
struct TestOutcome {
    success: bool,
    gas_used: u64,
    reason: Option<String>,
    logs: Vec<String>,
    setup_failed: Option<String>,
}
impl TestOutcome {
    fn fail_reason(&self, expected_revert: bool) -> String {
        if let Some(s) = &self.setup_failed {
            return format!("setUp reverted: {s}");
        }
        if expected_revert {
            return "expected revert, but call succeeded".to_string();
        }
        self.reason
            .clone()
            .unwrap_or_else(|| "reverted".to_string())
    }
}

fn run_one_test(
    art: &neo_devpack_solidity::cli::CompilationArtifacts,
    test_name: &str,
    has_setup: bool,
) -> TestOutcome {
    // Fresh VM per test → per-test isolation (setUp re-runs each time).
    let mut rt = match NeoRuntime::new(RuntimeConfig::default()) {
        Ok(rt) => rt,
        Err(e) => {
            return TestOutcome {
                success: false,
                gas_used: 0,
                reason: Some(format!("runtime init failed: {e}")),
                logs: Vec::new(),
                setup_failed: None,
            }
        }
    };

    let no_args: [StackItem; 0] = [];

    // The first call_method fires the `_deploy` prologue (constructor +
    // state-var initializers) exactly once. Run setUp first so the constructor
    // runs before it; then the test sees setUp's state.
    if has_setup {
        match rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "setUp", &no_args) {
            Ok(r) if r.success => {}
            Ok(r) => {
                return TestOutcome {
                    success: false,
                    gas_used: r.gas_used,
                    reason: None,
                    logs: collect_logs(&r),
                    setup_failed: Some(exception_message(&r)),
                }
            }
            Err(e) => {
                return TestOutcome {
                    success: false,
                    gas_used: 0,
                    reason: None,
                    logs: Vec::new(),
                    setup_failed: Some(format!("{e}")),
                }
            }
        }
    }

    match rt.call_method(
        &art.bytecode,
        &art.tokens,
        &art.manifest,
        test_name,
        &no_args,
    ) {
        Ok(r) => TestOutcome {
            success: r.success,
            gas_used: r.gas_used,
            reason: if r.success {
                None
            } else {
                Some(exception_message(&r))
            },
            logs: collect_logs(&r),
            setup_failed: None,
        },
        Err(e) => TestOutcome {
            success: false,
            gas_used: 0,
            reason: Some(format!("{e}")),
            logs: Vec::new(),
            setup_failed: None,
        },
    }
}

/// Decode a Solidity revert into a human-readable reason. On a Solidity revert
/// the runtime surfaces the raw `selector || abi.encode(args)` payload in
/// `return_data`; a genuine VM fault leaves it empty and we fall back to the
/// rendered exception message.
fn exception_message(r: &neo_devpack_solidity::runtime::ExecutionResult) -> String {
    let data = &r.return_data;
    // Error(string) — selector 0x08c379a0
    if data.len() >= 4 && data[..4] == [0x08, 0xc3, 0x79, 0xa0] {
        if let Some(s) = decode_abi_string(&data[4..]) {
            return format!("revert: {s}");
        }
    }
    // Panic(uint256) — selector 0x4e487b71
    if data.len() >= 36 && data[..4] == [0x4e, 0x48, 0x7b, 0x71] {
        let code = data[35]; // panic codes are small; low byte of the 32-byte word
        return format!("panic: 0x{code:02x} ({})", panic_desc(code));
    }
    // Non-empty payload that is not a known selector: a custom error name (the
    // IR currently surfaces the bare name bytes) or a raw string.
    if !data.is_empty() {
        if let Some(run) = longest_printable(data) {
            return format!("revert: {run}");
        }
        return format!("revert (0x{})", hex(&data[..data.len().min(16)]));
    }
    // No payload → genuine VM fault. Use the rendered message, trimmed.
    if let Some(e) = &r.exception {
        let m = e.message.trim();
        let m = m.strip_prefix("Execution failed: ").unwrap_or(m);
        let m = m.strip_prefix("THROW: ").unwrap_or(m);
        if !m.is_empty() {
            return m.to_string();
        }
    }
    "reverted".to_string()
}

/// `[32-byte offset][32-byte length][bytes]` ABI-encoded `string`.
fn decode_abi_string(d: &[u8]) -> Option<String> {
    if d.len() < 64 {
        return None;
    }
    let len = be_usize(&d[32..64]);
    let start = 64usize;
    if len > 4096 || d.len() < start + len {
        return None;
    }
    String::from_utf8(d[start..start + len].to_vec()).ok()
}

fn be_usize(b: &[u8]) -> usize {
    let mut v = 0usize;
    for &x in b.iter().skip(b.len().saturating_sub(8)) {
        v = (v << 8) | x as usize;
    }
    v
}

fn panic_desc(code: u8) -> &'static str {
    match code {
        0x01 => "assert(false)",
        0x11 => "arithmetic overflow/underflow",
        0x12 => "division or modulo by zero",
        0x21 => "invalid enum conversion",
        0x22 => "invalid storage byte array",
        0x31 => "pop on empty array",
        0x32 => "array index out of bounds",
        0x41 => "out of memory / too much allocated",
        0x51 => "call to invalid internal function",
        _ => "panic",
    }
}

/// The longest run of printable (non-control) bytes — used to pull a revert
/// string out of a selector-prefixed or padded payload.
fn longest_printable(d: &[u8]) -> Option<String> {
    let mut best = String::new();
    let mut cur = String::new();
    for &b in d {
        if (0x20..0x7f).contains(&b) {
            cur.push(b as char);
        } else {
            if cur.len() > best.len() {
                best = std::mem::take(&mut cur);
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() > best.len() {
        best = cur;
    }
    let t = best.trim().to_string();
    if t.len() >= 3 {
        Some(t)
    } else {
        None
    }
}

fn collect_logs(r: &neo_devpack_solidity::runtime::ExecutionResult) -> Vec<String> {
    r.logs
        .iter()
        .map(|l| match String::from_utf8(l.data.clone()) {
            Ok(s) if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\t') => s,
            _ => format!("0x{}", hex(&l.data)),
        })
        .collect()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn is_test_fn(name: &str) -> bool {
    name.starts_with("test")
}

fn contract_name(manifest: &Value) -> String {
    manifest
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("<contract>")
        .to_string()
}

/// No-argument ABI methods (test fns and setUp take no parameters).
fn method_names(manifest: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(methods) = manifest
        .get("abi")
        .and_then(|a| a.get("methods"))
        .and_then(|m| m.as_array())
    {
        for m in methods {
            let params_empty = m
                .get("parameters")
                .and_then(|p| p.as_array())
                .map(|p| p.is_empty())
                .unwrap_or(true);
            if !params_empty {
                continue;
            }
            if let Some(n) = m.get("name").and_then(|n| n.as_str()) {
                out.push(n.to_string());
            }
        }
    }
    out
}

fn collect_sol_files(paths: &[String]) -> Vec<PathBuf> {
    let roots: Vec<PathBuf> = if paths.is_empty() {
        let t = PathBuf::from("test");
        if t.is_dir() {
            vec![t]
        } else {
            vec![PathBuf::from(".")]
        }
    } else {
        paths.iter().map(PathBuf::from).collect()
    };

    let mut files = Vec::new();
    for root in roots {
        if root.is_file() {
            if root.extension().map(|e| e == "sol").unwrap_or(false) {
                files.push(root);
            }
        } else if root.is_dir() {
            let mut all = Vec::new();
            walk_sol(&root, &mut all);
            // Prefer Foundry-style *.t.sol; fall back to all *.sol.
            let t_sol: Vec<PathBuf> = all
                .iter()
                .filter(|p| p.to_string_lossy().ends_with(".t.sol"))
                .cloned()
                .collect();
            if t_sol.is_empty() {
                files.extend(all);
            } else {
                files.extend(t_sol);
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

fn walk_sol(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if p.is_dir() {
            // Skip the usual noise directories.
            if matches!(
                name.as_str(),
                "node_modules" | "target" | ".git" | "out" | "dist"
            ) {
                continue;
            }
            walk_sol(&p, out);
        } else if p.extension().map(|x| x == "sol").unwrap_or(false) {
            out.push(p);
        }
    }
}
