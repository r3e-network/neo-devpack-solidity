//! Behavioral tests closing the two PARTIAL findings flagged by
//! `docs/audits/AUDIT_v0.22_validation.md` §8 as the required pre-PR gates:
//!
//! 1. **S7 end-to-end** — the snapshot/restore *mechanism* was unit-tested in
//!    `src/runtime/execution/helpers/storage_ops.rs::s7_tests`, but no test
//!    exercised the full `Contract.Call → Storage.Put → THROW → catch` path.
//!    An off-by-one in the wiring (snapshot on the wrong frame, restore on the
//!    wrong unwind iteration) would have left those unit tests green. This
//!    file compiles a probe that writes storage, then drives a faulting
//!    self-call through `try/catch`, and asserts the callee's write was rolled
//!    back while the caller's survived.
//!
//! 2. **M-DEV1 behavioral** — the `to != address(this)` self-escrow guard was
//!    added to `devpack/standards/NEP11.sol` `_transfer`/`_mint` and
//!    compile-verified only. This file deploys a NEP-11 subclass, mints /
//!    escrows tokens to `address(this)`, and asserts the calls succeed and the
//!    contract ends up owning the tokens — the exact scenario the audit said
//!    was hard-blocked before the fix.
//!
//! Both tests would fail on the pre-fix code, so they are genuine regression
//! guards rather than tautological smoke.

#![allow(clippy::uninlined_format_args)]

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{ExecutionResult, NeoRuntime, RuntimeConfig};

/// Test-harness gas budget. `RuntimeConfig::default().gas_limit` is 10M; after
/// the S2 fix `Storage.Put` charges the mainnet-aligned 100_000/byte rate, so
/// the NEP-11 deploy + mint path can legitimately exceed 10M. These tests are
/// not gas-asserting, so they use a generous 1B budget.
fn test_runtime() -> NeoRuntime {
    NeoRuntime::new(RuntimeConfig {
        gas_limit: 1_000_000_000,
        ..RuntimeConfig::default()
    })
    .expect("runtime")
}

fn decode_uint_le(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    for (i, b) in bytes.iter().take(8).enumerate() {
        buf[i] = *b;
    }
    u64::from_le_bytes(buf)
}

fn assert_success(result: &ExecutionResult, what: &str) {
    assert!(
        result.success,
        "{what} must succeed; exception={:?}",
        result.exception.as_ref().map(|e| &e.message)
    );
}

// ============================================================================
// S7 — end-to-end storage rollback on a caught inner-call revert
// ============================================================================

/// A self-contained probe. `run()` writes the caller marker, then drives a
/// faulting self-call through `try/catch`. `faulty()` writes the callee marker
/// and reverts. On a correct runtime the callee write is rolled back to the
/// call-frame snapshot taken in `handle_contract_call`; the caller write
/// survives because it preceded the snapshot.
const S7_PROBE: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract S7RollbackProbe {
    mapping(uint256 => uint256) m;

    /// Caller path: stage the caller marker, then catch the faulting self-call.
    function run() external returns (bool caught) {
        m[1] = 0xAA; // caller write — MUST survive the caught revert
        try this.faulty() {
            return false; // faulty() always reverts; the success arm is unreachable
        } catch {
            return true;
        }
    }

    /// Callee path: stage the callee marker, then revert. Its write MUST be
    /// rolled back by the S7 snapshot/restore wiring (the call crosses the
    /// self-offsets contract-call boundary in `handle_contract_call`).
    function faulty() external {
        m[2] = 0xBB; // callee write — MUST be discarded on revert
        revert("s7-fault");
    }

    function readCaller() external view returns (uint256) { return m[1]; }
    function readCallee() external view returns (uint256) { return m[2]; }
}
"#;

