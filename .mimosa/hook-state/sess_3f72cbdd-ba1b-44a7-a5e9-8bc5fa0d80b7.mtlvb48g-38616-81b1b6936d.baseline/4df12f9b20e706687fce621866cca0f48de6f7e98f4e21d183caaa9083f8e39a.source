use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_contract(
        &mut self,
        hash: &[u8; 20],
        method: &str,
        params: StackItem,
    ) -> StackItem {
        let method_lower = method.to_ascii_lowercase();
        if let Some(name) = spec::native_contract_name(hash) {
            match name {
                "CryptoLib" => Self::invoke_native_cryptolib(&method_lower, params),
                "StdLib" => Self::invoke_native_stdlib(&method_lower, params),
                "NEO" => self.invoke_native_neo(&method_lower, params),
                "GAS" => self.invoke_native_gas(&method_lower, params),
                "Policy" => self.invoke_native_policy(&method_lower, params),
                "Oracle" => self.invoke_native_oracle(&method_lower, params),
                "ContractManagement" => {
                    self.invoke_native_contract_management(&method_lower, params)
                }
                "RoleManagement" => self.invoke_native_role_management(&method_lower, params),
                "Ledger" => self.invoke_native_ledger(&method_lower, params),
                "Notary" => self.invoke_native_notary(&method_lower, params),
                "Treasury" => self.invoke_native_treasury(&method_lower, params),
                _ => StackItem::Null,
            }
        } else {
            // Unknown contract – return null to allow graceful handling
            let _ = params;
            StackItem::Null
        }
    }
}
