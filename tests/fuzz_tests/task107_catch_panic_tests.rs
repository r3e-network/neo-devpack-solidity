//! Task #107 and Task #108 canonical Panic envelope round-trip harnesses.
//! Extracted from the run of `#[test]` functions that sat between Batch #44
//! and Batch #45 in the pre-split `tests/fuzz_tests.rs`. Contents unchanged.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_match)]
#![allow(clippy::partialeq_to_none)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Task #107 — Canonical Panic envelope round-trip ====================
//
// Task #107 migrated every Panic emission site in the IR from the legacy
// `PushLiteral(ByteString("Panic: 0xNN"))` + `Throw` shape to the canonical
// EVM envelope `keccak256("Panic(uint256)")[..4] || abi.encode(code)` via the
// shared `emit_panic(code)` helper in `src/ir/build/panic.rs`. Each test
// below drives one panic code end-to-end through `try { … } catch Panic(uint
// code) { return code; }` and asserts the caught `code` equals the expected
// Solidity-spec value, which is only possible if the canonical envelope is
// on `ExecutionResult.return_data`. A regression where the envelope is
// missing (legacy ByteString shape) or the code is wrong would surface as a
// mismatch here.

// Panic 0x11 — checked arithmetic overflow.  `2 ** 256 - 1 + 1` without
// `unchecked { }` must revert with Panic(0x11).
#[test]
fn task107_catch_panic_0x11_arith_overflow() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function willPanic() external pure returns (uint) {
        uint x = type(uint256).max;
        return x + 1;
    }
    function handle() external returns (uint) {
        try this.willPanic() returns (uint) { return 0xfe; }
        catch Panic(uint code) { return code; }
        catch Error(string memory) { return 0xfd; }
        catch (bytes memory) { return 0xfc; }
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("107 0x11 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("107 0x11 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "handle",
            &[] as &[StackItem],
        )
        .expect("107 0x11 call");
    assert!(
        r.success,
        "107 0x11 handle() must succeed (catch absorbed panic); exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        decode_uint_le(&r.return_data),
        num_bigint::BigUint::from(0x11u64),
        "107 0x11 handle() must return Panic 0x11 via `catch Panic(uint)`; \
         got rd_hex={} (0xfe=try-success, 0xfd=Error path, 0xfc=bytes path)",
        hex::encode(&r.return_data)
    );
}

// Panic 0x21 — enum-cast out-of-range.  `Status(uint8(3))` with only 3
// variants (0,1,2) must revert with Panic(0x21).
#[test]
fn task107_catch_panic_0x21_enum_cast() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    enum Status { Idle, Running, Done }
    function willPanic() external pure returns (Status) {
        uint8 bad = 3;
        return Status(bad);
    }
    function handle() external returns (uint) {
        try this.willPanic() returns (Status) { return 0xfe; }
        catch Panic(uint code) { return code; }
        catch Error(string memory) { return 0xfd; }
        catch (bytes memory) { return 0xfc; }
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("107 0x21 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("107 0x21 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "handle",
            &[] as &[StackItem],
        )
        .expect("107 0x21 call");
    assert!(
        r.success,
        "107 0x21 handle() must succeed (catch absorbed panic); exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        decode_uint_le(&r.return_data),
        num_bigint::BigUint::from(0x21u64),
        "107 0x21 handle() must return Panic 0x21 via `catch Panic(uint)`; \
         got rd_hex={} (0xfe=try-success, 0xfd=Error path, 0xfc=bytes path)",
        hex::encode(&r.return_data)
    );
}

// Panic 0x32 — array-index OOB.  `arr[5]` on a 3-length array must revert
// with Panic(0x32). Routes through the new IR-level bounds guard added by
// Task #107 on `lower_array_subscript_expression`.
#[test]
fn task107_catch_panic_0x32_array_oob() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function willPanic() external pure returns (uint) {
        uint[] memory arr = new uint[](3);
        arr[0] = 10; arr[1] = 20; arr[2] = 30;
        return arr[5];
    }
    function handle() external returns (uint) {
        try this.willPanic() returns (uint) { return 0xfe; }
        catch Panic(uint code) { return code; }
        catch Error(string memory) { return 0xfd; }
        catch (bytes memory) { return 0xfc; }
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("107 0x32 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("107 0x32 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "handle",
            &[] as &[StackItem],
        )
        .expect("107 0x32 call");
    assert!(
        r.success,
        "107 0x32 handle() must succeed (catch absorbed panic); exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        decode_uint_le(&r.return_data),
        num_bigint::BigUint::from(0x32u64),
        "107 0x32 handle() must return Panic 0x32 via `catch Panic(uint)`; \
         got rd_hex={} (0xfe=try-success, 0xfd=Error path, 0xfc=bytes path)",
        hex::encode(&r.return_data)
    );
}

