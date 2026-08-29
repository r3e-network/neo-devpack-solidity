//! Call and exception handling frames for execution context.
//!
//! Provides structures for managing function call frames and exception handling.

use super::stack::StackItem;
use super::state::OverlayEntry;
use std::collections::HashMap;

/// Call frame for function calls
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_address: u32,
    pub function_name: Option<String>,
    pub local_variables: HashMap<String, StackItem>,
    pub stack_base: usize,
    /// Saved caller locals — restored when returning from this frame.
    pub saved_locals: Vec<StackItem>,
    /// Saved caller args — restored when returning from this frame.
    pub saved_args: Vec<StackItem>,
    /// Task #123 — per-frame override for `msg.sender` (`CallingScriptHash`)
    /// reads. Populated when `handle_contract_call` enters a self-offsets
    /// "virtual contract boundary" (Task #83 sibling-merge dispatch), so the
    /// nested callee's `CallingScriptHash` reflects the intermediate caller's
    /// script hash instead of silently collapsing to `Transaction.Sender`.
    ///
    /// When `None`, the `GetCallingScriptHash` syscall falls back to the
    /// runtime's `caller_account` (i.e. `Transaction.Sender`), preserving the
    /// pre-Task-123 default for entry-depth calls.
    ///
    /// The override is *not* consulted by `GetScriptContainer` (which backs
    /// Solidity's `tx.origin` via `Transaction.Sender`), so `tx.origin` stays
    /// pinned to the outermost transaction signer across every nested frame —
    /// exactly matching EVM semantics.
    pub msg_sender_override: Option<Vec<u8>>,
    /// Task #160 — marks frames pushed by `handle_contract_call`'s self-offsets
    /// dispatch path (Task #83 sibling-merge via the 20-byte zero placeholder).
    ///
    /// The `invoke_native_contract` fast path always pushes exactly one
    /// `StackItem` result onto the caller's evaluation stack (empty/no-return
    /// callees surface as `Null`, see `execution_impl_part2_contract_call.rs`).
    /// The compiler's lowering for external contract calls relies on that
    /// invariant — for example `try_catch.rs` emits `Drop(ValueType::Any)` after
    /// a void external call to discard the implicit syscall result.
    ///
    /// The self-offsets dispatch branch, however, hands control directly to the
    /// target method's compiled offset and lets the callee's `RET` carry any
    /// return value back. When the callee is `void` (its `ReturnVoid` emits
    /// `0x40` without pushing anything), the caller's implicit `DROP` would hit
    /// an empty slot and fault — routing a happy-path `try Target(t).voidFn()`
    /// into the catch arm.
    ///
    /// When this flag is `true`, `return_from_function` synthesises a
    /// `StackItem::Null` result for void callees so the caller observes the
    /// same "one syscall result" contract regardless of dispatch flavour.
    pub syscall_result_expected: bool,
    /// S7 fix — snapshot of the storage overlay taken when this frame was
    /// pushed at a contract-call boundary. When the frame unwinds via an
    /// exception (`dispatch_exception`), the runtime restores this snapshot so
    /// the callee's dirty storage writes are discarded — matching Neo N3,
    /// which reverts storage to the call-boundary snapshot on any inner fault.
    ///
    /// `None` for plain internal function calls (CALL_L) — only
    /// `handle_contract_call`'s self-offsets dispatch arm populates it, since
    /// that's the path that crosses a virtual contract boundary whose
    /// storage semantics Neo N3 isolates per-call.
    pub storage_snapshot: Option<HashMap<Vec<u8>, OverlayEntry>>,
    /// S6 fix — saved caller CallFlags, populated only on frames pushed at a
    /// contract-call boundary (`handle_contract_call`'s self-offsets arm). The
    /// callee runs with the caller-requested flags (the operand the
    /// `System.Contract.Call` syscall popped off the stack); on return / unwind
    /// the caller's prior flags are restored from this slot.
    ///
    /// `None` for plain internal function calls (CALL_L) — Solidity internal
    /// calls do not cross a contract boundary and inherit the caller's flags
    /// unchanged, so there is nothing to save or restore.
    pub saved_call_flags: Option<u8>,
    /// neo-test `vm.expectRevert`: when a contract-call frame is pushed with an
    /// armed expectation, it is recorded here. A revert that unwinds to this
    /// frame is SWALLOWED (the call is treated as having returned), satisfying
    /// the expectation; a NORMAL return from this frame is a violation (the
    /// guarded call was expected to revert but didn't). `Some(None)` = expect
    /// any revert; `Some(Some(payload))` = expect this exact payload. `None`
    /// (the default for every frame) leaves both paths inert.
    pub expect_revert_guard: Option<Option<Vec<u8>>>,
}

impl CallFrame {
    /// Create a new call frame
    pub fn new(return_address: u32, stack_base: usize) -> Self {
        Self {
            return_address,
            function_name: None,
            local_variables: HashMap::new(),
            stack_base,
            saved_locals: Vec::new(),
            saved_args: Vec::new(),
            msg_sender_override: None,
            syscall_result_expected: false,
            storage_snapshot: None,
            saved_call_flags: None,
            expect_revert_guard: None,
        }
    }

    /// Check if this frame has local variables
    pub fn has_locals(&self) -> bool {
        !self.local_variables.is_empty()
    }
}

/// Exception handling frame for try-catch-finally blocks
#[derive(Debug, Clone)]
pub struct TryFrame {
    pub(crate) catch_target: Option<u32>,
    pub(crate) finally_target: Option<u32>,
    pub(crate) end_target: Option<u32>,
    pub(crate) state: TryFrameState,
    /// Call-stack depth at which this TRY was pushed. When a throw inside a
    /// callee is dispatched to this frame's catch handler, the call_stack
    /// must be unwound to this depth first — otherwise the catch body's RET
    /// pops the callee's frame and resumes the TRY body's post-call
    /// continuation (see batch31 H2b: catch `return 99` leaks into
    /// `x + 1` and yields 100 instead of 99).
    pub(crate) owner_call_depth: usize,
}

/// State of a try-catch-finally frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryFrameState {
    Try,
    Catch,
    Finally,
}

impl TryFrameState {
    /// Check if in error handling state
    pub fn is_handling_error(&self) -> bool {
        matches!(self, Self::Catch | Self::Finally)
    }

    /// Get state as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Try => "try",
            Self::Catch => "catch",
            Self::Finally => "finally",
        }
    }
}
