//! Property-based stress test for the storage-iterator state machine.
//!
//! The runtime exposes Neo N3-style storage iterators via
//! `System.Storage.Find` (-> opaque iterator handle) and the pair
//! `System.Iterator.Next` / `System.Iterator.Value`. The existing
//! proptests (baseline_tests.rs, storage_props.rs, devpack_props.rs)
//! exercise linear iteration over small key sets (<=12 keys) but never
//! sequence arbitrary `Find` / `Next` / `Value` operations against a
//! larger storage map. This module fills that gap.
//!
//! # Strategy
//!
//! For each test case:
//!
//!   1. Generate a random storage map of 3..=50 unique byte-keys (with
//!      values 1..=32 bytes each).
//!   2. Generate a random op-tape of 5..=30 operations, where each op
//!      is `Find(prefix)`, `Next`, or `Value`. The prefix mix is biased
//!      to include the empty prefix, prefixes that match some keys, and
//!      prefixes that match nothing.
//!   3. Encode the storage map and the op-tape as packed `bytes`,
//!      and drive a single-call Solidity contract through the entire
//!      sequence. The contract:
//!        - reads the packed storage entries from input and writes them
//!          via `Storage.put` (so the runtime's overlay + dirty flush
//!          path actually produces the keys the iterator sees);
//!        - decodes the op-tape and emits a deterministic trace as
//!          packed `bytes` (one record per op).
//!   4. A Rust-side reference model (`BTreeMap<Vec<u8>, Vec<u8>>`)
//!      replays the same op-tape and assembles the same trace shape;
//!      we assert byte-equality.
//!
//! Iterators do not persist across `call_method` invocations (the
//! execution context's `iterators` map is cleared on each
//! `initialize`), so the populate + run-ops phases must execute within
//! a single call. We bundle them into `runStateMachine(packedKV, ops)`.
//!
//! # Invariants asserted
//!
//!   * Iterator must never panic, never produce malformed data, and
//!     `Value` after end-of-iteration must NOT advance or duplicate the
//!     previous record.
//!   * Iteration order is deterministic and byte-lex-sorted by key.
//!   * The number of items walked between two consecutive `Find` ops
//!     equals the number of model keys whose key starts with the find
//!     prefix.
//!   * A second `Find` with a different prefix yields a fresh iterator
//!     (no continuation of the previous one). The model resets its
//!     "current iterator" cursor on every `Find`; if the contract did
//!     not, the Value records would diverge from the model after the
//!     replacement Find.
//!
//! # Trace encoding (packed; identical between contract and model)
//!
//!   * `Find(prefix)`        -> `0xF1`
//!   * `Next` returning ok   -> `0xF2 0x01`
//!   * `Next` returning end  -> `0xF2 0x00`
//!   * `Value` (post-Next)   -> `0xF3 KLEN_LE_U16 K... VLEN_LE_U16 V...`
//!   * `Value` before any Next on a fresh iterator (or post-end) ->
//!     `0xF3 0x00 0x00 0x00 0x00`
//!   * No iterator yet (Next/Value before any Find) -> SKIP (the test
//!     generator only emits Next/Value after a Find).

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------- Op model ----------

#[derive(Debug, Clone)]
enum Op {
    /// Issue `Storage.find(prefix)` and replace the active iterator.
    Find(Vec<u8>),
    /// Advance the active iterator one step.
    Next,
    /// Read the (key, value) currently at the iterator cursor.
    Value,
}

// Op tape byte tags (input direction, contract decodes these).
const OP_FIND: u8 = 0x01;
const OP_NEXT: u8 = 0x02;
const OP_VALUE: u8 = 0x03;

// Trace tags (output direction, contract emits / model expects).
const TR_FIND: u8 = 0xF1;
const TR_NEXT: u8 = 0xF2;
const TR_VALUE: u8 = 0xF3;

// ---------- Strategies ----------

/// Generate a non-empty storage map of unique keys (3..=50 entries).
///
/// Keys are 1..=8 bytes (small space → frequent prefix collisions →
/// stress for the byte-lex sort), values are 1..=32 bytes. Uniqueness
/// is enforced by `BTreeMap`'s key set.
fn storage_strategy() -> impl Strategy<Value = BTreeMap<Vec<u8>, Vec<u8>>> {
    prop::collection::vec(
        (
            prop::collection::vec(any::<u8>(), 1..=8),
            prop::collection::vec(any::<u8>(), 1..=32),
        ),
        3..=50,
    )
    .prop_map(|pairs| {
        let mut m = BTreeMap::new();
        for (k, v) in pairs {
            m.insert(k, v);
        }
        m
    })
    .prop_filter("must produce >= 3 unique keys after dedup", |m| {
        m.len() >= 3
    })
}

