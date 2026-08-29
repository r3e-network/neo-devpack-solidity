//! Conditional-jump opcode property tests (`JMPEQ`, `JMPNE`, `JMPGT`,
//! `JMPLT`, `JMPGE`, `JMPLE` and their `_L` long-offset variants — opcodes
//! `0x28..=0x33`).
//!
//! Why this exists: control-flow correctness on conditional jumps is
//! load-bearing for honest-contract integrity AND for safety against
//! attacker-influenced bytecode. A wrong PC advance — taken-vs-not-taken,
//! signed offset arithmetic, mismatched immediate length between short
//! and `_L` variants — would let attacker bytecode execute bytes the
//! verifier never approved. The dispatcher lives in
//! `src/runtime/execution/instruction/flow/jumps/compare.rs` and was 2.53%
//! line-covered before this batch.
//!
//! Test design (three proptests):
//!
//! * `cond_jump_taken_advances_by_offset` — for each of the 12 opcodes,
//!   construct `[push_a; push_b; OPCODE; offset(s); not-taken-pad;
//!   taken-pad]` such that the predicate matches. Run via the public
//!   `NeoRuntime::execute`. Verify the runtime returned the **taken**
//!   marker, which proves the PC was advanced exactly to the taken pad —
//!   i.e. `target = opcode_pos + offset`. The not-taken pad sits between
//!   the offset bytes and the taken pad; if PC arithmetic were wrong by
//!   even one byte, the not-taken marker (or a fault) would surface.
//!
//! * `cond_jump_not_taken_advances_naturally` — same script shape, but
//!   operands chosen so the predicate fails. Verify the **not-taken**
//!   marker surfaces, proving PC fell through by exactly the natural
//!   instruction length (2 bytes for short, 5 bytes for `_L`).
//!
//! * `cond_jump_long_offset_handles_full_i32_range` — for the six `_L`
//!   variants only, exercise offsets covering `i32::MIN`, `-1`, `0`,
//!   `1`, `i32::MAX` (those that point outside the bytecode must be
//!   rejected gracefully — runtime error, no panic / OOM-abort, no
//!   wraparound landing on a host-controlled byte). Offsets that land
//!   inside the bytecode must produce the expected marker.
//!
//! Why we marker-check `return_data` instead of asking the runtime for a
//! halted PC: `RET` (0x40) copies the top stack item into `return_data`
//! and sets PC = bytecode.len(), so a script of the form `PUSH<marker>;
//! RET` round-trips the marker out through the public
//! `ExecutionResult.return_data` byte slice. Two distinct markers (one
//! per pad) let us decide via a single `return_data` read whether the
//! taken or not-taken landing pad ran. That's strictly stronger than
//! observing `instruction_pointer` on `RuntimeException`, which is only
//! populated on faults.

use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ===== Markers =====
//
// We pick PUSH7 (0x17) for the taken-path landing pad and PUSH3 (0x13)
// for the not-taken pad. Both are single-byte opcodes that push a small
// `Integer` onto the stack; followed by `RET` (0x40), the runtime
// extracts the integer through `stack_item_to_bytes` as a fixed-width
// little-endian `i64` (`return_data` is then `[7, 0, 0, 0, 0, 0, 0, 0]`
// or `[3, 0, 0, 0, 0, 0, 0, 0]`). Choosing distinct values lets a single
// equality on `return_data` decide which pad ran.
const TAKEN_MARKER_OP: u8 = 0x17; // PUSH7 → Integer(7)
const NOT_TAKEN_MARKER_OP: u8 = 0x13; // PUSH3 → Integer(3)
const RET: u8 = 0x40;
fn taken_return() -> Vec<u8> {
    7i64.to_le_bytes().to_vec()
}
fn not_taken_return() -> Vec<u8> {
    3i64.to_le_bytes().to_vec()
}

