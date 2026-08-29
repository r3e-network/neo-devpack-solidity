use super::*;

impl NeoRuntime {
    /// Deploy contract and return address
    pub fn deploy_contract(
        &mut self,
        bytecode: &[u8],
        constructor_args: &[u8],
    ) -> Result<String, RuntimeError> {
        let deployer = self.execution_context.default_account.clone();

        // Generate contract address
        let address = self.generate_contract_address()?;

        // Store bytecode in state
        self.state_manager.set_code(&address, bytecode)?;

        // Execute constructor if present
        if !constructor_args.is_empty() {
            let result = self.execute(bytecode, constructor_args)?;
            if !result.success {
                return Err(RuntimeError::ExecutionError {
                    message: format!("Constructor failed: {:?}", result.exception),
                });
            }
        }

        // Advance deployer nonce so subsequent deployments derive a new address.
        let nonce = self.state_manager.get_nonce(&deployer)?;
        self.state_manager
            .set_nonce(&deployer, nonce.saturating_add(1))?;

        Ok(address)
    }

    fn generate_contract_address(&self) -> Result<String, RuntimeError> {
        // Generate deterministic contract address using deployer address + nonce
        use sha3::{Digest, Keccak256};

        let deployer = self.execution_context.default_account_bytes();
        let nonce = self
            .state_manager
            .get_nonce(&self.execution_context.default_account)?;

        let mut input = Vec::with_capacity(20 + std::mem::size_of::<u64>());
        input.extend_from_slice(deployer);
        input.extend_from_slice(&nonce.to_le_bytes());

        let hash = Keccak256::digest(&input);
        Ok(format!("0x{}", hex::encode(&hash[12..32]))) // Take last 20 bytes
    }
}