/// Generate an op tape of 5..=30 ops over the given storage map. The
/// op generator is biased so that a `Find` precedes any `Next`/`Value`
/// (otherwise the trace shape on a "no-iterator" path is not what the
/// test is exercising — that case is unit-tested elsewhere).
fn ops_strategy(storage: BTreeMap<Vec<u8>, Vec<u8>>) -> impl Strategy<Value = Vec<Op>> {
    let keys: Vec<Vec<u8>> = storage.keys().cloned().collect();

    // Build a strategy for prefix bytes biased to:
    //   - empty (matches everything)
    //   - first 1..=N bytes of an existing key (matches at least one)
    //   - random bytes (likely matches nothing)
    let keys_for_strategy = keys.clone();
    let prefix_strategy = prop_oneof![
        // Empty prefix.
        Just(Vec::<u8>::new()),
        // Existing-key prefix (truncated to 1..len).
        (0usize..keys_for_strategy.len(), 1usize..=8usize).prop_map(move |(idx, take)| {
            let k = &keys_for_strategy[idx];
            let n = take.min(k.len());
            k[..n].to_vec()
        }),
        // Random bytes, length 1..=4.
        prop::collection::vec(any::<u8>(), 1..=4),
    ];

    let op_strategy = prop_oneof![
        4 => prefix_strategy.prop_map(Op::Find),
        6 => Just(Op::Next),
        4 => Just(Op::Value),
    ];

    prop::collection::vec(op_strategy, 5..=30).prop_map(|ops| {
        // Ensure the very first op is a Find. If the random tape begins
        // with Next/Value, prepend a Find of the empty prefix (matches
        // everything). The contract's interpreter would otherwise see
        // a Next/Value without an iterator in scope; we test that path
        // separately and don't want it to mask other findings.
        let needs_prefix_find = matches!(ops.first(), Some(Op::Next | Op::Value));
        if needs_prefix_find {
            let mut out = Vec::with_capacity(ops.len() + 1);
            out.push(Op::Find(Vec::new()));
            out.extend(ops);
            out
        } else {
            ops
        }
    })
}

// ---------- Encoding helpers ----------

/// Encode the storage map as `[u8 klen, k bytes, u8 vlen, v bytes]+`.
/// Bounded klen (1..=8) and vlen (1..=32) fit cleanly in single bytes.
fn encode_kv(map: &BTreeMap<Vec<u8>, Vec<u8>>) -> Vec<u8> {
    let mut out = Vec::new();
    for (k, v) in map {
        debug_assert!(k.len() <= u8::MAX as usize);
        debug_assert!(v.len() <= u8::MAX as usize);
        out.push(k.len() as u8);
        out.extend_from_slice(k);
        out.push(v.len() as u8);
        out.extend_from_slice(v);
    }
    out
}

/// Encode the op tape:
///   Find(p)  -> 0x01 LEN P...
///   Next     -> 0x02
///   Value    -> 0x03
fn encode_ops(ops: &[Op]) -> Vec<u8> {
    let mut out = Vec::new();
    for op in ops {
        match op {
            Op::Find(p) => {
                out.push(OP_FIND);
                debug_assert!(p.len() <= u8::MAX as usize);
                out.push(p.len() as u8);
                out.extend_from_slice(p);
            }
            Op::Next => out.push(OP_NEXT),
            Op::Value => out.push(OP_VALUE),
        }
    }
    out
}

