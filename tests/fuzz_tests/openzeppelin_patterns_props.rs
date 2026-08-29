//! End-to-end proptests for OpenZeppelin-style real-world Solidity patterns.
//!
//! These are the four most-deployed access/state patterns in production EVM
//! contracts:
//!
//!   * `Ownable` — single-owner gate with `onlyOwner` modifier and
//!     `transferOwnership(address)`.
//!   * `Pausable` — `whenNotPaused` / `whenPaused` modifiers gated by a
//!     `bool paused` slot, owner toggles via `pause()` / `unpause()`.
//!   * `AccessControl` — role bitmap (`mapping(bytes32 => mapping(address =>
//!     bool))`) with `grantRole` / `revokeRole` admin gates.
//!   * `ERC20Permit`-style nonce — per-account `mapping(address => uint256)
//!     nonces` that increments on each call, pinned to the supplied user (NOT
//!     msg.sender — that is the load-bearing semantic for permit).
//!
//! `ReentrancyGuard` is intentionally NOT covered here — it is already
//! exhaustively fuzzed by `tests/fuzz_tests/reentrancy_props.rs`
//! (see `reentrancy_guard_blocks_self_call`, which uses the canonical
//! `_status` 1/2 toggle and exercises both try/catch and bubble-up shapes
//! across the same Task #70 self-dispatch path that `nonReentrant` relies
//! on). Duplicating it here would shadow the more-specific harness.
//!
//! Each test inlines the OZ pattern (no external imports — `compile_contracts`
//! cannot resolve `import "@openzeppelin/...";`) into a < 100-line contract,
//! deploys it via `NeoRuntime::call_method` / `call_method_with_deploy_args`
//! (mirroring the harness in `tests/fuzz_tests/native_contract_props.rs` and
//! `tests/fuzz_tests/batches_116_120.rs::batch124_erc20_full_lifecycle`), and
//! drives a multi-step sequence with cross-step assertions. Caller identity
//! is steered via `NeoRuntime::override_caller_account` (Task #176 sticky
//! caller) — the same mechanism `batch124` uses to thread distinct
//! `msg.sender` values into approve/transferFrom.
//!
//! A failing assertion in any of these tests is a real bug. If `onlyOwner`
//! does not fire on a stranger call, every Ownable contract on Neo silently
//! permits unauthorized state changes. If `whenNotPaused` fails to gate
//! while `paused == true`, every emergency-pause shutdown is a no-op. If
//! `hasRole(role, addr)` returns false after a successful `grantRole`, every
//! AccessControl-gated function rejects the granted account. If `nonces`
//! does not increment by exactly 1 (or affects the wrong account), every
//! permit-style off-chain signature flow signs the wrong nonce and the
//! second permit replays.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use num_bigint::BigUint;
use proptest::prelude::*;

