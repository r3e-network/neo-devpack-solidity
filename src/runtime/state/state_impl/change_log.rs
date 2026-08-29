use super::*;

impl StateManager {
    /// Get change count
    pub fn change_count(&self) -> u64 {
        self.change_count
    }

    /// Get recent changes
    pub fn get_recent_changes(&self, count: usize) -> &[StateChange] {
        let start = self.change_log.len().saturating_sub(count);
        &self.change_log[start..]
    }

    /// Clear change log
    pub fn clear_change_log(&mut self) {
        self.change_log.clear();
    }
}