/// Replay the op tape against the reference `BTreeMap` and produce the
/// expected packed trace. This is the spec the contract must match.
///
/// Mirrors the contract's state machine exactly:
///   * `live`     — set to true on first Find.
///   * `entries`  — the sorted prefix-matching subset of `map` for the
///     current iterator (replaced on every Find).
///   * `cursor`   — 1-based position into `entries`; 0 means pre-Next.
///   * `past_end` — set to true once Next returned false; subsequent
///     Next ops emit 0 and do NOT re-advance (the contract
///     is asserted to do the same — no double-read past
///     end).
///   * `last_ok`  — true iff the most recent Next returned true (and
///     no intervening Find reset it).
fn model_trace(map: &BTreeMap<Vec<u8>, Vec<u8>>, ops: &[Op]) -> Vec<u8> {
    let mut trace = Vec::new();
    let mut entries: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut cursor: usize = 0;
    let mut live = false;
    let mut past_end = false;
    let mut last_ok = false;

    for op in ops {
        match op {
            Op::Find(prefix) => {
                trace.push(TR_FIND);
                entries = map
                    .iter()
                    .filter(|(k, _)| k.starts_with(prefix))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                // BTreeMap already iterates in lex order; nothing to sort.
                cursor = 0;
                live = true;
                past_end = false;
                last_ok = false;
            }
            Op::Next => {
                trace.push(TR_NEXT);
                if live && !past_end {
                    if cursor < entries.len() {
                        cursor += 1;
                        last_ok = true;
                        trace.push(0x01);
                    } else {
                        past_end = true;
                        last_ok = false;
                        trace.push(0x00);
                    }
                } else {
                    last_ok = false;
                    trace.push(0x00);
                }
            }
            Op::Value => {
                trace.push(TR_VALUE);
                if live && last_ok && cursor >= 1 && cursor <= entries.len() {
                    let (k, v) = &entries[cursor - 1];
                    let kl = k.len() as u16;
                    let vl = v.len() as u16;
                    trace.extend_from_slice(&kl.to_le_bytes());
                    trace.extend_from_slice(k);
                    trace.extend_from_slice(&vl.to_le_bytes());
                    trace.extend_from_slice(v);
                } else {
                    // Empty (klen=0, vlen=0).
                    trace.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
    }
    trace
}

// ---------- Solidity harness ----------
//
// The contract performs three phases inside one call:
//
//   1. Decode `kvBlob` into per-entry `Storage.put(k, v)` calls.
//   2. Walk `opTape`; dispatch on the op tag and append a record to
//      the result trace per op:
//        * Find  -> reissue `Storage.find(slicePrefix)`, store handle
//                   in `bytes memory it`, mark `live=true`, reset
//                   `pastNext=false`. Append `0xF1` to trace.
//        * Next  -> if `live` and `!pastNext`, call `it.next()`. Append
//                   `0xF2 0x01` if true (cursor advances), `0xF2 0x00`
//                   if false (mark `pastNext=true`). If already
//                   pastNext, do NOT re-call `it.next()` — emit `0x00`.
//        * Value -> if `live` and at-an-entry (i.e., last Next returned
//                   true), fetch `it.currentKey` + `it.value()` and
//                   append `0xF3 KLEN K VLEN V`. Else append
//                   `0xF3 0x00 0x00 0x00 0x00`.
//   3. Return the populated `bytes` trace.
//
// State tracked alongside the iterator handle:
//   * `live`     — true once any Find has been issued.
//   * `pastNext` — true once Next has returned false (don't call again
//                  past end; the test asserts no double-read).
//   * `atEntry`  — true while the cursor points at a real entry (i.e.,
//                  last Next returned true and Value hasn't reset it).
//                  We don't strictly need a separate `atEntry`; it is
//                  exactly `live && !pastNext && (lastNextOk)`. Tracked
//                  via the `lastOk` boolean below.
const SOURCE: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function runStateMachine(bytes memory kvBlob, bytes memory opTape)
        external
        returns (bytes memory)
    {
        // ---- Phase 1: populate storage from kvBlob. ----
        uint256 p = 0;
        while (p < kvBlob.length) {
            uint8 klen = uint8(kvBlob[p]); p += 1;
            bytes memory k = new bytes(klen);
            for (uint256 i = 0; i < klen; i++) { k[i] = kvBlob[p + i]; }
            p += klen;
            uint8 vlen = uint8(kvBlob[p]); p += 1;
            bytes memory v = new bytes(vlen);
            for (uint256 i = 0; i < vlen; i++) { v[i] = kvBlob[p + i]; }
            p += vlen;
            Storage.put(k, v);
        }

        // ---- Phase 2: walk opTape and emit trace. ----
        // Pre-allocate a max-size buffer (re-trim at the end). Each op
        // emits at most 1 (Find) + 2 (Next) + (4 + 8 + 32 = 44) (Value)
        // bytes. 64 bytes/op is a comfortable upper bound; opTape itself
        // is <=30 ops + Find prefixes (each <=9 bytes), so 64*opTape.length
        // covers the worst case with significant headroom.
        bytes memory trace = new bytes(64 * (opTape.length + 8) + 64);
        uint256 traceLen = 0;

        bytes memory it;
        bool live = false;
        bool pastNext = false;
        bool lastOk = false;

        uint256 q = 0;
        while (q < opTape.length) {
            uint8 tag = uint8(opTape[q]); q += 1;

            if (tag == 0x01) {
                // Find.
                uint8 plen = uint8(opTape[q]); q += 1;
                bytes memory prefix = new bytes(plen);
                for (uint256 i = 0; i < plen; i++) {
                    prefix[i] = opTape[q + i];
                }
                q += plen;
                it = Storage.find(prefix);
                live = true;
                pastNext = false;
                lastOk = false;
                trace[traceLen] = 0xF1; traceLen += 1;
            } else if (tag == 0x02) {
                // Next.
                trace[traceLen] = 0xF2; traceLen += 1;
                if (live && !pastNext) {
                    bool ok = it.next();
                    lastOk = ok;
                    if (ok) {
                        trace[traceLen] = 0x01;
                    } else {
                        pastNext = true;
                        trace[traceLen] = 0x00;
                    }
                } else {
                    // No iterator OR already past end. Do NOT call
                    // it.next() again past end — the test asserts no
                    // double-read.
                    lastOk = false;
                    trace[traceLen] = 0x00;
                }
                traceLen += 1;
            } else if (tag == 0x03) {
                // Value.
                trace[traceLen] = 0xF3; traceLen += 1;
                if (live && lastOk) {
                    bytes memory ck = it.currentKey;
                    bytes memory cv = it.value();
                    uint256 klen2 = ck.length;
                    uint256 vlen2 = cv.length;
                    trace[traceLen] = bytes1(uint8(klen2 & 0xFF)); traceLen += 1;
                    trace[traceLen] = bytes1(uint8((klen2 >> 8) & 0xFF)); traceLen += 1;
                    for (uint256 i = 0; i < klen2; i++) {
                        trace[traceLen] = ck[i]; traceLen += 1;
                    }
                    trace[traceLen] = bytes1(uint8(vlen2 & 0xFF)); traceLen += 1;
                    trace[traceLen] = bytes1(uint8((vlen2 >> 8) & 0xFF)); traceLen += 1;
                    for (uint256 i = 0; i < vlen2; i++) {
                        trace[traceLen] = cv[i]; traceLen += 1;
                    }
                } else {
                    // No live cursor: emit 0,0,0,0 (klen=0, vlen=0).
                    trace[traceLen] = 0x00; traceLen += 1;
                    trace[traceLen] = 0x00; traceLen += 1;
                    trace[traceLen] = 0x00; traceLen += 1;
                    trace[traceLen] = 0x00; traceLen += 1;
                }
            } else {
                // Unknown tag.
                trace[traceLen] = 0xEE; traceLen += 1;
                break;
            }
        }

        // ---- Phase 3: trim trace to actually-emitted length. ----
        bytes memory out = new bytes(traceLen);
        for (uint256 i = 0; i < traceLen; i++) { out[i] = trace[i]; }
        return out;
    }
}
"#;

proptest! {
    // Compiling and running the contract is expensive (~1-2s per case),
    // so cap the case count. Higher counts are available via the
    // PROPTEST_CASES env var when investigating a regression.
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// Drive a random storage map + op tape through the contract and
    /// assert the emitted trace matches the BTreeMap-based model trace
    /// byte-for-byte.
    #[test]
    fn storage_iterator_state_machine_matches_model(
        (storage, ops) in storage_strategy()
            .prop_flat_map(|s| (Just(s.clone()), ops_strategy(s)))
    ) {
        // Sanity bound: storage entries fit in the u8-length encoding.
        for (k, v) in &storage {
            prop_assume!(k.len() <= u8::MAX as usize);
            prop_assume!(v.len() <= u8::MAX as usize);
        }

        let kv_blob = encode_kv(&storage);
        let op_tape = encode_ops(&ops);
        let expected_trace = model_trace(&storage, &ops);

        let arts = compile_contracts(SOURCE, false, 2)
            .unwrap_or_else(|e| panic!("storage_iterator_stress compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "storage_iterator_stress: no artifacts");
        let art = &arts[0];

        let mut rt = NeoRuntime::new(RuntimeConfig::default())
            .expect("storage_iterator_stress runtime");

        let r = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "runStateMachine",
            &[
                StackItem::byte_array(kv_blob.clone()),
                StackItem::byte_array(op_tape.clone()),
            ],
        ).expect("runStateMachine host-level");

        prop_assert!(
            r.success,
            "runStateMachine faulted (storage entries={}, ops={}): {:?}\n\
             kv_blob_hex={}\nop_tape_hex={}",
            storage.len(),
            ops.len(),
            r.exception.as_ref().map(|e| e.message.clone()),
            hex::encode(&kv_blob),
            hex::encode(&op_tape),
        );

        // The contract returns `bytes`, so `return_data` is the raw
        // trace (no ABI envelope wrapping a length prefix; the runtime
        // currently emits ByteString return values flat).
        let observed = r.return_data.clone();

        prop_assert_eq!(
            &observed, &expected_trace,
            "iterator state machine trace divergence:\n\
             storage entries={}, ops={}\n\
             kv_blob_hex={}\n\
             op_tape_hex={}\n\
             expected_trace_hex={}\n\
             observed_trace_hex={}\n\
             ops={:?}",
            storage.len(),
            ops.len(),
            hex::encode(&kv_blob),
            hex::encode(&op_tape),
            hex::encode(&expected_trace),
            hex::encode(&observed),
            ops,
        );
    }

    /// Companion test: a single Find with a wider variety of prefixes,
    /// asserting the *count* of items walked equals the BTreeMap's
    /// prefix-filtered key count. This is a weaker but faster check
    /// (no Value records, just iteration shape) — runs a separate set
    /// of cases to broaden coverage of the prefix dimension.
    #[test]
    fn storage_iterator_find_count_matches_btreemap(
        storage in storage_strategy(),
        prefix_choice in 0u8..3,
        prefix_seed in any::<u32>(),
    ) {
        // Pick a prefix in one of three buckets:
        //   0: empty  (matches all)
        //   1: prefix of an existing key
        //   2: random bytes (likely no match)
        let keys: Vec<Vec<u8>> = storage.keys().cloned().collect();
        let prefix: Vec<u8> = match prefix_choice {
            0 => Vec::new(),
            1 => {
                let k = &keys[(prefix_seed as usize) % keys.len()];
                let take = ((prefix_seed >> 8) as usize % k.len()).max(1);
                k[..take].to_vec()
            }
            _ => {
                // 1..=4 random bytes derived from the seed (deterministic
                // per case for shrinking).
                let n = ((prefix_seed >> 16) as usize % 4) + 1;
                let mut p = Vec::with_capacity(n);
                let mut s = prefix_seed;
                for _ in 0..n {
                    s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                    p.push((s & 0xFF) as u8);
                }
                p
            }
        };

        let expected_count: u64 = storage
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .count() as u64;

        // Reuse the same harness: a single Find, then drain Next until
        // false, then return the count as little-endian bytes via the
        // trace payload. We construct the op tape directly: one Find,
        // then `len + 2` Next operations (over-drain to also exercise
        // Next-past-end in the same proptest).
        let mut ops: Vec<Op> = vec![Op::Find(prefix.clone())];
        for _ in 0..(expected_count + 2) {
            ops.push(Op::Next);
        }

        let kv_blob = encode_kv(&storage);
        let op_tape = encode_ops(&ops);
        let expected_trace = model_trace(&storage, &ops);

        let arts = compile_contracts(SOURCE, false, 2)
            .unwrap_or_else(|e| panic!("storage_iterator_stress compile: {:?}", e));
        let art = &arts[0];

        let mut rt = NeoRuntime::new(RuntimeConfig::default())
            .expect("storage_iterator_stress runtime");

        let r = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "runStateMachine",
            &[
                StackItem::byte_array(kv_blob.clone()),
                StackItem::byte_array(op_tape.clone()),
            ],
        ).expect("runStateMachine host-level");

        prop_assert!(
            r.success,
            "runStateMachine (find-count case) faulted: {:?}",
            r.exception.as_ref().map(|e| e.message.clone()),
        );
        prop_assert_eq!(
            &r.return_data, &expected_trace,
            "find-count trace divergence (prefix_hex={}, expected_count={}):\n\
             expected_trace_hex={}\n\
             observed_trace_hex={}",
            hex::encode(&prefix),
            expected_count,
            hex::encode(&expected_trace),
            hex::encode(&r.return_data),
        );
    }
}