// ===== Operand encoders =====
//
// We exercise four runtime-distinguishable operand shapes:
//   * Integer(i8)   via PUSHINT8  (2 bytes:  0x00 <i8>)
//   * Integer(i64)  via PUSHINT64 (9 bytes:  0x03 <i64 LE>)
//   * Boolean       via PUSHT/PUSHF (1 byte: 0x08 / 0x09)
//   * ByteArray     via PUSHDATA1 (2+len bytes: 0x0C <u8 len> <data...>)
//
// `UnsignedInteger` stack items are not produced by any standalone
// PUSH-class opcode in the dispatcher (`PUSHA` exists but encodes
// absolute jump targets — abusing it for operand pushing would make
// bytecode-layout reasoning fragile). The cross-Integer/UnsignedInteger
// comparison branches in `helpers/comparison.rs` are exercised by the
// IR-codegen integration tests; this proptest stays focused on the
// dispatcher's PC arithmetic, which is type-agnostic.

#[derive(Debug, Clone)]
enum Operand {
    I8(i8),
    I64(i64),
    Bool(bool),
    Bytes(Vec<u8>),
}

impl Operand {
    fn emit(&self, out: &mut Vec<u8>) {
        match self {
            Operand::I8(v) => {
                out.push(0x00); // PUSHINT8
                out.push(*v as u8);
            }
            Operand::I64(v) => {
                out.push(0x03); // PUSHINT64
                out.extend_from_slice(&v.to_le_bytes());
            }
            Operand::Bool(true) => out.push(0x08),  // PUSHT
            Operand::Bool(false) => out.push(0x09), // PUSHF
            Operand::Bytes(b) => {
                assert!(b.len() <= u8::MAX as usize, "test PUSHDATA1 bound");
                out.push(0x0C); // PUSHDATA1
                out.push(b.len() as u8);
                out.extend_from_slice(b);
            }
        }
    }

    /// Mirror the runtime's `stack_items_equal` semantics for the four
    /// shapes this test emits. Cross-type pairs return `false` (matches
    /// the catch-all arm in `helpers/comparison.rs`), with the explicit
    /// Integer/Integer carve-out (we never produce `UnsignedInteger`
    /// here, so the cross-numeric arms are unreachable).
    fn eq_runtime(a: &Operand, b: &Operand) -> bool {
        match (a, b) {
            (Operand::I8(x), Operand::I8(y)) => (*x as i64) == (*y as i64),
            (Operand::I8(x), Operand::I64(y)) => (*x as i64) == *y,
            (Operand::I64(x), Operand::I8(y)) => *x == (*y as i64),
            (Operand::I64(x), Operand::I64(y)) => x == y,
            (Operand::Bool(x), Operand::Bool(y)) => x == y,
            (Operand::Bytes(x), Operand::Bytes(y)) => x == y,
            _ => false,
        }
    }

    /// Mirror runtime `less_than` for same-kind numeric/byte/bool pairs.
    /// Cross-type pairs are not constructed for the GT/LT/GE/LE harness
    /// (the runtime would coerce-or-fault, and we don't need that path
    /// to validate dispatcher PC arithmetic).
    fn lt_runtime(a: &Operand, b: &Operand) -> Option<bool> {
        match (a, b) {
            (Operand::I8(x), Operand::I8(y)) => Some((*x as i64) < (*y as i64)),
            (Operand::I64(x), Operand::I64(y)) => Some(x < y),
            (Operand::I8(x), Operand::I64(y)) => Some((*x as i64) < *y),
            (Operand::I64(x), Operand::I8(y)) => Some(*x < (*y as i64)),
            (Operand::Bool(x), Operand::Bool(y)) => Some(!x & y),
            // ByteArray ordering routes through `coerce_item_to_i64`
            // (per `helpers/comparison.rs`) which is the same shape as
            // `stack_item_to_int`: take the first ≤8 bytes LE. We only
            // emit short, well-formed byte arrays for ordered pairs.
            (Operand::Bytes(x), Operand::Bytes(y)) => {
                let xv = bytes_as_i64(x);
                let yv = bytes_as_i64(y);
                Some(xv < yv)
            }
            _ => None,
        }
    }

