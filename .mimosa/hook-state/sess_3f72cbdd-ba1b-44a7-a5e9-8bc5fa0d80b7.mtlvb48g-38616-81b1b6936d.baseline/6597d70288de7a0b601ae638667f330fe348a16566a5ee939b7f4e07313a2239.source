use super::*;
// S6 fix — CallFlags gates + propagation in `handle_contract_call`.
use super::syscalls::{CALL_FLAG_ALL, CALL_FLAG_ALLOW_CALL};

impl ExecutionContext {
    pub(crate) fn handle_contract_call(&mut self) -> Result<(), RuntimeError> {
        // S6 fix — AllowCall gate. Neo N3 faults `System.Contract.Call` unless
        // the active context's CallFlags carry AllowCall (0b0100). A
        // read-only-shaped context without that bit (e.g. host-armed
        // ReadStates-only) may not initiate any contract call. The gate fires
        // before the stack is touched so a rejecting context leaves the eval
        // stack intact for the fault handler.
        if self.active_call_flags & CALL_FLAG_ALLOW_CALL == 0 {
            return Err(RuntimeError::ExecutionError {
                message: "System.Contract.Call requires CallFlag.AllowCall".to_string(),
            });
        }
        // Neo N3 syscall convention: the first argument is at the top of the stack.
        // For System.Contract.Call(hash, method, flags, args), the evaluation stack
        // order is: [args, flags, method, hash].
        let contract_item = self.pop_stack()?;
        let method_item = self.pop_stack()?;
        // S6 fix — the flags operand is the MAXIMUM CallFlags the caller grants
        // the callee. Neo N3 rejects any value outside the 4-bit mask; the callee
        // then runs with exactly these flags (see the self-offsets arm below).
        let requested_flags = Self::stack_item_to_int(self.pop_stack()?) as u8;
        if requested_flags & !CALL_FLAG_ALL != 0 {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "System.Contract.Call: invalid CallFlags 0x{requested_flags:02X} (exceeds All = 0x0F)"
                ),
            });
        }
        let params = self.pop_stack()?;

        let method = String::from_utf8(Self::stack_item_to_bytes(method_item)).unwrap_or_default();
        let contract_bytes = Self::stack_item_to_bytes(contract_item);
        let mut hash = [0u8; 20];
        for (i, b) in contract_bytes.iter().take(20).enumerate() {
            hash[i] = *b;
        }

        // Foundry-style cheatcodes (neo-test): a call to the well-known HEVM
        // cheatcode address is intercepted and serviced by the runtime itself
        // (vm.prank / warp / roll / deal / expectRevert / …). Handled before
        // the self-offsets, native, and manifest-permission paths so the cheat
        // address never needs a manifest permission and never collides with a
        // real contract. Returns one StackItem result, matching the shape the
        // external-call caller site expects.
        if Self::is_cheatcode_address(&contract_bytes) {
            return self.handle_cheatcode(&method, &params);
        }

        // Task #70: route `this.someFn()` self external calls to the
        // compiled method offset from the manifest, so `invoke_native_contract`
        // doesn't return `Null` (silently 0/false for typed returns).
        //
        // Task #83 (batch #33 K1): also route the 20-byte zero placeholder
        // that `new Contract()` pushes for factory-deploy receivers — the
        // analyse phase merges the sibling's public methods into the host
        // (see `analyse_all_sources`), so `self_method_offsets` already
        // carries an entry for the target method name.
        let is_zero_placeholder = hash.iter().all(|b| *b == 0);
        if hash[..] == self.default_account_bytes[..] || is_zero_placeholder {
            // Task #126 — if the explicitly-named method isn't in the merged
            // self-offsets table (e.g. `Target(t).nonExistentMethod()` where
            // `TargetImpl` declares only `fallback()`), route to the
            // `fallback` entry instead. This mirrors Solidity's ABI
            // dispatcher: an unknown selector on a contract with a
            // `fallback()` falls through to that fallback. Without this
            // redirect, the unknown-method path would drop into
            // `invoke_native_contract`, which returns `Null` for the zero-
            // placeholder hash — silently absorbing the call and letting
            // the caller's try-arm fire with a bogus zero/empty result
            // instead of propagating the fallback's revert envelope.
            let resolved_offset = self
                .self_method_offsets
                .get(&method)
                .copied()
                .or_else(|| self.self_method_offsets.get("fallback").copied());
            if let Some(target_offset) = resolved_offset {
                let args_vec = match &params {
                    StackItem::Array(items) => items.borrow().clone(),
                    _ => Vec::new(),
                };
                // The SYSCALL opcode handler normally post-increments the
                // instruction pointer by 5 (see instruction/syscall.rs). The
                // return address must therefore be `instruction_pointer + 5`
                // (i.e. past the SYSCALL). We set IP directly to the target
                // method's compiled offset and raise the `suppress_ip_advance`
                // flag so the SYSCALL dispatcher skips its post-increment.
                let return_address = self.instruction_pointer.wrapping_add(5);
                // The nested callee's `msg.sender` (`CallingScriptHash`) must
                // read back the *immediate caller's* `address(this)` — i.e.
                // the bundle's executing script hash (`default_account_bytes`).
                // In the merged sibling-dispatch model (Task #83
                // `analyse_all_sources`), the caller always executes as the
                // bundle, so its `address(this)` IS `default_account_bytes`.
                //
                // This is what makes the canonical test-runner pattern hold:
                //   B b = new B(); b.deposit();   // bal[msg.sender] += 1
                //   assert(b.balanceOf(address(this)) == 1);
                // `msg.sender` inside `deposit` now equals the calling
                // contract's `address(this)`, so the storage slot keyed on
                // `msg.sender` is the same slot read back via `address(this)`.
                //
                // It also keeps the nested `msg.sender != tx.origin` invariant
                // honest. `GetEntryScriptHash` is now a DISTINCT entry-script
                // identity (see `entry_script_hash`), so the `msg.sender`
                // lowering's `CallingScriptHash == EntryScriptHash` top-level
                // short-circuit does NOT fire for this inner hop
                // (`default_account_bytes != entry_script_hash`). Meanwhile
                // `Transaction.Sender` (which backs `tx.origin`) stays pinned
                // to the entry caller. So `msg.sender` (the bundle hash) and
                // `tx.origin` (the entry hash) diverge here exactly as they do
                // on a real node, where `CallingScriptHash` rolls over to the
                // direct caller while `EntryScriptHash`/`Transaction.Sender`
                // remain anchored to the transaction's entry frame.
                // A `vm.prank(addr)` (one-shot) or `vm.startPrank(addr)`
                // (sticky) overrides the caller identity for this hop, exactly
                // like Foundry: only `msg.sender` changes, `tx.origin`
                // (Transaction.Sender) stays pinned to the entry caller.
                let caller_executing_hash = if let Some(p) = self.pending_prank.take() {
                    p
                } else if let Some(s) = self.sticky_prank.clone() {
                    s
                } else {
                    self.default_account_bytes.clone()
                };
                // Task #197 — push the call frame BEFORE unpacking args onto
                // the evaluation stack. The frame's `stack_base` snapshots
                // `self.stack.len()` at the moment of creation, and
                // `return_from_function` later uses
                // `self.stack.len() > frame.stack_base` to decide whether the
                // callee produced a return value. If the args are pushed
                // first, the stack_base is inflated by `args.len()`, and the
                // callee's INITSLOT pops them back down — so a single-return
                // callee leaves `stack.len() == stack_base` on the caller's
                // side, the check reads as "no return", and the Task #160
                // `syscall_result_expected` path synthesises a spurious Null
                // that clobbers the real return value. Moving the frame
                // push ahead of the arg push keeps stack_base at the
                // pre-call baseline, so a callee producing exactly one
                // result lands at `stack_base + 1` and the check fires
                // correctly.
                self.push_call_frame(return_address)?;
                // S7 fix — capture the storage-overlay snapshot BEFORE the
                // mutable borrow of call_stack below (snapshot_storage_overlay
                // borrows self too, which would conflict with last_mut).
                let storage_snapshot = Some(self.snapshot_storage_overlay());
                // S6 fix — capture the caller's flags BEFORE the mutable borrow
                // so we can (a) stash them on the frame for restore-on-return
                // and (b) arm the callee's active flags after the borrow ends.
                let caller_flags = self.active_call_flags;
                // neo-test `vm.expectRevert`: transfer a one-shot armed
                // expectation onto THIS call's frame, so a revert that unwinds
                // here is swallowed (and a normal return is a violation).
                let expect_revert_guard = self.expect_revert.take();
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.msg_sender_override = Some(caller_executing_hash);
                    frame.expect_revert_guard = expect_revert_guard;
                    // Task #160 — mark the frame so `return_from_function`
                    // synthesises a `StackItem::Null` result for void callees.
                    // The caller site emitted by `try_catch.rs` (and the generic
                    // `lower_expression_statement` path) assumes every external
                    // call leaves exactly one syscall result on the evaluation
                    // stack — matching what `invoke_native_contract` does on
                    // the non-zero-placeholder branch below. Without this flag,
                    // a void target's `ReturnVoid` (`0x40`) pushes nothing, and
                    // the caller's implicit `DROP` faults into the catch arm
                    // — turning a happy-path `try Target(t).voidFn() {}` into
                    // an unintended catch firing.
                    frame.syscall_result_expected = true;
                    // S7 fix — attach the snapshot captured above.
                    frame.storage_snapshot = storage_snapshot;
                    // S6 fix — record the caller's flags so `return_from_function`
                    // / `dispatch_exception` can restore them when this frame
                    // unwinds. Only set on contract-call frames (plain CALL_L
                    // frames stay `None`), so internal Solidity calls never
                    // disturb the active flags.
                    frame.saved_call_flags = Some(caller_flags);
                }
                // S6 fix — the callee now runs with the caller-granted flags.
                // This is what makes the Notify/AllowCall/WriteStates gates
                // fire inside compiler-emitted `view`/`pure` callees (which
                // carry 0x05 = ReadStates|AllowCall), not just in host-armed
                // restricted contexts. The caller's flags are restored when the
                // frame returns or unwinds.
                self.active_call_flags = requested_flags;
                // Unpack the params array and push args in reverse so that the
                // target method's INITSLOT pops them into arg slots in order.
                for arg in args_vec.iter().rev() {
                    self.push_stack(arg.clone())?;
                }
                self.instruction_pointer = target_offset;
                self.syscall_suppress_ip_advance = true;
                // Do NOT push a result here: the target method's RET will
                // leave its return value on the evaluation stack, which is
                // exactly what the caller (the SYSCALL site) expects.
                return Ok(());
            }
        }

        // S6 follow-up — manifest permission check. Neo N3 faults any
        // `System.Contract.Call` whose target hash + method is not declared in
        // the calling contract's manifest `permissions` array. Self-calls are
        // exempt by construction (they took the self-offsets return above; the
        // compiler deliberately omits self-permissions — see
        // `permissions/calls.rs`). When no manifest is loaded (raw `execute()`
        // path) `manifest_permits` short-circuits to `true`.
        if !self.manifest_permits(&hash, &method) {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "System.Contract.Call: manifest does not permit call to {method} on 0x{}",
                    hex::encode(hash.iter().rev().copied().collect::<Vec<u8>>())
                ),
            });
        }

        let result = self.invoke_native_contract(&hash, &method, params);
        self.push_stack(result)?;
        Ok(())
    }

    /// The transaction's entry-script hash (`System.Runtime.GetEntryScriptHash`).
    ///
    /// On a real Neo node the entry script hash is the hash of the
    /// transaction's invocation script — a value that is DISTINCT from any
    /// deployed contract's own script hash. That distinctness is load-bearing
    /// for the `msg.sender` lowering in
    /// `src/cli/bytecode/bytecode_helpers/array_runtime.rs`
    /// (`RuntimeValue::MsgSender`): it collapses `msg.sender` to
    /// `Transaction.Sender` only when `CallingScriptHash == EntryScriptHash`,
    /// which on-chain is true exactly for the genuine top-level entry frame
    /// (the entry script invoked the contract directly) and false for any
    /// inner contract-to-contract hop (where `CallingScriptHash` has rolled
    /// over to the caller contract).
    ///
    /// The embedded runtime previously conflated entry / executing by
    /// returning `default_account_bytes` for both syscalls, which made the
    /// short-circuit misfire whenever the entry contract called a merged
    /// sibling (`CallingScriptHash` == the bundle hash == `EntryScriptHash`),
    /// leaking `tx.origin` into a nested `msg.sender`. Deriving a separate,
    /// deterministic entry identity restores the on-chain relationship.
    pub fn entry_script_hash(&self) -> Vec<u8> {
        Self::derive_entry_script_hash(&self.default_account_bytes)
    }

    /// Deterministic derivation of the distinct entry-script hash from the
    /// bundle's executing script hash. Domain-separated so it never collides
    /// with `default_account_bytes` (the executing hash / `address(this)`) nor
    /// with any other `Hash160` preimage the compiler emits. Determinism keeps
    /// `tx.origin` / `EntryScriptHash` byte content stable across runs.
    fn derive_entry_script_hash(executing: &[u8]) -> Vec<u8> {
        use ripemd::Ripemd160;
        use sha2::{Digest, Sha256};
        const DOMAIN: &[u8] = b"neo-devpack-solidity/entry-script-hash/v1";
        let mut preimage = Vec::with_capacity(DOMAIN.len() + executing.len());
        preimage.extend_from_slice(DOMAIN);
        preimage.extend_from_slice(executing);
        let sha = Sha256::digest(&preimage);
        let hash: [u8; 20] = Ripemd160::digest(sha).into();
        hash.to_vec()
    }

    /// Task #123 — resolve the active `msg.sender` (`CallingScriptHash`) for
    /// the current frame.
    ///
    /// Walks the call stack from the most recent frame downwards, returning
    /// the first `msg_sender_override` that is populated. When no frame
    /// carries an override (entry-depth execution, or a plain `CALL` frame
    /// from the Solidity internal-call lowering), this returns `None` and the
    /// `System.Runtime.GetCallingScriptHash` handler falls through to its
    /// pre-Task-123 `caller_account`-first behaviour.
    pub(crate) fn active_msg_sender_override(&self) -> Option<Vec<u8>> {
        for frame in self.call_stack.iter().rev() {
            if let Some(bytes) = frame.msg_sender_override.as_ref() {
                return Some(bytes.clone());
            }
        }
        None
    }

    /// The well-known Foundry/HEVM cheatcode address,
    /// `0x7109709ECfa91a80626fF3989D68f67F5b1DD12D`. neo-test's `Vm` interface
    /// (bundled `neo-std/Vm.sol`) is bound to this address; the runtime
    /// services calls to it directly (see `handle_cheatcode`).
    const CHEATCODE_ADDRESS: [u8; 20] = [
        0x71, 0x09, 0x70, 0x9E, 0xCF, 0xA9, 0x1A, 0x80, 0x62, 0x6F, 0xF3, 0x98, 0x9D, 0x68, 0xF6,
        0x7F, 0x5B, 0x1D, 0xD1, 0x2D,
    ];

    /// Whether `bytes` is the cheatcode address in either byte order (the
    /// compiler may push an address literal big-endian or little-endian
    /// depending on the call shape; matching both keeps the cheat robust).
    fn is_cheatcode_address(bytes: &[u8]) -> bool {
        if bytes.len() < 20 {
            return false;
        }
        let head = &bytes[..20];
        if head == Self::CHEATCODE_ADDRESS {
            return true;
        }
        let mut reversed = Self::CHEATCODE_ADDRESS;
        reversed.reverse();
        head == reversed
    }

    /// Service a Foundry-style cheatcode call. Mutates runtime state for the
    /// recognised cheats and always pushes exactly one `StackItem` result
    /// (cheats return `void`, so `Null`), matching the shape the external-call
    /// caller site expects. Unknown cheats are accepted as no-ops so a newer
    /// `Vm.sol` surface never faults an older runtime.
    fn handle_cheatcode(&mut self, method: &str, params: &StackItem) -> Result<(), RuntimeError> {
        let args: Vec<StackItem> = match params {
            StackItem::Array(items) => items.borrow().clone(),
            _ => Vec::new(),
        };
        let arg_int = |i: usize| -> i64 {
            args.get(i)
                .map(|a| Self::stack_item_to_int(a.clone()))
                .unwrap_or(0)
        };
        let arg_bytes = |i: usize| -> Vec<u8> {
            args.get(i)
                .map(|a| Self::stack_item_to_bytes(a.clone()))
                .unwrap_or_default()
        };

        match method {
            // vm.warp(uint256) — set block.timestamp (SECONDS, Foundry
            // semantics) for subsequent reads. Neo's `GetTime` is milliseconds
            // and the `block.timestamp` lowering divides by 1000, so store
            // seconds * 1000 to round-trip exactly.
            "warp" => {
                self.timestamp = Some((arg_int(0).max(0) as u64).saturating_mul(1000));
            }
            // vm.roll(uint256) — set block.number / Ledger.currentIndex.
            "roll" => {
                self.block_height = Some(arg_int(0).max(0) as u64);
            }
            // vm.prank(address) — the NEXT cross-contract call observes
            // `address` as msg.sender (one-shot). tx.origin is unaffected.
            "prank" => {
                self.pending_prank = Some(arg_bytes(0));
            }
            // vm.startPrank(address) — every subsequent call until stopPrank.
            "startPrank" => {
                self.sticky_prank = Some(arg_bytes(0));
            }
            // vm.stopPrank() — clear both prank modes.
            "stopPrank" => {
                self.pending_prank = None;
                self.sticky_prank = None;
            }
            // vm.deal(address, uint256) — set the account's GAS balance.
            "deal" => {
                self.gas_balances
                    .insert(arg_bytes(0), arg_int(1).max(0) as u64);
            }
            // vm.label(address, string) — cosmetic in Foundry; accept + ignore.
            "label" => {}
            // vm.assume(bool) — reject the current input when false. In the
            // non-fuzzing runner this surfaces as a revert the runner reports.
            "assume" if arg_int(0) == 0 => {
                return Err(RuntimeError::ExecutionError {
                    message: "vm.assume(false): rejected input".to_string(),
                });
            }
            "assume" => {}
            // vm.expectRevert() / vm.expectRevert(bytes) — arm the next-call
            // revert expectation. `expectRevert()` (no args) matches any
            // revert; `expectRevert(payload)` requires an exact payload match.
            "expectRevert" => {
                self.expect_revert = Some(if args.is_empty() {
                    None
                } else {
                    Some(arg_bytes(0))
                });
            }
            _ => {}
        }

        // Cheats return void — push one Null to match the external-call shape.
        self.push_stack(StackItem::Null)?;
        Ok(())
    }
}
