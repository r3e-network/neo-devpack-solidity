use super::*;

pub(crate) fn resolve_builtin_call(expr: &Expression) -> Option<BuiltinCall> {
    if let Expression::MemberAccess(_, inner, member) = expr {
        if let Expression::Variable(base) = inner.as_ref() {
            let member_name = member.name.as_str();
            match base.name.as_str() {
                "Runtime" => return resolve_runtime_member(member_name),
                "abi" => return resolve_abi_member(member_name),
                "Storage" => return resolve_storage_member(member_name),
                "Syscalls" => return resolve_syscalls_member(member_name),
                "NativeCalls" => return resolve_native_calls_member(member_name),
                "Neo" => return resolve_neo_member(member_name),
                // Task — `StdLib.<method>(...)` and `CryptoLib.<method>(...)`
                // in Solidity source must lower to a `BuiltinCall::NativeCall`
                // to the matching Neo N3 native contract. Before this was
                // wired, the compiler fell through to the generic function-
                // call handler, which evaluated and dropped the args then
                // pushed PUSH0 as the result. Fuzz regression: the
                // Solidity→bytecode path for `StdLib.itoa(12345, 10)` and
                // `CryptoLib.sha256(bytes)` returned zeros at runtime even
                // though the underlying native implementations are correct
                // (see baseline_tests.rs::callt_stdlib_itoa_roundtrip_via_token).
                "StdLib" => return resolve_stdlib_member(member_name),
                "CryptoLib" => return resolve_cryptolib_member(member_name),
                _ => {}
            }
        }
    }

    if let Expression::Variable(identifier) = expr {
        if identifier.name == "ecrecover" {
            return Some(BuiltinCall::Ecrecover);
        }
        if identifier.name == "keccak256" {
            return Some(BuiltinCall::Keccak256);
        }
        // Solidity exposes `sha256(bytes)` and `ripemd160(bytes)` as
        // global built-in hashers (EVM precompiles 0x02 and 0x03). Route
        // them to the CryptoLib native contract's `sha256`/`ripemd160`
        // methods. Before this wiring the compiler fell through to the
        // generic function-call handler, which evaluated and dropped the
        // argument and pushed 0 — so `sha256(b"abc")` returned 8 zero
        // bytes at runtime instead of the canonical digest. `keccak256`
        // already has a dedicated `BuiltinCall::Keccak256` variant
        // because its EVM semantics differ from Neo's default; `sha256`
        // and `ripemd160` have a direct 1:1 native mapping.
        if identifier.name == "sha256" {
            return Some(BuiltinCall::NativeCall {
                contract: NativeContract::CryptoLib,
                method: "sha256".to_string(),
            });
        }
        if identifier.name == "ripemd160" {
            return Some(BuiltinCall::NativeCall {
                contract: NativeContract::CryptoLib,
                method: "ripemd160".to_string(),
            });
        }
        if identifier.name == "type" {
            return Some(BuiltinCall::TypeOf);
        }
    }

    None
}

/// The builtin-library base names the compiler lowers as intrinsics
/// (their devpack Solidity bodies are never compiled).
pub(crate) const BUILTIN_LIBRARY_BASES: &[&str] = &[
    "Runtime",
    "abi",
    "Storage",
    "Syscalls",
    "NativeCalls",
    "Neo",
];