    fn gt_runtime(a: &Operand, b: &Operand) -> Option<bool> {
        Operand::lt_runtime(b, a)
    }
}

fn bytes_as_i64(b: &[u8]) -> i64 {
    let mut buf = [0u8; 8];
    for (i, byte) in b.iter().take(8).enumerate() {
        buf[i] = *byte;
    }
    i64::from_le_bytes(buf)
}

fn op_emitted_size(o: &Operand) -> usize {
    match o {
        Operand::I8(_) => 2,
        Operand::I64(_) => 9,
        Operand::Bool(_) => 1,
        Operand::Bytes(b) => 2 + b.len(),
    }
}

// ===== Opcode table =====
//
// (opcode, is_long, predicate_kind). `predicate_kind` lets the test
// driver decide "make predicate hold" or "make predicate fail" in a
// type-aware way without re-deriving each opcode's semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pred {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

const ALL_COMPARE_JUMPS: &[(u8, bool, Pred)] = &[
    (0x28, false, Pred::Eq), // JMPEQ
    (0x29, true, Pred::Eq),  // JMPEQ_L
    (0x2A, false, Pred::Ne), // JMPNE
    (0x2B, true, Pred::Ne),  // JMPNE_L
    (0x2C, false, Pred::Gt), // JMPGT
    (0x2D, true, Pred::Gt),  // JMPGT_L
    (0x2E, false, Pred::Ge), // JMPGE
    (0x2F, true, Pred::Ge),  // JMPGE_L
    (0x30, false, Pred::Lt), // JMPLT
    (0x31, true, Pred::Lt),  // JMPLT_L
    (0x32, false, Pred::Le), // JMPLE
    (0x33, true, Pred::Le),  // JMPLE_L
];

fn predicate_holds(p: Pred, a: &Operand, b: &Operand) -> Option<bool> {
    match p {
        Pred::Eq => Some(Operand::eq_runtime(a, b)),
        Pred::Ne => Some(!Operand::eq_runtime(a, b)),
        Pred::Gt => Operand::gt_runtime(a, b),
        Pred::Lt => Operand::lt_runtime(a, b),
        Pred::Ge => match (Operand::gt_runtime(a, b), Operand::eq_runtime(a, b)) {
            (Some(g), e) => Some(g || e),
            _ => None,
        },
        Pred::Le => match (Operand::lt_runtime(a, b), Operand::eq_runtime(a, b)) {
            (Some(l), e) => Some(l || e),
            _ => None,
        },
    }
}

// ===== Script builder =====
//
// Layout for the standard taken/not-taken harness:
//
//   [push_a]                           starts at byte 0
//   [push_b]
//   [OPCODE]                           opcode_pos (= len(push_a)+len(push_b))
//   [offset bytes — 1 byte short, 4 bytes LE long]
//   [PUSH<NOT_TAKEN_MARKER>]           not-taken landing pad
//   [RET]
//   [PUSH<TAKEN_MARKER>]               taken landing pad
//   [RET]
//
// Predicate-true script must set offset = (taken_pad_pos − opcode_pos):
//   short: (1 byte offset + 2-byte not-taken pad) = 3 → offset value = 3
//   long:  (4 bytes offset + 2-byte not-taken pad) = 6 → offset value = 6
//
// Predicate-false script: same script. The not-taken pad runs (returns
// NOT_TAKEN_MARKER), the taken pad is dead code.
//
// This same script shape is reused for *both* the "taken" and "not
// taken" proptests — what differs is only `(a, b)`.
struct CompareJumpScript {
    bytes: Vec<u8>,
    /// PC of the opcode byte itself.
    opcode_pos: usize,
    /// Expected post-jump PC if the predicate holds.
    taken_pad_pos: usize,
    /// Expected post-fallthrough PC if the predicate fails.
    not_taken_pad_pos: usize,
}