proptest! {
    // 4 cases — each test compiles + deploys + drives a multi-step contract
    // sequence (1-3s per case). The semantic property is invariant in the
    // role-byte / nonce-count fuzz axis; we shrink only across the chosen
    // role identifier or nonce-call count, not the contract structure.
    #![proptest_config(ProptestConfig::with_cases(4))]

    /// **a. `oz_ownable_only_owner_modifier`**
    ///
    /// Inlined `Ownable` mirror of the OpenZeppelin pattern: constructor sets
    /// `owner = msg.sender`, `onlyOwner` modifier reverts non-owners,
    /// `transferOwnership(address)` rotates the owner slot.
    ///
    /// Sequence:
    ///   1. Deploy as alice — alice is owner, `getOwner()` returns alice.
    ///   2. alice calls `transferOwnership(bob)` — must succeed, owner = bob.
    ///   3. bob calls `transferOwnership(carol)` — must succeed, owner = carol.
    ///   4. alice calls `transferOwnership(alice)` — must REVERT (alice is no
    ///      longer owner). Owner must still equal carol after the failed call.
    ///
    /// What a failure means:
    ///   - Step 2/3 fails: the `onlyOwner` modifier rejected the legitimate
    ///     owner — either msg.sender is wrong (Task #176 sticky-caller
    ///     regression) or the owner-write inside `transferOwnership` did not
    ///     persist.
    ///   - Step 4 SUCCEEDS: critical access-control bypass — the modifier did
    ///     not fire. Every Ownable contract on the chain is unguarded.
    ///   - Step 4 owner-readback != carol: even if the reverting call's
    ///     storage write was rolled back, owner must still be carol from
    ///     step 3. A different value indicates revert-time storage rollback
    ///     leaked or the modifier wrote owner before the require fired.
    #[test]
    fn oz_ownable_only_owner_modifier(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Ownable {
    address private _owner;

    constructor() {
        _owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == _owner, "Ownable: caller is not the owner");
        _;
    }

    function getOwner() external view returns (address) {
        return _owner;
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Ownable: new owner is the zero address");
        _owner = newOwner;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Ownable compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "Ownable produced no artifacts");
        let art = &arts[0];

        // Uniform-byte addresses keep LE/BE byte order indistinguishable so
        // the test isolates dispatch logic from any address-encoding
        // convention (mirroring batch124).
        let alice: [u8; 20] = [0xAA; 20];
        let bob: [u8; 20] = [0xBB; 20];
        let carol: [u8; 20] = [0xCC; 20];
        let alice_hex = format!("0x{}", hex::encode(alice));
        let bob_hex = format!("0x{}", hex::encode(bob));

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Step 1: deploy as alice + readback owner.
        rt.override_caller_account(&alice_hex)
            .expect("alice override");
        let r0 = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "getOwner", &[] as &[StackItem], None,
            )
            .expect("Ownable deploy + getOwner host-level");
        prop_assert!(r0.success,
            "Step 1 (deploy as alice + getOwner): must succeed; exc={:?}.",
            r0.exception.as_ref().map(|e| &e.message));
        // Owner readback should contain alice's bytes (0xAA repeating).
        prop_assert!(r0.return_data.iter().any(|b| *b == 0xAA),
            "Step 1: getOwner() after deploy must contain alice (0xAA*) bytes; \
             rd_hex={}. If the return is alice-free, the constructor's \
             `_owner = msg.sender` either ran in the wrong scope (msg.sender \
             not alice) or the override_caller_account didn't propagate \
             into the deploy entrypoint.",
            hex::encode(&r0.return_data));

        // Step 2: alice transfers ownership to bob.
        rt.override_caller_account(&alice_hex)
            .expect("alice re-override");
        let r1 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "transferOwnership",
                &[StackItem::byte_array(bob.to_vec())],
            )
            .expect("alice.transferOwnership(bob) host-level");
        prop_assert!(r1.success,
            "Step 2: alice.transferOwnership(bob) must succeed (alice IS the \
             owner); exc={:?}. If it reverts citing 'caller is not the \
             owner', either msg.sender inside the modifier is not alice \
             (Task #176 sticky-caller regression) or _owner does not equal \
             alice (constructor write didn't land).",
            r1.exception.as_ref().map(|e| &e.message));

        // Step 3: bob transfers ownership to carol.
        rt.override_caller_account(&bob_hex)
            .expect("bob override");
        let r2 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "transferOwnership",
                &[StackItem::byte_array(carol.to_vec())],
            )
            .expect("bob.transferOwnership(carol) host-level");
        prop_assert!(r2.success,
            "Step 3: bob.transferOwnership(carol) must succeed (bob IS now \
             the owner after step 2); exc={:?}. If it reverts, the owner \
             write from step 2 did not persist to bob, or msg.sender inside \
             the modifier is not bob.",
            r2.exception.as_ref().map(|e| &e.message));

        // Step 4: alice (now NOT owner) tries to transfer — must revert.
        rt.override_caller_account(&alice_hex)
            .expect("alice override (post-rotation, no longer owner)");
        let r3 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "transferOwnership",
                &[StackItem::byte_array(alice.to_vec())],
            )
            .expect("alice.transferOwnership(alice) host-level");
        prop_assert!(!r3.success,
            "CRITICAL Step 4: alice (no longer owner — owner is carol) \
             called transferOwnership and IT SUCCEEDED. The `onlyOwner` \
             modifier did not fire. Every Ownable contract on Neo is \
             unguarded. rd_hex={}",
            hex::encode(&r3.return_data));
        let exc3 = r3.exception.as_ref().map(|e| e.message.as_str()).unwrap_or("");
        let in_exc = exc3.to_lowercase().contains("not the owner")
            || exc3.to_lowercase().contains("ownable");
        let in_rd = r3.return_data
            .windows(b"not the owner".len())
            .any(|w| w.eq_ignore_ascii_case(b"not the owner"))
            || r3.return_data
                .windows(b"Ownable".len())
                .any(|w| w == b"Ownable");
        prop_assert!(in_exc || in_rd,
            "Step 4 reverted but the message did NOT cite the modifier's \
             revert string. exc_msg={:?}, rd_hex={}. If neither contains \
             'Ownable' / 'not the owner', the revert came from a different \
             source (e.g. dispatch fault) and we cannot confirm the \
             modifier itself fired.",
            exc3, hex::encode(&r3.return_data));

        // Step 5: confirm owner is still carol after the failed call.
        let r4 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "getOwner", &[] as &[StackItem],
            )
            .expect("getOwner post-revert host-level");
        prop_assert!(r4.success, "Step 5: getOwner() must succeed; exc={:?}",
            r4.exception.as_ref().map(|e| &e.message));
        prop_assert!(r4.return_data.iter().any(|b| *b == 0xCC),
            "Step 5: getOwner() after the failed alice call must still \
             contain carol's bytes (0xCC*); got rd_hex={}. If the bytes \
             show alice (0xAA) the failed transferOwnership leaked a \
             storage write past the require — which would be a revert-\
             time rollback bug.",
            hex::encode(&r4.return_data));
    }

    /// **b. `oz_pausable_blocks_when_paused`**
    ///
    /// Inlined `Pausable` mirror: a `bool _paused` slot guards `whenNotPaused`
    /// / `whenPaused` modifiers; only the owner can flip it via `pause()` and
    /// `unpause()`.
    ///
    /// Sequence:
    ///   1. Deploy as alice (owner) — `protectedCall()` returns the sentinel.
    ///   2. alice calls `pause()` — paused becomes true.
    ///   3. `protectedCall()` MUST revert ("Pausable: paused").
    ///   4. alice calls `unpause()` — paused becomes false.
    ///   5. `protectedCall()` succeeds again.
    ///
    /// What a failure means:
    ///   - Step 3 succeeds: `whenNotPaused` did not fire while paused == true
    ///     — every Pausable shutdown is a no-op.
    ///   - Step 5 reverts: pause toggle didn't restore — the unpause write
    ///     never landed, or `_paused` was read-locked.
    #[test]
    fn oz_pausable_blocks_when_paused(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Pausable {
    address private _owner;
    bool private _paused;

    constructor() {
        _owner = msg.sender;
        _paused = false;
    }

    modifier onlyOwner() {
        require(msg.sender == _owner, "Pausable: not owner");
        _;
    }
    modifier whenNotPaused() {
        require(!_paused, "Pausable: paused");
        _;
    }
    modifier whenPaused() {
        require(_paused, "Pausable: not paused");
        _;
    }

    function isPaused() external view returns (bool) {
        return _paused;
    }
    function pause() external onlyOwner whenNotPaused {
        _paused = true;
    }
    function unpause() external onlyOwner whenPaused {
        _paused = false;
    }
    function protectedCall() external whenNotPaused returns (uint256) {
        return 0xCAFE;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Pausable compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "Pausable produced no artifacts");
        let art = &arts[0];

        let alice: [u8; 20] = [0xAA; 20];
        let alice_hex = format!("0x{}", hex::encode(alice));
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Step 1: deploy + protectedCall must succeed (paused=false default).
        rt.override_caller_account(&alice_hex)
            .expect("alice override (deployer)");
        let r0 = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "protectedCall", &[] as &[StackItem], None,
            )
            .expect("Pausable deploy+protectedCall host-level");
        prop_assert!(r0.success,
            "Step 1: protectedCall() on a freshly-deployed (unpaused) \
             contract must succeed; exc={:?}. If it reverts citing \
             'paused', the constructor's `_paused = false` initializer \
             didn't write OR bool-default-false logic regressed (a fresh \
             slot reads as zero / false anyway, so a revert here would \
             indicate the require itself read the wrong slot).",
            r0.exception.as_ref().map(|e| &e.message));
        let v0 = decode_uint_le(&r0.return_data);
        prop_assert_eq!(v0.clone(), BigUint::from(0xCAFEu64),
            "Step 1: protectedCall() must return 0xCAFE; got {} (rd_hex={}).",
            v0, hex::encode(&r0.return_data));

        // Step 2: alice pauses.
        rt.override_caller_account(&alice_hex).expect("alice");
        let r1 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "pause", &[] as &[StackItem])
            .expect("pause host-level");
        prop_assert!(r1.success,
            "Step 2: alice.pause() must succeed (alice is owner, not paused); \
             exc={:?}. Failure here means either the owner check rejects \
             alice (sticky-caller regression) OR whenNotPaused rejected \
             even though _paused is false (default-bool read regression).",
            r1.exception.as_ref().map(|e| &e.message));

        // Step 3: protectedCall MUST revert.
        let r2 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "protectedCall", &[] as &[StackItem])
            .expect("protectedCall host-level");
        prop_assert!(!r2.success,
            "CRITICAL Step 3: protectedCall() succeeded while paused=true. \
             The whenNotPaused modifier did not fire. Every Pausable \
             contract on Neo is incapable of emergency shutdown. \
             rd_hex={}", hex::encode(&r2.return_data));
        let exc2 = r2.exception.as_ref().map(|e| e.message.as_str()).unwrap_or("");
        let in_exc = exc2.to_lowercase().contains("paused");
        let in_rd = r2.return_data
            .windows(b"paused".len())
            .any(|w| w.eq_ignore_ascii_case(b"paused"));
        prop_assert!(in_exc || in_rd,
            "Step 3: protectedCall reverted but the message did NOT cite \
             'paused'. exc={:?}, rd_hex={}. The revert source is unclear.",
            exc2, hex::encode(&r2.return_data));

        // Step 4: alice unpauses.
        rt.override_caller_account(&alice_hex).expect("alice");
        let r3 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "unpause", &[] as &[StackItem])
            .expect("unpause host-level");
        prop_assert!(r3.success,
            "Step 4: alice.unpause() must succeed (alice is owner, paused); \
             exc={:?}. Failure here means whenPaused rejected even though \
             pause() succeeded — the _paused write didn't persist.",
            r3.exception.as_ref().map(|e| &e.message));

        // Step 5: protectedCall succeeds again.
        let r4 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "protectedCall", &[] as &[StackItem])
            .expect("protectedCall post-unpause host-level");
        prop_assert!(r4.success,
            "Step 5: protectedCall() must succeed after unpause; exc={:?}. \
             Failure means the unpause write didn't flip _paused back to \
             false (storage commit regression).",
            r4.exception.as_ref().map(|e| &e.message));
        let v4 = decode_uint_le(&r4.return_data);
        prop_assert_eq!(v4.clone(), BigUint::from(0xCAFEu64),
            "Step 5: protectedCall() post-unpause must return 0xCAFE; got \
             {} (rd_hex={}).", v4, hex::encode(&r4.return_data));
    }

    /// **c. `oz_access_control_role_grants`**
    ///
    /// Inlined `AccessControl` mirror: `mapping(bytes32 => mapping(address =>
    /// bool)) _roles`, with `hasRole`, `grantRole`, `revokeRole`. Only the
    /// admin (set in the constructor for simplicity — the OZ original uses
    /// per-role admins, which we collapse to a single `DEFAULT_ADMIN_ROLE`
    /// bound to the deployer for a < 100-line contract).
    ///
    /// Sequence:
    ///   1. Deploy as alice (admin) — `hasRole(ROLE_X, bob)` is false.
    ///   2. alice calls `grantRole(ROLE_X, bob)`.
    ///   3. `hasRole(ROLE_X, bob)` is now true.
    ///   4. alice calls `revokeRole(ROLE_X, bob)`.
    ///   5. `hasRole(ROLE_X, bob)` is false again.
    #[test]
    fn oz_access_control_role_grants(role_byte in 1u8..=255u8) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract AccessControl {{
    bytes32 public constant DEFAULT_ADMIN_ROLE = bytes32(0);
    mapping(bytes32 => mapping(address => bool)) private _roles;

    constructor() {{
        _roles[DEFAULT_ADMIN_ROLE][msg.sender] = true;
    }}

    modifier onlyAdmin() {{
        require(_roles[DEFAULT_ADMIN_ROLE][msg.sender],
                "AccessControl: missing admin role");
        _;
    }}

    function hasRole(bytes32 role, address account) external view returns (bool) {{
        return _roles[role][account];
    }}
    function grantRole(bytes32 role, address account) external onlyAdmin {{
        _roles[role][account] = true;
    }}
    function revokeRole(bytes32 role, address account) external onlyAdmin {{
        _roles[role][account] = false;
    }}
    function ROLE_X() external pure returns (bytes32) {{
        return bytes32(uint256({}));
    }}
}}"#, role_byte);
        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("AccessControl compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "AccessControl produced no artifacts");
        let art = &arts[0];

        let alice: [u8; 20] = [0xAA; 20];
        let bob: [u8; 20] = [0xBB; 20];
        let alice_hex = format!("0x{}", hex::encode(alice));
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // ROLE_X is bytes32(uint256(role_byte)) — for a one-byte uint, the
        // bytes32 right-pads with zeros at LSBs (actually left-pads with
        // zeros at MSBs in big-endian: 0x00..00<role_byte>). The runtime
        // stores bytes32 as raw big-endian; we mirror that here for the
        // call-site value.
        let mut role_x = [0u8; 32];
        role_x[31] = role_byte;

        // Step 1: deploy + check role NOT granted yet.
        rt.override_caller_account(&alice_hex).expect("alice");
        let r0 = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "hasRole",
                &[
                    StackItem::byte_array(role_x.to_vec()),
                    StackItem::byte_array(bob.to_vec()),
                ],
                None,
            )
            .expect("AccessControl deploy + hasRole(bob) host-level");
        prop_assert!(r0.success,
            "Step 1 (deploy + hasRole): exc={:?}",
            r0.exception.as_ref().map(|e| &e.message));
        // bob has NO role — return must decode to 0/false.
        let v0_zero = r0.return_data.is_empty()
            || r0.return_data.iter().all(|b| *b == 0);
        prop_assert!(v0_zero,
            "Step 1: hasRole(ROLE_X, bob) on a fresh contract must be false; \
             got rd_hex={}. If non-zero, the role-mapping default-false read \
             regressed (a fresh nested mapping should never read truthy).",
            hex::encode(&r0.return_data));

        // Step 2: alice grants ROLE_X to bob.
        rt.override_caller_account(&alice_hex).expect("alice");
        let r1 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "grantRole",
                &[
                    StackItem::byte_array(role_x.to_vec()),
                    StackItem::byte_array(bob.to_vec()),
                ],
            )
            .expect("alice.grantRole host-level");
        prop_assert!(r1.success,
            "Step 2: alice.grantRole(ROLE_X, bob) must succeed (alice has \
             admin role from ctor); exc={:?}. Failure means the admin \
             check rejected alice — either msg.sender isn't alice (Task \
             #176 sticky-caller regression) OR the ctor write to \
             _roles[DEFAULT_ADMIN_ROLE][alice] didn't land (nested mapping \
             write regression).",
            r1.exception.as_ref().map(|e| &e.message));

        // Step 3: hasRole(ROLE_X, bob) is now true.
        let r2 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "hasRole",
                &[
                    StackItem::byte_array(role_x.to_vec()),
                    StackItem::byte_array(bob.to_vec()),
                ],
            )
            .expect("hasRole(bob) post-grant host-level");
        prop_assert!(r2.success, "Step 3: hasRole post-grant exc={:?}",
            r2.exception.as_ref().map(|e| &e.message));
        let v2_truthy = !r2.return_data.is_empty()
            && r2.return_data.iter().any(|b| *b != 0);
        prop_assert!(v2_truthy,
            "Step 3: hasRole(ROLE_X, bob) after grantRole must be true; got \
             rd_hex={}. If false, the grantRole write landed on a different \
             slot than hasRole reads (nested-mapping key derivation \
             mismatch — `_roles[role][account]` write vs read uses \
             different hash inputs). Role byte=0x{:02x}.",
            hex::encode(&r2.return_data), role_byte);

        // Step 4: alice revokes.
        rt.override_caller_account(&alice_hex).expect("alice");
        let r3 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "revokeRole",
                &[
                    StackItem::byte_array(role_x.to_vec()),
                    StackItem::byte_array(bob.to_vec()),
                ],
            )
            .expect("alice.revokeRole host-level");
        prop_assert!(r3.success,
            "Step 4: alice.revokeRole(ROLE_X, bob) must succeed; exc={:?}",
            r3.exception.as_ref().map(|e| &e.message));

        // Step 5: hasRole is false again.
        let r4 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "hasRole",
                &[
                    StackItem::byte_array(role_x.to_vec()),
                    StackItem::byte_array(bob.to_vec()),
                ],
            )
            .expect("hasRole(bob) post-revoke host-level");
        prop_assert!(r4.success, "Step 5: hasRole post-revoke exc={:?}",
            r4.exception.as_ref().map(|e| &e.message));
        let v4_zero = r4.return_data.is_empty()
            || r4.return_data.iter().all(|b| *b == 0);
        prop_assert!(v4_zero,
            "CRITICAL Step 5: hasRole(ROLE_X, bob) after revokeRole must be \
             false; got rd_hex={}. If still truthy, revokeRole's `= false` \
             write either didn't land OR landed on a different slot than \
             the post-grant read targeted. Either way: revoke is broken, \
             every AccessControl revoke is a no-op.",
            hex::encode(&r4.return_data));
    }

    /// **e. `oz_erc20_permit_nonce_increments`**
    ///
    /// Simplified `ERC20Permit`-style nonce: `mapping(address => uint256)
    /// nonces`, with `permit(address user)` that increments the user's
    /// nonce by exactly 1. Each call to `permit(alice)` must increment
    /// `nonces(alice)` by 1 and leave `nonces(bob)` untouched.
    ///
    /// We drive a fuzz-chosen number of `permit(alice)` calls (1..=8) and
    /// confirm the readback matches the call count, with a single bob check
    /// at the end to confirm cross-account isolation.
    ///
    /// What a failure means:
    ///   - Final `nonces(alice)` != call_count: per-account counter
    ///     under/overflow, or the storage write was clobbered.
    ///   - Final `nonces(bob)` != 0: the mapping key derivation collapsed
    ///     distinct accounts into the same slot — every permit-style
    ///     signature replays across users (devastating).
    #[test]
    fn oz_erc20_permit_nonce_increments(call_count in 1u8..=8u8) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Permit {
    mapping(address => uint256) private _nonces;

    function nonces(address user) external view returns (uint256) {
        return _nonces[user];
    }

    function permit(address user) external {
        // Real ERC20Permit would also verify a signature, recover the
        // signer, etc. — but the nonce-counter semantic is independent of
        // the signature path and is what every off-chain permit signer
        // assumes. This test isolates that semantic.
        _nonces[user] = _nonces[user] + 1;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Permit compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "Permit produced no artifacts");
        let art = &arts[0];

        let alice: [u8; 20] = [0xAA; 20];
        let bob: [u8; 20] = [0xBB; 20];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Step 1: deploy + initial nonce(alice) read = 0.
        let r0 = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "nonces",
                &[StackItem::byte_array(alice.to_vec())],
                None,
            )
            .expect("Permit deploy + nonces(alice) host-level");
        prop_assert!(r0.success, "Step 1: deploy+nonces(alice) exc={:?}",
            r0.exception.as_ref().map(|e| &e.message));
        let v0 = decode_uint_le(&r0.return_data);
        prop_assert_eq!(v0.clone(), BigUint::from(0u64),
            "Step 1: nonces(alice) on fresh contract must be 0; got {} \
             (rd_hex={}). A non-zero default is a fresh-mapping read \
             regression.", v0, hex::encode(&r0.return_data));

        // Step 2: call permit(alice) `call_count` times.
        for i in 0..call_count {
            let r = rt
                .call_method(
                    &art.bytecode, &art.tokens, &art.manifest,
                    "permit",
                    &[StackItem::byte_array(alice.to_vec())],
                )
                .expect("permit(alice) host-level");
            prop_assert!(r.success,
                "Step 2.{}: permit(alice) call must succeed; exc={:?}",
                i, r.exception.as_ref().map(|e| &e.message));
        }

        // Step 3: nonces(alice) == call_count.
        let r1 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "nonces",
                &[StackItem::byte_array(alice.to_vec())],
            )
            .expect("nonces(alice) host-level");
        prop_assert!(r1.success, "Step 3: nonces(alice) exc={:?}",
            r1.exception.as_ref().map(|e| &e.message));
        let v1 = decode_uint_le(&r1.return_data);
        prop_assert_eq!(v1.clone(), BigUint::from(call_count as u64),
            "CRITICAL Step 3: nonces(alice) after {} permit() calls must \
             equal {}; got {} (rd_hex={}). If lower, increments were lost \
             across calls — storage_overlay is not committing the write \
             back to the persistent slot. If higher, a single permit \
             incremented more than once. If equal to 0 with call_count > 0, \
             the read+write key derivation diverges and writes are \
             scattered across distinct slots.",
            call_count, call_count, v1, hex::encode(&r1.return_data));

        // Step 4: nonces(bob) is still 0 — cross-account isolation.
        let r2 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "nonces",
                &[StackItem::byte_array(bob.to_vec())],
            )
            .expect("nonces(bob) host-level");
        prop_assert!(r2.success, "Step 4: nonces(bob) exc={:?}",
            r2.exception.as_ref().map(|e| &e.message));
        let v2 = decode_uint_le(&r2.return_data);
        prop_assert_eq!(v2.clone(), BigUint::from(0u64),
            "CRITICAL Step 4: nonces(bob) must be 0 (we never called \
             permit(bob)); got {} (rd_hex={}). A non-zero value means the \
             permit(alice) write LEAKED into bob's slot — every \
             ERC20Permit signature replays cross-account. The mapping key \
             derivation is collapsing distinct addresses into the same \
             slot.",
            v2, hex::encode(&r2.return_data));
    }
}
