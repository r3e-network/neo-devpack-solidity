//! Regression tests for internal function-pointer lowering (Solidity
//! `function(...) internal` type variables — Task #186).
//!
//! These call through a code POINTER via NeoVM `CALLA`. Real NeoVM's `CALLA`
//! requires a `Pointer` produced by `PUSHA` (opcode 0x0A, a signed offset
//! relative to the opcode); the previous lowering pushed the target as a bare
//! `PUSHINT32` Integer (0x02), which faults on-chain ("not a Pointer") even
//! though the local runtime — which models `CALLA` as popping an integer
//! position — masked it. These tests pin both the runtime behavior and the
//! conformant opcode shape.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

// Passing the function as a PARAMETER forces an indirect call (`CALLA`): the
// callee `apply_` cannot statically resolve which function `fn` refers to.
const FN_PTR_SRC: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function add(uint256 a, uint256 b) internal pure returns (uint256) { return a + b; }
    function mul(uint256 a, uint256 b) internal pure returns (uint256) { return a * b; }
    function apply_(function(uint256, uint256) internal pure returns (uint256) fn, uint256 x, uint256 y)
        internal pure returns (uint256) { return fn(x, y); }
    function run() public pure returns (uint256) {
        return apply_(add, 20, 22) + apply_(mul, 6, 7);   // 42 + 42 = 84
    }
}"#;

#[test]
fn internal_function_pointer_executes_correctly() {
    let arts = compile_contracts(FN_PTR_SRC, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let res = rt.execute(&art.bytecode, &[]).expect("host-level execute");
    assert!(
        res.success,
        "fn-pointer run() must succeed; exc={:?}",
        res.exception
    );
    let mut v: u128 = 0;
    for (i, b) in res.return_data.iter().enumerate().take(16) {
        v |= (*b as u128) << (8 * i);
    }
    assert_eq!(
        v, 84,
        "add then mul through a function pointer must yield 84"
    );
}

// A function-pointer LOCAL (assigned from a function reference) must dispatch
// through CALLA too. Previously these silently returned 0: only PARAMETERS got
// a function-pointer binding, so `f(args)` fell through to the compatibility
// path that drops the arguments and pushes 0.
const FN_PTR_LOCAL_SRC: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function add(uint256 a, uint256 b) internal pure returns (uint256) { return a + b; }
    function mul(uint256 a, uint256 b) internal pure returns (uint256) { return a * b; }
    function run() public pure returns (uint256) {
        function(uint256, uint256) internal pure returns (uint256) f = add;
        uint256 s = f(20, 22);       // 42 via the local pointer
        f = mul;                     // reassign the pointer
        return s + f(6, 7);          // 42 + 42 = 84
    }
}"#;

#[test]
fn function_pointer_local_executes_correctly() {
    let arts = compile_contracts(FN_PTR_LOCAL_SRC, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let res = rt.execute(&art.bytecode, &[]).expect("host-level execute");
    assert!(
        res.success,
        "fn-pointer-local run() must succeed; exc={:?}",
        res.exception
    );
    let mut v: u128 = 0;
    for (i, b) in res.return_data.iter().enumerate().take(16) {
        v |= (*b as u128) << (8 * i);
    }
    assert_eq!(
        v, 84,
        "function-pointer LOCAL must call add then mul (84), not silently return 0"
    );
}

// A zero-initialized (never-assigned) internal function-pointer local must
// REVERT when called — Solidity raises Panic(0x51). Previously this silently
// returned 0 (the compatibility fallback). Now the local dispatches through
// CALLA, and CALLA on the Null slot faults cleanly instead of executing
// arbitrary code or returning a bogus value.
#[test]
fn uninitialized_function_pointer_local_reverts() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function run() public pure returns (uint256) {
        function(uint256) internal pure returns (uint256) f;
        return f(5);
    }
}"#;
    let arts = compile_contracts(src, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let res = rt.execute(&art.bytecode, &[]).expect("host-level execute");
    assert!(
        !res.success,
        "calling a zero-initialized function pointer must revert, not silently return 0"
    );
}

#[test]
fn function_pointer_uses_pusha_and_calla_not_pushint32() {
    let arts = compile_contracts(FN_PTR_SRC, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let bc = &art.bytecode;
    // CALLA (0x36) is emitted for the indirect call.
    assert!(
        bc.contains(&0x36),
        "function-pointer call must emit CALLA (0x36)"
    );
    // PUSHA (0x0A) is emitted to materialize the code pointer that CALLA consumes.
    // On real NeoVM CALLA faults unless its operand is a Pointer from PUSHA.
    assert!(
        bc.contains(&0x0A),
        "function-pointer offset must be pushed via PUSHA (0x0A), not PUSHINT32 (0x02)"
    );
}
