//! Real-node differential test harness (audit gap #1, AUDIT_REPORT_v0.21.md §5).
//!
//! The audit's #1 finding: compiler-emitted bytecode is exercised only through
//! the in-tree NeoVM simulator. A bug that emits valid-looking bytecode which
//! the simulator accepts but a real Neo N3 node rejects (or evaluates
//! differently) is invisible to the existing test suite. This harness closes
//! that gap by running the SAME compiled contract in BOTH the embedded
//! `NeoRuntime` AND a real Neo-Express node, then diffing the integer results.
//!
//! ## What it covers
//!
//! Six `pure` methods exercising opcodes NOT already covered by the encoding
//! smoke suite (`examples/test_neoxp_encoding_smoke.sh`): POW (`**`), XOR (`^`),
//! SHL (`<<`), nested arithmetic + MOD, a complex bitwise expression
//! (`(a & b) | (a ^ b)`), and integer DIV (`/`).
//!
//! ## Why it is opt-in
//!
//! Running requires the .NET SDK + Neo-Express (`neoxp`) plus a pre-built
//! `neo-solc` release binary — none of which are available in a default
//! `cargo test` invocation. The whole file is gated behind
//! `#![cfg(feature = "neoxp-diff")]` and every test is `#[ignore]`, so:
//!
//!   - `cargo test`                                   → file is compiled out
//!   - `cargo test --features neoxp-diff`             → compiled, tests skipped
//!   - `cargo test --features neoxp-diff -- --ignored`→ compiled + RUN (CI only)
//!
//! The dedicated `neoxp-diff` CI job (`ci.yml`) installs dotnet + Neo.Express,
//! builds `neo-solc`, and runs the ignored tests.

