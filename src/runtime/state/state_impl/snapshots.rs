use super::*;

impl StateManager {
    /// Create state snapshot
    pub fn create_snapshot(&mut self, description: String) -> u64 {
        let snapshot_id = self.snapshots.len() as u64;
        let snapshot = StateSnapshot {
            id: snapshot_id,
            timestamp: self.current_timestamp(),
            accounts: self.accounts.clone(),
            description,
        };

        self.snapshots.push(snapshot);
        snapshot_id
    }

    /// Get state snapshot
    pub fn get_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            id: u64::MAX,
            timestamp: self.current_timestamp(),
            accounts: self.accounts.clone(),
            description: "Current state".to_string(),
        }
    }

    /// Restore from snapshot
    pub fn restore_snapshot(&mut self, snapshot: StateSnapshot) -> Result<(), RuntimeError> {
        // Record changes for each account
        for (address, account) in &snapshot.accounts {
            if let Some(current_account) = self.accounts.get(address) {
                if current_account.balance != account.balance {
                    self.record_change(StateChange {
                        change_type: StateChangeType::BalanceChange,
                        account: address.clone(),
                        key: None,
                        old_value: Some(current_account.balance.to_le_bytes().to_vec()),
                        new_value: account.balance.to_le_bytes().to_vec(),
                    });
                }
            }
        }

        self.accounts = snapshot.accounts;
        Ok(())
    }
}
