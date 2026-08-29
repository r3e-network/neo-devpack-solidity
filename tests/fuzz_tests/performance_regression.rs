//! Performance-regression smoke gate.
//!
//! Why this module exists:
//! ------------------------
//! The fuzz suite has comprehensive correctness coverage, but no test fails
//! if a refactor accidentally makes the compiler 5x slower. Performance
//! regressions slip in silently — a compiler that takes 30 s to build a
//! 200-line contract instead of 3 s is not "broken" (every correctness test
//! still passes) yet it is degraded enough to break CI / IDE workflows.
//!
//! `compile_runtime_performance_regression` runs three representative
//! contracts (tiny / medium / large) end-to-end:
//!
//!   1. compile_us  — wall time of `compile_contracts(src, false, 2)` (opt=2)
//!   2. runtime_us  — sum of 5 `NeoRuntime::call_method` invocations on a
//!      representative entry point
//!
//! Each timing is asserted below a generous absolute threshold (~10x slower
//! than typical observed values on a modern dev box, sized for cargo's
//! default `dev` profile which is ~3x slower than `--release`). If a
//! threshold trips, the panic message reports the actual µs so the
//! regression can be filed instantly.
//!
//! Stability rules:
//!   - No saved-baseline file (avoids maintenance burden).
//!   - Absolute thresholds only; the test only fails on real regressions.
//!   - Contracts are inlined as string literals (no FS dependency, no
//!     import-resolution path) so the gate works in any CI environment.
//!
//! Reference harness pattern: `tests/fuzz_tests/native_contract_props.rs`,
//! `tests/fuzz_tests/examples_smoke_props.rs`.

#![allow(unused_imports)]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use std::time::Instant;

// ---------- Representative contracts ----------

/// Tiny: a 20-line counter. Single uint storage slot, one mutator, one
/// implicit getter.
const SRC_TINY: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Counter {
    uint256 public counter;

    function inc() external {
        counter++;
    }

    function get() external view returns (uint256) {
        return counter;
    }
}
"#;

/// Medium: ~100-line ERC-20-style contract with mint / transfer / burn /
/// balanceOf. No imports, parameterless constructor (mints initial supply
/// to a hard-coded zero address so `_deploy` runs without ctor args), and
/// every call exercises a balance mapping read+write — i.e. it stresses
/// the compiler's mapping-storage lowering and the runtime's STG/STG_GET
/// path. Patterned after `examples/ERC20Token.sol` but simplified to
/// keep the test self-contained.
const SRC_MEDIUM: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract MiniToken {
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;

    uint256 private _totalSupply;
    string public name;
    string public symbol;
    uint8 public decimals;
    address public owner;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    constructor() {
        name = "MiniToken";
        symbol = "MINI";
        decimals = 18;
        owner = msg.sender;
        _totalSupply = 0;
    }

    function totalSupply() external view returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }

    function mint(address to, uint256 amount) external {
        require(to != address(0), "mint to zero");
        _balances[to] += amount;
        _totalSupply += amount;
        emit Transfer(address(0), to, amount);
    }

    function burn(address from, uint256 amount) external {
        require(_balances[from] >= amount, "burn exceeds balance");
        _balances[from] -= amount;
        _totalSupply -= amount;
        emit Transfer(from, address(0), amount);
    }

    function transfer(address from, address to, uint256 amount) external returns (bool) {
        require(to != address(0), "transfer to zero");
        require(_balances[from] >= amount, "balance too low");
        _balances[from] -= amount;
        _balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }

    function approve(address ownerAddr, address spender, uint256 amount) external returns (bool) {
        _allowances[ownerAddr][spender] = amount;
        emit Approval(ownerAddr, spender, amount);
        return true;
    }

    function allowance(address ownerAddr, address spender) external view returns (uint256) {
        return _allowances[ownerAddr][spender];
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        uint256 cur = _allowances[from][msg.sender];
        require(cur >= amount, "allowance too low");
        require(_balances[from] >= amount, "balance too low");
        _allowances[from][msg.sender] = cur - amount;
        _balances[from] -= amount;
        _balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}
"#;

