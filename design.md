# P3 Deep Architecture Improvements — Design Document

> **Date**: 2026-07-03  
> **Project**: neo-devpack-solidity v0.27.0  
> **Phase**: Brainstorming (Superpowers Phase 1)

---

## Overview

Four P3 improvements identified in the audit. Each is analyzed below with concrete design proposals and feasibility assessment.

---

## 1. Oracle Request Data Enrichment

### Current State
- `OracleRequest` struct has 8 fields (id, original_tx_hash, url, filter, callback_contract, callback_method, user_data, gas_for_response)
- Runtime `getrequests` returns only request IDs as raw byte arrays
- No `getrequest` (singular) match arm exists — falls through to `StackItem::Null`
- Solidity `NativeOracle.sol` exposes `getOracleRequests()` returning an `Iterator`

### Design
Add `"getrequest"` match arm to `invoke_native_oracle` in `oracle.rs`. When a request ID is passed, deserialize the ID, look up the request in `self.oracle_requests`, and return the full `OracleRequest` as a struct StackItem.

Also enrich `getrequests` to optionally support filtering/structured returns.

```rust
"getrequest" => {
    let id = Self::extract_first_int(&params);
    if let Some(req) = self.oracle_requests.get(&id) {
        // Return structured OracleRequest as NeoVM struct
        oracle_request_to_stack_item(req)
    } else {
        StackItem::Null
    }
}
```

### Effort: **Low** (~30 min)
### Risk: **Low** — pure additive, no existing behavior changes
### Impact: **Medium** — enables richer oracle testing in embedded runtime

---

## 2. Gas Precision Optimization

### Current State
- All opcodes use fixed costs (0-512 range)
- Three dynamic scaling paths already exist: hash per-byte (50/byte), Storage.Put (100k/byte), CheckMultisig (1k/verify)
- Known gaps documented: "Dynamic costs for large integer operations are approximated"
- Dead code: `named_operation_cost()` in `execution_gas.rs` (legacy EVM-like costs, never called)
- `GasTracker` is redundant: double-counts what `ExecutionContext.gas_used` already tracks

### Design

**2a. Arithmetic operand-size scaling** — Add dynamic surcharge for MUL/DIV/MOD/POW/SQRT based on operand byte lengths. On Neo N3, these scale with BigInt size. The formula: `gas += max(a.len(), b.len()) * ARITH_PER_BYTE_GAS` where `ARITH_PER_BYTE_GAS = 3`.

**2b. Remove dead GasTracker code** — `named_operation_cost()` and the EVM-style cost constants (`SSTORE=200`, `SLOAD=100`, etc.) in `execution_gas.rs` are never called by the production dispatch path. Remove them to reduce confusion.

**2c. Document remaining precision gaps** — Some NeoVM opcodes (e.g., PACK/UNPACK for large structs) should scale with element count but currently have fixed costs. Document these as known limitations.

### Effort: **Medium** (~2 hours)
- 2a: ~45 min (add ARITH_PER_BYTE_GAS constant + surcharge in dispatch.rs)
- 2b: ~30 min (audit all call sites, remove dead code)
- 2c: ~15 min (documentation)
### Risk: **Medium** — gas cost changes could break existing tests that check exact gas consumption
### Impact: **Medium** — better alignment with N3 cost model, cleaner codebase

---

## 3. Streaming Iterator Replacement

### Current State
- `IteratorState` holds `entries: Vec<StackItem>` — fully materialized
- `build_storage_entries()` queries storage host, merges overlay, sorts, shapes all entries at `Find` time
- Memory cap: `storage_limit / MIN_ENTRY_BYTES`
- `Iterator.Next` just increments `index` cursor; `Iterator.Value` reads from pre-built `Vec`
- Documented gap acknowledged in `parity-and-limitations.md`

### Design

Redesign `IteratorState` to hold a lazy query cursor instead of a pre-fetched `Vec`:

```rust
pub struct IteratorState {
    /// Pre-fetched entries (small batch for start)
    pub(crate) entries: Vec<StackItem>,
    /// Current position
    pub(crate) index: usize,
    /// Cursor for fetching more entries on demand (None = fully materialized)
    pub(crate) cursor: Option<StreamingCursor>,
}

pub struct StreamingCursor {
    /// Storage prefix being iterated
    pub(crate) prefix: Vec<u8>,
    /// Options flags
    pub(crate) options: i64,
    /// Last key returned (for pagination)
    pub(crate) last_key: Option<Vec<u8>>,
    /// Whether we've exhausted the storage host
    pub(crate) exhausted: bool,
}
```

