use super::*;

// ===========================================================================
// Task #99 — yul (inline assembly) lowering.
//
// Scope (per the task plan):
//   * mstore/mload/return  — the three memory opcodes exercised by the
//     `batch39_n3_yul_mstore_mload_return_ignored_until_task_99` harness.
//   * `let x := expr` / `x := expr` — yul local variable declarations and
//     assignments (the harness uses `let v := mload(0x40)`).
//   * A handful of pure-arithmetic opcodes: add/sub/mul/div + iszero/eq/lt/gt
//     which show up alongside mstore/mload in real yul idioms. Everything
//     else (for/switch/sload/sstore/call/...) falls back to the legacy no-op
//     warning path so contracts using more exotic yul still compile.
//
// Memory model:
//   Yul exposes a flat byte-addressed memory. We materialise it as a NeoVM
//   Buffer allocated on first mstore/mload inside the block. Because we
//   can't know the maximum offset ahead of time without a second static
//   pass, we pre-size conservatively to 256 bytes (enough for the
//   `mstore(0x40, ...)` free-memory-pointer slot plus a 32-byte return slot
//   at offset 0). Accesses above that bound trap at runtime with
//   `MEMCPY: range out of bounds` — an acceptable signal for "this yul block
//   is too ambitious for the stub."
//
// Return handling:
//   yul `return(offset, length)` terminates the function with the memory
//   slice as its return payload. For a Solidity function declared
//   `returns (uintN)` / `returns (intN)` / `returns (bool)` (the common
//   case, and the one exercised by the batch39_n3 harness), we re-interpret
//   the slice as a big-endian integer so the main-frame RET emits the
//   expected 32-byte BE-packed value. Otherwise the raw buffer is returned.

/// Lower a yul block, returning `true` iff every statement in the block was
/// successfully lowered. A `false` result causes the enclosing assembly
/// statement to fall back to the legacy no-op warning path, preserving
/// compilation for contracts using unsupported yul constructs.
pub(crate) fn lower_yul_block(
    block: &solang_parser::pt::YulBlock,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Snapshot the instruction stream length so we can roll back on failure.
    let snapshot = instructions.len();

    let mut state = YulLoweringState::new();

    for stmt in &block.statements {
        if !lower_yul_statement(stmt, &mut state, ctx, instructions) {
            // Roll back any partial IR so the enclosing caller can emit the
            // legacy no-op warning without leaving orphan instructions.
            instructions.truncate(snapshot);
            return false;
        }
    }

    // Prepend memory-buffer AND transient-map initialisation at the snapshot
    // boundary. The memory buffer is always per-block (yul memory is scratch,
    // cleared each call to `assembly`). The transient map is initialised
    // only in the block that first introduces `__yul_transient` — later
    // blocks in the same function resolve the slot by name and reuse the
    // already-live map (EIP-1153 per-tx persistence across yul blocks).
    // Task #184 mirrors the transient-map pattern for `__yul_returndata`:
    // allocated function-wide, initialised once in the first block that
    // references it.
    let needs_memory_init = state.memory_local.is_some();
    let needs_transient_init = state.transient_allocated_here;
    let needs_returndata_init = state.returndata_allocated_here;

    if needs_memory_init || needs_transient_init || needs_returndata_init {
        let mut init: Vec<Instruction> = Vec::new();
        if let Some(mem_local) = state.memory_local {
            init.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(YUL_MEMORY_BYTES),
            )));
            init.push(Instruction::NewBuffer);
            init.push(Instruction::StoreLocal(mem_local));
        }
        if needs_transient_init {
            // `transient_allocated_here ⇒ transient_local` is a compiler
            // invariant (`ensure_transient` sets both together). Under
            // `panic = "abort"` a future refactor that broke the pairing
            // would abort the process instantly; degrade to a diagnostic
            // and roll back this block instead.
            let Some(transient_local) = state.transient_local else {
                ctx.record_error(
                    "internal: transient_allocated_here set but transient_local was not populated",
                );
                instructions.truncate(snapshot);
                return false;
            };
            init.push(Instruction::NewMap);
            init.push(Instruction::StoreLocal(transient_local));
        }
        if needs_returndata_init {
            let Some(rd_local) = state.returndata_local else {
                ctx.record_error(
                    "internal: returndata_allocated_here set but returndata_local was not populated",
                );
                instructions.truncate(snapshot);
                return false;
            };
            // Task #184 — seed with an empty ByteString so `GetSize` returns 0
            // and `returndatacopy(_, _, 0)` is a no-op. `NewBuffer(0)` would
            // emit a zero-length Buffer which also reports Size 0, but the
            // empty-ByteString literal matches the shape of a real callee's
            // ByteString return value so the follow-up "stash after call"
            // plumbing drops into the same slot without a type change.
            init.push(Instruction::PushLiteral(
                LiteralValue::ByteArray(Vec::new()),
            ));
            init.push(Instruction::StoreLocal(rd_local));
        }
        let tail = instructions.split_off(snapshot);
        instructions.extend(init);
        instructions.extend(tail);
    }

    true
}

