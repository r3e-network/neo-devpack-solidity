use super::*;

impl ExecutionContext {
    /// Size of the last return buffer
    pub fn returndatasize(&self) -> usize {
        self.return_data.len()
    }

    /// Copy bytes from the last return buffer into memory
    pub fn returndatacopy(
        &mut self,
        memory_offset: usize,
        return_offset: usize,
        length: usize,
    ) -> Result<(), RuntimeError> {
        if length == 0 {
            return Ok(());
        }

        let end = return_offset
            .checked_add(length)
            .ok_or(RuntimeError::ExecutionError {
                message: "Return data copy overflow".to_string(),
            })?;

        if end > self.return_data.len() {
            return Err(RuntimeError::ExecutionError {
                message: "Return data access out of bounds".to_string(),
            });
        }

        let slice = self.return_data[return_offset..end].to_vec();
        self.write_memory(memory_offset, &slice)
    }

    /// Replace the current return buffer
    pub fn set_return_data(&mut self, data: Vec<u8>) {
        self.return_data = data;
    }

    /// Access the return buffer
    pub fn return_data(&self) -> &[u8] {
        &self.return_data
    }

    /// Access logs
    pub fn logs(&self) -> &[LogEntry] {
        &self.logs
    }

    /// Task #27 (runtime slice) — the raw bytes popped by the most recent
    /// `THROW` opcode, preserved verbatim (not UTF-8 lossy'd into a String).
    ///
    /// The bridge reads this on the error path and routes it into
    /// `ExecutionResult.return_data`, modeling the EVM convention that a
    /// revert surfaces `return_data = selector || abi.encode(args)`.
    ///
    /// Returns an empty slice if no THROW has executed yet.
    pub fn revert_payload(&self) -> &[u8] {
        &self.revert_payload
    }
}
