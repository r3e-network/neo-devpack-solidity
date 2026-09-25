use super::*;

impl ExecutionResult {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.success && self.exception.is_none()
    }

    /// Get gas efficiency (percentage of gas limit used)
    pub fn gas_efficiency(&self) -> f64 {
        if self.gas_limit == 0 {
            0.0
        } else {
            (self.gas_used as f64) / (self.gas_limit as f64)
        }
    }

    /// Get remaining gas
    pub fn gas_remaining(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }

    /// Check if execution ran out of gas
    pub fn out_of_gas(&self) -> bool {
        matches!(
            self.exception,
            Some(RuntimeException {
                exception_type: ExceptionType::OutOfGas,
                ..
            })
        )
    }

    /// Get return data as string (if valid UTF-8)
    pub fn return_string(&self) -> Option<String> {
        String::from_utf8(self.return_data.clone()).ok()
    }

    /// Get return data as hex string
    pub fn return_hex(&self) -> String {
        hex::encode(&self.return_data)
    }

    /// Check if there were any state changes
    pub fn has_state_changes(&self) -> bool {
        !self.state_changes.is_empty()
    }

    /// Get the number of logs emitted
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }
}