/// Size in bytes of the Yul memory buffer. Chosen to cover the standard
/// `0x40` free-memory-pointer slot (64–95) plus a 32-byte return slot at
/// 0–31 plus some scratch space. Yul blocks that mstore above this bound
/// will trap at runtime with `MEMCPY: range out of bounds`.
pub(crate) const YUL_MEMORY_BYTES: u64 = 256;

/// Per-block lowering state. Tracks the NeoVM local slot holding the yul
/// memory buffer (lazily allocated on first mstore/mload) and a map of yul
/// variable names to NeoVM local slots.
pub(crate) struct YulLoweringState {
    memory_local: Option<usize>,
    /// Task #100 — the transient-storage map local. Unlike `memory_local`,
    /// this is allocated at the FUNCTION level (via the shared
    /// `__yul_transient` name) so it persists across multiple `assembly { }`
    /// blocks in the same function — which is what EIP-1153 requires
    /// (tstore in block A; tload in block B must return the stored value).
    transient_local: Option<usize>,
    /// True iff this block is the one that first introduced the transient
    /// map. The block prelude will emit `NEWMAP + StoreLocal` so the map is
    /// initialised before any tstore/tload. Subsequent yul blocks in the
    /// same function find the slot via `ctx.resolve_local("__yul_transient")`
    /// and skip the init.
    transient_allocated_here: bool,
    /// Task #184 — the yul-visible returndata buffer. EVM exposes the result
    /// of the most-recent external call through `returndatasize` / `returndatacopy`.
    /// NeoVM has no native returndata concept, so we model it as a per-function
    /// ByteArray local `__yul_returndata` that is initialised to the empty
    /// buffer (returndatasize = 0). Because Task #184 covers the degenerate
    /// "no preceding external call" surface only, the buffer starts empty and
    /// `returndatacopy` panics (Panic 0x32 / "returndata out of bounds") on
    /// any non-zero-length read. A follow-up task can extend this to stash
    /// the callee's return bytes after `Target(t).f()` / CALLT / DYNCALL so
    /// `returndatacopy` recovers the real payload.
    returndata_local: Option<usize>,
    /// True iff this block is the one that first introduced the returndata
    /// buffer. The block prelude will emit `PUSH0 ; NEWBUFFER ; StoreLocal`
    /// so the buffer is initialised (to zero length) before any
    /// returndatacopy. Subsequent yul blocks in the same function find the
    /// slot via `ctx.resolve_local("__yul_returndata")` and skip the init.
    returndata_allocated_here: bool,
    yul_locals: std::collections::HashMap<String, usize>,
}

impl YulLoweringState {
    pub(crate) fn new() -> Self {
        Self {
            memory_local: None,
            transient_local: None,
            transient_allocated_here: false,
            returndata_local: None,
            returndata_allocated_here: false,
            yul_locals: std::collections::HashMap::new(),
        }
    }

    /// Lazily allocate the yul memory buffer local. The caller is responsible
    /// for emitting the NEWBUFFER + StoreLocal prelude — we do this centrally
    /// in `lower_yul_block` after the whole body has been lowered so the
    /// allocation always sits before the first use.
    pub(crate) fn ensure_memory(&mut self, ctx: &mut LoweringContext) -> usize {
        if let Some(slot) = self.memory_local {
            return slot;
        }
        let slot = ctx.allocate_local("__yul_memory".to_string(), None);
        self.memory_local = Some(slot);
        slot
    }

