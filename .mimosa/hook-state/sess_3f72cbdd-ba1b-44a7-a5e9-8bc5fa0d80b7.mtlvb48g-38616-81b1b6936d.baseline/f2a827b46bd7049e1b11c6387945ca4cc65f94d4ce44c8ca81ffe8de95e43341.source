use super::*;

pub(crate) fn require_native_method(
    native_methods: &mut BTreeMap<String, BTreeSet<String>>,
    contract: ir::NativeContract,
    method: &str,
) {
    let hash_le = codegen::native_contract_hash(contract);
    let hash_be = hash_le.iter().rev().copied().collect::<Vec<_>>();
    let contract_str = format!("0x{}", hex::encode(hash_be));
    native_methods
        .entry(contract_str)
        .or_default()
        .insert(method.to_string());
}

/// S6 follow-up — record a raw `(hash_le, method)` native-call target
/// discovered by the bytecode scan into the same `"0x<be>" → {methods}` shape
/// `collect_native_permissions` produces. The hash is the eval-stack
/// little-endian UInt160 (reversed to big-endian for the manifest key).
fn require_raw_native(
    native_methods: &mut BTreeMap<String, BTreeSet<String>>,
    hash_le: &[u8],
    method: &str,
) {
    let hash_be: Vec<u8> = hash_le.iter().rev().copied().collect();
    let contract_str = format!("0x{}", hex::encode(hash_be));
    native_methods
        .entry(contract_str)
        .or_default()
        .insert(method.to_string());
}

/// S6 follow-up — bytecode-level native-call permission scan.
///
/// The IR-level `collect_native_permissions` only sees native calls that
/// surface as `BuiltinCall::NativeCall`. A few codegen paths emit native
/// calls directly to bytecode WITHOUT an IR-level marker — most notably the
/// storage key-derivation helpers (`keccak256` / `serialize`) reached via
/// `StoreState(computed_slot)` for fixed-size array elements inside structs.
/// Their calls show up only as `System.Contract.Call` SYSCALL sites or
/// `CALLT` (method-token) references in the final script. This pass scans
/// both surfaces and records every native target so the manifest grants the
/// permissions Neo N3 would otherwise fault for on-chain.
pub(crate) fn collect_bytecode_native_permissions(
    bytecode: &[u8],
    tokens: &[crate::neo::MethodToken],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    // CALLT (0x37) — each references a method token by u16 index. The token
    // carries the exact (hash, method) the VM dispatches, so it is always
    // authoritative regardless of which IR path emitted it.
    let mut i = 0;
    while i < bytecode.len() {
        if bytecode[i] == 0x37 && i + 2 < bytecode.len() {
            let idx = u16::from_le_bytes([bytecode[i + 1], bytecode[i + 2]]);
            if let Some(token) = tokens.get(idx as usize) {
                if !token.method.is_empty() {
                    require_raw_native(&mut out, &token.hash, &token.method);
                }
            }
            i += 3;
            continue;
        }
        i += 1;
    }

    // System.Contract.Call SYSCALL sites. The compiler's lowering pushes
    // `[args, flags, method, hash]` (hash on top) immediately before the
    // SYSCALL, so scanning backward from each site finds the 20-byte hash
    // PUSHDATA and then the method-name string PUSHDATA just beneath it.
    let scc_id = {
        let d = sha2::Sha256::digest(b"System.Contract.Call");
        [d[0], d[1], d[2], d[3]]
    };
    let mut i = 0;
    while i + 4 < bytecode.len() {
        if bytecode[i] == 0x41 && bytecode[i + 1..i + 5] == scc_id {
            if let Some((hash_le, method)) = scan_scc_operands(bytecode, i) {
                require_raw_native(&mut out, &hash_le, &method);
            }
            i += 5;
            continue;
        }
        i += 1;
    }

    out
}

/// Walk backward from a `System.Contract.Call` SYSCALL at `scc_off` and
/// recover the `(hash_le, method)` operands the compiler pushed. The hash is
/// the nearest preceding `PUSHDATA1 0x14` (20-byte UInt160, little-endian as
/// carried on the eval stack); the method is the nearest preceding short
/// ASCII-string `PUSHDATA1` before the hash. Returns `None` if either operand
/// can't be located (e.g. a dynamic target the scan can't statically resolve —
/// those are already covered by the wildcard fallback in `infer_permissions`).
fn scan_scc_operands(bytecode: &[u8], scc_off: usize) -> Option<(Vec<u8>, String)> {
    // Nearest preceding 20-byte PUSHDATA1 = the contract hash.
    let mut hash_off = None;
    let mut j = scc_off;
    while j > 0 {
        j -= 1;
        if bytecode[j] == 0x0C && j + 1 < scc_off && bytecode[j + 1] == 0x14 && j + 22 <= scc_off {
            hash_off = Some(j + 2);
            break;
        }
    }
    let hash_start = hash_off?;
    let hash_le = bytecode[hash_start..hash_start + 20].to_vec();

    // Nearest preceding short ASCII PUSHDATA1 before the hash = the method name.
    let mut method = String::new();
    let mut j = hash_start - 2; // index of the hash's PUSHDATA1 opcode
    while j > 0 {
        j -= 1;
        if bytecode[j] == 0x0C && j + 1 < hash_start {
            let len = bytecode[j + 1] as usize;
            let start = j + 2;
            if len > 0 && len <= 64 && start + len <= hash_start {
                if let Ok(s) = std::str::from_utf8(&bytecode[start..start + len]) {
                    if s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                        method = s.to_string();
                        break;
                    }
                }
            }
        }
    }

    if method.is_empty() {
        return None;
    }
    Some((hash_le, method))
}