#![cfg(feature = "neoxp-diff")]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use num_bigint::BigUint;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// The differential Solidity contract. All methods are `pure` and return a
/// single `uint256`, so both the simulator and the real node surface the
/// result as a bare integer (no ABI-decoded tuple wrapper). Methods are
/// intentionally chosen to cover opcodes NOT in the smoke suite.
const DIFF_SOURCE: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract DiffTest {
    function pow_test(uint256 base, uint256 exp) public pure returns (uint256) {
        return base ** exp;
    }
    function xor_test(uint256 a, uint256 b) public pure returns (uint256) {
        return a ^ b;
    }
    function shl_test(uint256 a, uint8 n) public pure returns (uint256) {
        return a << n;
    }
    function nested_mod(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {
        return (a + b) % c;
    }
    function bitwise_complex(uint256 a, uint256 b) public pure returns (uint256) {
        return (a & b) | (a ^ b);
    }
    function fn_div(uint256 a, uint256 b) public pure returns (uint256) {
        return a / b;
    }
    // ---- Wide-value probes (large but < 2^255, so representable) ----
    function pow_wide() public pure returns (uint256) { return uint256(3) ** 100; }
    function mul_wide() public pure returns (uint256) {
        return (uint256(1) << 100) * (uint256(3) << 100);
    }
}"#;

/// One differential probe: invoke `method` with `args` and expect `expected`.
/// Values fit comfortably in `i64` so they round-trip through both the
/// simulator's `StackItem::Integer` and neoxp's JSON integer arguments. The
/// `expected` is a `BigUint` so boundary probes (values ≥ 2^255) can express
/// the full uint256 range.
struct Case {
    method: &'static str,
    args: &'static [i64],
    expected: BigUint,
}

/// Build the probe set at runtime (`BigUint` is not const-constructible).
/// `&[...]` arg literals get static-promoted, satisfying `&'static [i64]`.
fn cases() -> Vec<Case> {
    vec![
        // POW (base ** exp)
        Case {
            method: "pow_test",
            args: &[2, 10],
            expected: BigUint::from(1024u64),
        },
        Case {
            method: "pow_test",
            args: &[3, 4],
            expected: BigUint::from(81u64),
        },
        // XOR (a ^ b)
        Case {
            method: "xor_test",
            args: &[0xFF, 0x0F],
            expected: BigUint::from(0xF0u64),
        },
        Case {
            method: "xor_test",
            args: &[0xAA, 0x55],
            expected: BigUint::from(0xFFu64),
        },
        // SHL (a << n)
        Case {
            method: "shl_test",
            args: &[1, 8],
            expected: BigUint::from(256u64),
        },
        Case {
            method: "shl_test",
            args: &[7, 3],
            expected: BigUint::from(56u64),
        },
        // Nested arithmetic + MOD ((a + b) % c)
        Case {
            method: "nested_mod",
            args: &[100, 23, 50],
            expected: BigUint::from(23u64),
        },
        Case {
            method: "nested_mod",
            args: &[10, 20, 7],
            expected: BigUint::from(2u64),
        },
        // Complex bitwise ((a & b) | (a ^ b)) == (a | b)
        Case {
            method: "bitwise_complex",
            args: &[0xF0, 0x3C],
            expected: BigUint::from(0xFCu64),
        },
        Case {
            method: "bitwise_complex",
            args: &[0xAA, 0x0F],
            expected: BigUint::from(0xAFu64),
        },
        // DIV (a / b)
        Case {
            method: "fn_div",
            args: &[100, 7],
            expected: BigUint::from(14u64),
        },
        Case {
            method: "fn_div",
            args: &[255, 16],
            expected: BigUint::from(15u64),
        },
        // ---- Wide-value probes (large but < 2^255, so representable) ----
        // These exercise the multi-limb / BigInt code paths for values well
        // beyond i64 but still inside the signed-32-byte range, catching
        // truncation / encoding divergences a small-value probe can't see.
        // Values >= 2^255 ([2^255, 2^256-1]) are a KNOWN representation
        // limitation — they need a 33-byte signed form NeoVM rejects, and the
        // compiler emits a validation warning for them — so they are
        // intentionally excluded here (the harness would FAULT on both sides
        // only if the simulator were also strict; today the simulator is
        // lenient, so such probes surface as simulator/node divergences that
        // track the open [2^255, 2^256-1] lowering gap, not new regressions).
        Case {
            method: "pow_wide",
            args: &[],
            expected: BigUint::from(3u8).pow(100u32),
        },
        Case {
            method: "mul_wide",
            args: &[],
            expected: (BigUint::from(1u8) << 100u32) * (BigUint::from(3u8) << 100u32),
        },
    ]
}

// ---------------------------------------------------------------------------
// Binary resolvers
// ---------------------------------------------------------------------------

/// Locate the `neo-solc` release binary. Prefers `$NEO_SOLC`, then the
/// workspace `target/release` / `target/debug` dirs. Panics with a actionable
/// message if absent — the harness is `#[ignore]` and only runs in CI where
/// the binary is pre-built (`cargo build --release`).
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

/// Locate the `neoxp` CLI. Mirrors `examples/test_neoxp_encoding_smoke.sh`:
/// `$NEOXP` → workspace `build/dotnet-tools/neoxp` → `PATH`.
fn resolve_neoxp() -> PathBuf {
    if let Ok(p) = std::env::var("NEOXP") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return path;
        }
    }
    let vendored = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build/dotnet-tools/neoxp");
    if vendored.is_file() {
        return vendored;
    }
    // Fall back to PATH; `which` is resolved by the OS when the arg has no `/`.
    PathBuf::from("neoxp")
}

// ---------------------------------------------------------------------------
// Simulator side (embedded NeoRuntime)
// ---------------------------------------------------------------------------

/// Run `method(args)` under the in-tree simulator and return the result as a
/// `BigUint`. The single-`uint256` return surfaces as minimum-width
/// little-endian bytes (see `differential_abi_encode_uint256_single`), so we
/// decode with `BigUint::from_bytes_le`.
fn run_in_simulator(
    bytecode: &[u8],
    tokens: &[neo_devpack_solidity::neo::MethodToken],
    manifest: &Value,
    method: &str,
    args: &[i64],
) -> BigUint {
    let mut rt = NeoRuntime::new(RuntimeConfig::default())
        .expect("NeoRuntime::new failed in differential harness");
    let stack_args: Vec<StackItem> = args.iter().map(|v| StackItem::Integer(*v)).collect();
    let r = rt
        .call_method(bytecode, tokens, manifest, method, &stack_args)
        .unwrap_or_else(|e| panic!("simulator call_method({method}) errored: {e:?}"));
    assert!(
        r.success,
        "simulator {method}({args:?}) faulted: {}",
        r.exception
            .as_ref()
            .map(|e| e.message.as_str())
            .unwrap_or("(no exception detail)")
    );
    BigUint::from_bytes_le(&r.return_data)
}