/// Introspection over the complete builtin intrinsic surface:
/// `(base, supported members)` for every builtin library base.
///
/// INVARIANT (pinned by `tests/gap_hasrole_tests.rs`): every member listed
/// here MUST actually lower — either via the `resolve_*_member` table in this
/// file / `syscalls.rs` / `native_calls.rs`, or via a bespoke handler in
/// `src/ir/expressions/calls/builtins/member_*.rs`. A whitelisted member
/// without a lowering is an uncallable intrinsic: the diagnostic for
/// `Base.member(...)` would then claim the member is supported while every
/// call fails (this happened with `Syscalls.hasRole`, which resolved to
/// `None` while being advertised). Add new members here only together with
/// their lowering.
pub fn builtin_intrinsic_surface() -> Vec<(&'static str, &'static [&'static str])> {
    BUILTIN_LIBRARY_BASES
        .iter()
        .map(|base| {
            (
                *base,
                builtin_library_supported_members(base)
                    // Invariant: `base` iterates the hardcoded
                    // BUILTIN_LIBRARY_BASES table, every entry of which has a
                    // matching arm, so this cannot be None. Safe under
                    // `panic = "abort"`.
                    .expect("every builtin library base has a member whitelist"),
            )
        })
        .collect()
}

pub(crate) fn builtin_library_supported_members(base: &str) -> Option<&'static [&'static str]> {
    match base {
        "Runtime" => Some(&[
            "notify",
            "notifyIndexed",
            "checkWitness",
            "requireWitness",
            "checkAnyWitness",
            "checkAllWitnesses",
            "checkMultiSigWitness",
            "gasLeft",
            "burnGas",
            "log",
            "getTime",
            "getTrigger",
            "getInvocationCounter",
            "getCurrentSigners",
            "getCallFlags",
            "getScriptContainer",
            "loadScript",
            "initializeServices",
            "getNetwork",
            "getPlatform",
            "getAddressVersion",
            "getRandom",
            "getExecutingScriptHash",
            "getCallingScriptHash",
            "getEntryScriptHash",
        ]),
        "abi" => Some(&[
            "encode",
            "encodePacked",
            "encodeCall",
            "encodeWithSignature",
            "encodeWithSelector",
            "decode",
        ]),
        // Only members with faithful Neo N3 lowerings are listed here. The
        // former `*Local` family lowered to fictional `System.Storage.Local.*`
        // syscalls that do not exist in Neo N3's interop table (instant FAULT
        // on real nodes), and convenience helpers such as `batchPut`,
        // `batchGet`, `batchDelete`, `count`, `findValues`, `findKeys`,
        // `clearPrefix`, `exists`, `isValidKey`, and `getUsage` silently
        // miscompiled to single raw syscalls with the wrong arity/semantics.
        // All of them now fail compilation with a loud diagnostic instead.
        "Storage" => Some(&[
            "find",
            "put",
            "get",
            "remove",
            "asReadOnly",
            "initializeContext",
            "getContext",
            "getReadOnlyContext",
            "putContractMetadata",
        ]),
        "Syscalls" => Some(&[
            "contractCall",
            "contractCallWithFlags",
            "getCallFlags",
            "contractCreate",
            "contractUpdate",
            "contractDestroy",
            "createStandardAccount",
            "createMultisigAccount",
            "notify",
            "getCurrentIndex",
            "getCurrentHash",
            "getBlock",
            "getTransaction",
            "getTransactionHeight",
            "getTransactionFromBlock",
            "getTransactionSigners",
            "getTransactionVMState",
            "getExecutingScriptHash",
            "getCallingScriptHash",
            "getEntryScriptHash",
            "getScriptContainer",
            "loadScript",
            "getStorageContext",
            "getReadOnlyStorageContext",
            "storageAsReadOnly",
            "storageGet",
            "storagePut",
            "storageDelete",
            "storageFind",
            "checkWitness",
            "getTime",
            "gasLeft",
            "getPlatform",
            "getTrigger",
            "getNotifications",
            "log",
            "getCurrentSigners",
            "checkSig",
            "checkMultisig",
            "sha256",
            "ripemd160",
            "verifyWithECDsa",
            "murmur32",
            "keccak256",
            "recoverSecp256K1",
            "verifyWithEd25519",
            "bls12381Serialize",
            "bls12381Deserialize",
            "bls12381Equal",
            "bls12381Add",
            "bls12381Mul",
            "bls12381Pairing",
            "bls12381G1Add",
            "bls12381G1Mul",
            "bls12381G2Add",
            "bls12381G2Mul",
            "bls12381G1Neg",
            "bls12381G2Neg",
            // Alias for `keccak256` — the devpack's `Syscalls.sol` exposes
            // it as `neoKeccak256` so the name doesn't shadow Solidity's
            // bare `keccak256` intrinsic. Unwired here before the audit.
            "neoKeccak256",
            "serialize",
            "deserialize",
            "itoa",
            "atoi",
            "jsonSerialize",
            "jsonDeserialize",
            "base64Encode",
            "base64Decode",
            "base64UrlEncode",
            "base64UrlDecode",
            "base58Encode",
            "base58Decode",
            "base58CheckEncode",
            "base58CheckDecode",
            // StdLib.hexEncode / hexDecode — real StdLib methods (see
            // StdLib.cs `HexEncode` / `HexDecode`), previously believed
            // absent and left unmapped.
            "hexEncode",
            "hexDecode",
            "memoryCompare",
            "memorySearch",
            "stringSplit",
            "strLen",
            "iteratorNext",
            "iteratorValue",
            "getCurrentRandom",
            "getNetwork",
            "getAddressVersion",
            "burnGas",
            "getInvocationCounter",
            "getFeePerByte",
            "getExecFeeFactor",
            "getExecPicoFeeFactor",
            "getStoragePrice",
            "getMillisecondsPerBlock",
            "getMaxValidUntilBlockIncrement",
            "getMaxTraceableBlocks",
            "getAttributeFee",
            "isBlocked",
            "oracleRequest",
            "getOraclePrice",
            "getDesignatedByRole",
            "scriptHashToAddress",
            "addressToScriptHash",
            "isValidAddress",
            "getContractScript",
            "contractExists",
        ]),
        "NativeCalls" => Some(&[
            "neoTotalSupply",
            "neoBalanceOf",
            "neoTransfer",
            "neoDecimals",
            "neoSymbol",
            "vote",
            "getCandidates",
            "registerCandidate",
            "unregisterCandidate",
            "getGasPerBlock",
            "getRegisterPrice",
            "setRegisterPrice",
            "setGasPerBlock",
            "getAccountState",
            "unclaimedGas",
            "getCandidateVote",
            "getCommittee",
            "getCommitteeAddress",
            "isCommittee",
            "getNextBlockValidators",
            "getAllCandidates",
            "isValidator",
            "gasTotalSupply",
            "gasBalanceOf",
            "gasTransfer",
            "gasDecimals",
            "gasSymbol",
            "deployContract",
            "updateContract",
            "destroyContract",
            "getContract",
            "listContracts",
            "hasMethod",
            "getMinimumDeploymentFee",
            "setMinimumDeploymentFee",
            "getContractById",
            "isContract",
            "getFeePerByte",
            "setFeePerByte",
            "getExecFeeFactor",
            "getExecPicoFeeFactor",
            "setExecFeeFactor",
            "getStoragePrice",
            "getMillisecondsPerBlock",
            "setMillisecondsPerBlock",
            "getMaxValidUntilBlockIncrement",
            "setMaxValidUntilBlockIncrement",
            "getMaxTraceableBlocks",
            "setMaxTraceableBlocks",
            "getAttributeFee",
            "setAttributeFee",
            "setStoragePrice",
            "blockAccount",
            "unblockAccount",
            "isBlocked",
            "getBlockedAccounts",
            "recoverFund",
            "setWhitelistFeeContract",
            "removeWhitelistFeeContract",
            "getWhitelistFeeContracts",
            "requestOracleData",
            "getOraclePrice",
            "setOraclePrice",
            "oracleFinish",
            "oracleVerify",
            "oracleRequest",
            "designateAsRole",
            "getDesignatedByRole",
            "currentIndex",
            "currentHash",
            "getBlock",
            "getBlockByIndex",
            "getBlockByHash",
            "getTransaction",
            "getTransactionHeight",
            "getTransactionFromBlock",
            "getTransactionSigners",
            "getTransactionVMState",
            "notaryVerify",
            "notaryBalanceOf",
            "notaryExpirationOf",
            "notaryLockDepositUntil",
            "notaryWithdraw",
            "notaryGetMaxNotValidBeforeDelta",
            "notarySetMaxNotValidBeforeDelta",
            "notaryOnNEP17Payment",
            "treasuryVerify",
            "treasuryOnNEP17Payment",
            "treasuryOnNEP11Payment",
            "isNativeContract",
            "getNativeContractName",
            "getAllNativeContracts",
            "estimateNativeCallGas",
            "batchNativeCalls",
            "getNetworkConfiguration",
            "getNativeContractManifest",
            "safeNativeCall",
            "externalNativeCall",
        ]),
        "Neo" => Some(&[
            "verifySignature",
            "callContract",
            "deployContract",
            "getNeoBalance",
            "getGasBalance",
            "getFeePerByte",
            "getGasPrice",
            "getStoragePrice",
            "getCommittee",
            "getValidators",
            "isCommittee",
            "isValidator",
            "getRandom",
            "getBlockHeight",
            "getCurrentBlock",
            "getBlockByIndex",
            "getBlockTime",
            "getTransaction",
            "getTransactionHeight",
            "transactionExists",
            "getNetworkMagic",
            "verifyWithWitness",
            "sha256Hash",
            "ripemd160Hash",
            // `transferNeo`/`transferGas` are lowered by
            // `try_lower_value_transfer_helpers` (value_transfer.rs) to the
            // NEO/GAS native `transfer` methods rather than via
            // `resolve_neo_member`, but they are part of the supported
            // devpack surface and belong in this diagnostic list.
            "transferNeo",
            "transferGas",
        ]),
        _ => None,
    }
}

