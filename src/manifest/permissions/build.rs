use super::*;

pub(crate) fn collect_contract_call_permissions(
    ir_module: &ir::Module,
) -> (BTreeMap<String, PermissionMethods>, bool) {
    let mut needs_wildcard = false;
    let mut contract_methods: BTreeMap<String, PermissionMethods> = BTreeMap::new();

    for function in &ir_module.functions {
        if needs_wildcard {
            break;
        }

        for req in analyze_contract_calls(function, ir_module) {
            let contract = req.contract.unwrap_or_else(|| "*".to_string());
            let methods = if let Some(method) = req.method {
                let mut set = std::collections::BTreeSet::new();
                set.insert(method);
                PermissionMethods::Some(set)
            } else {
                PermissionMethods::All
            };

            contract_methods
                .entry(contract.clone())
                .and_modify(|existing| existing.merge_in(methods.clone()))
                .or_insert(methods);

            if contract == "*" && matches!(contract_methods.get("*"), Some(PermissionMethods::All))
            {
                needs_wildcard = true;
                break;
            }
        }
    }

    (contract_methods, needs_wildcard)
}

/// Infer contract permissions based on method signatures and behavior
pub(crate) fn infer_permissions(
    metadata: &ContractMetadata,
    ir_module: &ir::Module,
    bytecode: &[u8],
    tokens: &[crate::neo::MethodToken],
) -> Vec<serde_json::Value> {
    // Neo N3 enforces contract call permissions via the manifest. Native contract
    // calls (StdLib/CryptoLib/etc) are executed through `System.Contract.Call` and
    // therefore also require explicit permissions, otherwise the contract will
    // fault on-chain.
    //
    // We keep manifests minimal by emitting:
    // - Exact permissions for native contracts when the contract only calls native contracts.
    // - Precise permissions for contract calls (`Syscalls.contractCall`) when the
    //   contract hash and/or method name are compile-time constants.
    // - Wildcard permissions only when the contract performs fully dynamic
    //   contract calls (both target + method unknown at compile time), because the
    //   destination can be user-controlled.
    let (contract_methods, needs_wildcard) = collect_contract_call_permissions(ir_module);
    let mut native_methods = collect_native_permissions(ir_module);

    // S6 follow-up — merge the bytecode-level scan. This catches native calls
    // the IR-level analysis can't see (storage key-derivation helpers emitted
    // directly to bytecode via StoreState(computed_slot), etc.). See
    // `collect_bytecode_native_permissions`.
    for (contract, methods) in collect_bytecode_native_permissions(bytecode, tokens) {
        native_methods.entry(contract).or_default().extend(methods);
    }

    // Task #55: the IR-level lowering for `new Contract(...)` does not emit a
    // `CallBuiltin::DeployContract`, so neither `collect_contract_call_permissions`
    // nor `collect_native_permissions` can discover the intent. Walk the parsed
    // AST to detect the pattern and wire the `ContractManagement.deploy`
    // permission ourselves; if the emitted bytecode ever starts performing the
    // deployment, the manifest will already authorize it.
    if contract_uses_new_contract(metadata) {
        let hash_le = codegen::native_contract_hash(ir::NativeContract::ContractManagement);
        let hash_be = hash_le.iter().rev().copied().collect::<Vec<_>>();
        let contract_str = format!("0x{}", hex::encode(hash_be));
        native_methods
            .entry(contract_str)
            .or_default()
            .insert("deploy".to_string());
    }

    if needs_wildcard {
        return vec![json!({ "contract": "*", "methods": "*" })];
    }

    let mut merged = contract_methods;

    for (contract, methods) in native_methods {
        let methods = PermissionMethods::Some(methods);
        merged
            .entry(contract)
            .and_modify(|existing| existing.merge_in(methods.clone()))
            .or_insert(methods);
    }

    merged
        .into_iter()
        .map(|(contract, methods)| {
            let methods_json = match methods {
                PermissionMethods::All => json!("*"),
                PermissionMethods::Some(set) => json!(set.into_iter().collect::<Vec<_>>()),
            };
            json!({
                "contract": contract,
                "methods": methods_json,
            })
        })
        .collect()
}