// ---------------------------------------------------------------------------
// Real-node side (Neo-Express)
// ---------------------------------------------------------------------------

/// A live Neo-Express chain with the `DiffTest` contract already deployed.
/// Dropping it cleans up the on-disk chain + neoxp home.
struct NeoxpEnv {
    // `home`/`work` tempdirs are kept alive for the struct's lifetime; their
    // paths are cached as plain `PathBuf` so subprocess spawns can borrow them
    // without holding a borrow on the `TempDir`.
    _home: TempDir,
    _work: TempDir,
    home: PathBuf,
    neoxp: PathBuf,
    chain: PathBuf,
    contract_hash: String,
}

impl NeoxpEnv {
    /// Stand up a fresh chain and deploy the compiled `.nef`. Reproduces the
    /// exact `neoxp` command sequence from
    /// `examples/test_neoxp_encoding_smoke.sh`:
    ///
    /// ```text
    /// neoxp create -f -o <chain>
    /// neoxp transfer -i <chain> 100 GAS genesis node1
    /// neoxp contract hash  -i <chain> <nef> node1
    /// neoxp contract deploy -i <chain> <nef> node1
    /// ```
    ///
    /// `HOME` is redirected into a throwaway dir because neoxp stashes
    /// per-user state there (same trick the smoke script uses).
    fn deploy(nef_path: &Path) -> NeoxpEnv {
        let home = tempfile::tempdir().expect("neoxp home tempdir");
        let home_path = home.path().to_path_buf();
        let work = tempfile::tempdir().expect("neoxp work tempdir");
        let neoxp = resolve_neoxp();
        let chain = work.path().join("chain.neo-express");

        // 1. Create a fresh single-blockchain node config.
        Self::run(
            &neoxp,
            home.path(),
            &["create", "-f", "-o", chain_str(&chain)],
        );

        // 2. Fund the deployer account so deploy + invoke have GAS.
        Self::run(
            &neoxp,
            home.path(),
            &[
                "transfer",
                "-i",
                chain_str(&chain),
                "100",
                "GAS",
                "genesis",
                "node1",
            ],
        );

        // 3. Compute the contract hash from the .nef BEFORE deploy (neoxp
        //    derives the hash deterministically from the script bytes).
        let hash_out = Self::run(
            &neoxp,
            home.path(),
            &[
                "contract",
                "hash",
                "-i",
                chain_str(&chain),
                nef_str(nef_path),
                "node1",
            ],
        );
        let contract_hash = hash_out.trim().replace('\r', "");
        assert!(
            !contract_hash.is_empty(),
            "neoxp contract hash returned empty output"
        );

        // 4. Deploy.
        Self::run(
            &neoxp,
            home.path(),
            &[
                "contract",
                "deploy",
                "-i",
                chain_str(&chain),
                nef_str(nef_path),
                "node1",
            ],
        );

        NeoxpEnv {
            _home: home,
            _work: work,
            home: home_path,
            neoxp,
            chain,
            contract_hash,
        }
    }

    /// Invoke `method(args)` on the deployed contract via
    /// `neoxp contract invoke -r -j` and return the integer result.
    /// Mirrors the smoke script's invoke JSON + result parsing
    /// (`.state` == HALT, `.stack[0].value` == decimal string).
    fn invoke(&self, method: &str, args: &[i64]) -> BigUint {
        let invoke = json!({
            "contract": self.contract_hash,
            "operation": method,
            "args": args,
        });
        let invoke_path = self
            .chain
            .with_file_name(format!("{method}.neo-invoke.json"));
        fs::write(&invoke_path, invoke.to_string()).expect("write invoke json");

        let out = Self::run(
            &self.neoxp,
            &self.home,
            &[
                "contract",
                "invoke",
                "-r",
                "-j",
                "-i",
                chain_str(&self.chain),
                invoke_path.to_str().unwrap(),
                "node1",
            ],
        );

        let v: Value = serde_json::from_str(out.trim()).unwrap_or_else(|e| {
            panic!("neoxp invoke({method}) returned non-JSON: {e}\nraw: {out}")
        });
        let state = v["state"].as_str().unwrap_or("");
        assert_eq!(
            state, "HALT",
            "neoxp {method}({args:?}) did not HALT (state={state}); full: {v}"
        );
        let value = v["stack"][0]["value"]
            .as_str()
            .unwrap_or_else(|| panic!("neoxp {method} result has no stack[0].value: {v}"));
        BigUint::parse_bytes(value.as_bytes(), 10)
            .unwrap_or_else(|| panic!("neoxp {method} value `{value}` is not a decimal integer"))
    }