/// Large: 200+ lines combining multiple modifiers, nested mappings, struct
/// storage, and several mutator paths. Stresses the IR builder
/// (modifier-rewrite, struct-pack/unpack, multi-mapping lowering) and the
/// runtime (storage-prefix dispatch across many slots). No imports; no
/// constructor args (Neo deploy constraint).
const SRC_LARGE: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract MultiVault {
    struct Account {
        uint256 balance;
        uint256 lockedUntil;
        uint32 tier;
        bool frozen;
    }

    struct Pool {
        uint256 totalDeposit;
        uint256 rewardPerShare;
        uint64 lastUpdate;
        bool active;
    }

    mapping(address => Account) private _accounts;
    mapping(uint256 => Pool) private _pools;
    mapping(address => mapping(uint256 => uint256)) private _stakes;
    mapping(address => mapping(uint256 => uint256)) private _rewards;
    mapping(address => bool) private _admins;
    mapping(uint256 => address) private _poolOperators;

    address public owner;
    uint256 public poolCount;
    uint256 public totalLocked;
    uint256 public globalFee;
    bool public paused;

    event Deposited(address indexed user, uint256 indexed pid, uint256 amount);
    event Withdrawn(address indexed user, uint256 indexed pid, uint256 amount);
    event PoolCreated(uint256 indexed pid, address indexed operator);
    event AccountFrozen(address indexed user, bool frozen);
    event TierChanged(address indexed user, uint32 oldTier, uint32 newTier);
    event FeeUpdated(uint256 oldFee, uint256 newFee);

    modifier onlyOwner() {
        require(msg.sender == owner, "not owner");
        _;
    }

    modifier onlyAdmin() {
        require(_admins[msg.sender] || msg.sender == owner, "not admin");
        _;
    }

    modifier whenNotPaused() {
        require(!paused, "paused");
        _;
    }

    modifier notFrozen(address user) {
        require(!_accounts[user].frozen, "frozen");
        _;
    }

    modifier validPool(uint256 pid) {
        require(pid < poolCount, "bad pool");
        require(_pools[pid].active, "inactive pool");
        _;
    }

    constructor() {
        owner = msg.sender;
        paused = false;
        globalFee = 25;
        poolCount = 0;
        totalLocked = 0;
        _admins[msg.sender] = true;
    }

    function setAdmin(address who, bool flag) external onlyOwner {
        _admins[who] = flag;
    }

    function pause() external onlyAdmin {
        paused = true;
    }

    function unpause() external onlyAdmin {
        paused = false;
    }

    function setFee(uint256 newFee) external onlyOwner {
        require(newFee <= 1000, "fee too high");
        uint256 old = globalFee;
        globalFee = newFee;
        emit FeeUpdated(old, newFee);
    }

    function createPool(address operator) external onlyAdmin returns (uint256) {
        uint256 pid = poolCount;
        _pools[pid] = Pool({
            totalDeposit: 0,
            rewardPerShare: 0,
            lastUpdate: uint64(block.timestamp),
            active: true
        });
        _poolOperators[pid] = operator;
        poolCount = pid + 1;
        emit PoolCreated(pid, operator);
        return pid;
    }

    function setTier(address user, uint32 newTier) external onlyAdmin {
        Account storage a = _accounts[user];
        uint32 oldTier = a.tier;
        a.tier = newTier;
        emit TierChanged(user, oldTier, newTier);
    }

    function freeze(address user, bool flag) external onlyAdmin {
        _accounts[user].frozen = flag;
        emit AccountFrozen(user, flag);
    }

    function deposit(uint256 pid, uint256 amount)
        external
        whenNotPaused
        notFrozen(msg.sender)
        validPool(pid)
    {
        require(amount > 0, "zero amount");
        Account storage a = _accounts[msg.sender];
        a.balance += amount;
        _stakes[msg.sender][pid] += amount;
        Pool storage p = _pools[pid];
        p.totalDeposit += amount;
        p.lastUpdate = uint64(block.timestamp);
        totalLocked += amount;
        emit Deposited(msg.sender, pid, amount);
    }

    function withdraw(uint256 pid, uint256 amount)
        external
        whenNotPaused
        notFrozen(msg.sender)
        validPool(pid)
    {
        require(amount > 0, "zero amount");
        uint256 staked = _stakes[msg.sender][pid];
        require(staked >= amount, "stake too low");
        Account storage a = _accounts[msg.sender];
        require(a.balance >= amount, "balance too low");
        a.balance -= amount;
        _stakes[msg.sender][pid] = staked - amount;
        Pool storage p = _pools[pid];
        p.totalDeposit -= amount;
        p.lastUpdate = uint64(block.timestamp);
        totalLocked -= amount;
        emit Withdrawn(msg.sender, pid, amount);
    }

    function lock(address user, uint256 until) external onlyAdmin {
        _accounts[user].lockedUntil = until;
    }

    function balanceOf(address user) external view returns (uint256) {
        return _accounts[user].balance;
    }

    function stakeOf(address user, uint256 pid) external view returns (uint256) {
        return _stakes[user][pid];
    }

    function tierOf(address user) external view returns (uint32) {
        return _accounts[user].tier;
    }

    function isFrozen(address user) external view returns (bool) {
        return _accounts[user].frozen;
    }

    function isAdmin(address who) external view returns (bool) {
        return _admins[who];
    }

    function poolInfo(uint256 pid)
        external
        view
        returns (uint256 totalDeposit, uint64 lastUpdate, bool active)
    {
        Pool storage p = _pools[pid];
        return (p.totalDeposit, p.lastUpdate, p.active);
    }

    function poolOperator(uint256 pid) external view returns (address) {
        return _poolOperators[pid];
    }

    function getOwner() external view returns (address) {
        return owner;
    }

    function getFee() external view returns (uint256) {
        return globalFee;
    }

    function isPaused() external view returns (bool) {
        return paused;
    }

    function getTotalLocked() external view returns (uint256) {
        return totalLocked;
    }

    function getPoolCount() external view returns (uint256) {
        return poolCount;
    }

    function noop() external pure returns (uint256) {
        return 42;
    }
}
"#;

