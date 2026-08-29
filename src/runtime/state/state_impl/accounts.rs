use super::*;

impl StateManager {
    /// Get account state
    pub fn get_account(&self, address: &str) -> Option<&AccountState> {
        self.accounts.get(address)
    }

    /// Get account balance
    pub fn get_balance(&self, address: &str) -> Result<u64, RuntimeError> {
        Ok(self
            .accounts
            .get(address)
            .map(|account| account.balance)
            .unwrap_or(0))
    }

    /// Set account balance
    pub fn set_balance(&mut self, address: &str, balance: u64) -> Result<(), RuntimeError> {
        let old_balance = self.get_balance(address)?;

        let created_at = self.current_timestamp();
        let account = self
            .accounts
            .entry(address.to_string())
            .or_insert_with(|| AccountState {
                address: address.to_string(),
                balance: 0,
                nonce: 0,
                code: None,
                code_hash: None,
                storage_root: None,
                created_at,
            });

        account.balance = balance;

        // Record state change
        self.record_change(StateChange {
            change_type: StateChangeType::BalanceChange,
            account: address.to_string(),
            key: None,
            old_value: Some(old_balance.to_le_bytes().to_vec()),
            new_value: balance.to_le_bytes().to_vec(),
        });

        Ok(())
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &str) -> Result<u64, RuntimeError> {
        Ok(self
            .accounts
            .get(address)
            .map(|account| account.nonce)
            .unwrap_or(0))
    }

    /// Set account nonce
    pub fn set_nonce(&mut self, address: &str, nonce: u64) -> Result<(), RuntimeError> {
        let old_nonce = self.get_nonce(address)?;

        let created_at = self.current_timestamp();
        let account = self
            .accounts
            .entry(address.to_string())
            .or_insert_with(|| AccountState {
                address: address.to_string(),
                balance: 0,
                nonce: 0,
                code: None,
                code_hash: None,
                storage_root: None,
                created_at,
            });

        account.nonce = nonce;

        // Record state change
        self.record_change(StateChange {
            change_type: StateChangeType::NonceChange,
            account: address.to_string(),
            key: None,
            old_value: Some(old_nonce.to_le_bytes().to_vec()),
            new_value: nonce.to_le_bytes().to_vec(),
        });

        Ok(())
    }

    /// Get contract code
    pub fn get_code(&self, address: &str) -> Option<&[u8]> {
        self.accounts
            .get(address)
            .and_then(|account| account.code.as_ref())
            .map(|code| code.as_slice())
    }

    /// Set contract code
    pub fn set_code(&mut self, address: &str, code: &[u8]) -> Result<(), RuntimeError> {
        let old_code = self.get_code(address).map(|c| c.to_vec());

        let created_at = self.current_timestamp();
        let code_hash = self.calculate_hash(code);

        let account = self
            .accounts
            .entry(address.to_string())
            .or_insert_with(|| AccountState {
                address: address.to_string(),
                balance: 0,
                nonce: 0,
                code: None,
                code_hash: None,
                storage_root: None,
                created_at,
            });
        account.code = Some(code.to_vec());
        account.code_hash = Some(code_hash);

        // Record state change
        self.record_change(StateChange {
            change_type: StateChangeType::CodeChange,
            account: address.to_string(),
            key: None,
            old_value: old_code,
            new_value: code.to_vec(),
        });

        Ok(())
    }

    /// Create new account
    pub fn create_account(
        &mut self,
        address: &str,
        initial_balance: u64,
    ) -> Result<(), RuntimeError> {
        if self.accounts.contains_key(address) {
            return Err(RuntimeError::StateError {
                message: format!("Account {address} already exists"),
            });
        }

        let account = AccountState {
            address: address.to_string(),
            balance: initial_balance,
            nonce: 0,
            code: None,
            code_hash: None,
            storage_root: None,
            created_at: self.current_timestamp(),
        };

        self.accounts.insert(address.to_string(), account);

        // Record state change
        self.record_change(StateChange {
            change_type: StateChangeType::AccountCreation,
            account: address.to_string(),
            key: None,
            old_value: None,
            new_value: initial_balance.to_le_bytes().to_vec(),
        });

        Ok(())
    }

    /// Delete account
    pub fn delete_account(&mut self, address: &str) -> Result<(), RuntimeError> {
        if let Some(account) = self.accounts.remove(address) {
            // Record state change
            self.record_change(StateChange {
                change_type: StateChangeType::AccountDeletion,
                account: address.to_string(),
                key: None,
                old_value: Some(account.balance.to_le_bytes().to_vec()),
                new_value: vec![],
            });
        }

        Ok(())
    }

    /// Check if account exists
    pub fn account_exists(&self, address: &str) -> bool {
        self.accounts.contains_key(address)
    }

    /// Transfer balance between accounts
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), RuntimeError> {
        let from_balance = self.get_balance(from)?;
        let to_balance = self.get_balance(to)?;

        if from_balance < amount {
            return Err(RuntimeError::StateError {
                message: "Insufficient balance".to_string(),
            });
        }

        self.set_balance(from, from_balance - amount)?;
        self.set_balance(to, to_balance + amount)?;

        Ok(())
    }
}