    /// Run `neoxp` with `HOME` redirected and return stdout. Panics on
    /// non-zero exit, printing combined stdout+stderr for diagnosis.
    fn run(bin: &Path, home: &Path, args: &[&str]) -> String {
        let output = Command::new(bin)
            .args(args)
            .env("HOME", home)
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn neoxp {args:?}: {e}"));
        if !output.status.success() {
            panic!(
                "neoxp {args:?} exited {:?}\nstdout:\n{}\nstderr:\n{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        String::from_utf8_lossy(&output.stdout).into_owned()
    }
}

fn chain_str(chain: &Path) -> &str {
    chain.to_str().expect("chain path is not valid UTF-8")
}

fn nef_str(nef: &Path) -> &str {
    nef.to_str().expect("nef path is not valid UTF-8")
}

// ---------------------------------------------------------------------------
// The differential test
// ---------------------------------------------------------------------------

/// For every case: run the compiled bytecode in the simulator AND on a real
/// Neo-Express node, then assert both agree with the Rust-computed expected
/// value. A divergence here means the compiler emitted bytecode that the
/// in-tree simulator and a real Neo N3 node evaluate differently — exactly
/// the audit gap this harness exists to close.
#[test]
#[ignore = "requires dotnet + Neo.Express; run with --ignored under the neoxp-diff feature"]
fn differential_simulator_matches_real_node() {
    // --- Compile once via the shipped binary (produces the .nef the real
    //     node will run), and once via the Rust API (gives the in-memory
    //     artifacts the simulator needs). Both compile the same source. ---
    let neo_solc = resolve_neo_solc();

    let work = tempfile::tempdir().expect("compile work tempdir");
    let sol_path = work.path().join("DiffTest.sol");
    fs::write(&sol_path, DIFF_SOURCE).expect("write DiffTest.sol");

    let out_prefix = work.path().join("DiffTest");
    let status = Command::new(&neo_solc)
        .arg(&sol_path)
        .arg("-o")
        .arg(&out_prefix)
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn neo-solc: {e}"));
    assert!(status.success(), "neo-solc failed to compile DiffTest.sol");

    let nef_path = work.path().join("DiffTest.nef");
    let manifest_path = work.path().join("DiffTest.manifest.json");
    assert!(
        nef_path.is_file(),
        "neo-solc did not emit {}",
        nef_path.display()
    );
    assert!(
        manifest_path.is_file(),
        "neo-solc did not emit {}",
        manifest_path.display()
    );

    // In-memory artifacts for the simulator side.
    let arts =
        compile_contracts(DIFF_SOURCE, false, 2).expect("compile_contracts(DiffTest) failed");
    let art = &arts[0];

    // --- Stand up the real node + deploy once for all cases. ---
    let env = NeoxpEnv::deploy(&nef_path);

    for case in cases() {
        let sim = run_in_simulator(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            case.method,
            case.args,
        );
        let node = env.invoke(case.method, case.args);

        assert_eq!(
            sim, node,
            "DIVERGENCE for {}({:?}): simulator={} node={}",
            case.method, case.args, sim, node
        );
        assert_eq!(
            sim, case.expected,
            "WRONG RESULT for {}({:?}): got {} expected {} \
             (simulator and node agree but both disagree with the oracle — \
             likely a bad test fixture, not a compiler bug)",
            case.method, case.args, sim, case.expected
        );
    }
}