pub(crate) fn collect_native_permissions(
    ir_module: &ir::Module,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut native_methods: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for function in &ir_module.functions {
        for block in &function.basic_blocks {
            for instr in &block.instructions {
                match instr {
                    ir::Instruction::CallBuiltin { builtin, .. } => match builtin {
                        ir::BuiltinCall::ContractCall | ir::BuiltinCall::ContractCallWithFlags => {
                            // Builtin lowering always deserializes args and serializes return.
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "deserialize",
                            );
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "serialize",
                            );
                        }
                        ir::BuiltinCall::NativeCall { contract, method } => {
                            // Backwards-compatible alias: older devpacks used
                            // `ContractManagement.listContracts`, but the Neo native contract
                            // exposes this as `getContractHashes`.
                            if matches!(contract, ir::NativeContract::ContractManagement)
                                && method == "listContracts"
                            {
                                require_native_method(
                                    &mut native_methods,
                                    ir::NativeContract::ContractManagement,
                                    "getContractHashes",
                                );
                            } else {
                                require_native_method(
                                    &mut native_methods,
                                    *contract,
                                    method.as_str(),
                                );
                            }
                        }
                        ir::BuiltinCall::DeployContract => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::ContractManagement,
                                "deploy",
                            );
                        }
                        ir::BuiltinCall::GetContract => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::ContractManagement,
                                "getContract",
                            );
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "serialize",
                            );
                        }
                        ir::BuiltinCall::GetContractScript => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::ContractManagement,
                                "getContract",
                            );
                        }
                        ir::BuiltinCall::GetNeoAccountState => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::Neo,
                                "getAccountState",
                            );
                        }
                        ir::BuiltinCall::AbiEncode | ir::BuiltinCall::AbiEncodePacked => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "serialize",
                            );
                        }
                        ir::BuiltinCall::AbiDecode => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "deserialize",
                            );
                        }
                        ir::BuiltinCall::RuntimeNotify | ir::BuiltinCall::NotifySerialized => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "deserialize",
                            );
                        }
                        ir::BuiltinCall::Keccak256 => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::CryptoLib,
                                "keccak256",
                            );
                        }
                        ir::BuiltinCall::Ecrecover | ir::BuiltinCall::PrecompileEcrecover => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::CryptoLib,
                                "recoverSecp256K1",
                            );
                            // Task #20: ecrecover lowers to
                            //   recoverSecp256K1 → SUBSTR(1,64) → keccak256 → RIGHT 20
                            // so keccak256 is also a required CryptoLib method.
                            // Task #H6a extends the same lowering to the 0x01
                            // precompile staticcall routing.
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::CryptoLib,
                                "keccak256",
                            );
                        }
                        ir::BuiltinCall::VerifySignature => {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::CryptoLib,
                                "verifyWithECDsa",
                            );
                        }
                        ir::BuiltinCall::Syscall(name) => {
                            let _ = name;
                        }
                        _ => {}
                    },
                    ir::Instruction::LoadRuntimeValue(value) => {
                        if matches!(value, ir::RuntimeValue::BlockNumber) {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::Ledger,
                                "currentIndex",
                            );
                        }
                    }
                    ir::Instruction::LoadMappingElement { key_types, .. }
                    | ir::Instruction::StoreMappingElement { key_types, .. }
                        if !key_types.is_empty() =>
                    {
                        // Mapping storage slots are derived via:
                        //   StdLib.serialize(key) + CryptoLib.keccak256(...)
                        //
                        // When `key_types` is empty, the slot is the base slot and no hashing is
                        // performed, so avoid adding unnecessary permissions.
                        require_native_method(
                            &mut native_methods,
                            ir::NativeContract::StdLib,
                            "serialize",
                        );
                        require_native_method(
                            &mut native_methods,
                            ir::NativeContract::CryptoLib,
                            "keccak256",
                        );
                    }
                    ir::Instruction::LoadStructField { key_types, .. }
                    | ir::Instruction::StoreStructField { key_types, .. } => {
                        // Struct field storage slots always require at least one keccak256
                        // (base slot + field key). If the struct is nested under a mapping,
                        // additional key serialization + hashing occurs.
                        require_native_method(
                            &mut native_methods,
                            ir::NativeContract::CryptoLib,
                            "keccak256",
                        );
                        if !key_types.is_empty() {
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::StdLib,
                                "serialize",
                            );
                        }
                    }
                    ir::Instruction::LoadStructArrayElement { key_types, .. }
                    | ir::Instruction::StoreStructArrayElement { key_types, .. } => {
                        // Array elements stored under a struct field require:
                        // - keccak256 for the field-slot derivation and the element-slot derivation
                        // - StdLib.serialize for the element index (and any mapping keys)
                        require_native_method(
                            &mut native_methods,
                            ir::NativeContract::CryptoLib,
                            "keccak256",
                        );
                        require_native_method(
                            &mut native_methods,
                            ir::NativeContract::StdLib,
                            "serialize",
                        );
                        if !key_types.is_empty() {
                            // mapping keys also require serialize/keccak256; already covered above,
                            // but keep this for clarity in case the rules evolve.
                            require_native_method(
                                &mut native_methods,
                                ir::NativeContract::CryptoLib,
                                "keccak256",
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    native_methods
}