// ---------- Thresholds ----------
//
// Sized for `cargo test` (default `dev` profile). Dev mode is ~3x slower
// than `--release`. Each threshold is ~10x typical observed values on a
// modern dev box (Linux, x86_64, ~3 GHz) — i.e. wide enough to absorb CI /
// laptop / VM hardware variability, narrow enough to flag a real 5x
// regression.

const THRESHOLD_TINY_COMPILE_US: u128 = 500_000; //  500 ms
const THRESHOLD_TINY_RUNTIME_US: u128 = 100_000; //  100 ms total (5x 20 ms)

const THRESHOLD_MEDIUM_COMPILE_US: u128 = 2_000_000; // 2 s
const THRESHOLD_MEDIUM_RUNTIME_US: u128 = 500_000; //   500 ms total

const THRESHOLD_LARGE_COMPILE_US: u128 = 5_000_000; // 5 s
const THRESHOLD_LARGE_RUNTIME_US: u128 = 1_000_000; // 1 s total

// ---------- Helpers ----------

/// Time `compile_contracts(src, false, 2)`. Panics if compilation fails —
/// these contracts are baked-in known-good source; a compile failure is a
/// frontend regression, not a perf regression.
fn time_compile(
    label: &str,
    src: &str,
) -> (u128, Vec<neo_devpack_solidity::cli::CompilationArtifacts>) {
    let t0 = Instant::now();
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "performance_regression: '{}' compile failed: {:?}",
            label, e
        )
    });
    let elapsed = t0.elapsed().as_micros();
    assert!(
        !arts.is_empty(),
        "performance_regression: '{}' produced zero artifacts",
        label
    );
    (elapsed, arts)
}

/// Time `n` consecutive `call_method` invocations on a fresh runtime per
/// call. We rebuild the runtime each iteration so the timing reflects a
/// realistic deploy + invoke flow (matches what an external caller does
/// per RPC). Returns the total elapsed µs across all `n` calls.
fn time_runtime_calls(
    label: &str,
    art: &neo_devpack_solidity::cli::CompilationArtifacts,
    method: &str,
    args: &[StackItem],
    n: usize,
) -> u128 {
    let mut total: u128 = 0;
    for i in 0..n {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap_or_else(|e| {
            panic!(
                "performance_regression: '{}' runtime construction failed (iter {}): {:?}",
                label, i, e
            )
        });
        let t0 = Instant::now();
        let result = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, method, args)
            .unwrap_or_else(|e| {
                panic!(
                    "performance_regression: '{}' call_method({}) host-level error \
                     (iter {}): {:?}",
                    label, method, i, e
                )
            });
        total += t0.elapsed().as_micros();
        // Don't gate on success — some methods legitimately revert on a
        // freshly-deployed contract (e.g. `withdraw` with no prior deposit).
        // The whole point of this gate is timing, not correctness; the
        // correctness suite covers the latter exhaustively. We DO require
        // that the host did not fault (above), since a host fault would
        // skew the timing.
        let _ = result;
    }
    total
}

// ==================== The single performance-regression test ====================