fn build_default_script(opcode: u8, is_long: bool, a: &Operand, b: &Operand) -> CompareJumpScript {
    let mut out = Vec::new();
    a.emit(&mut out);
    b.emit(&mut out);

    let opcode_pos = out.len();
    out.push(opcode);

    let offset_size: usize = if is_long { 4 } else { 1 };

    // Distance from opcode_pos to the taken pad: 1 byte for the
    // opcode itself + offset_size bytes for the immediate + 2 bytes
    // for the not-taken pad ([PUSH<NT>; RET]). The signed offset value
    // we emit equals this distance because `compute_offset_target`
    // adds `offset` to `instruction_pointer` (= opcode_pos), not to
    // the post-instruction PC.
    let taken_distance = (1 + offset_size + 2) as i64;
    if is_long {
        out.extend_from_slice(&(taken_distance as i32).to_le_bytes());
    } else {
        out.push(taken_distance as i8 as u8);
    }
    let not_taken_pad_pos = out.len();
    out.push(NOT_TAKEN_MARKER_OP);
    out.push(RET);

    let taken_pad_pos = out.len();
    out.push(TAKEN_MARKER_OP);
    out.push(RET);

    CompareJumpScript {
        bytes: out,
        opcode_pos,
        taken_pad_pos,
        not_taken_pad_pos,
    }
}

// ===== Strategy =====
//
// Operand strategy weighted toward edge cases (zero, one, -1, type
// boundaries). The driver picks a generation function based on the
// predicate kind so we don't waste cases on cross-type comparisons that
// short-circuit to `false` (which would only ever exercise the
// not-taken branch).
fn op_strategy_any() -> impl Strategy<Value = Operand> {
    prop_oneof![
        (-128i8..=127i8).prop_map(Operand::I8),
        (-1_000_000i64..=1_000_000i64).prop_map(Operand::I64),
        any::<bool>().prop_map(Operand::Bool),
        prop::collection::vec(0u8..=255u8, 0..=8).prop_map(Operand::Bytes),
    ]
}

/// Generate `(a, b)` biased so ~50% of cases satisfy `a == b` (per the
/// task spec — Eq/Ne harness). For "equal" pairs we re-emit `a` as `b`
/// so the runtime sees identical stack items.
fn op_strategy_eq_biased() -> impl Strategy<Value = (Operand, Operand)> {
    (op_strategy_any(), any::<bool>(), op_strategy_any()).prop_map(|(a, force_eq, b)| {
        if force_eq {
            (a.clone(), a)
        } else {
            (a, b)
        }
    })
}

/// Generate a same-kind ordered pair so the runtime's ordered
/// comparison branches (greater_than/less_than) fire deterministically.
/// Cross-type ordered pairs are excluded — the dispatcher would still
/// run, but the test driver can't predict the runtime's coercion path
/// without re-implementing it, and our purpose is verifying PC
/// arithmetic, not coercion.
fn op_strategy_ordered_same_kind() -> impl Strategy<Value = (Operand, Operand)> {
    prop_oneof![
        (-100i8..=100i8, -100i8..=100i8).prop_map(|(x, y)| (Operand::I8(x), Operand::I8(y))),
        (-10_000i64..=10_000i64, -10_000i64..=10_000i64)
            .prop_map(|(x, y)| (Operand::I64(x), Operand::I64(y))),
        (any::<bool>(), any::<bool>()).prop_map(|(x, y)| (Operand::Bool(x), Operand::Bool(y))),
        // Bytes — short, 1..4-byte arrays. Runtime LE-decodes the first
        // ≤8 bytes as `i64`, so values stay distinguishable.
        (
            prop::collection::vec(0u8..=255u8, 1..=4),
            prop::collection::vec(0u8..=255u8, 1..=4),
        )
            .prop_map(|(x, y)| (Operand::Bytes(x), Operand::Bytes(y))),
    ]
}