    /// Lazily allocate the yul transient-storage map local. Unlike
    /// `ensure_memory`, this is keyed on a FUNCTION-wide name so later yul
    /// blocks in the same function pick up the same slot (and the same
    /// already-initialised map). The block that first introduces the local
    /// sets `transient_allocated_here` so `lower_yul_block` emits the
    /// NEWMAP prelude before the body.
    pub(crate) fn ensure_transient(&mut self, ctx: &mut LoweringContext) -> usize {
        if let Some(slot) = self.transient_local {
            return slot;
        }
        // If an earlier yul block in the SAME function already allocated
        // `__yul_transient`, reuse that slot — the map is already live and
        // any stored values must persist (EIP-1153 per-tx semantics).
        if let Some(existing) = ctx.resolve_local("__yul_transient") {
            self.transient_local = Some(existing);
            return existing;
        }
        // First use anywhere in the function: allocate the slot and mark
        // this block as the one responsible for emitting the NEWMAP
        // init prelude.
        let slot = ctx.allocate_local("__yul_transient".to_string(), None);
        self.transient_local = Some(slot);
        self.transient_allocated_here = true;
        slot
    }

    /// Task #184 — lazily allocate the yul returndata buffer local. Mirrors
    /// `ensure_transient` in that the slot is function-scoped (so a hypothetical
    /// future "stash return value after Target(t).f()" shim can write to the
    /// same buffer that `returndatacopy` reads from across yul blocks). The
    /// block that first introduces the slot sets `returndata_allocated_here`
    /// so `lower_yul_block` emits a `PUSH 0 ; NEWBUFFER ; StoreLocal` prelude
    /// — an empty buffer models `returndatasize() == 0`, which is what the
    /// minimal Task #184 surface needs (no preceding external call).
    pub(crate) fn ensure_returndata(&mut self, ctx: &mut LoweringContext) -> usize {
        if let Some(slot) = self.returndata_local {
            return slot;
        }
        if let Some(existing) = ctx.resolve_local("__yul_returndata") {
            self.returndata_local = Some(existing);
            return existing;
        }
        let slot = ctx.allocate_local("__yul_returndata".to_string(), None);
        self.returndata_local = Some(slot);
        self.returndata_allocated_here = true;
        slot
    }
}

/// Lower a slice of yul statements with the standard "stop on first
/// failure" semantics. Used by the `Block`, `If`, `For` (init / body /
/// post), and `Switch` (case body, default body) arms of
/// `lower_yul_statement` — six sites that previously inlined the same
/// `for s in stmts { if !lower_yul_statement(s, …) { return false; } }`
/// loop. Returns `true` iff every statement lowered successfully.
fn lower_yul_block_stmts(
    stmts: &[solang_parser::pt::YulStatement],
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    for s in stmts {
        if !lower_yul_statement(s, state, ctx, instructions) {
            return false;
        }
    }
    true
}

/// Lower a yul 2-arg arithmetic / bitwise call (`add`/`sub`/`mul` and
/// `and`/`or`/`xor`) and emit a single `BinaryOp`. The arity check,
/// recursive argument lowering, and operator lookup were duplicated
/// across the two match arms. `op_for` returns the `BinaryOperator` for
/// the matched name; the caller is responsible for restricting `name`
/// to the set the closure handles.
fn lower_two_arg_yul_call(
    call: &solang_parser::pt::YulFunctionCall,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    op_for: impl Fn(&str) -> BinaryOperator,
) -> bool {
    if call.arguments.len() != 2 {
        return false;
    }
    if !lower_yul_expression(&call.arguments[0], state, ctx, instructions) {
        return false;
    }
    if !lower_yul_expression(&call.arguments[1], state, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::BinaryOp(op_for(&call.id.name)));
    true
}

mod dispatch;
pub(crate) use dispatch::*;
mod opcodes;
pub(crate) use opcodes::*;
