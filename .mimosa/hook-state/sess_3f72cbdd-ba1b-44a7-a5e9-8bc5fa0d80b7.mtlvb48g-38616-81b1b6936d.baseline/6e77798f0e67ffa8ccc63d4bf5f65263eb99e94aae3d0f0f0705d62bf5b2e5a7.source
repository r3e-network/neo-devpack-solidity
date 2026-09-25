use super::*;

impl VMBridge {
    /// Create new VM bridge
    pub fn new(config: &RuntimeConfig) -> Result<Self, RuntimeError> {
        Ok(Self {
            config: config.clone(),
            contract_account: config.contract_account.clone(),
        })
    }
}