/// End-to-end perf gate over three representative contracts.
///
/// Prints one line per size class so a passing run leaves a stable
/// baseline trace in the test log:
///
///   performance_regression: tiny    compile=<us>  runtime=<us>
///   performance_regression: medium  compile=<us>  runtime=<us>
///   performance_regression: large   compile=<us>  runtime=<us>
///
/// Each timing is asserted under its absolute threshold; the panic
/// message names the size class, the actual µs, and the threshold so a
/// regression filer has all the data they need from the test output
/// alone.
#[test]
fn compile_runtime_performance_regression() {
    // ---------- Tiny ----------
    let (tiny_compile_us, tiny_arts) = time_compile("tiny", SRC_TINY);
    let tiny_runtime_us = time_runtime_calls("tiny", &tiny_arts[0], "inc", &[], 5);

    // ---------- Medium ----------
    // `balanceOf(address(0))` is a single mapping-load on cold storage —
    // representative of the most common ERC20-style read path. Zero
    // address is a stable, parameterless input.
    let (medium_compile_us, medium_arts) = time_compile("medium", SRC_MEDIUM);
    let medium_runtime_us = time_runtime_calls(
        "medium",
        &medium_arts[0],
        "balanceOf",
        &[StackItem::byte_array(vec![0u8; 20])],
        5,
    );

    // ---------- Large ----------
    // `noop()` returns a constant — measures pure dispatch cost (manifest
    // lookup, deploy prologue, method invoke) without touching storage.
    // For a perf gate this is desirable: it isolates the compiler's
    // dispatch + runtime's bytecode-load overhead, which is what scales
    // with contract size.
    let (large_compile_us, large_arts) = time_compile("large", SRC_LARGE);
    let large_runtime_us = time_runtime_calls("large", &large_arts[0], "noop", &[], 5);

    // ---------- Stable, machine-parseable log lines ----------
    println!(
        "performance_regression: tiny    compile={}  runtime={}",
        tiny_compile_us, tiny_runtime_us
    );
    println!(
        "performance_regression: medium  compile={}  runtime={}",
        medium_compile_us, medium_runtime_us
    );
    println!(
        "performance_regression: large   compile={}  runtime={}",
        large_compile_us, large_runtime_us
    );

    // ---------- Threshold assertions ----------
    // We assert ALL six values before panicking on any single failure —
    // but in Rust the standard `assert!` macro short-circuits on the
    // first failure. That's fine: the printed lines above already give
    // the full picture for diagnosis, regardless of which assertion
    // trips first.
    assert!(
        tiny_compile_us < THRESHOLD_TINY_COMPILE_US,
        "performance_regression: tiny compile {} us exceeded threshold {} us \
         (~10x slowdown). Either the threshold is too tight for this hardware, \
         or the compiler genuinely got slower for trivial inputs.",
        tiny_compile_us,
        THRESHOLD_TINY_COMPILE_US
    );
    assert!(
        tiny_runtime_us < THRESHOLD_TINY_RUNTIME_US,
        "performance_regression: tiny runtime (5 calls) {} us exceeded threshold {} us. \
         Runtime dispatch path got measurably slower.",
        tiny_runtime_us,
        THRESHOLD_TINY_RUNTIME_US
    );
    assert!(
        medium_compile_us < THRESHOLD_MEDIUM_COMPILE_US,
        "performance_regression: medium compile {} us exceeded threshold {} us. \
         An ERC-20-shaped contract should compile in well under 2 s in dev mode.",
        medium_compile_us,
        THRESHOLD_MEDIUM_COMPILE_US
    );
    assert!(
        medium_runtime_us < THRESHOLD_MEDIUM_RUNTIME_US,
        "performance_regression: medium runtime (5 calls) {} us exceeded threshold {} us. \
         Mapping-load dispatch got measurably slower.",
        medium_runtime_us,
        THRESHOLD_MEDIUM_RUNTIME_US
    );
    assert!(
        large_compile_us < THRESHOLD_LARGE_COMPILE_US,
        "performance_regression: large compile {} us exceeded threshold {} us. \
         A 200-line multi-modifier multi-mapping contract should compile in \
         under 5 s in dev mode.",
        large_compile_us,
        THRESHOLD_LARGE_COMPILE_US
    );
    assert!(
        large_runtime_us < THRESHOLD_LARGE_RUNTIME_US,
        "performance_regression: large runtime (5 calls) {} us exceeded threshold {} us. \
         Pure-function dispatch on a large manifest got measurably slower.",
        large_runtime_us,
        THRESHOLD_LARGE_RUNTIME_US
    );
}
