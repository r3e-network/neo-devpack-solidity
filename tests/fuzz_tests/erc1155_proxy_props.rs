//! End-to-end proptests for two more highly-deployed real-world Solidity
//! patterns that the sibling `openzeppelin_patterns_props.rs` does not
//! cover:
//!
//!   * ERC-1155 (multi-token / nested-id mapping) — `mint`, `burn`,
//!     `balanceOf(owner, id)`, plus `balanceOfBatch(owners, ids)` returning a
//!     `uint256[]`. The batch-balance variant pins the dynamic-array return
//!     wire format (Task #121 / #137 — `offset || length || BE-32 elements`).
//!   * Proxy-style upgrade lifecycle — `delegatecall` is hard-rejected by
//!     this compiler (see CHANGELOG and the IR rejection path
//!     `src/ir/expressions/calls/...`), so the proxy semantic on Neo is
//!     `ContractManagement.update`. This test focuses on the EVM-ecosystem
//!     "proxy upgrade" intuition: the storage at `counter` flows through
//!     a NEF/manifest swap and is read by a v2-only function. It is a
//!     SLIGHTLY broader form of
//!     `contract_upgrade_props.rs::contract_upgrade_storage_persists_compatible_layout`
//!     — same compatible-layout principle, but driven through a
//!     `increment()`-style accumulator over multiple calls (matching how
//!     EVM proxy upgrades typically encounter live state) rather than a
//!     single `setCounter(v)`.
//!
//! What a failing assertion in any of these tests means:
//!
//!   - `erc1155_mint_burn_lifecycle`: if bob.id1 is not 70 after mint(100)
//!     + burn(30), either the nested-mapping write did not commit (Task
//!       #176 / nested-mapping-key derivation regression) or the read+write
//!       paths used different key derivations.
//!   - `erc1155_batch_balance_query`: if the batch return cannot be parsed
//!     into a length-prefixed `uint256[]`, the dynamic-array return wire
//!     format regressed (Task #121 / #137 — every contract that returns
//!     `T[]` is broken).
//!   - `proxy_storage_collision_safe`: if v2.getCounter() != 3 after the
//!     update, storage is being rebound to bytecode hash rather than to
//!     the contract hash — every `ContractManagement.update`-style proxy
//!     loses state (catastrophic).

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use num_bigint::BigUint;
use proptest::prelude::*;

/// Compile a single contract source and return the (bytecode, tokens,
/// manifest-bytes, manifest-Value) tuple needed to call methods and feed
/// the NEF + manifest to `NativeCalls.updateContract`. Mirrors the helper
/// in `contract_upgrade_props.rs`.
fn compile_one(
    source: &str,
    label: &str,
) -> (
    Vec<u8>,
    Vec<neo_devpack_solidity::neo::MethodToken>,
    Vec<u8>,
    serde_json::Value,
) {
    let arts = compile_contracts(source, false, 2)
        .unwrap_or_else(|e| panic!("erc1155_proxy {} compile: {:?}", label, e));
    assert!(
        !arts.is_empty(),
        "erc1155_proxy {} compile produced no artifacts",
        label
    );
    let art = arts[0].clone();
    let manifest_bytes = serde_json::to_vec(&art.manifest)
        .unwrap_or_else(|e| panic!("erc1155_proxy {} manifest serialize: {:?}", label, e));
    (art.bytecode, art.tokens, manifest_bytes, art.manifest)
}