pub(crate) fn resolve_abi_member(member: &str) -> Option<BuiltinCall> {
    match member {
        "encode" => Some(BuiltinCall::AbiEncode),
        "encodePacked" => Some(BuiltinCall::AbiEncodePacked),
        "encodeCall" => Some(BuiltinCall::AbiEncodeCall),
        "encodeWithSignature" => Some(BuiltinCall::AbiEncodeWithSignature),
        "encodeWithSelector" => Some(BuiltinCall::AbiEncodeWithSignature),
        "decode" => Some(BuiltinCall::AbiDecode),
        _ => None,
    }
}

/// `StdLib.<method>(...)` — resolve bare member-access into a NativeCall to
/// the StdLib native contract (hash c0ef39cee0e4e925c6c2a06a79e1440dd86fceac).
/// Solidity source can write either `StdLib.itoa(v, 10)` or the devpack's
/// `Syscalls.itoa(v, 10)`; both must land at the same CALLT. See
/// `resolve_syscalls_member` for the sibling dispatch.
pub(crate) fn resolve_stdlib_member(member: &str) -> Option<BuiltinCall> {
    match member {
        "serialize" | "deserialize" | "jsonSerialize" | "jsonDeserialize" | "itoa" | "atoi"
        | "base64Encode" | "base64Decode" | "base64UrlEncode" | "base64UrlDecode"
        | "base58Encode" | "base58Decode" | "base58CheckEncode" | "base58CheckDecode"
        | "memoryCompare" | "memorySearch" | "stringSplit" | "strLen" => {
            Some(BuiltinCall::NativeCall {
                contract: NativeContract::StdLib,
                method: member.to_string(),
            })
        }
        _ => None,
    }
}

