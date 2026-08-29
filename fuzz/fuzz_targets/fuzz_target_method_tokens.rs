//! Fuzz target: NeoVM CALLT (opcode 0x37) dispatch under random
//! method-token tables.
//!
//! Why: CALLT consumes a 16-bit operand indexing into a NEF method-token
//! table. Real contracts only ever use small, well-formed token tables
//! (StdLib, CryptoLib hashes), but an attacker-supplied NEF could feed
//! arbitrary tokens. The default `fuzz_target_runtime_exec` target calls
//! `NeoRuntime::execute` with an EMPTY token slice, so any CALLT in its
//! random bytecode immediately bails on "token index N out of bounds".
//! That short-circuits the dispatch path we care about — the actual
//! token-resolution logic, the `invoke_native_contract` shim, and the
//! `arg_count`/`has_return_value` post-pop bookkeeping in
//! `src/runtime/execution/instruction/flow/calls.rs`.
//!
//! Invariant under test: `NeoRuntime::execute_with_tokens(bc, &[], tokens)`
//! is total at the host level — for any `(bc, tokens)` pair the runtime
//! returns either `Ok(_)` with a runtime exception or an `Err(_)` runtime
//! error, and never produces a Rust-side panic, OOM, or aborts the host
//! process.
//!
//! Termination: same bound as `fuzz_target_runtime_exec` — `gas_limit =
//! 100_000` keeps each iteration under ~1ms even for tight `JMP -1` loops.

#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use neo_devpack_solidity::neo::MethodToken;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

/// Cap on the bytecode prefix length. Plenty of room for a handful of
/// stack-priming opcodes plus a CALLT (3 bytes: 0x37 + u16 LE token index)
/// and a HALT-style terminator, but small enough that random PUSHDATA
/// truncation paths still dominate iteration time.
const MAX_BYTECODE_LEN: usize = 32;

/// Lower / upper bounds on the random method-token table size. At least
/// one token guarantees the CALLT operand has a chance of landing on a
/// valid slot; eight is more than enough variation without ballooning
/// the input corpus.
const MIN_TOKENS: usize = 1;
const MAX_TOKENS: usize = 8;

/// Cap on random method-name length. Below the Neo spec ceiling of 32
/// bytes (`MAX_TOKEN_METHOD_LENGTH`), so any name we generate is
/// well-formed; matches the task's 1..=16 spec.
const MAX_METHOD_NAME_LEN: usize = 16;

/// Cap on parameter counts we synthesise. The CALLT handler pops
/// `parameters_count` items off the stack, so larger values almost always
/// hit "insufficient stack items"; 0..=8 keeps the popping path
/// reachable without burning fuzz iterations on a guaranteed-fail branch.
const MAX_PARAM_COUNT: u16 = 8;

fn fuzz_config() -> RuntimeConfig {
    let mut cfg = RuntimeConfig::default();
    cfg.gas_limit = 100_000;
    cfg.memory_limit = 256 * 1024;
    cfg.storage_limit = 256 * 1024;
    cfg
}

/// Build a random `MethodToken`. Each field is chosen independently so
/// that the dispatch path's per-field branches (param-count loop,
/// `has_return_value` post-push, `call_flags` checks in
/// `invoke_native_contract`) all see broad value coverage.
fn arbitrary_token(u: &mut Unstructured) -> arbitrary::Result<MethodToken> {
    let mut hash = [0u8; 20];
    u.fill_buffer(&mut hash)?;

    // ASCII method name, 1..=16 bytes. Restricting to ASCII keeps the
    // varbytes serializer happy and avoids invalid UTF-8 panics if any
    // downstream code formats the name; the runtime dispatch only
    // compares bytes so the alphabet doesn't matter for coverage.
    let name_len = u.int_in_range(1..=MAX_METHOD_NAME_LEN)?;
    let mut name_bytes = Vec::with_capacity(name_len);
    for _ in 0..name_len {
        // Printable ASCII (0x20..=0x7e) — keeps `String::from_utf8`
        // infallible and round-trippable through the manifest.
        name_bytes.push(u.int_in_range::<u8>(0x20..=0x7e)?);
    }
    let method =
        String::from_utf8(name_bytes).expect("printable-ASCII bytes are always valid UTF-8");

    let parameters_count: u16 = u.int_in_range(0..=MAX_PARAM_COUNT)?;
    let has_return_value: bool = bool::arbitrary(u)?;
    let call_flags: u8 = u8::arbitrary(u)?;

    Ok(MethodToken {
        hash,
        method,
        parameters_count,
        has_return_value,
        call_flags,
    })
}

