use super::*;

impl StateManager {
    /// Query state with filters
    pub fn query_state(&self, query: StateQuery) -> Vec<&AccountState> {
        self.accounts
            .values()
            .filter(|account| {
                if let Some(ref addr) = query.account {
                    account.address == *addr
                } else {
                    true
                }
            })
            .collect()
    }

    /// Get state statistics
    pub fn get_statistics(&self) -> StateStatistics {
        let total_accounts = self.accounts.len();
        let total_balance: u64 = self.accounts.values().map(|a| a.balance).sum();
        let contracts = self.accounts.values().filter(|a| a.code.is_some()).count();

        StateStatistics {
            total_accounts,
            total_balance,
            contract_accounts: contracts,
            external_accounts: total_accounts - contracts,
            total_changes: self.change_count,
            snapshots_count: self.snapshots.len(),
        }
    }
}