/// `CryptoLib.<method>(...)` — resolve bare member-access into a NativeCall
/// to the CryptoLib native contract (hash 1bf575ab11896884136110a35a12886cde0b66c72).
/// Keeps parity with `Syscalls.sha256` / `Syscalls.ripemd160` / etc.
pub(crate) fn resolve_cryptolib_member(member: &str) -> Option<BuiltinCall> {
    match member {
        "sha256"
        | "ripemd160"
        | "verifyWithECDsa"
        | "murmur32"
        | "keccak256"
        | "recoverSecp256K1"
        | "verifyWithEd25519"
        | "bls12381Serialize"
        | "bls12381Deserialize"
        | "bls12381Equal"
        | "bls12381Add"
        | "bls12381Mul"
        | "bls12381Pairing"
        | "bls12381G1Add"
        | "bls12381G1Mul"
        | "bls12381G1Neg"
        | "bls12381G2Add"
        | "bls12381G2Mul"
        | "bls12381G2Neg" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::CryptoLib,
            method: member.to_string(),
        }),
        _ => None,
    }
}

/// CI probe (devpack agent): the devpack `Runtime`/`Storage`/`Neo` libraries
/// are compiler intrinsics whose Solidity bodies are never compiled, so every
/// non-private function they declare MUST have an intrinsic lowering —
/// otherwise the shipped sources advertise an API that hard-fails at compile
/// time (or worse, used to silently miscompile, e.g. the fictional
/// `System.Storage.Local.*` syscall family). These tests keep the `.sol`
/// surface, the `builtin_library_supported_members` diagnostic whitelist, and
/// the actual lowerings in sync.
#[cfg(test)]
mod devpack_intrinsic_surface_tests {
    use super::*;
    use std::path::PathBuf;