// Panic 0x41 — abi.decode short buffer.  `abi.decode(hex"00", (uint256))`
// must revert with Panic(0x41) because the buffer is 1 byte but the
// expected static layout requires 32 bytes.
#[test]
fn task107_catch_panic_0x41_abi_decode_short() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function willPanic() external pure returns (uint) {
        bytes memory short = hex"00";
        return abi.decode(short, (uint256));
    }
    function handle() external returns (uint) {
        try this.willPanic() returns (uint) { return 0xfe; }
        catch Panic(uint code) { return code; }
        catch Error(string memory) { return 0xfd; }
        catch (bytes memory) { return 0xfc; }
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("107 0x41 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("107 0x41 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "handle",
            &[] as &[StackItem],
        )
        .expect("107 0x41 call");
    assert!(
        r.success,
        "107 0x41 handle() must succeed (catch absorbed panic); exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        decode_uint_le(&r.return_data),
        num_bigint::BigUint::from(0x41u64),
        "107 0x41 handle() must return Panic 0x41 via `catch Panic(uint)`; \
         got rd_hex={} (0xfe=try-success, 0xfd=Error path, 0xfc=bytes path)",
        hex::encode(&r.return_data)
    );
}

// ==================== Task #108 — Runtime-side INT256_MIN / -1 Panic envelope ====================
//
// Task #108 migrates the last runtime-side Panic emission site that still
// pushed a raw `"Panic: 0x11"` ByteString message:
// `src/runtime/execution/helpers/arithmetic/basic_ops.rs` (the
// `div_stack_items_wide` INT256_MIN / -1 signed-div overflow case). The new
// shape populates `ExecutionContext::revert_payload` with the canonical
// 36-byte envelope:
//   `keccak256("Panic(uint256)")[..4] || abi.encode(0x11)`
//     = `0x4e487b71` || `[0x00*31, 0x11]`
// so `try { … } catch Panic(uint code) { … }` can decode code = 0x11. Without
// this migration, the legacy UTF-8 bytes `"Panic: 0x11"` land on the catch
// handler's stack and `abi.decode` fails — the `catch Panic(uint)` arm never
// binds and the caller sees the `catch (bytes)` fallback instead.
//
// Repro: `int256 x = type(int256).min; return x / int256(-1);` with both
// operands forced to the wide BigInt path (here via `int256` variables, which
// the runtime handles as 32-byte signed-LE ByteArrays → `cmp_needs_bigint_path`
// routes through `div_stack_items_wide` where the guard lives at
// `basic_ops.rs:306`).
#[test]
fn task108_catch_panic_int256_min_div_neg_one_routes_canonical() {
    use neo_devpack_solidity::runtime::types::StackItem;
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function willPanic() external pure returns (int256) {
        int256 x = type(int256).min;
        int256 y = int256(-1);
        return x / y;
    }
    function handle() external returns (uint) {
        try this.willPanic() returns (int256) { return 0xfe; }
        catch Panic(uint code) { return code; }
        catch Error(string memory) { return 0xfd; }
        catch (bytes memory) { return 0xfc; }
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("108 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("108 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "handle",
            &[] as &[StackItem],
        )
        .expect("108 call");
    assert!(
        r.success,
        "108 handle() must succeed (catch absorbed panic); exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        decode_uint_le(&r.return_data),
        num_bigint::BigUint::from(0x11u64),
        "108 handle() must return Panic 0x11 via `catch Panic(uint)` for \
         INT256_MIN / -1; got rd_hex={} (0xfe=try-success, 0xfd=Error path, \
         0xfc=bytes path — the legacy \"Panic: 0x11\" ByteString shape would \
         route through 0xfc since UTF-8 bytes don't decode as Panic(uint256))",
        hex::encode(&r.return_data)
    );
}