/// Synthesise a small random bytecode prefix that *probably* contains a
/// CALLT. The deliberately-skewed CALLT-emit probability (~50%) lets the
/// fuzzer also explore non-CALLT prefixes (raw random opcodes), which
/// keeps the corpus useful for the surrounding interpreter dispatch even
/// when libFuzzer's mutation hits CALLT-free regions.
fn arbitrary_bytecode(
    u: &mut Unstructured,
    token_count: usize,
) -> arbitrary::Result<Vec<u8>> {
    let len = u.int_in_range(3..=MAX_BYTECODE_LEN)?;
    let mut bytecode = Vec::with_capacity(len);

    // Emit some random preamble opcodes (may push items onto the stack
    // so the eventual CALLT has args to pop). Stop one CALLT-instruction
    // (3 bytes) short of the cap so we always have room for the CALLT
    // tail when we choose to emit one.
    let preamble_len = u.int_in_range(0..=len.saturating_sub(3))?;
    for _ in 0..preamble_len {
        bytecode.push(u8::arbitrary(u)?);
    }

    // Emit a CALLT roughly half the time — the rest of the time the
    // bytecode is whatever random bytes the fuzzer generated, which
    // gives us coverage of the dispatch loop's non-CALLT branches.
    if bool::arbitrary(u)? {
        bytecode.push(0x37); // CALLT
        // 16-bit operand. ~50/50 between "in-range index" (covers the
        // happy-path token resolution) and "fully-random index" (covers
        // the out-of-bounds rejection branch). We cannot just always
        // emit `0..token_count` because the out-of-bounds path is also
        // load-bearing — it's the attacker-supplied-NEF case.
        let token_index: u16 = if bool::arbitrary(u)? && token_count > 0 {
            u.int_in_range(0..=(token_count as u16 - 1))?
        } else {
            u16::arbitrary(u)?
        };
        bytecode.extend_from_slice(&token_index.to_le_bytes());
    }

    // Pad with random bytes up to the chosen length so the CALLT is
    // followed by additional dispatch (RET fall-throughs, more random
    // opcodes, etc.) — exercises the post-CALLT `instruction_pointer +=
    // 3` advance and any continuation-path bugs.
    while bytecode.len() < len {
        bytecode.push(u8::arbitrary(u)?);
    }

    Ok(bytecode)
}

fuzz_target!(|data: &[u8]| {
    let _ = std::panic::catch_unwind(|| {
        let mut u = Unstructured::new(data);

        // Build the method-token table first so the bytecode generator
        // knows how many slots are addressable when it picks an
        // in-range index. Bail cleanly if `arbitrary` runs out of
        // entropy — libFuzzer will retry with a longer corpus entry.
        let token_count = match u.int_in_range(MIN_TOKENS..=MAX_TOKENS) {
            Ok(n) => n,
            Err(_) => return,
        };
        let mut tokens = Vec::with_capacity(token_count);
        for _ in 0..token_count {
            match arbitrary_token(&mut u) {
                Ok(t) => tokens.push(t),
                Err(_) => return,
            }
        }

        let bytecode = match arbitrary_bytecode(&mut u, tokens.len()) {
            Ok(bc) => bc,
            Err(_) => return,
        };

        let mut rt = match NeoRuntime::new(fuzz_config()) {
            Ok(rt) => rt,
            Err(_) => return,
        };

        // Drive the bytecode AND the token table into the
        // CALLT-aware execution path. Any host-level crash (rust panic,
        // OOM, runaway loop) is a bug — graceful runtime exceptions and
        // `Err(RuntimeError::_)` are both expected outcomes.
        let _ = rt.execute_with_tokens(&bytecode, &[], &tokens);
    });
});