#[test]
fn s7_inner_call_revert_rolls_back_callee_storage_keeps_caller_state() {
    let arts = compile_contracts(S7_PROBE, false, 2).expect("S7 probe must compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "S7RollbackProbe")
        .expect("S7RollbackProbe artifact");
    let mut rt = test_runtime();

    // Drive the faulting self-call. `run()` must succeed and report that the
    // catch arm fired.
    let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "run", &[]);
    let r = r.expect("run() invocation");
    assert_success(&r, "S7 run()");
    assert!(
        decode_uint_le(&r.return_data) != 0,
        "S7: run() must report the catch arm fired (non-zero); got {:?}",
        r.return_data
    );

    // Caller write survives: the snapshot was taken *after* `m[1] = 0xAA`.
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "readCaller", &[])
        .expect("readCaller invocation");
    assert_success(&r, "S7 readCaller()");
    assert_eq!(
        decode_uint_le(&r.return_data),
        0xAA,
        "S7: caller's pre-call write must survive the caught revert"
    );

    // The load-bearing S7 assertion: the callee's dirty write was rolled back
    // to the frame snapshot on revert, so it never reaches the committed
    // overlay. Pre-fix this returned 0xBB (callee write leaked to top-level).
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "readCallee", &[])
        .expect("readCallee invocation");
    assert_success(&r, "S7 readCallee()");
    assert_eq!(
        decode_uint_le(&r.return_data),
        0,
        "S7: callee's Storage.Put inside the faulting inner call must be \
         rolled back — got {:#x} (leaked to the committed overlay)",
        decode_uint_le(&r.return_data)
    );
}

// ============================================================================
// M-DEV1 — NEP-11 self-escrow behavioral test
// ============================================================================

/// Minimal NEP-11 subclass exercising the `to == address(this)` paths that the
/// M-DEV1 guard unblocks (`_mint` and `_transfer` self-call short-circuit).
///
/// On real Neo N3 a contract that doesn't declare `onNEP11Payment` faults when
/// called through `Contract.Call(self, "onNEP11Payment", …)`. The bundled
/// runtime's self-offsets routing silently absorbs a missing method, so to
/// faithfully model the on-chain "receiver rejects the transfer" case the
/// probe itself implements `INEP11Receiver` with an always-reverting callback.
/// The `_transfer` catch arm turns that into `revert NEP11InvalidReceiver(to)`
/// — exactly the hard-block the M-DEV1 `to != address(this)` short-circuit
/// removes. With the guard in place the callback is never invoked on a self
/// transfer, so the escrow succeeds.
const NEP11_SELVESCROW_PROBE: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract NEP11SelfEscrowProbe is NEP11, INEP11Receiver {
    constructor() NEP11("Self", "SELF", 0, "", 0, false) {}

    /// M-DEV1 mint path: minting directly to address(this) must succeed.
    /// (_mint's catch arm is lenient, so this is a happy-path pin, not the
    /// load-bearing guard — see escrowToSelf below.)
    function mintToSelf(bytes memory tokenId) public onlyMinter {
        _mint(address(this), tokenId, "");
    }

    /// M-DEV1 transfer path: mint to the owner, then self-escrow via
    /// _transfer(owner, address(this), ...). _transfer's catch arm is strict
    /// (`revert NEP11InvalidReceiver`), so without the self-escrow guard this
    /// reverts; with the guard the callback is skipped and it succeeds.
    /// Mirrors the production pattern in CompleteNEP11NFT.listToken.
    function escrowToSelf(bytes memory tokenId) public onlyMinter {
        _mint(msg.sender, tokenId, "");
        _transfer(msg.sender, address(this), tokenId, "");
    }

    function ownAddress() external view returns (address) { return address(this); }

    /// INEP11Receiver — always rejects. The M-DEV1 guard must ensure this is
    /// never reached on a self transfer; if it is, _transfer's catch arm
    /// converts the revert into `NEP11InvalidReceiver(address(this))`.
    function onNEP11Payment(address, uint256, bytes calldata, Any calldata) external pure {
        revert("self-escrow probe rejects inbound transfer");
    }
}
"#;

fn devpack_source(probe: &str) -> String {
    [
        include_str!("../devpack/contracts/Syscalls.sol"),
        include_str!("../devpack/contracts/NativeContracts.sol"),
        include_str!("../devpack/contracts/NativeCalls.sol"),
        include_str!("../devpack/libraries/Storage.sol"),
        include_str!("../devpack/libraries/Runtime.sol"),
        include_str!("../devpack/libraries/Neo.sol"),
        include_str!("../devpack/contracts/FrameworkBase.sol"),
        include_str!("../devpack/standards/NEP11.sol"),
        probe,
    ]
    .join("\n")
}

/// The M-DEV1 probe is a NEP-11 subclass; the caller that deploys it becomes
/// `_minter`, so all mint/escrow calls below are made under this identity.
const MINTER: &str = "0x0011223344556677889900112233445566778899";