**Iterator.Next** logic:
1. Increment `index`
2. If `index < entries.len()`, return `true` (entry already fetched)
3. If `cursor.is_some()` and `!cursor.exhausted`:
   - Fetch next batch from storage host (e.g., 100 entries)
   - Merge overlay entries
   - Append to `entries`
   - If new entries, return `true`
4. Return `false`

This is a **hybrid approach**: small initial fetch for responsiveness, then lazy batch fetching for memory efficiency. This avoids a complete rewrite while delivering streaming benefits.

### Effort: **High** (~4-6 hours)
- Redesign IteratorState + StreamingCursor: ~1h
- Rewrite build_storage_entries to support paging: ~1.5h
- Rewrite Iterator.Next to fetch on demand: ~1h
- Update overlay merging to handle streaming state: ~1h
- Test with large datasets: ~1h
### Risk: **High** — core runtime change, could break iterator behavior in subtle ways
### Impact: **High** — fixes a documented architectural limitation, reduces memory pressure

---

## 4. ByteString/Buffer Type Distinction

### Current State
- Simulator uses single `StackItem::ByteArray` for both ByteString (0x28) and Buffer (0x30)
- Compiler already inserts `CONVERT 0x28` after Buffer-producing ops (CAT, SUBSTR, NEWBUFFER)
- EQUAL is not type-strict in simulator (Buffer == ByteString returns true, should be false)
- REVERSEITEMS on ByteString works in simulator (should fault on real NeoVM)
- `StdLib.serialize` type tag embedding not modeled

### Design

Add an internal type tag field to `StackItem::ByteArray` to distinguish Buffer from ByteString at runtime:

```rust
pub enum StackItem {
    // ...
    ByteArray {
        data: Rc<RefCell<Vec<u8>>>,
        type_tag: ByteArrayType,  // NEW
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteArrayType {
    ByteString,  // 0x28
    Buffer,      // 0x30
}
```

Changes needed:
1. **StackItem enum** — add `type_tag` to ByteArray variant
2. **byte_array() constructor** — add type_tag parameter (default ByteString for backward compat)
3. **EQUAL** — check type_tag for ByteArray comparisons
4. **ISTYPE(0x28)** — return true only for ByteString-tagged arrays
5. **ISTYPE(0x30)** — return true only for Buffer-tagged arrays
6. **NEWBUFFER/CAT/SUBSTR/LEFT/RIGHT** — emit Buffer-tagged ByteArrays
7. **CONVERT 0x30** — tag as Buffer
8. **CONVERT 0x28** — tag as ByteString
9. **MEMCPY** — destination must be Buffer-tagged (assert or convert)
10. **REVERSEITEMS** — reject ByteString-tagged arrays

This is a **non-breaking change** because:
- The compiler already inserts CONVERT 0x28 normalization
- All byte-producing ops already have CONVERT 0x28 following them
- The only behavior change is making EQUAL type-strict, which is what NeoVM does

### Effort: **High** (~3-4 hours)
- StackItem enum change: ~30 min
- All ByteArray construction sites (~40 call sites): ~1.5h
- EQUAL/ISTYPE logic: ~30 min
- CONVERT logic: ~30 min
- Buffer-producing ops: ~30 min
- Test updates: ~1h
### Risk: **High** — touches core StackItem type used everywhere; many call sites to update
### Impact: **High** — fixes a fundamental simulation fidelity gap

---

## Implementation Priority

Given the effort/risk/impact trade-offs:

| # | Item | Effort | Risk | Impact | Priority |
|---|------|--------|------|--------|----------|
| 1 | Oracle request data | Low | Low | Medium | **First** |
| 2a | Arithmetic gas scaling | Medium | Medium | Medium | Second |
| 2b | Dead GasTracker removal | Low | Low | Low | Quick win |
| 4 | ByteString/Buffer | High | High | High | Core fix |
| 3 | Streaming Iterator | High | High | High | Most complex |

**Recommended execution order**: 1 → 2b → 2a → 4 → 3

---

## Verification Plan

After each item:
1. `cargo check` — must be clean (0 errors, 0 warnings)
2. `cargo test --no-run` — all 55 test targets compile
3. Run affected test binaries: `cargo test -p neo-devpack-solidity <test_name>`
4. For gas changes: run `tests/runtime_gas_tests` and `tests/conformance/gas_regression_tests`
5. For iterator changes: run `tests/runtime_storage_iterator_tests`

---

*Design document v1. Ready for validation.*
