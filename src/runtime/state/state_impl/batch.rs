use super::*;

impl StateManager {
    /// Execute state batch
    pub fn execute_batch(&mut self, batch: StateBatch) -> Result<(), RuntimeError> {
        if batch.atomic {
            // Create snapshot for rollback
            let snapshot_id = self.create_snapshot("Batch execution".to_string());

            // Execute all changes
            for change in &batch.changes {
                match self.apply_change(change) {
                    Ok(_) => {}
                    Err(e) => {
                        // Rollback on error
                        if let Some(snapshot) = self.snapshots.get(snapshot_id as usize) {
                            self.restore_snapshot(snapshot.clone())?;
                        }
                        return Err(e);
                    }
                }
            }
        } else {
            // Execute changes non-atomically (best-effort apply, analogous to
            // Ethereum's `Multicall3.tryAggregate(requireSuccess = false, ...)`).
            // Each malformed change is silently skipped; subsequent changes
            // continue to apply. Callers that need all-or-nothing semantics
            // must set `atomic: true` (which uses a snapshot for rollback).
            // See `tests/runtime_state_batch_tests.rs` for the canonical
            // contract and rationale.
            for change in &batch.changes {
                let _ = self.apply_change(change);
            }
        }

        Ok(())
    }

    fn apply_change(&mut self, change: &StateChange) -> Result<(), RuntimeError> {
        match change.change_type {
            StateChangeType::BalanceChange => {
                let new_balance =
                    u64::from_le_bytes(change.new_value.as_slice().try_into().map_err(|_| {
                        RuntimeError::StateError {
                            message: "Invalid balance format".to_string(),
                        }
                    })?);
                self.set_balance(&change.account, new_balance)
            }
            StateChangeType::NonceChange => {
                let new_nonce =
                    u64::from_le_bytes(change.new_value.as_slice().try_into().map_err(|_| {
                        RuntimeError::StateError {
                            message: "Invalid nonce format".to_string(),
                        }
                    })?);
                self.set_nonce(&change.account, new_nonce)
            }
            StateChangeType::CodeChange => self.set_code(&change.account, &change.new_value),
            StateChangeType::AccountCreation => {
                let initial_balance =
                    u64::from_le_bytes(change.new_value.as_slice().try_into().map_err(|_| {
                        RuntimeError::StateError {
                            message: "Invalid balance format".to_string(),
                        }
                    })?);
                self.create_account(&change.account, initial_balance)
            }
            StateChangeType::AccountDeletion => self.delete_account(&change.account),
            StateChangeType::StorageChange => {
                // Storage changes are handled by storage manager
                Ok(())
            }
        }
    }
}
