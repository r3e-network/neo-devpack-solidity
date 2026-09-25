use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_contract_management(
        &mut self,
        method: &str,
        params: StackItem,
    ) -> StackItem {
        match method {
            "getcontract" => {
                if let StackItem::Array(args) = params {
                    if let Some(StackItem::ByteArray {
                        data: hash_bytes, ..
                    }) = args.borrow().first()
                    {
                        let hash_slice = hash_bytes.borrow();
                        if let Some(state) = self.lookup_contract(hash_slice.as_slice()) {
                            return self.contract_to_stackitem(&state);
                        }
                        // Task #189 — self-registry fallback. The isolated
                        // test-harness runtime does not call `deploy` before
                        // invoking user methods, so `contract_registry` is
                        // empty. When a compiled `address(this).code` lowering
                        // routes through `ContractManagement.getContract(addr)`
                        // with `addr = default_account_bytes` (the executing
                        // script hash derived from Hash160(bytecode) in
                        // `initialize`), returning Null causes the downstream
                        // PICKITEM on .nef to fault with
                        // "PICKITEM: unsupported target Null". Synthesize a
                        // minimal ContractState backed by the actually-executing
                        // bytecode so the PICKITEM(index=3 / .nef) returns the
                        // script bytes — matching the EVM expectation that
                        // `address(this).code` yields the current contract's
                        // runtime bytecode.
                        if hash_slice.as_slice() == self.default_account_bytes.as_slice()
                            && !self.bytecode.is_empty()
                        {
                            let mut hash_le = [0u8; 20];
                            let copy_len = self.default_account_bytes.len().min(20);
                            hash_le[..copy_len]
                                .copy_from_slice(&self.default_account_bytes[..copy_len]);
                            let synthetic = ContractState {
                                id: 0,
                                hash: hash_le,
                                nef: self.bytecode.clone(),
                                manifest: Vec::new(),
                                update_counter: 0,
                            };
                            return self.contract_to_stackitem(&synthetic);
                        }
                    }
                }
                StackItem::Null
            }
            "deploy" => {
                if let StackItem::Array(args) = params {
                    let args = args.borrow();
                    if args.len() >= 2 {
                        let nef = Self::stack_item_to_bytes(args[0].clone());
                        let manifest = Self::stack_item_to_bytes(args[1].clone());
                        let state = self.register_contract(nef, manifest);
                        return self.contract_to_stackitem(&state);
                    }
                }
                StackItem::Null
            }
            "update" => {
                if let StackItem::Array(args) = params {
                    let args = args.borrow();
                    if args.len() >= 2 {
                        let nef = Self::stack_item_to_bytes(args[0].clone());
                        let manifest = Self::stack_item_to_bytes(args[1].clone());
                        if let Some(hash) = self.contract_registry.keys().next().cloned() {
                            if let Some(state) =
                                self.update_contract(&hash, nef.clone(), manifest.clone())
                            {
                                return self.contract_to_stackitem(&state);
                            }
                        }
                        let state = self.register_contract(nef, manifest);
                        return self.contract_to_stackitem(&state);
                    }
                }
                StackItem::Null
            }
            // `ContractManagement.isContract(addr)` — returns true when the
            // supplied hash is the currently-executing contract OR has been
            // deployed into the in-memory registry. The embedded test-harness
            // runtime does not call `deploy` before invoking methods (see
            // the self-registry fallback in `getcontract` above), so without
            // this explicit self-check the Solidity-level
            // `NativeCalls.isContract(address(this))` would return `false`
            // even though the contract is actively executing.
            "iscontract" => {
                if let StackItem::Array(args) = params {
                    if let Some(StackItem::ByteArray {
                        data: hash_bytes, ..
                    }) = args.borrow().first()
                    {
                        let hash_slice = hash_bytes.borrow();
                        // Self-check: compiled contract is running — treat
                        // its `default_account_bytes` as an existing contract.
                        if hash_slice.as_slice() == self.default_account_bytes.as_slice()
                            && !self.bytecode.is_empty()
                        {
                            return StackItem::Boolean(true);
                        }
                        // Registry check: anything deployed via `deploy` lives here.
                        return StackItem::Boolean(
                            self.lookup_contract(hash_slice.as_slice()).is_some(),
                        );
                    }
                }
                StackItem::Boolean(false)
            }
            // `ContractManagement.hasMethod(addr, method, paramCount)` —
            // returns true when the method exists on the target contract.
            // For the currently-executing contract, consult the `self_method_offsets`
            // table populated from the manifest.
            "hasmethod" => {
                if let StackItem::Array(args) = params {
                    let args_ref = args.borrow();
                    if args_ref.len() >= 2 {
                        if let Some(StackItem::ByteArray {
                            data: hash_bytes, ..
                        }) = args_ref.first()
                        {
                            let hash_slice = hash_bytes.borrow();
                            let method_bytes = Self::stack_item_to_bytes(args_ref[1].clone());
                            let method = String::from_utf8(method_bytes).unwrap_or_default();
                            if hash_slice.as_slice() == self.default_account_bytes.as_slice()
                                && self.self_method_offsets.contains_key(&method)
                            {
                                return StackItem::Boolean(true);
                            }
                            if let Some(state) = self.lookup_contract(hash_slice.as_slice()) {
                                if let Ok(manifest_json) =
                                    serde_json::from_slice::<serde_json::Value>(&state.manifest)
                                {
                                    if let Some(methods) = manifest_json
                                        .get("abi")
                                        .and_then(|abi| abi.get("methods"))
                                        .and_then(|m| m.as_array())
                                    {
                                        let exists = methods.iter().any(|m| {
                                            m.get("name").and_then(|n| n.as_str())
                                                == Some(method.as_str())
                                        });
                                        return StackItem::Boolean(exists);
                                    }
                                }
                            }
                        }
                    }
                }
                StackItem::Boolean(false)
            }
            // `ContractManagement.getMinimumDeploymentFee()` — Neo N3
            // default is 10 GAS (10 * 10^8 = 1_000_000_000 fractions). The
            // embedded runtime has no policy-driven state, so return the
            // canonical default so Solidity-level tests see a non-zero
            // value instead of `Null` (which decodes to 0 and falsely
            // passes `require(fee > 0)`-style guards without running the
            // real Policy call).
            "getminimumdeploymentfee" => StackItem::Integer(1_000_000_000),
            // ContractManagement.destroyContract(hash) — removes the contract
            // from the in-memory registry. Returns Null (matching Neo N3 which
            // returns void). No state persistence in the embedded runtime.
            // NOTE: the on-chain method name is `destroy`; `destroycontract`
            // is kept as a legacy alias for older compiled artifacts.
            "destroycontract" | "destroy" => {
                if let StackItem::Array(args) = &params {
                    if let Some(StackItem::ByteArray {
                        data: hash_bytes, ..
                    }) = args.borrow().first()
                    {
                        let hash_slice = hash_bytes.borrow();
                        self.contract_registry.remove(hash_slice.as_slice());
                    }
                }
                StackItem::Null
            }
            // ContractManagement.getContractById(id) — returns the ContractState
            // for the given integer ID, or Null if not found.
            "getcontractbyid" => {
                if let StackItem::Array(args) = &params {
                    if let Some(id_item) = args.borrow().first() {
                        let id = Self::extract_first_int(id_item) as u32;
                        for state in self.contract_registry.values() {
                            if state.id == id {
                                return self.contract_to_stackitem(state);
                            }
                        }
                    }
                }
                StackItem::Null
            }
            // ContractManagement.getContractHashes() — the REAL Neo N3 method
            // for enumerating deployed contracts (there is no `listContracts`
            // on-chain; `listcontracts` is kept as a legacy alias of the
            // embedded runtime only). Returns the array of deployed contract
            // states in the registry.
            "getcontracthashes" | "listcontracts" => {
                let contracts: Vec<StackItem> = self
                    .contract_registry
                    .values()
                    .map(|state| self.contract_to_stackitem(state))
                    .collect();
                StackItem::array(contracts)
            }
            // ContractManagement.setMinimumDeploymentFee(fee) — governance
            // operation. No-op in the embedded runtime; return Null.
            "setminimumdeploymentfee" => StackItem::Null,
            _ => StackItem::Null,
        }
    }
}