    /// Members lowered by dedicated expansion handlers rather than
    /// `resolve_*_member`:
    /// - Runtime: `try_lower_runtime_member_builtin`
    ///   (src/ir/expressions/calls/builtins/member_runtime.rs)
    /// - Storage: `try_lower_storage_member_builtin`
    ///   (src/ir/expressions/calls/builtins/member_storage.rs)
    /// - Neo: `try_lower_neo_member_builtin`
    ///   (src/ir/expressions/calls/builtins/member_neo.rs) and
    ///   `try_lower_value_transfer_helpers`
    ///   (src/ir/expressions/calls/value_transfer.rs for
    ///   `transferGas`/`transferNeo`)
    pub(crate) fn special_case_lowerings(base: &str) -> &'static [&'static str] {
        match base {
            "Runtime" => &[
                "initializeServices",
                "notifyIndexed",
                "notify",
                "requireWitness",
                "checkAnyWitness",
                "checkAllWitnesses",
                "checkMultiSigWitness",
            ],
            "Storage" => &["putContractMetadata"],
            "Neo" => &[
                "isCommittee",
                "getCommittee",
                "getValidators",
                "isValidator",
                "transferGas",
                "transferNeo",
            ],
            _ => &[],
        }
    }

    pub(crate) fn has_lowering(base: &str, member: &str) -> bool {
        let resolved = match base {
            "Runtime" => resolve_runtime_member(member).is_some(),
            "Storage" => resolve_storage_member(member).is_some(),
            "Neo" => resolve_neo_member(member).is_some(),
            other => panic!("unexpected library base '{other}'"),
        };
        resolved || special_case_lowerings(base).contains(&member)
    }

    pub(crate) fn devpack_library_path(base: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("devpack")
            .join("libraries")
            .join(format!("{base}.sol"))
    }

    /// Extract the names of all non-private function declarations from a
    /// devpack library source. Private helpers (declared `private` or named
    /// with a leading underscore) are not part of the public surface.
    pub(crate) fn declared_public_functions(source: &str) -> Vec<String> {
        let mut names = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix("function ") else {
                continue;
            };
            let Some(name) = crate::utils::method_name_from_signature(rest) else {
                continue;
            };
            if name.starts_with('_') {
                continue;
            }
            // The visibility keyword may appear on the declaration line or a
            // follow-up line; devpack sources keep `private` on the same line.
            if trimmed.contains(" private ") || trimmed.ends_with(" private") {
                continue;
            }
            names.push(name.clone());
        }
        names
    }

    #[test]
    pub(crate) fn every_declared_devpack_library_function_has_an_intrinsic_lowering() {
        for base in ["Runtime", "Storage", "Neo"] {
            let path = devpack_library_path(base);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let declared = declared_public_functions(&source);
            assert!(
                !declared.is_empty(),
                "no function declarations found in {} — extraction regressed?",
                path.display()
            );

            let missing: Vec<&String> = declared
                .iter()
                .filter(|name| !has_lowering(base, name))
                .collect();
            assert!(
                missing.is_empty(),
                "devpack/libraries/{base}.sol declares functions with no compiler \
                 lowering (calls would hard-fail; prune them or add a lowering): {missing:?}"
            );
        }
    }

    #[test]
    pub(crate) fn every_declared_devpack_library_function_is_whitelisted() {
        // The "supported intrinsics" diagnostic emitted for unsupported
        // members lists `builtin_library_supported_members`; everything the
        // shipped sources declare should appear there.
        for base in ["Runtime", "Storage", "Neo"] {
            let path = devpack_library_path(base);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let whitelist =
                builtin_library_supported_members(base).expect("builtin library whitelist");
            let missing: Vec<String> = declared_public_functions(&source)
                .into_iter()
                .filter(|name| !whitelist.contains(&name.as_str()))
                .collect();
            assert!(
                missing.is_empty(),
                "devpack/libraries/{base}.sol declares functions missing from \
                 builtin_library_supported_members: {missing:?}"
            );
        }
    }

    #[test]
    pub(crate) fn every_whitelisted_member_has_an_intrinsic_lowering() {
        // The converse direction: the whitelist is what the compiler prints
        // as "supported intrinsics", so a whitelisted member without a
        // lowering would advertise an API that still hard-fails.
        for base in ["Runtime", "Storage", "Neo"] {
            let whitelist =
                builtin_library_supported_members(base).expect("builtin library whitelist");
            let missing: Vec<&&str> = whitelist
                .iter()
                .filter(|member| !has_lowering(base, member))
                .collect();
            assert!(
                missing.is_empty(),
                "builtin_library_supported_members(\"{base}\") lists members \
                 with no compiler lowering: {missing:?}"
            );
        }
    }

    #[test]
    pub(crate) fn fictional_local_storage_syscalls_are_not_resolvable() {
        // Regression: `Storage.putLocal`/`getLocal`/`removeLocal`/`findLocal`
        // and the matching `Syscalls.storage*Local` wrappers lowered to
        // `System.Storage.Local.*` — syscalls that do not exist in Neo N3's
        // interop table, so compiled contracts faulted on real nodes. They
        // must stay unresolvable (callers now get the loud "unsupported
        // builtin library call" diagnostic).
        for member in ["putLocal", "getLocal", "removeLocal", "findLocal"] {
            assert!(
                resolve_storage_member(member).is_none(),
                "Storage.{member} must not resolve to an intrinsic"
            );
            assert!(
                !builtin_library_supported_members("Storage")
                    .expect("Storage whitelist")
                    .contains(&member),
                "Storage.{member} must not be whitelisted"
            );
        }
        for member in [
            "storageGetLocal",
            "storagePutLocal",
            "storageDeleteLocal",
            "storageFindLocal",
        ] {
            assert!(
                resolve_syscalls_member(member).is_none(),
                "Syscalls.{member} must not resolve to an intrinsic"
            );
            assert!(
                !builtin_library_supported_members("Syscalls")
                    .expect("Syscalls whitelist")
                    .contains(&member),
                "Syscalls.{member} must not be whitelisted"
            );
        }
    }

    #[test]
    pub(crate) fn miscompiling_storage_helpers_are_not_resolvable() {
        // Regression: these documented helpers used to lower to a single raw
        // syscall with the wrong arity/semantics (e.g. `count` returned an
        // iterator handle, `batchPut` issued one put with the arrays as
        // key/value, `getUsage` was a hardcoded PUSH0 stub). They must stay
        // unresolvable until faithful expansions exist.
        for member in [
            "batchPut",
            "batchGet",
            "batchDelete",
            "count",
            "countLocal",
            "findValues",
            "findLocalValues",
            "findKeys",
            "findLocalKeys",
            "clearPrefix",
            "exists",
            "isValidKey",
            "getUsage",
        ] {
            assert!(
                resolve_storage_member(member).is_none(),
                "Storage.{member} must not resolve to an intrinsic"
            );
            assert!(
                !builtin_library_supported_members("Storage")
                    .expect("Storage whitelist")
                    .contains(&member),
                "Storage.{member} must not be whitelisted"
            );
        }
    }

    #[test]
    pub(crate) fn no_resolver_emits_syscalls_outside_the_neo_n3_interop_table() {
        // Every BuiltinCall::Syscall name produced by the resolvers must be a
        // real Neo N3 ApplicationEngine interop. Guards against reintroducing
        // fictional names like System.Storage.Local.* or
        // System.Storage.GetUsage (interop IDs are sha256-derived, so an
        // unknown name compiles fine and FAULTs on real nodes).
        const CANONICAL: &[&str] = &[
            "System.Contract.Call",
            "System.Contract.CallNative",
            "System.Contract.CreateMultisigAccount",
            "System.Contract.CreateStandardAccount",
            "System.Contract.GetCallFlags",
            "System.Contract.NativeOnPersist",
            "System.Contract.NativePostPersist",
            "System.Crypto.CheckMultisig",
            "System.Crypto.CheckSig",
            "System.Iterator.Next",
            "System.Iterator.Value",
            "System.Runtime.BurnGas",
            "System.Runtime.CheckWitness",
            "System.Runtime.CurrentSigners",
            "System.Runtime.GasLeft",
            "System.Runtime.GetAddressVersion",
            "System.Runtime.GetCallingScriptHash",
            "System.Runtime.GetEntryScriptHash",
            "System.Runtime.GetExecutingScriptHash",
            "System.Runtime.GetInvocationCounter",
            "System.Runtime.GetNetwork",
            "System.Runtime.GetNotifications",
            "System.Runtime.GetRandom",
            "System.Runtime.GetScriptContainer",
            "System.Runtime.GetTime",
            "System.Runtime.GetTrigger",
            "System.Runtime.LoadScript",
            "System.Runtime.Log",
            "System.Runtime.Notify",
            "System.Runtime.Platform",
            "System.Storage.AsReadOnly",
            "System.Storage.Delete",
            "System.Storage.Find",
            "System.Storage.Get",
            "System.Storage.GetContext",
            "System.Storage.GetReadOnlyContext",
            "System.Storage.Put",
        ];

        let probe = |label: &str, builtin: Option<BuiltinCall>| {
            if let Some(BuiltinCall::Syscall(name)) = builtin {
                assert!(
                    CANONICAL.contains(&name.as_str()),
                    "{label} lowers to syscall '{name}' which is not in the \
                     Neo N3 interop table"
                );
            }
        };

        for base in ["Runtime", "Storage", "Syscalls", "Neo"] {
            let members = builtin_library_supported_members(base).expect("whitelist");
            for member in members {
                let builtin = match base {
                    "Runtime" => resolve_runtime_member(member),
                    "Storage" => resolve_storage_member(member),
                    "Syscalls" => resolve_syscalls_member(member),
                    "Neo" => resolve_neo_member(member),
                    _ => unreachable!(),
                };
                probe(&format!("{base}.{member}"), builtin);
            }
        }
    }
}
