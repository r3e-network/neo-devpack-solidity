use super::*;

impl GasTracker {
    /// Create new gas tracker
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            base_cost: 0, // NeoVM has no base transaction cost in execution context
        }
    }

    /// Reset gas tracker
    pub fn reset(&mut self, new_limit: u64) {
        self.limit = new_limit;
        self.used = 0;
    }

    /// Consume a fixed gas amount for an operation.
    /// The primary interpreter charges via `spec::opcode_gas` and dynamic
    /// surcharges in `execute_instruction`; this method is used by bridge
    /// helper methods that bypass the main opcode dispatch.
    pub fn consume_gas(&mut self, _operation: &str, amount: u64) -> Result<(), RuntimeError> {
        if self.used + amount > self.limit {
            return Err(RuntimeError::OutOfGas {
                used: self.used + amount,
                limit: self.limit,
            });
        }

        self.used += amount;
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Get used gas
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Sync gas usage with execution context accounting.
    pub fn sync_from_execution(&mut self, execution_used: u64) {
        self.used = execution_used;
    }

    /// Base transaction cost that seeds gas tracking.
    pub fn base_cost(&self) -> u64 {
        self.base_cost
    }

    /// Get gas limit
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Check if out of gas
    pub fn out_of_gas(&self) -> bool {
        self.used >= self.limit
    }

    /// Get usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        (self.used as f64 / self.limit as f64) * 100.0
    }
}
