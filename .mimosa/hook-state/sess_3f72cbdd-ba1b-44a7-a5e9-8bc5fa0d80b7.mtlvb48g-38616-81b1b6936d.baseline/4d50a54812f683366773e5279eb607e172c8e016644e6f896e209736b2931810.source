//! Criterion benchmarks for `compile_contracts`.
//!
//! Establishes the first compile-time measurement baseline for the compiler,
//! covering simple and complex contracts across optimizer levels O0/O2/O3.
//!
//! Run quick mode during development:
//!   cargo bench -- --warm-up-time 1 --measurement-time 2 --sample-size 10
//! Full statistical run (default criterion settings):
//!   cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neo_devpack_solidity::cli::compile_contracts;

/// Small single-contract source. Exercises the core pipeline (parse → IR →
/// optimize → codegen) without inheritance, events, or struct storage.
const SIMPLE_TOKEN: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract SimpleToken {
    mapping(address => uint256) public balances;
    uint256 public totalSupply;
    function transfer(address to, uint256 amount) public returns (bool) {
        require(balances[msg.sender] >= amount, "insufficient");
        balances[msg.sender] -= amount;
        balances[to] += amount;
        return true;
    }
}
"#;

/// Moderately-sized inline contract used as a fallback when the external
/// `examples/ERC20Token.sol` either cannot be read or does not compile in a
/// standalone single-file context. Keeps the benchmark self-contained and
/// panic-free regardless of the surrounding repo layout.
const COMPLEX_FALLBACK: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract ComplexToken {
    struct Allowance { address spender; uint256 amount; }
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    mapping(address => Allowance[]) private _explicitAllowances;
    uint256 private _totalSupply;
    uint8 private _decimals;
    string private _name;
    string private _symbol;
    address private _owner;
    bool private _paused;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Pause();
    event Unpause();

    modifier onlyOwner() {
        require(msg.sender == _owner, "not owner");
        _;
    }

    constructor() {
        _name = "Complex";
        _symbol = "CPX";
        _decimals = 18;
        _owner = msg.sender;
        _totalSupply = 1_000_000 * 10 ** 18;
        _balances[msg.sender] = _totalSupply;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        require(!_paused, "paused");
        require(_balances[msg.sender] >= amount, "insufficient");
        _balances[msg.sender] -= amount;
        _balances[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        _allowances[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        require(!_paused, "paused");
        require(_balances[from] >= amount, "insufficient");
        require(_allowances[from][msg.sender] >= amount, "allowance");
        _balances[from] -= amount;
        _balances[to] += amount;
        _allowances[from][msg.sender] -= amount;
        emit Transfer(from, to, amount);
        return true;
    }

    function pause() public onlyOwner { _paused = true; emit Pause(); }
    function unpause() public onlyOwner { _paused = false; emit Unpause(); }

    function batchTransfer(address[] calldata recipients, uint256 amount) public returns (bool) {
        require(!_paused, "paused");
        for (uint256 i = 0; i < recipients.length; i++) {
            require(_balances[msg.sender] >= amount, "insufficient");
            _balances[msg.sender] -= amount;
            _balances[recipients[i]] += amount;
            emit Transfer(msg.sender, recipients[i], amount);
        }
        return true;
    }
}
"#;

/// Resolve the "complex" benchmark source.
///
/// Prefers `examples/ERC20Token.sol` when it both reads and compiles
/// successfully as a single source string (real-world representative weight);
/// otherwise falls back to the self-contained inline `COMPLEX_FALLBACK`. The
/// probe compile runs once at setup, so a failing external file never causes a
/// benchmark panic.
fn resolve_complex_source() -> String {
    std::fs::read_to_string("examples/ERC20Token.sol")
        .ok()
        .filter(|src| compile_contracts(src, false, 2).is_ok())
        .unwrap_or_else(|| COMPLEX_FALLBACK.to_string())
}

fn benchmark_compilation(c: &mut Criterion) {
    let simple = SIMPLE_TOKEN;
    let complex = resolve_complex_source();

    let mut group = c.benchmark_group("compile");

    group.bench_function("simple_O0", |b| {
        b.iter(|| {
            compile_contracts(black_box(simple), false, 0).unwrap();
        })
    });

    group.bench_function("simple_O2", |b| {
        b.iter(|| {
            compile_contracts(black_box(simple), false, 2).unwrap();
        })
    });

    group.bench_function("simple_O3", |b| {
        b.iter(|| {
            compile_contracts(black_box(simple), false, 3).unwrap();
        })
    });

    group.bench_function("complex_O2", |b| {
        b.iter(|| {
            compile_contracts(black_box(&complex), false, 2).unwrap();
        })
    });

    group.bench_function("complex_O3", |b| {
        b.iter(|| {
            compile_contracts(black_box(&complex), false, 3).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_compilation);
criterion_main!(benches);