proptest! {
    // 4 cases — each test compiles + deploys + drives a multi-step contract
    // sequence (1-3s per case). The semantic property is invariant in the
    // fuzz axis; we shrink only across the fuzz seed / scalar parameter.
    #![proptest_config(ProptestConfig::with_cases(4))]

    /// **a. `erc1155_mint_burn_lifecycle`**
    ///
    /// Self-contained ERC-1155 mirror: `mapping(uint256 => mapping(address
    /// => uint256)) balances`, with `mint(to, id, amount)`,
    /// `burn(from, id, amount)`, and `balanceOf(owner, id)`. Safe-transfer
    /// hooks are intentionally elided — those require an EIP-1155
    /// acceptance-check on the receiver, which is too involved for a
    /// self-contained property test (and orthogonal to the
    /// nested-mapping storage semantic this test pins).
    ///
    /// Sequence:
    ///   1. Deploy as alice (owner-by-deploy).
    ///   2. mint(bob, 1, 100); mint(bob, 2, 50); mint(carol, 1, 20).
    ///   3. burn(bob, 1, 30).
    ///   4. Assert: balanceOf(bob, 1) == 70, balanceOf(bob, 2) == 50,
    ///      balanceOf(carol, 1) == 20, balanceOf(carol, 2) == 0,
    ///      balanceOf(bob, 99) == 0 (untouched id).
    ///
    /// What a failure means:
    ///   - balanceOf(bob, 1) != 70: either mint's `+= amount` did not
    ///     accumulate (write clobbered read, or the storage overlay did
    ///     not flush) OR burn's `-= amount` underflowed past the live
    ///     balance OR the (id, owner) pair hashed to a different slot on
    ///     read vs write (nested-mapping derivation mismatch).
    ///   - balanceOf(carol, 1) != 20: cross-account isolation broken —
    ///     the `(id=1, bob)` write leaked into `(id=1, carol)`.
    ///   - balanceOf(bob, 99) != 0: the (id=1, bob) write or the (id=2,
    ///     bob) write leaked into a never-touched id slot — id-key
    ///     collision in the outer mapping.
    #[test]
    fn erc1155_mint_burn_lifecycle(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract ERC1155Lite {
    // Outer key = token id; inner key = owner. Order matters for slot
    // derivation: a swap (`balances[owner][id]`) would change every slot
    // and break interop with any inlined OZ ERC-1155.
    mapping(uint256 => mapping(address => uint256)) private _balances;

    address private _owner;
    constructor() {
        _owner = msg.sender;
    }
    modifier onlyOwner() {
        require(msg.sender == _owner, "ERC1155Lite: not owner");
        _;
    }

    function balanceOf(address owner, uint256 id) external view returns (uint256) {
        return _balances[id][owner];
    }
    function mint(address to, uint256 id, uint256 amount) external onlyOwner {
        _balances[id][to] = _balances[id][to] + amount;
    }
    function burn(address from, uint256 id, uint256 amount) external onlyOwner {
        uint256 cur = _balances[id][from];
        require(cur >= amount, "ERC1155Lite: burn exceeds balance");
        _balances[id][from] = cur - amount;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("ERC1155Lite compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "ERC1155Lite produced no artifacts");
        let art = &arts[0];

        let alice: [u8; 20] = [0xAA; 20];
        let bob: [u8; 20] = [0xBB; 20];
        let carol: [u8; 20] = [0xCC; 20];
        let alice_hex = format!("0x{}", hex::encode(alice));

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Step 1: deploy as alice. balanceOf(bob, 1) on a fresh contract
        // must be 0 (nested-mapping default-zero read).
        rt.override_caller_account(&alice_hex)
            .expect("alice override (deployer)");
        let r0 = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "balanceOf",
                &[
                    StackItem::byte_array(bob.to_vec()),
                    StackItem::UnsignedInteger(1),
                ],
                None,
            )
            .expect("ERC1155Lite deploy + balanceOf(bob,1) host-level");
        prop_assert!(r0.success,
            "Step 1 (deploy + balanceOf): exc={:?}",
            r0.exception.as_ref().map(|e| &e.message));
        let v0 = decode_uint_le(&r0.return_data);
        prop_assert_eq!(v0.clone(), BigUint::from(0u64),
            "Step 1: balanceOf(bob, 1) on a fresh contract must be 0; got \
             {} (rd_hex={}). A non-zero default is a fresh nested-mapping \
             read regression.",
            v0, hex::encode(&r0.return_data));

        // Step 2a: alice mints (bob, id=1, amount=100).
        rt.override_caller_account(&alice_hex).expect("alice");
        let r1 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "mint",
                &[
                    StackItem::byte_array(bob.to_vec()),
                    StackItem::UnsignedInteger(1),
                    StackItem::UnsignedInteger(100),
                ],
            )
            .expect("mint(bob,1,100) host-level");
        prop_assert!(r1.success,
            "Step 2a: alice.mint(bob, 1, 100) must succeed; exc={:?}. \
             If the modifier rejects alice, sticky-caller (Task #176) \
             regressed; if the nested-mapping write itself faults, the \
             outer key (id=1) is being interpreted as an unsupported type.",
            r1.exception.as_ref().map(|e| &e.message));

        // Step 2b: alice mints (bob, id=2, amount=50).
        rt.override_caller_account(&alice_hex).expect("alice");
        let r2 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "mint",
                &[
                    StackItem::byte_array(bob.to_vec()),
                    StackItem::UnsignedInteger(2),
                    StackItem::UnsignedInteger(50),
                ],
            )
            .expect("mint(bob,2,50) host-level");
        prop_assert!(r2.success,
            "Step 2b: alice.mint(bob, 2, 50) must succeed; exc={:?}.",
            r2.exception.as_ref().map(|e| &e.message));

        // Step 2c: alice mints (carol, id=1, amount=20).
        rt.override_caller_account(&alice_hex).expect("alice");
        let r3 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "mint",
                &[
                    StackItem::byte_array(carol.to_vec()),
                    StackItem::UnsignedInteger(1),
                    StackItem::UnsignedInteger(20),
                ],
            )
            .expect("mint(carol,1,20) host-level");
        prop_assert!(r3.success,
            "Step 2c: alice.mint(carol, 1, 20) must succeed; exc={:?}.",
            r3.exception.as_ref().map(|e| &e.message));

        // Step 3: alice burns (bob, id=1, amount=30).
        rt.override_caller_account(&alice_hex).expect("alice");
        let r4 = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "burn",
                &[
                    StackItem::byte_array(bob.to_vec()),
                    StackItem::UnsignedInteger(1),
                    StackItem::UnsignedInteger(30),
                ],
            )
            .expect("burn(bob,1,30) host-level");
        prop_assert!(r4.success,
            "Step 3: alice.burn(bob, 1, 30) must succeed; exc={:?}. \
             If exc cites 'burn exceeds balance', the read of \
             _balances[1][bob] in burn() saw a value < 30 — i.e. the \
             mint(bob,1,100) write from step 2a did NOT persist into the \
             slot that burn reads. That's a nested-mapping read/write \
             slot-derivation mismatch and is the SAME bug-class as the \
             AccessControl regression (sibling file).",
            r4.exception.as_ref().map(|e| &e.message));

        // Step 4: assert all final balances.
        let cases: &[(&str, [u8; 20], u64, u64)] = &[
            ("bob, id=1 (mint 100 - burn 30)", bob, 1, 70),
            ("bob, id=2 (mint 50, untouched)", bob, 2, 50),
            ("carol, id=1 (mint 20)", carol, 1, 20),
            ("carol, id=2 (never touched)", carol, 2, 0),
            ("bob, id=99 (never touched)", bob, 99, 0),
        ];
        for (label, who, id, expected) in cases {
            let r = rt
                .call_method(
                    &art.bytecode, &art.tokens, &art.manifest,
                    "balanceOf",
                    &[
                        StackItem::byte_array(who.to_vec()),
                        StackItem::UnsignedInteger(*id),
                    ],
                )
                .expect("balanceOf host-level");
            prop_assert!(r.success,
                "Step 4 [{}]: balanceOf must succeed; exc={:?}",
                label, r.exception.as_ref().map(|e| &e.message));
            let got = decode_uint_le(&r.return_data);
            prop_assert_eq!(got.clone(), BigUint::from(*expected),
                "CRITICAL Step 4 [{}]: expected {}, got {} (rd_hex={}). \
                 If non-zero where 0 was expected, a write LEAKED into a \
                 never-touched slot — id/owner key derivation is \
                 collapsing distinct keys into the same slot. If short of \
                 expected (e.g. 0 where 70 was expected), the mint or burn \
                 write did not persist OR landed at a different slot than \
                 the read targets.",
                label, expected, got, hex::encode(&r.return_data));
        }
    }

    /// **b. `erc1155_batch_balance_query`**
    ///
    /// Same `_balances` storage as variant a, with an added
    /// `balanceOfBatch(address[] owners, uint256[] ids)` returning
    /// `uint256[]`. The batch read is the canonical EIP-1155 multi-balance
    /// query and exercises the `T[]` return wire format end-to-end:
    /// the runtime must build a NeoVM Array of values and the codegen
    /// must marshal it back across the external call boundary as the
    /// EVM-canonical `offset(32) || length(32) || BE-32 elements`
    /// shape (Task #121 / #137).
    ///
    /// The test drives the call from the Rust side rather than from a
    /// same-contract internal helper, because Solidity-side
    /// `uint256[] memory got = this_or_another_function(...)` would
    /// require the compiler to round-trip the encoded `T[]` payload
    /// through an in-contract decode — a separate concern from what
    /// real ERC-1155 callers exercise (an off-chain client decoding the
    /// response).
    ///
    /// Sequence:
    ///   1. Deploy the contract.
    ///   2. mint(a, 1, 11), mint(a, 2, 22), mint(b, 1, 33), mint(b, 2, 44),
    ///      mint(c, 7, 77).
    ///   3. Sanity: balanceOf(a, 1) == 11 etc. (single-read agreement).
    ///   4. Call balanceOfBatch with [(a,1), (a,2), (b,1), (b,2), (c,7)].
    ///   5. Parse the EVM-canonical `offset || length || BE-32 elements`
    ///      response and assert each element matches.
    ///
    /// What a failure means:
    ///   - balanceOfBatch reverts: the `uint256[]` return wire format
    ///     itself faulted at marshal time — every contract returning
    ///     `T[]` is broken (Task #121/#137 regression).
    ///   - Length-32-LE-decode != 5: the runtime built the array but the
    ///     length header is corrupted (likely a width/endianness mismatch
    ///     between the codegen and the wire format).
    ///   - An element != its expected value: the runtime wrote elements
    ///     in the wrong order, packed them at the wrong stride, or read
    ///     the storage slot via a key derivation that diverges from the
    ///     single-read path.
    #[test]
    fn erc1155_batch_balance_query(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract ERC1155Batch {
    mapping(uint256 => mapping(address => uint256)) private _balances;

    function mint(address to, uint256 id, uint256 amount) external {
        _balances[id][to] = _balances[id][to] + amount;
    }

    function balanceOf(address owner, uint256 id) external view returns (uint256) {
        return _balances[id][owner];
    }

    function balanceOfBatch(address[] memory owners, uint256[] memory ids)
        external
        view
        returns (uint256[] memory)
    {
        require(owners.length == ids.length, "ERC1155Batch: length mismatch");
        uint256[] memory out = new uint256[](owners.length);
        for (uint256 i = 0; i < owners.length; i++) {
            out[i] = _balances[ids[i]][owners[i]];
        }
        return out;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("ERC1155Batch compile: {:?}. If \
                this fires on `balanceOfBatch(address[],uint256[]) returns \
                (uint256[])`, the dynamic-array parameter+return type pair \
                regressed in the front-end.", e));
        prop_assert!(!arts.is_empty(), "ERC1155Batch produced no artifacts");
        let art = &arts[0];

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Three uniform-byte addresses — same convention as variant a
        // and the OZ-pattern siblings.
        let a: [u8; 20] = [0xAA; 20];
        let b: [u8; 20] = [0xBB; 20];
        let c: [u8; 20] = [0xCC; 20];

        // ---- Step 1+2: deploy + mint a known fixture. ----
        // (id=1, a)=11, (id=2, a)=22, (id=1, b)=33, (id=2, b)=44, (id=7, c)=77.
        let mints: &[([u8; 20], u64, u64)] = &[
            (a, 1, 11),
            (a, 2, 22),
            (b, 1, 33),
            (b, 2, 44),
            (c, 7, 77),
        ];
        // Use the first mint to drive the deploy so we don't need a
        // distinct constructor + bare-mint sequencing dance.
        let (m0_addr, m0_id, m0_amt) = mints[0];
        let r_deploy = rt
            .call_method_with_deploy_args(
                &art.bytecode, &art.tokens, &art.manifest,
                "mint",
                &[
                    StackItem::byte_array(m0_addr.to_vec()),
                    StackItem::UnsignedInteger(m0_id),
                    StackItem::UnsignedInteger(m0_amt),
                ],
                None,
            )
            .expect("ERC1155Batch deploy + first mint host-level");
        prop_assert!(r_deploy.success,
            "Step 1+2.0: deploy + first mint must succeed; exc={:?}.",
            r_deploy.exception.as_ref().map(|e| &e.message));
        for (i, (addr, id, amt)) in mints.iter().enumerate().skip(1) {
            let r = rt
                .call_method(
                    &art.bytecode, &art.tokens, &art.manifest,
                    "mint",
                    &[
                        StackItem::byte_array(addr.to_vec()),
                        StackItem::UnsignedInteger(*id),
                        StackItem::UnsignedInteger(*amt),
                    ],
                )
                .expect("mint host-level");
            prop_assert!(r.success,
                "Step 2.{}: mint({:?}, {}, {}) must succeed; exc={:?}.",
                i, addr, id, amt,
                r.exception.as_ref().map(|e| &e.message));
        }

        // ---- Step 3: sanity-check single-read balanceOf. ----
        for (addr, id, expected) in mints {
            let r = rt
                .call_method(
                    &art.bytecode, &art.tokens, &art.manifest,
                    "balanceOf",
                    &[
                        StackItem::byte_array(addr.to_vec()),
                        StackItem::UnsignedInteger(*id),
                    ],
                )
                .expect("balanceOf host-level");
            prop_assert!(r.success,
                "Step 3 sanity: balanceOf must succeed; exc={:?}",
                r.exception.as_ref().map(|e| &e.message));
            let got = decode_uint_le(&r.return_data);
            prop_assert_eq!(got.clone(), BigUint::from(*expected),
                "Step 3 sanity: balanceOf({:?}, {}) must equal {} \
                 (matching the per-call mint we just performed); got {} \
                 (rd_hex={}). If this fails, the bug is in the \
                 single-read storage path and variant a's lifecycle test \
                 should have caught it — investigate that first.",
                addr, id, expected, got, hex::encode(&r.return_data));
        }

        // ---- Step 4: call balanceOfBatch with all 5 (owner, id) pairs. ----
        let owners_arg = StackItem::array(
            mints.iter().map(|(o, _, _)| StackItem::byte_array(o.to_vec())).collect()
        );
        let ids_arg = StackItem::array(
            mints.iter().map(|(_, id, _)| StackItem::UnsignedInteger(*id)).collect()
        );
        let r_batch = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "balanceOfBatch",
                &[owners_arg, ids_arg],
            )
            .expect("balanceOfBatch host-level");
        prop_assert!(r_batch.success,
            "Step 4: balanceOfBatch must succeed; exc={:?}. If exc cites \
             abi.encode / dynamic-array marshal, the `uint256[]` return \
             wire format faulted (Task #121/#137 regression). If exc \
             cites a length mismatch, the array-parameter length read \
             diverged inside the function.",
            r_batch.exception.as_ref().map(|e| &e.message));

        // ---- Step 5: parse the EVM-canonical `T[]` shape:
        //
        //   bytes [0..32]:   offset (== 0x20 = 32; little-endian on the
        //                    NeoVM ByteString in Task #121/#137 still
        //                    produces a 32-byte BE-padded value; we
        //                    accept either by tolerating leading-zero
        //                    runs and matching the trailing significant
        //                    byte == 0x20).
        //   bytes [32..64]:  length (BE-32 uint256).
        //   bytes [64..]:    `length` × 32 bytes per element, BE.
        //
        // Some envelopes elide the leading offset (the "flat tail" form
        // — `length || elements...` directly). We probe for both: if
        // bytes[0..32] decodes to 32 (== sizeof header), treat the
        // tail as starting at 32; otherwise assume bytes[0..32] is
        // already the length.
        let rd = &r_batch.return_data;
        prop_assert!(rd.len() >= 32 + 5 * 32,
            "Step 5: balanceOfBatch return must be at least 32 (length) \
             + 5*32 (elements) = 192 bytes; got {} bytes (rd_hex={}). \
             If shorter, the dynamic-array marshal truncated the \
             payload — Task #121/#137 regression candidate.",
            rd.len(), hex::encode(rd));

        // BE-32 decoder: read 32 bytes and interpret as a big-endian
        // uint256. Bound to u64 here because all our values fit.
        let read_be32 = |slice: &[u8]| -> BigUint {
            BigUint::from_bytes_be(slice)
        };

        let head = read_be32(&rd[0..32]);
        let (len_off, body_off) = if head == BigUint::from(32u64) {
            // EVM-canonical envelope: head is offset, length follows.
            (32usize, 64usize)
        } else {
            // Flat-tail envelope: head is the length itself.
            (0usize, 32usize)
        };

        let len_val = read_be32(&rd[len_off..len_off + 32]);
        prop_assert_eq!(len_val.clone(), BigUint::from(5u64),
            "Step 5: balanceOfBatch return length header must be 5; got \
             {} (rd_hex={}, head_offset={}, body_offset={}). If 0, the \
             length wasn't written. If a multiple of 32 (32, 64, 96, \
             128, 160, 224, ...), the header was double-counted as bytes \
             instead of element count — that is the exact bug class \
             this test pins.",
            len_val, hex::encode(rd), len_off, body_off);

        // Element-by-element comparison. EVM uint256 elements are
        // 32-byte big-endian; our values fit in u64 so the high 24
        // bytes of each element should be zero.
        for (i, (_, _, expected)) in mints.iter().enumerate() {
            let elem_start = body_off + i * 32;
            let elem_end = elem_start + 32;
            prop_assert!(elem_end <= rd.len(),
                "Step 5: element {} ends at byte {} but return is {} \
                 bytes; payload truncated.", i, elem_end, rd.len());
            let elem_be = read_be32(&rd[elem_start..elem_end]);
            prop_assert_eq!(elem_be.clone(), BigUint::from(*expected),
                "Step 5: element {} must equal {}; got {} \
                 (be_hex={}). If wrong but the length is correct, the \
                 inner balanceOfBatch loop wrote elements in the wrong \
                 order OR read a different storage slot than the \
                 single-read balanceOf. Index {} corresponds to \
                 (owner={:?}, id={}).",
                i, expected, elem_be,
                hex::encode(&rd[elem_start..elem_end]),
                i, mints[i].0, mints[i].1);
        }
    }

    /// **c. `proxy_storage_collision_safe`**
    ///
    /// EVM-ecosystem proxy upgrade pattern, translated to Neo's native
    /// upgrade primitive. `delegatecall` is hard-rejected by this
    /// compiler (per the IR rejection path), so the equivalent path on
    /// Neo is `ContractManagement.update`: the contract hash + storage
    /// persist, only the NEF + manifest swap. This test focuses on the
    /// "live state flowing through an upgrade" intuition: drive several
    /// `increment()` calls on v1, swap to v2 (which retains `counter`
    /// at the same SHA-256(name)-derived slot but ALSO adds
    /// `getDoubled()`), and confirm:
    ///
    ///   - v2.counter() returns the v1-accumulated value (3).
    ///   - v2.getDoubled() returns 6.
    ///
    /// This is a slightly broader form of
    /// `contract_upgrade_props.rs::contract_upgrade_storage_persists_compatible_layout`
    /// — same compatible-layout principle, but driven through a
    /// multi-call accumulator (the way EVM proxy upgrades actually
    /// encounter live state in production) rather than a single
    /// `setCounter(v)`.
    ///
    /// What a failure means:
    ///   - v1.counter() != 3 after 3 increment() calls: the live
    ///     `counter += 1` did not persist across calls — separate from
    ///     the upgrade question; would surface as a storage-flush bug.
    ///   - v2.counter() != 3 after upgrade: storage rebound to bytecode
    ///     hash on update — every proxy-style upgrade loses state.
    ///   - v2.getDoubled() != 6: v2 read the v1 counter slot
    ///     incorrectly (slot drifted across upgrade despite identical
    ///     field name).
    #[test]
    fn proxy_storage_collision_safe(_seed in any::<u8>()) {
        // v1: counter + increment + upgrade hook.
        let v1_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Impl {
    uint256 public counter;
    function increment() external { counter = counter + 1; }
    function upgrade(bytes calldata nef, bytes calldata manifest) external {
        NativeCalls.updateContract(nef, manifest);
    }
}"#;

        // v2: same `counter` slot key (SHA-256("counter")) — adds
        // getDoubled() but keeps the storage layout untouched.
        let v2_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Impl {
    uint256 public counter;
    function increment() external { counter = counter + 1; }
    function getDoubled() external view returns (uint256) { return counter * 2; }
    function upgrade(bytes calldata nef, bytes calldata manifest) external {
        NativeCalls.updateContract(nef, manifest);
    }
}"#;

        let (v1_bc, v1_tk, _v1_mb, v1_manifest) = compile_one(v1_src, "proxy-v1");
        let (v2_bc, v2_tk, v2_mb, v2_manifest) = compile_one(v2_src, "proxy-v2");

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Step 1: drive 3 increment() calls on v1.
        for i in 0..3u32 {
            let r = rt.call_method(
                &v1_bc, &v1_tk, &v1_manifest, "increment", &[],
            ).expect("v1 increment host-level");
            prop_assert!(r.success,
                "v1 increment() call #{} must succeed; exc={:?}. If exc \
                 cites a panic, `counter + 1` overflowed (impossible at \
                 small N) OR the read inside `counter = counter + 1` \
                 diverged from the write — separate storage-overlay bug.",
                i, r.exception.as_ref().map(|e| &e.message));
        }

        // Step 2: confirm v1 sees counter == 3 BEFORE the upgrade. If
        // this fails, the upgrade-step assertion below would be ambiguous
        // — we want the pre-upgrade state pinned to a known value first.
        let r_pre = rt.call_method(&v1_bc, &v1_tk, &v1_manifest, "counter", &[])
            .expect("v1 counter() pre-upgrade host-level");
        prop_assert!(r_pre.success,
            "v1.counter() pre-upgrade must succeed; exc={:?}.",
            r_pre.exception.as_ref().map(|e| &e.message));
        let pre_v = decode_uint_le(&r_pre.return_data);
        prop_assert_eq!(pre_v.clone(), BigUint::from(3u64),
            "v1.counter() pre-upgrade must equal 3 (we called \
             increment() 3 times); got {} (rd_hex={}). If 0, NONE of the \
             increments persisted (storage-overlay flush bug). If 1, only \
             the LAST write persisted (read-modify-write didn't see the \
             prior write — overlay/commit ordering bug). If 2, the FIRST \
             write was lost (overlay clobbered an in-progress write).",
            pre_v, hex::encode(&r_pre.return_data));

        // Step 3: upgrade via NativeCalls.updateContract(v2_nef, v2_mb).
        let r_up = rt.call_method(
            &v1_bc, &v1_tk, &v1_manifest, "upgrade",
            &[
                StackItem::byte_array(v2_bc.clone()),
                StackItem::byte_array(v2_mb.clone()),
            ],
        ).expect("v1.upgrade(v2 nef, v2 manifest) host-level");
        prop_assert!(r_up.success,
            "v1.upgrade(v2_nef, v2_manifest) must succeed; exc={:?}. If \
             this fires, ContractManagement.update dispatch regressed \
             (file under contract_management.rs::\"update\").",
            r_up.exception.as_ref().map(|e| &e.message));

        // Step 4: switch to v2 bytecode/manifest. counter() must still
        // return 3 — storage MUST persist across the NEF/manifest swap.
        let r_post_c = rt.call_method(&v2_bc, &v2_tk, &v2_manifest, "counter", &[])
            .expect("v2 counter() post-upgrade host-level");
        prop_assert!(r_post_c.success,
            "v2.counter() post-upgrade must succeed; exc={:?}. If this \
             fires citing 'manifest.abi.methods has no entry', the \
             post-upgrade manifest swap didn't take effect.",
            r_post_c.exception.as_ref().map(|e| &e.message));
        let post_c = decode_uint_le(&r_post_c.return_data);
        prop_assert_eq!(post_c.clone(), BigUint::from(3u64),
            "CRITICAL: v2.counter() post-upgrade MUST equal 3 (the \
             v1-accumulated value); got {} (rd_hex={}). If 0, the \
             counter slot was wiped on upgrade — runtime BUG: storage is \
             being rebound to bytecode hash rather than to contract \
             hash, every proxy-style upgrade on Neo loses state. This is \
             the same failure mode the sibling \
             contract_upgrade_storage_persists_compatible_layout test \
             also guards against — but with a DIFFERENT live-state \
             generation pattern (multi-call accumulator vs single \
             setCounter), so a bug that surfaces only one of the two \
             would indicate an interaction between the storage-overlay \
             flush and the update handler.",
            post_c, hex::encode(&r_post_c.return_data));

        // Step 5: v2-only function reads the persisted counter.
        let r_doubled = rt.call_method(&v2_bc, &v2_tk, &v2_manifest, "getDoubled", &[])
            .expect("v2 getDoubled() host-level");
        prop_assert!(r_doubled.success,
            "v2.getDoubled() (v2-only fn) must succeed post-upgrade; \
             exc={:?}. If exc cites 'no entry', the manifest swap \
             didn't land — file under ContractManagement.update manifest \
             rebinding.",
            r_doubled.exception.as_ref().map(|e| &e.message));
        let got_doubled = decode_uint_le(&r_doubled.return_data);
        prop_assert_eq!(got_doubled.clone(), BigUint::from(6u64),
            "v2.getDoubled() must return 2 * counter = 2 * 3 = 6; got {} \
             (rd_hex={}). If 0, the counter slot is unreadable from v2 \
             (slot key drifted across upgrade despite identical field \
             name — SHA-256(\"counter\") should be stable across a \
             rename-free upgrade). If a different non-zero value, the \
             multiplication path regressed (separate from the upgrade \
             question).",
            got_doubled, hex::encode(&r_doubled.return_data));
    }
}
