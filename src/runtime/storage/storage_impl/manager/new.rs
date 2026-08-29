use super::*;

impl StorageManager {
    /// Create new storage manager
    pub fn new(_config: &RuntimeConfig) -> Result<Self, RuntimeError> {
        Ok(Self {
            storage: HashMap::new(),
            read_count: 0,
            write_count: 0,
            gas_costs: StorageGasCosts::default(),
        })
    }
}