fn nep11_probe_call(
    rt: &mut NeoRuntime,
    art: &neo_devpack_solidity::cli::CompilationArtifacts,
    name: &str,
    args: &[StackItem],
) -> ExecutionResult {
    rt.call_method(&art.bytecode, &art.tokens, &art.manifest, name, args)
        .unwrap_or_else(|e| panic!("{name} invocation failed: {e:?}"))
}

#[test]
fn m_dev1_nep11_mint_to_self_succeeds_and_contract_owns_token() {
    let source = devpack_source(NEP11_SELVESCROW_PROBE);
    let arts = compile_contracts(&source, false, 2).expect("NEP-11 self-escrow compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "NEP11SelfEscrowProbe")
        .expect("NEP11SelfEscrowProbe artifact");

    let mut rt = test_runtime();
    rt.override_caller_account(MINTER)
        .expect("minter caller override");

    // Learn the contract's own address (RIPEMD160(SHA256(bytecode))) so we can
    // assert ownership afterwards. address(this) is resolved inside the
    // contract, so this avoids re-deriving the hash on the host side.
    let own = nep11_probe_call(&mut rt, art, "ownAddress", &[]);
    assert_success(&own, "ownAddress()");
    let self_addr = own.return_data.clone();
    assert_eq!(self_addr.len(), 20, "address(this) is a 20-byte Hash160");

    // M-DEV1 mint path — the load-bearing assertion. Pre-fix this reverted
    // with NEP11InvalidReceiver(address(this)).
    let id1 = StackItem::byte_array(b"self-mint-1".to_vec());
    let r = nep11_probe_call(&mut rt, art, "mintToSelf", std::slice::from_ref(&id1));
    assert_success(
        &r,
        "M-DEV1 mintToSelf — self-mint must not hit onNEP11Payment",
    );

    // The contract now owns the token.
    let r = nep11_probe_call(&mut rt, art, "ownerOf", std::slice::from_ref(&id1));
    assert_success(&r, "ownerOf(self-mint-1)");
    assert_eq!(
        r.return_data, self_addr,
        "M-DEV1: self-minted token must be owned by address(this)"
    );

    // M-DEV1 transfer path — mint to the owner, then escrow to self.
    let id2 = StackItem::byte_array(b"self-escrow-2".to_vec());
    let r = nep11_probe_call(&mut rt, art, "escrowToSelf", std::slice::from_ref(&id2));
    assert_success(
        &r,
        "M-DEV1 escrowToSelf — self-transfer must not hit onNEP11Payment",
    );

    let r = nep11_probe_call(&mut rt, art, "ownerOf", std::slice::from_ref(&id2));
    assert_success(&r, "ownerOf(self-escrow-2)");
    assert_eq!(
        r.return_data, self_addr,
        "M-DEV1: self-escrowed token must be owned by address(this)"
    );
}

// ============================================================================
// S6 follow-up — compiler-emitted CallFlags propagation
// ============================================================================

/// The audit's S6 follow-up (v0.22 validation §3) named the runtime's failure
/// to "auto-propagate a restricted flag when a compiler-emitted staticcall
/// enters a nested Contract.Call" — the Storage.Put gate only fired in
/// host-driven tests. `handle_contract_call` now saves the caller's flags on
/// the call frame, arms the callee with the caller-requested operand, and
/// restores on return.
///
/// This probe compiles a `view → view` self-call. The compiler emits
/// `CallFlags(0x05 = ReadStates|AllowCall)` for the call (see
/// `src/ir/expressions/calls/member_calls.rs`), so a propagated callee reads
/// `GetCallFlags() == 5`. Without propagation it inherits the top-level `0x0F`
/// and returns 15 — the value this test asserts against.
const S6_PROPAGATION_PROBE: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract S6FlagPropagationProbe {
    function flagsHere() external view returns (uint8) {
        return Runtime.getCallFlags();
    }
    function outerView() external view returns (uint8) {
        // `this.flagsHere()` from a view context — the compiler lowers this to
        // System.Contract.Call with CallFlags = 0x05 (ReadStates|AllowCall).
        return this.flagsHere();
    }
}
"#;

fn runtime_devpack_source(probe: &str) -> String {
    [
        include_str!("../devpack/contracts/Syscalls.sol"),
        include_str!("../devpack/contracts/NativeContracts.sol"),
        include_str!("../devpack/contracts/NativeCalls.sol"),
        include_str!("../devpack/libraries/Storage.sol"),
        include_str!("../devpack/libraries/Runtime.sol"),
        probe,
    ]
    .join("\n")
}

#[test]
fn s6_view_self_call_propagates_restricted_callflags() {
    let source = runtime_devpack_source(S6_PROPAGATION_PROBE);
    let arts = compile_contracts(&source, false, 2).expect("S6 propagation compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "S6FlagPropagationProbe")
        .expect("S6FlagPropagationProbe artifact");
    let mut rt = test_runtime();

    // Sanity: at top level the active flags are CallFlags::All (0x0F).
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "flagsHere", &[])
        .expect("flagsHere invocation");
    assert_success(&r, "flagsHere()");
    assert_eq!(
        decode_uint_le(&r.return_data),
        0x0F,
        "top-level GetCallFlags must be 0x0F (All); got {:?}",
        r.return_data
    );

    // The load-bearing S6 propagation assertion: the `view → view` self-call
    // carries CallFlags 0x05, so the callee observes 0x05 — NOT the top-level
    // 0x0F. Pre-fix this returned 0x0F (flags never propagated into the callee
    // frame, so the Notify/WriteStates/AllowCall gates could never fire inside
    // a compiler-emitted call).
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "outerView", &[])
        .expect("outerView invocation");
    assert_success(&r, "outerView()");
    assert_eq!(
        decode_uint_le(&r.return_data),
        0x05,
        "S6: a view→view self-call must propagate the caller-requested \
         CallFlags (0x05) into the callee; got {:?} — flags were not propagated",
        r.return_data
    );
}