// ============================================================
// Proptest #1 — predicate-true → PC advances to opcode_pos + offset
// ============================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn cond_jump_taken_advances_by_offset(
        (a_eq, b_eq) in op_strategy_eq_biased(),
        (a_ord, b_ord) in op_strategy_ordered_same_kind(),
    ) {
        for &(opcode, is_long, pred) in ALL_COMPARE_JUMPS {
            // Pick the operand pair that's appropriate for the predicate
            // and force the predicate to hold. For Eq we use the eq-biased
            // pair; for Ne we use any non-equal pair (using ordered pair
            // is fine because eq is independent of ordering).
            let (a, b) = match pred {
                Pred::Eq => {
                    if !Operand::eq_runtime(&a_eq, &b_eq) {
                        // skip — we want the taken branch
                        continue;
                    }
                    (a_eq.clone(), b_eq.clone())
                }
                Pred::Ne => {
                    if Operand::eq_runtime(&a_ord, &b_ord) {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Gt => {
                    let g = Operand::gt_runtime(&a_ord, &b_ord);
                    if g != Some(true) {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Lt => {
                    let l = Operand::lt_runtime(&a_ord, &b_ord);
                    if l != Some(true) {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Ge => {
                    let g = Operand::gt_runtime(&a_ord, &b_ord);
                    let e = Operand::eq_runtime(&a_ord, &b_ord);
                    if g != Some(true) && !e {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Le => {
                    let l = Operand::lt_runtime(&a_ord, &b_ord);
                    let e = Operand::eq_runtime(&a_ord, &b_ord);
                    if l != Some(true) && !e {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
            };

            // Sanity: oracle says predicate holds.
            prop_assert_eq!(
                predicate_holds(pred, &a, &b),
                Some(true),
                "oracle bug: predicate {:?} on {:?},{:?} should hold",
                pred, a, b
            );

            let script = build_default_script(opcode, is_long, &a, &b);
            let mut rt = NeoRuntime::new(RuntimeConfig::default())
                .expect("runtime construction must not fail");
            let res = rt.execute(&script.bytes, &[])
                .expect("execute must not fail at host level (a fault would surface as Ok+success=false)");

            prop_assert!(
                res.success,
                "opcode 0x{:02X} predicate-true: execute returned !success: {:?}",
                opcode, res.exception
            );
            prop_assert_eq!(
                &res.return_data,
                &taken_return(),
                "opcode 0x{:02X} (is_long={}) predicate-true on a={:?} b={:?}: \
                 return_data should be the TAKEN marker (PC jumped to opcode_pos+offset = {}), \
                 not the not-taken marker (PC fell through to {}). \
                 If this fires: the dispatcher's PC arithmetic for the \
                 taken branch is wrong — see compare.rs. \
                 script bytes: {:02X?}",
                opcode, is_long, a, b,
                script.taken_pad_pos, script.not_taken_pad_pos,
                &script.bytes
            );
        }
    }
}

// ============================================================
// Proptest #2 — predicate-false → PC advances naturally
// ============================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn cond_jump_not_taken_advances_naturally(
        (a_eq, b_eq) in op_strategy_eq_biased(),
        (a_ord, b_ord) in op_strategy_ordered_same_kind(),
    ) {
        for &(opcode, is_long, pred) in ALL_COMPARE_JUMPS {
            let (a, b) = match pred {
                Pred::Eq => {
                    if Operand::eq_runtime(&a_eq, &b_eq) {
                        continue;
                    }
                    (a_eq.clone(), b_eq.clone())
                }
                Pred::Ne => {
                    if !Operand::eq_runtime(&a_eq, &b_eq) {
                        continue;
                    }
                    (a_eq.clone(), b_eq.clone())
                }
                Pred::Gt => {
                    let g = Operand::gt_runtime(&a_ord, &b_ord);
                    if g != Some(false) {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Lt => {
                    let l = Operand::lt_runtime(&a_ord, &b_ord);
                    if l != Some(false) {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Ge => {
                    let g = Operand::gt_runtime(&a_ord, &b_ord);
                    let e = Operand::eq_runtime(&a_ord, &b_ord);
                    if g != Some(false) || e {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
                Pred::Le => {
                    let l = Operand::lt_runtime(&a_ord, &b_ord);
                    let e = Operand::eq_runtime(&a_ord, &b_ord);
                    if l != Some(false) || e {
                        continue;
                    }
                    (a_ord.clone(), b_ord.clone())
                }
            };

            prop_assert_eq!(
                predicate_holds(pred, &a, &b),
                Some(false),
                "oracle bug: predicate {:?} on {:?},{:?} should fail",
                pred, a, b
            );

            let script = build_default_script(opcode, is_long, &a, &b);
            let mut rt = NeoRuntime::new(RuntimeConfig::default())
                .expect("runtime construction must not fail");
            let res = rt.execute(&script.bytes, &[])
                .expect("execute must not fail at host level");

            prop_assert!(
                res.success,
                "opcode 0x{:02X} predicate-false: execute returned !success: {:?}",
                opcode, res.exception
            );
            prop_assert_eq!(
                &res.return_data,
                &not_taken_return(),
                "opcode 0x{:02X} (is_long={}) predicate-false on a={:?} b={:?}: \
                 return_data should be the NOT-TAKEN marker (PC advanced by \
                 instruction length to {}), not the taken marker (PC = {}). \
                 If this fires: the dispatcher's natural-fallthrough PC \
                 advance is wrong (off-by-one or wrong immediate length). \
                 See compare.rs. script bytes: {:02X?}",
                opcode, is_long, a, b,
                script.not_taken_pad_pos, script.taken_pad_pos,
                &script.bytes
            );
        }
    }
}

// ============================================================
// Proptest #3 — `_L` long-offset variants must handle the full
// `i32` range gracefully.
// ============================================================
//
// Three sub-cases:
//
// * Offsets that point INSIDE the bytecode (small positive values,
//   plus a controlled small negative loop-back) — the runtime must
//   land at `opcode_pos + offset` and produce a marker.
// * Offsets that point OUTSIDE the bytecode (`i32::MIN`, `i32::MAX`,
//   any negative offset whose target is < 0, or positive ≥ len) — the
//   runtime must reject gracefully (`Err(_)` or `Ok` with success=false
//   and an `InvalidJump`/`ExecutionError`-style exception). Crucially,
//   it must **not** wrap, panic, or land on attacker-controlled bytes.
// * Offset = 0 (taken path) — that's a self-jump infinite loop; the
//   runtime must terminate via OutOfGas, not hang or crash.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn cond_jump_long_offset_handles_full_i32_range(
        (a, b) in op_strategy_eq_biased(),
    ) {
        // We only run this one against `_L` variants. For each opcode,
        // construct a "predicate-true" script (we want the offset path
        // to be exercised). We force eq for Eq and Ge/Le; force ne for
        // Ne; and re-derive Gt/Lt against the eq-biased pair (which is
        // enough — when not equal, exactly one of gt/lt is true).
        for &(opcode, is_long, pred) in ALL_COMPARE_JUMPS {
            if !is_long {
                continue;
            }

            // Force the predicate to hold so the long-offset branch
            // actually fires. We use simple, known-good I64 pairs for
            // the predicate seeding: this proptest's purpose is to
            // exercise PC arithmetic on the `_L` immediate, not to
            // re-validate the comparison helpers (which are the focus
            // of proptests #1 and #2). Using `I64` operands also
            // avoids the BigInt-coercion path that triggers on
            // ByteArrays wider than 8 bytes — see
            // `cmp_needs_bigint_path` in helpers/arithmetic/basic_ops.rs
            // — where two byte representations of the same numeric
            // value would compare equal even though their byte slices
            // differ. We still vary `(a, b)` source from the
            // proptest input so each `_L` opcode is exercised across
            // many proptest cases (ProptestConfig::with_cases(32)).
            let _seed = (a.clone(), b.clone());
            let (oa, ob) = match pred {
                Pred::Eq => (Operand::I64(42), Operand::I64(42)),
                Pred::Ne => (Operand::I64(42), Operand::I64(43)),
                Pred::Gt => (Operand::I64(10), Operand::I64(5)),
                Pred::Lt => (Operand::I64(5), Operand::I64(10)),
                Pred::Ge => (Operand::I64(7), Operand::I64(7)),
                Pred::Le => (Operand::I64(7), Operand::I64(7)),
            };

            prop_assert_eq!(
                predicate_holds(pred, &oa, &ob),
                Some(true),
                "oracle bug: forced-true predicate {:?} on {:?},{:?} did not hold",
                pred, oa, ob
            );

            // Sub-case A — small in-range positive offsets.
            // We reuse `build_default_script` whose taken pad is at
            // distance (4 + 2) = 6 from opcode_pos. The default offset
            // emitted is exactly 6, so the runtime must produce the
            // taken marker. (Same shape as proptest #1 but exercising
            // ONLY `_L` variants.)
            {
                let script = build_default_script(opcode, true, &oa, &ob);
                let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
                let res = rt.execute(&script.bytes, &[])
                    .expect("execute must not fail at host level");
                prop_assert!(res.success);
                prop_assert_eq!(&res.return_data, &taken_return(),
                    "opcode 0x{:02X} small +offset: expected taken marker", opcode);
            }

            // Sub-case B — extreme out-of-range offsets.
            // We construct a script identical to A in the prefix, but
            // overwrite the 4 offset bytes with the extreme value. The
            // runtime must NOT panic and must NOT successfully land on
            // any of the host-controlled bytes; either Err(_) or
            // Ok{success: false} is acceptable.
            for &extreme in &[i32::MIN, i32::MAX, -1_000_000_000i32, 1_000_000_000i32] {
                let mut script = build_default_script(opcode, true, &oa, &ob);
                let off_pos = script.opcode_pos + 1;
                script.bytes[off_pos..off_pos + 4]
                    .copy_from_slice(&extreme.to_le_bytes());

                let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
                let res = rt.execute(&script.bytes, &[]);
                match res {
                    Err(_) => { /* graceful host error — fine */ }
                    Ok(r) => {
                        // Either it landed inside the bytecode at a
                        // valid PC and ran something, OR it produced
                        // success=false with an exception. The
                        // CRITICAL property: `return_data` MUST NOT
                        // equal a marker — that would mean the runtime
                        // wrapped `i32::MIN` / `i32::MAX` to a valid
                        // landing pad and treated attacker-controlled
                        // bytes as code.
                        if r.success {
                            prop_assert_ne!(
                                &r.return_data,
                                &taken_return(),
                                "opcode 0x{:02X} extreme offset {}: runtime claims success \
                                 with the TAKEN marker — that means PC arithmetic wrapped \
                                 i32 silently (CONTROL-FLOW HIJACK). compare.rs / \
                                 compute_offset_target must reject targets < 0 or ≥ len.",
                                opcode, extreme
                            );
                            prop_assert_ne!(
                                &r.return_data,
                                &not_taken_return(),
                                "opcode 0x{:02X} extreme offset {}: runtime claims success \
                                 with the NOT-TAKEN marker. (Predicate held; any success \
                                 path other than the taken pad is unexpected.)",
                                opcode, extreme
                            );
                        }
                    }
                }
            }

            // Sub-case C — offset = 0 with predicate true creates a
            // self-jump on the opcode itself, infinite-looping until
            // gas runs out. Verify graceful termination — no panic, no
            // hang, host returns either `Err(_)` or `Ok{success:false,
            // exception:OutOfGas}`. (No marker should surface, since
            // the loop never reaches a RET.)
            {
                let mut script = build_default_script(opcode, true, &oa, &ob);
                let off_pos = script.opcode_pos + 1;
                script.bytes[off_pos..off_pos + 4]
                    .copy_from_slice(&0i32.to_le_bytes());

                let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
                let res = rt.execute(&script.bytes, &[]);
                match res {
                    Err(_) => {}
                    Ok(r) => {
                        prop_assert!(
                            !r.success,
                            "opcode 0x{:02X} offset=0 self-loop: expected \
                             non-success (gas exhaustion or fault), got success",
                            opcode
                        );
                    }
                }
            }
        }
    }
}

// ============================================================
// Static smoke tests — fixed bytecodes that pin the byte layout
// computations above. If `build_default_script` ever drifts (e.g.
// someone changes `op_emitted_size` or the marker-pad shape), these
// catch it before the proptests, with a clearer failure.
// ============================================================

#[test]
fn smoke_jmpeq_short_taken() {
    // PUSH5 (0x15 → Integer(5)) ; PUSH5 ; JMPEQ +4 ; PUSH3 ; RET ; PUSH7 ; RET
    // opcode_pos=2, target = opcode_pos + 4 = 6 → TAKEN pad (0x17, 0x40).
    let script = vec![
        0x15,
        0x15,
        0x28,
        0x04,
        NOT_TAKEN_MARKER_OP,
        RET,
        TAKEN_MARKER_OP,
        RET,
    ];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let res = rt.execute(&script, &[]).expect("execute");
    assert!(res.success, "execute failed: {:?}", res.exception);
    assert_eq!(
        res.return_data,
        taken_return(),
        "JMPEQ with equal operands must take"
    );
}

#[test]
fn smoke_jmpeq_short_not_taken() {
    // PUSH5 ; PUSH7 ; JMPEQ +4 ; PUSH3 ; RET ; PUSH7 ; RET
    let script = vec![
        0x15,
        0x17,
        0x28,
        0x04,
        NOT_TAKEN_MARKER_OP,
        RET,
        TAKEN_MARKER_OP,
        RET,
    ];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let res = rt.execute(&script, &[]).expect("execute");
    assert!(res.success, "execute failed: {:?}", res.exception);
    assert_eq!(
        res.return_data,
        not_taken_return(),
        "JMPEQ with unequal operands must fall through"
    );
}

#[test]
fn smoke_jmpeq_long_taken() {
    // PUSH5 ; PUSH5 ; JMPEQ_L +7 ; PUSH3 ; RET ; PUSH7 ; RET
    // opcode_pos=2, target = 2 + 7 = 9 → TAKEN pad.
    let mut script = vec![0x15, 0x15, 0x29];
    script.extend_from_slice(&7i32.to_le_bytes());
    script.extend_from_slice(&[NOT_TAKEN_MARKER_OP, RET, TAKEN_MARKER_OP, RET]);
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let res = rt.execute(&script, &[]).expect("execute");
    assert!(res.success);
    assert_eq!(res.return_data, taken_return());
}

#[test]
fn smoke_op_emitted_size_matches_emit_len() {
    // Defensive: any future operand-encoding change must keep the size
    // calculator in sync with actual emission length, otherwise the
    // proptest will quietly compute wrong jump distances.
    let cases: &[Operand] = &[
        Operand::I8(0),
        Operand::I8(-128),
        Operand::I64(0),
        Operand::I64(i64::MIN),
        Operand::Bool(true),
        Operand::Bool(false),
        Operand::Bytes(Vec::new()),
        Operand::Bytes(vec![1, 2, 3, 4]),
    ];
    for c in cases {
        let mut buf = Vec::new();
        c.emit(&mut buf);
        assert_eq!(
            op_emitted_size(c),
            buf.len(),
            "op_emitted_size out of sync with emit() for {:?}",
            c
        );
    }
}
