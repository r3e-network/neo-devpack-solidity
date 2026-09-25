use super::*;

impl StateManager {
    /// Create new state manager
    pub fn new(_config: &RuntimeConfig) -> Result<Self, RuntimeError> {
        Ok(Self {
            accounts: HashMap::new(),
            snapshots: Vec::new(),
            change_log: Vec::new(),
            change_count: 0,
        })
    }
}