// ============================================================================
// M-IR2 — logical-OP right-operand bool normalization
// ============================================================================

/// The audit (AUDIT_REPORT_v0.21 §M-IR2) noted that `||`/`&&` left the right
/// operand's raw value on the stack without a bool-normalizing CONVERT when the
/// short-circuit fell through to the right branch. Solidity's type system
/// guarantees `bool` operands, so the gap is only reachable via inline-assembly
/// injection or a frontend type-inference miss — but the emitted result must
/// still be a canonical Boolean for any downstream consumer that expects one.
///
/// The fix adds `CONVERT Boolean` (0xDB 0x20) after each right-operand
/// lowering. This probe pins both the structural emission and the behavioral
/// truth table.
const M_IR2_PROBE: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract LogicalProbe {
    function or_(bool a, bool b) external pure returns (bool) { return a || b; }
    function and_(bool a, bool b) external pure returns (bool) { return a && b; }
}
"#;

#[test]
fn m_ir2_logical_ops_normalize_right_operand_to_canonical_bool() {
    let arts = compile_contracts(M_IR2_PROBE, false, 0).expect("M-IR2 probe compile");
    let art = arts
        .iter()
        .find(|a| a.metadata.name == "LogicalProbe")
        .expect("LogicalProbe artifact");

    // Structural: each logical operator emits exactly one CONVERT Boolean
    // (0xDB 0x20) at the right-operand join point. Two operators ⇒ ≥2.
    // (A regression that drops the normalization lands at 0.)
    let needle = [0xDBu8, 0x20u8];
    let convert_count = art.bytecode.windows(2).filter(|w| w == &needle).count();
    assert!(
        convert_count >= 2,
        "M-IR2: expected ≥2 CONVERT Boolean (one per ||/&& right operand); \
         found {convert_count} — right-operand bool normalization regressed"
    );

    // Behavioral: the full truth table must be correct (and canonical).
    let mut rt = test_runtime();
    for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "or_",
                &[StackItem::Boolean(a), StackItem::Boolean(b)],
            )
            .expect("or_ invocation");
        assert_success(&r, "or_()");
        assert_eq!(
            r.return_data,
            [(a || b) as u8],
            "M-IR2: or_({a}, {b}) truth table"
        );

        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "and_",
                &[StackItem::Boolean(a), StackItem::Boolean(b)],
            )
            .expect("and_ invocation");
        assert_success(&r, "and_()");
        assert_eq!(
            r.return_data,
            [(a && b) as u8],
            "M-IR2: and_({a}, {b}) truth table"
        );
    }
}
